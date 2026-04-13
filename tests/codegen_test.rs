//! codegen_test.rs
//!
//! Integration tests for the CodegenBackend trait and PlainPhpBackend.
//! Tests use `run_dry()` to generate PHP into memory (no filesystem writes)
//! and assert on the rendered content without personal data or real endpoints.

use openapi_php::cli::GenerateMode;
use openapi_php::config::Framework;
use openapi_php::config::PhpVersion;
use openapi_php::generator::{CodegenBackend, CodegenContext, PlainPhpBackend, run_dry_filtered};
use openapi_php::parser;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ─── run_dry returns all expected files ───────────────────────────────────────

#[test]
fn run_dry_simple_returns_expected_paths() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let paths: Vec<String> = files.keys().map(|p| p.display().to_string()).collect();
    assert!(paths.contains(&"Models/Item.php".to_string()));
    assert!(paths.contains(&"Models/ItemStatus.php".to_string()));
    assert!(paths.contains(&"Models/CreateItemRequest.php".to_string()));
    assert!(paths.contains(&"Client/ApiClient.php".to_string()));
}

#[test]
fn run_dry_petstore_returns_all_models() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    // At minimum: Pet, PetStatus, ApiClient
    assert!(files.contains_key(&PathBuf::from("Models/Pet.php")));
    assert!(files.contains_key(&PathBuf::from("Models/PetStatus.php")));
    assert!(files.contains_key(&PathBuf::from("Client/ApiClient.php")));
}

// ─── Generated PHP structure tests ───────────────────────────────────────────

#[test]
fn model_has_declare_strict_and_namespace() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let item = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(item.contains("declare(strict_types=1);"));
    assert!(item.contains("namespace App\\Generated\\Models;"));
    assert!(item.contains("final class Item"));
}

#[test]
fn model_has_from_array_and_to_array() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let item = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(item.contains("public static function fromArray(array $data): self"));
    assert!(item.contains("public function toArray(): array"));
    assert!(item.contains("return new self("));
    assert!(item.contains("return array_filter("));
}

#[test]
fn model_datetime_uses_date_time_immutable() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let item = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(
        item.contains("\\DateTimeImmutable"),
        "Expected \\DateTimeImmutable type hint"
    );
    assert!(
        item.contains("new \\DateTimeImmutable("),
        "Expected new \\DateTimeImmutable() in fromArray"
    );
    assert!(
        item.contains("\\DateTimeInterface::RFC3339"),
        "Expected RFC3339 format in toArray"
    );
}

#[test]
fn enum_has_backed_type_and_cases() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let status = files[&PathBuf::from("Models/ItemStatus.php")].as_str();

    assert!(status.contains("declare(strict_types=1);"));
    assert!(status.contains("enum ItemStatus: string"));
    // Variants depend on fixture; just confirm at least one `case` line
    assert!(status.contains("case "), "Expected at least one enum case");
}

#[test]
fn client_has_psr18_constructor() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let client = files[&PathBuf::from("Client/ApiClient.php")].as_str();

    assert!(client.contains("use Psr\\Http\\Client\\ClientInterface;"));
    assert!(client.contains("use Psr\\Http\\Message\\RequestFactoryInterface;"));
    assert!(client.contains("private readonly ClientInterface $httpClient"));
    assert!(client.contains("final class ApiClient"));
    assert!(client.contains("private const BASE_URL ="));
}

#[test]
fn client_has_assert_successful_and_decode_json() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let client = files[&PathBuf::from("Client/ApiClient.php")].as_str();

    assert!(
        client.contains("assertSuccessful("),
        "Missing assertSuccessful helper"
    );
    assert!(client.contains("decodeJson("), "Missing decodeJson helper");
    assert!(
        client.contains("JSON_THROW_ON_ERROR"),
        "Missing JSON_THROW_ON_ERROR"
    );
}

#[test]
fn client_throws_docblock() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let client = files[&PathBuf::from("Client/ApiClient.php")].as_str();

    assert!(client.contains("@throws \\Psr\\Http\\Client\\ClientExceptionInterface"));
    // Endpoints without error cases still emit the generic RuntimeException docblock
    assert!(client.contains("@throws \\RuntimeException On unexpected non-2xx response"));
}

// ─── run_dry_filtered tests ───────────────────────────────────────────────────

#[test]
fn dry_run_models_mode_excludes_client_files() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let files = run_dry_filtered(
        &spec,
        "App\\Test",
        &GenerateMode::Models,
        &Framework::Plain,
        None,
        &PhpVersion::Php82,
    )
    .unwrap();

    // Every returned path must be under Models/
    for path in files.keys() {
        assert!(
            path.starts_with("Models"),
            "Expected only Models/ files, got: {}",
            path.display()
        );
    }
    // Sanity: at least one model was returned
    assert!(!files.is_empty(), "Expected at least one model file");
}

