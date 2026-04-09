use crate::ir::ResolvedSpec;
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

fn render_client(spec: &ResolvedSpec, namespace: &str) -> String {
    let mut out = String::new();

    out.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Client;\n\n"));
    out.push_str("use Psr\\Http\\Client\\ClientInterface;\n");
    out.push_str("use Psr\\Http\\Message\\RequestFactoryInterface;\n");
    out.push_str("use Psr\\Http\\Message\\StreamFactoryInterface;\n\n");
    out.push_str(&format!("/** {} API Client (auto-generated) */\n", spec.title));
    out.push_str("final class ApiClient\n{\n");
    out.push_str(&format!(
        "    private const BASE_URL = '{}';\n\n",
        spec.base_url
    ));
    out.push_str("    public function __construct(\n");
    out.push_str("        private readonly ClientInterface $httpClient,\n");
    out.push_str("        private readonly RequestFactoryInterface $requestFactory,\n");
    out.push_str("        private readonly StreamFactoryInterface $streamFactory,\n");
    out.push_str("        private readonly string $baseUrl = self::BASE_URL,\n");
    out.push_str("    ) {}\n");

    for ep in &spec.endpoints {
        let fn_name = escape_reserved(&to_camel_case(&ep.operation_id));
        let method_str = ep.method.as_str();

        // Collect parameters
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

        let return_type = ep
            .response
            .as_ref()
            .map(|s| schema_to_php_type(s, false))
            .unwrap_or_else(|| "void".to_string());

        out.push('\n');
        if let Some(summary) = &ep.summary {
            out.push_str(&format!("    /** {summary} */\n"));
        }
        if ep.deprecated {
            out.push_str("    #[\\Deprecated]\n");
        }

        let params_str = params.join(", ");
        out.push_str(&format!(
            "    public function {fn_name}({params_str}): {return_type}\n    {{\n"
        ));

        // Build path with substituted variables
        let path_expr = build_path_expr(&ep.path, &ep.path_params);
        out.push_str(&format!(
            "        $request = $this->requestFactory\n            ->createRequest('{method_str}', $this->baseUrl . {path_expr});\n"
        ));

        if ep.request_body.is_some() {
            out.push_str("        $stream = $this->streamFactory->createStream(json_encode($body, JSON_THROW_ON_ERROR));\n");
            out.push_str("        $request = $request->withBody($stream)->withHeader('Content-Type', 'application/json');\n");
        }

        out.push_str("        $response = $this->httpClient->sendRequest($request);\n");
        if return_type == "void" {
            // nothing to return
        } else {
            out.push_str(
                "        return json_decode((string) $response->getBody(), true, 512, JSON_THROW_ON_ERROR);\n",
            );
        }
        out.push_str("    }\n");
    }

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

    // Strip leading slash already in the sprintf string; keep it.
    format!("sprintf('{fmt}', {args})")
}
