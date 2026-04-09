use crate::ir::{ResolvedSchema, ResolvedSpec};
use anyhow::Result;
use std::path::Path;

use super::models::{escape_reserved, schema_to_php_type, to_camel_case};

pub fn generate(spec: &ResolvedSpec, output: &Path, namespace: &str) -> Result<()> {
    let client_dir = output.join("Client");
    std::fs::create_dir_all(&client_dir)?;

    let php = render_client(spec, namespace);
    let path = client_dir.join("ApiClient.php");
    std::fs::write(&path, php)?;
    println!("  📄 Client/ApiClient.php");

    Ok(())
}

/// How a method returns its response.
enum ReturnKind {
    /// No response body (void)
    Void,
    /// Response is a known model class — use `fromArray`
    Ref(String),
    /// Response is an array/primitive — use raw `decodeJson`
    Array,
}

/// Resolve the PHP return type and how the response should be decoded.
fn resolve_return(response: &Option<ResolvedSchema>) -> (String, ReturnKind) {
    match response {
        None => ("void".to_string(), ReturnKind::Void),
        Some(ResolvedSchema::Ref(name)) => (name.to_string(), ReturnKind::Ref(name.to_string())),
        Some(schema) => {
            let php_type = schema_to_php_type(schema, false);
            (php_type, ReturnKind::Array)
        }
    }
}