#[test]
fn dry_run_all_files_start_with_php_open_tag() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    let files = run_dry_filtered(
        &spec,
        "App\\Test",
        &GenerateMode::All,
        &Framework::Plain,
        None,
        &PhpVersion::Php82,
    )
    .unwrap();

    assert!(!files.is_empty(), "Expected generated files");
    for (path, content) in &files {
        assert!(
            content.starts_with("<?php"),
            "File {} does not start with <?php",
            path.display()
        );
    }
}

// ─── PHP version conditional output ──────────────────────────────────────────

#[test]
fn php82_uses_readonly_class() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();
    let content = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(content.contains("readonly final class Item"), "8.2 should use readonly class");
    assert!(!content.contains("public readonly"), "8.2 should not have per-property readonly");
}

#[test]
fn php81_uses_per_property_readonly() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php81,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();
    let content = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(content.contains("final class Item"), "8.1 should use plain final class");
    assert!(!content.contains("readonly final class"), "8.1 should not use readonly class");
    assert!(content.contains("public readonly"), "8.1 should have per-property readonly");
}

// ─── Auth injection tests ─────────────────────────────────────────────────────

#[test]
fn parse_bearer_auth_scheme() {
    let spec = parser::load_and_resolve(&fixture("bearer_auth.yaml")).unwrap();
    assert!(
        spec.security_schemes.iter().any(|s| s.name == "BearerAuth"),
        "Expected BearerAuth scheme in resolved spec"
    );
}

#[test]
fn parse_api_key_auth_scheme() {
    let spec = parser::load_and_resolve(&fixture("bearer_auth.yaml")).unwrap();
    assert!(
        spec.security_schemes.iter().any(|s| s.name == "ApiKeyHeader"),
        "Expected ApiKeyHeader scheme in resolved spec"
    );
}

#[test]
fn no_security_schemes_on_simple_spec() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    assert!(
        spec.security_schemes.is_empty(),
        "simple.yaml has no securitySchemes"
    );
}

#[test]
fn client_ctx_has_bearer_auth_flag() {
    use openapi_php::generator::php::context::build_client_ctx;

    let spec = parser::load_and_resolve(&fixture("bearer_auth.yaml")).unwrap();
    let ctx = build_client_ctx(&spec, "App\\Generated");
    assert!(ctx.has_bearer_auth, "has_bearer_auth should be true");
    assert!(ctx.has_api_key_header_auth, "has_api_key_header_auth should be true");
    assert!(!ctx.auth_schemes.is_empty(), "auth_schemes should not be empty");
}

#[test]
fn client_ctx_no_auth_on_simple_spec() {
    use openapi_php::generator::php::context::build_client_ctx;

    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = build_client_ctx(&spec, "App\\Generated");
    assert!(!ctx.has_bearer_auth, "has_bearer_auth should be false for simple.yaml");
    assert!(!ctx.has_api_key_header_auth);
    assert!(ctx.auth_schemes.is_empty());
}

#[test]
fn client_ctx_bearer_auth_scheme_fields() {
    use openapi_php::generator::php::context::build_client_ctx;

    let spec = parser::load_and_resolve(&fixture("bearer_auth.yaml")).unwrap();
    let ctx = build_client_ctx(&spec, "App\\Generated");

    let bearer = ctx
        .auth_schemes
        .iter()
        .find(|s| s.constructor_param.contains("bearerToken"))
        .expect("Expected bearer auth scheme in auth_schemes");

    assert!(
        bearer.constructor_param.contains("?string $bearerToken"),
        "constructor_param should declare nullable bearerToken"
    );
    assert_eq!(bearer.header_name, "Authorization");
    assert!(
        bearer.header_prefix.contains("Bearer"),
        "header_prefix should include Bearer"
    );
}

#[test]
fn client_ctx_api_key_scheme_fields() {
    use openapi_php::generator::php::context::build_client_ctx;

    let spec = parser::load_and_resolve(&fixture("bearer_auth.yaml")).unwrap();
    let ctx = build_client_ctx(&spec, "App\\Generated");

    let api_key = ctx
        .auth_schemes
        .iter()
        .find(|s| s.header_name.contains("X-API-Key"))
        .expect("Expected API key auth scheme in auth_schemes");

    assert!(
        api_key.constructor_param.contains("?string $"),
        "constructor_param should declare a nullable string property"
    );
    assert_eq!(api_key.header_prefix, "");
}
