//! validate_test.rs
//!
//! Mirrors what the CLI `validate` sub-command does — calls
//! `parser::load_and_resolve` and checks the resolved IR envelope (title,
//! version, base_url, schema count, endpoint count).  Tests run without a
//! pre-built binary.
//!
//! Error-path tests (missing file, malformed YAML) call `parser::load` because
//! those errors surface before the resolver runs.

use openapi_php::parser;
use pretty_assertions::assert_eq;
use std::path::{Path, PathBuf};

// ─── Helper ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ─── simple.yaml via ResolvedSpec ─────────────────────────────────────────────

#[test]
fn validate_simple_returns_ok() {
    let result = parser::load_and_resolve(&fixture("simple.yaml"));
    assert!(
        result.is_ok(),
        "simple.yaml must resolve cleanly: {:?}",
        result.err()
    );
}

#[test]
fn validate_simple_info_fields() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    assert_eq!(spec.title, "Simple API");
    assert_eq!(spec.version, "1.0.0");
}

#[test]
fn validate_simple_server_url() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    assert_eq!(spec.base_url, "https://api.example.com");
}

/// The resolver should produce 3 named schemas: Item, ItemStatus, CreateItemRequest.
#[test]
fn validate_simple_schema_count() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    assert_eq!(
        spec.schemas.len(),
        3,
        "simple.yaml must resolve to 3 schemas, got {}",
        spec.schemas.len()
    );
}

/// The resolver should produce 4 endpoints: listItems, createItem, getItem, deleteItem.
#[test]
fn validate_simple_endpoint_count() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    assert_eq!(
        spec.endpoints.len(),
        4,
        "simple.yaml must have 4 endpoints, got {}",
        spec.endpoints.len()
    );
}

// ─── petstore.yaml via ResolvedSpec ───────────────────────────────────────────

#[test]
fn validate_petstore_returns_ok() {
    let result = parser::load_and_resolve(&fixture("petstore.yaml"));
    assert!(
        result.is_ok(),
        "petstore.yaml must resolve cleanly: {:?}",
        result.err()
    );
}

#[test]
fn validate_petstore_info_fields() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    assert_eq!(spec.title, "Fictional Petstore API");
    assert_eq!(spec.version, "1.0.0");
}

#[test]
fn validate_petstore_server_url() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    assert_eq!(spec.base_url, "https://petstore.example.com/v1");
}

/// Resolver must produce 9 named schemas:
/// Pet, NewPet, Category, Tag, PetStatus, DomesticPet, PetOrError, ApiResponse, Error.
#[test]
fn validate_petstore_schema_count() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    assert_eq!(
        spec.schemas.len(),
        9,
        "petstore.yaml must resolve to 9 schemas, got {}",
        spec.schemas.len()
    );
}

/// Resolver must produce 5 endpoints:
/// listPets, createPet, getPetById, updatePet, deletePet.
#[test]
fn validate_petstore_endpoint_count() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    assert_eq!(
        spec.endpoints.len(),
        5,
        "petstore.yaml must have 5 endpoints, got {}",
        spec.endpoints.len()
    );
}

// ─── Error-path tests ─────────────────────────────────────────────────────────

/// Loading a path that doesn't exist must return an Err, not panic.
#[test]
fn validate_missing_file_returns_error() {
    let result = parser::load(Path::new("/nonexistent/fixture/spec.yaml"));
    assert!(
        result.is_err(),
        "loading a non-existent file must return an error"
    );
}

/// Loading a file with invalid YAML must return an Err, not panic.
#[test]
fn validate_invalid_yaml_returns_error() {
    let dir = std::env::temp_dir();
    let path = dir.join("openapi_php_test_invalid.yaml");
    std::fs::write(&path, b": this is not: valid: yaml: [\n").unwrap();
    let result = parser::load(&path);
    let _ = std::fs::remove_file(&path); // best-effort cleanup
    assert!(
        result.is_err(),
        "loading malformed YAML must return an error"
    );
}
