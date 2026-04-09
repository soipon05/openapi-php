//! codegen_test.rs
//!
//! Integration tests for the CodegenBackend trait and PlainPhpBackend.
//! Tests use `run_dry()` to generate PHP into memory (no filesystem writes)
//! and assert on the rendered content without personal data or real endpoints.

use openapi_php::generator::{CodegenBackend, CodegenContext, PlainPhpBackend};
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
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new();
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
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new();
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
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
    let item = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(item.contains("declare(strict_types=1);"));
    assert!(item.contains("namespace App\\Generated\\Models;"));
    assert!(item.contains("final class Item"));
}

#[test]
fn model_has_from_array_and_to_array() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
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
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
    let item = files[&PathBuf::from("Models/Item.php")].as_str();

    assert!(item.contains("\\DateTimeImmutable"), "Expected \\DateTimeImmutable type hint");
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
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
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
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
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
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
    let client = files[&PathBuf::from("Client/ApiClient.php")].as_str();

    assert!(client.contains("assertSuccessful("), "Missing assertSuccessful helper");
    assert!(client.contains("decodeJson("), "Missing decodeJson helper");
    assert!(client.contains("JSON_THROW_ON_ERROR"), "Missing JSON_THROW_ON_ERROR");
}

#[test]
fn client_throws_docblock() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = PlainPhpBackend::new().run_dry(&ctx).unwrap();
    let client = files[&PathBuf::from("Client/ApiClient.php")].as_str();

    assert!(client.contains("@throws \\Psr\\Http\\Client\\ClientExceptionInterface"));
    assert!(client.contains("@throws \\RuntimeException On non-2xx response"));
}
