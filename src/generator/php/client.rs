use crate::parser::types::OpenApi;
use anyhow::Result;
use std::path::Path;

pub fn generate(spec: &OpenApi, output: &Path, namespace: &str) -> Result<()> {
    let client_dir = output.join("Client");
    std::fs::create_dir_all(&client_dir)?;

    let php = render_client(spec, namespace);
    let path = client_dir.join("ApiClient.php");
    std::fs::write(&path, php)?;
    println!("  📄 Client/ApiClient.php");

    Ok(())
}

fn render_client(spec: &OpenApi, namespace: &str) -> String {
    let title = &spec.info.title;
    let base_url = spec.servers.as_ref()
        .and_then(|s| s.first())
        .map(|s| s.url.as_str())
        .unwrap_or("");

    let mut out = String::new();

    out.push_str("<?php\n\n");
    out.push_str("declare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Client;\n\n"));
    out.push_str("use Psr\\Http\\Client\\ClientInterface;\n");
    out.push_str("use Psr\\Http\\Message\\RequestFactoryInterface;\n");
    out.push_str("use Psr\\Http\\Message\\StreamFactoryInterface;\n\n");
    out.push_str(&format!("/** {title} API Client (auto-generated) */\n"));
    out.push_str("final class ApiClient\n{\n");
    out.push_str(&format!("    private const BASE_URL = '{base_url}';\n\n"));
    out.push_str("    public function __construct(\n");
    out.push_str("        private readonly ClientInterface $httpClient,\n");
    out.push_str("        private readonly RequestFactoryInterface $requestFactory,\n");
    out.push_str("        private readonly StreamFactoryInterface $streamFactory,\n");
    out.push_str("        private readonly string $baseUrl = self::BASE_URL,\n");
    out.push_str("    ) {}\n");

    if let Some(paths) = &spec.paths {
        for (path, item) in paths {
            for (method, op) in [
                ("get", &item.get),
                ("post", &item.post),
                ("put", &item.put),
                ("patch", &item.patch),
                ("delete", &item.delete),
            ] {
                let Some(op) = op else { continue };
                let fn_name = op.operation_id.as_deref()
                    .unwrap_or("unknownOperation")
                    .to_string();
                let fn_name = to_camel_case(&fn_name);

                out.push('\n');
                if let Some(summary) = &op.summary {
                    out.push_str(&format!("    /** {summary} */\n"));
                }
                out.push_str(&format!("    public function {fn_name}(): mixed\n    {{\n"));
                out.push_str(&format!("        $request = $this->requestFactory\n"));
                out.push_str(&format!("            ->createRequest('{method}', $this->baseUrl . '{path}');\n"));
                out.push_str("        $response = $this->httpClient->sendRequest($request);\n");
                out.push_str("        return json_decode((string) $response->getBody(), true);\n");
                out.push_str("    }\n");
            }
        }
    }

    out.push_str("}\n");
    out
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}