fn render_client(spec: &ResolvedSpec, namespace: &str) -> String {
    let mut out = String::new();

    // Issue 5: only import StreamFactory when at least one endpoint has a request body
    let needs_stream_factory = spec.endpoints.iter().any(|ep| ep.request_body.is_some());

    // Issue 3: collect model Refs used as return types for `use` statements
    let mut model_refs: Vec<String> = spec
        .endpoints
        .iter()
        .filter_map(|ep| {
            if let Some(ResolvedSchema::Ref(name)) = &ep.response {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();
    model_refs.sort();
    model_refs.dedup();

    out.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Client;\n\n"));
    out.push_str("use Psr\\Http\\Client\\ClientInterface;\n");
    out.push_str("use Psr\\Http\\Message\\RequestFactoryInterface;\n");
    if needs_stream_factory {
        out.push_str("use Psr\\Http\\Message\\StreamFactoryInterface;\n");
    }
    for model_name in &model_refs {
        out.push_str(&format!("use {namespace}\\Models\\{model_name};\n"));
    }
    out.push('\n');
    out.push_str(&format!("/** {} API Client (auto-generated) */\n", spec.title));
    out.push_str("final class ApiClient\n{\n");
    out.push_str(&format!(
        "    private const BASE_URL = '{}';\n\n",
        spec.base_url
    ));
    out.push_str("    public function __construct(\n");
    out.push_str("        private readonly ClientInterface $httpClient,\n");
    out.push_str("        private readonly RequestFactoryInterface $requestFactory,\n");
    if needs_stream_factory {
        out.push_str("        private readonly StreamFactoryInterface $streamFactory,\n");
    }
    out.push_str("        private readonly string $baseUrl = self::BASE_URL,\n");
    out.push_str("    ) {}\n");

    for ep in &spec.endpoints {
        let fn_name = escape_reserved(&to_camel_case(&ep.operation_id));
        let method_str = ep.method.as_str();

        // Collect typed parameters
        let mut params: Vec<String> = Vec::new();
        for p in &ep.path_params {
            let php_type = schema_to_php_type(&p.schema, !p.required);
            params.push(format!("{php_type} ${}", p.php_name));
        }
        for p in &ep.query_params {
            let php_type = schema_to_php_type(&p.schema, !p.required);
            params.push(format!("{php_type} ${}", p.php_name));
        }
        if let Some(rb) = &ep.request_body {
            let php_type = schema_to_php_type(&rb.schema, !rb.required);
            params.push(format!("{php_type} $body"));
        }

        let (return_type, return_kind) = resolve_return(&ep.response);
        let has_json =
            !matches!(return_kind, ReturnKind::Void) || ep.request_body.is_some();

        // Issue 6: @throws docblock
        out.push('\n');
        out.push_str("    /**\n");
        if let Some(summary) = &ep.summary {
            out.push_str(&format!("     * {summary}\n"));
            out.push_str("     *\n");
        }
        out.push_str("     * @throws \\Psr\\Http\\Client\\ClientExceptionInterface\n");
        out.push_str("     * @throws \\RuntimeException On non-2xx response\n");
        if has_json {
            out.push_str("     * @throws \\JsonException On JSON error\n");
        }
        out.push_str("     */\n");

        if ep.deprecated {
            out.push_str("    #[\\Deprecated]\n");
        }

        let params_str = params.join(", ");
        out.push_str(&format!(
            "    public function {fn_name}({params_str}): {return_type}\n    {{\n"
        ));

        // Issue 4: build URI — inline when no query params, separate $uri with http_build_query otherwise
        let path_expr = build_path_expr(&ep.path, &ep.path_params);
        if ep.query_params.is_empty() {
            out.push_str(&format!(
                "        $request = $this->requestFactory\n            ->createRequest('{method_str}', $this->baseUrl . {path_expr});\n"
            ));
        } else {
            out.push_str(&format!(
                "        $uri = $this->baseUrl . {path_expr} . '?' . http_build_query([\n"
            ));
            for p in &ep.query_params {
                out.push_str(&format!("            '{}' => ${},\n", p.name, p.php_name));
            }
            out.push_str("        ]);\n");
            out.push_str(&format!(
                "        $request = $this->requestFactory->createRequest('{method_str}', $uri);\n"
            ));
        }

        if ep.request_body.is_some() {
            out.push_str("        $stream = $this->streamFactory->createStream(json_encode($body, JSON_THROW_ON_ERROR));\n");
            out.push_str("        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');\n");
        }

        out.push_str("        $response = $this->httpClient->sendRequest($request);\n");

        // Issue 1: assert 2xx status
        out.push_str(&format!(
            "        $this->assertSuccessful($response, '{method_str}', '{}');\n",
            ep.path
        ));

        // Issue 2 & 3: decode and return
        match return_kind {
            ReturnKind::Void => {}
            ReturnKind::Ref(ref name) => {
                out.push_str(&format!(
                    "        return {name}::fromArray($this->decodeJson($response));\n"
                ));
            }
            ReturnKind::Array => {
                out.push_str("        return $this->decodeJson($response);\n");
            }
        }
        out.push_str("    }\n");
    }

    // Issue 2: decodeJson helper
    out.push_str("\n    /** @return array<string, mixed> */\n");
    out.push_str("    private function decodeJson(\\Psr\\Http\\Message\\ResponseInterface $response): array\n    {\n");
    out.push_str("        /** @var array<string, mixed> $data */\n");
    out.push_str("        $data = json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);\n");
    out.push_str("        return $data;\n");
    out.push_str("    }\n");

    // Issue 1: assertSuccessful helper
    out.push_str("\n    private function assertSuccessful(\n");
    out.push_str("        \\Psr\\Http\\Message\\ResponseInterface $response,\n");
    out.push_str("        string $method,\n");
    out.push_str("        string $uri,\n");
    out.push_str("    ): void {\n");
    out.push_str("        $status = $response->getStatusCode();\n");
    out.push_str("        if ($status >= 200 && $status < 300) {\n");
    out.push_str("            return;\n");
    out.push_str("        }\n");
    out.push_str("        throw new \\RuntimeException(\n");
    out.push_str("            sprintf('HTTP %d error: %s %s', $status, $method, $uri),\n");
    out.push_str("            $status,\n");
    out.push_str("        );\n");
    out.push_str("    }\n");

    out.push_str("}\n");
    out
}

fn build_path_expr(path: &str, path_params: &[crate::ir::ResolvedParam]) -> String {
    if path_params.is_empty() {
        return format!("'{path}'");
    }
    // Convert /users/{userId}/posts → sprintf('/users/%s/posts', $userId)
    let fmt = path
        .split('/')
        .map(|seg| {
            if seg.starts_with('{') && seg.ends_with('}') {
                "%s".to_string()
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    let args: String = path_params
        .iter()
        .map(|p| format!("${}", p.php_name))
        .collect::<Vec<_>>()
        .join(", ");

    format!("sprintf('{fmt}', {args})")
}
