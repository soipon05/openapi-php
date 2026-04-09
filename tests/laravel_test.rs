//! laravel_test.rs
//!
//! Integration tests for LaravelPhpBackend.
//! Uses `run_dry()` to generate PHP into memory without touching the filesystem.

use openapi_php::generator::{CodegenBackend, CodegenContext, LaravelPhpBackend};
use openapi_php::parser;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn laravel_generates_form_requests() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new();
    let files = backend.run_dry(&ctx).unwrap();

    let key = PathBuf::from("Http/Requests/ItemRequest.php");
    assert!(
        files.contains_key(&key),
        "Expected Http/Requests/ItemRequest.php to be generated; got: {:?}",
        files.keys().collect::<Vec<_>>()
    );
    let content = &files[&key];
    assert!(
        content.contains("FormRequest"),
        "Expected FormRequest in ItemRequest.php"
    );
    assert!(
        content.contains("public function rules()"),
        "Expected rules() method"
    );
    assert!(
        content.contains("declare(strict_types=1)"),
        "Expected strict_types declaration"
    );
}

#[test]
fn laravel_generates_resources() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new();
    let files = backend.run_dry(&ctx).unwrap();

    let key = PathBuf::from("Http/Resources/ItemResource.php");
    assert!(
        files.contains_key(&key),
        "Expected Http/Resources/ItemResource.php to be generated"
    );
    let content = &files[&key];
    assert!(
        content.contains("JsonResource"),
        "Expected JsonResource in ItemResource.php"
    );
    assert!(
        content.contains("public function toArray("),
        "Expected toArray() method"
    );
    assert!(
        content.contains("@mixin"),
        "Expected @mixin docblock for IDE support"
    );
}

#[test]
fn laravel_generates_routes() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new();
    let files = backend.run_dry(&ctx).unwrap();

    let key = PathBuf::from("routes/api.php");
    assert!(
        files.contains_key(&key),
        "Expected routes/api.php to be generated"
    );
    let content = &files[&key];
    assert!(
        content.contains("Route::"),
        "Expected Route:: calls in routes/api.php"
    );
    assert!(
        content.contains("use Illuminate\\Support\\Facades\\Route"),
        "Expected Route facade import"
    );
    // simple.yaml has GET + POST /items and GET + DELETE /items/{id}
    assert!(content.contains("Route::get("), "Expected Route::get()");
    assert!(content.contains("Route::post("), "Expected Route::post()");
    assert!(content.contains("Route::delete("), "Expected Route::delete()");
}

#[test]
fn laravel_still_generates_models() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new();
    let files = backend.run_dry(&ctx).unwrap();

    // DTOs must be generated alongside Laravel-specific files
    assert!(
        files.contains_key(&PathBuf::from("Models/Item.php")),
        "Expected Models/Item.php"
    );
    assert!(
        files.contains_key(&PathBuf::from("Models/ItemStatus.php")),
        "Expected Models/ItemStatus.php (enum)"
    );
    assert!(
        files.contains_key(&PathBuf::from("Models/CreateItemRequest.php")),
        "Expected Models/CreateItemRequest.php"
    );
    let item = &files[&PathBuf::from("Models/Item.php")];
    assert!(
        item.contains("final class Item"),
        "Expected final class Item in DTO"
    );
}
