//! OpenAPI 3.1 specific feature coverage.
//!
//! OAS 3.1 adds JSON Schema 2020-12 features on top of OAS 3.0:
//! - `type: [T, "null"]` nullable arrays (replacing `nullable: true`)
//! - `const` keyword for fixed literal values
//! - Top-level `webhooks:` key
//! - `info.summary`, `license.identifier` (SPDX) metadata
//!
//! The fixture `oas31_comprehensive.yaml` exercises all of these at once.
//! These tests verify they parse and generate without panicking or producing
//! output that PHPStan would reject. The PHP syntax/PHPStan guarantees are
//! separately enforced by the `scripts/phpstan-check.sh` gate in CI — here
//! we lock in the Rust-visible behavior.

use openapi_php::cli::GenerateMode;
use openapi_php::config::{Framework, PhpVersion};
use openapi_php::parser;
use openapi_php::{generator, ir::ResolvedSchema};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn generate_plain(fixture_name: &str, namespace: &str) -> BTreeMap<PathBuf, String> {
    let spec = parser::load_and_resolve(&fixture(fixture_name)).unwrap();
    generator::run_dry_filtered(
        &spec,
        namespace,
        &GenerateMode::All,
        &Framework::Plain,
        None,
        &PhpVersion::Php82,
        false,
    )
    .unwrap()
}

// ─── Parse-level guarantees ───────────────────────────────────────────────────

/// The comprehensive OAS 3.1 fixture parses cleanly and produces the expected
/// surface: one Item schema, two endpoints. The top-level `webhooks:` block
/// must not crash the parser (we currently drop webhooks — they are tolerated).
#[test]
fn comprehensive_oas31_spec_parses_and_resolves() {
    let spec = parser::load_and_resolve(&fixture("oas31_comprehensive.yaml")).unwrap();

    assert_eq!(spec.title, "OAS 3.1 Comprehensive Test");
    assert_eq!(spec.version, "1.2.3");
    assert_eq!(spec.endpoints.len(), 2);
    assert!(spec.schemas.contains_key("Item"));
}

/// OAS 3.1 `type: [T, "null"]` on a query parameter resolves to a nullable
/// primitive — not to a union class, not to an error.
#[test]
fn nullable_type_array_on_query_param_is_optional_scalar() {
    let spec = parser::load_and_resolve(&fixture("oas31_comprehensive.yaml")).unwrap();
    let list_items = spec
        .endpoints
        .iter()
        .find(|e| e.operation_id == "listItems")
        .unwrap();
    let cursor = list_items
        .query_params
        .iter()
        .find(|p| p.name == "cursor")
        .unwrap();
    assert!(!cursor.required, "cursor must be optional");
    assert!(
        matches!(&cursor.schema, ResolvedSchema::Primitive(_)),
        "type: [string, null] must resolve to a primitive, got {:?}",
        cursor.schema
    );
}

// ─── Codegen-level guarantees ─────────────────────────────────────────────────

/// Required-after-optional property ordering in the source spec must not leak
/// into the PHP constructor: PHP 8+ deprecates optional-before-required params.
/// The generator sorts required-non-default first.
#[test]
fn required_fields_are_emitted_before_optional_in_constructor() {
    let files = generate_plain("oas31_comprehensive.yaml", "App\\Oas31");
    let item = &files[&PathBuf::from("Models/Item.php")];

    let id_pos = item.find("public int $id").unwrap();
    let name_pos = item.find("public string $name").unwrap();
    let kind_pos = item.find("public string $kind").unwrap();
    let desc_pos = item.find("public ?string $description").unwrap();
    let score_pos = item.find("public ?float $score").unwrap();

    // All required (non-nullable) params before any nullable-with-default.
    assert!(
        id_pos < desc_pos && name_pos < desc_pos && kind_pos < desc_pos,
        "required params must precede optional:\n{item}"
    );
    assert!(
        desc_pos < score_pos, // stable sort preserves within-group order
        "optional param order must be preserved (description before score):\n{item}"
    );
}

/// OAS 3.1 nullable via `type: [T, "null"]` generates `?T` in PHP,
/// identical to the OAS 3.0 `nullable: true` behavior.
#[test]
fn type_array_with_null_generates_nullable_php_type() {
    let files = generate_plain("oas31_comprehensive.yaml", "App\\Oas31");
    let item = &files[&PathBuf::from("Models/Item.php")];

    assert!(
        item.contains("public ?string $description"),
        "type: [string, null] must emit ?string:\n{item}"
    );
    assert!(
        item.contains("public ?float $score"),
        "type: [number, null] must emit ?float:\n{item}"
    );
}

/// The emitted PHPStan `@phpstan-type ItemData array{…}` shape agrees with the
/// constructor ordering — required keys first, optionals marked with `?`.
#[test]
fn phpstan_shape_reflects_required_vs_optional() {
    let files = generate_plain("oas31_comprehensive.yaml", "App\\Oas31");
    let item = &files[&PathBuf::from("Models/Item.php")];

    // Required keys: no `?` marker
    assert!(item.contains("'id': int,"));
    assert!(item.contains("'name': string,"));
    assert!(item.contains("'kind': string,"));
    // Optional + nullable keys: `?` marker, `|null` in type
    assert!(item.contains("'description'?: string|null,"));
    assert!(item.contains("'score'?: float|null,"));
}

/// OAS 3.1 nullable `type: [string, null]` on a query parameter must not emit
/// `is_bool` cast (no bool params) nor crash on URI building. The generated
/// client must pass PHP syntax check.
#[test]
fn oas31_client_renders_without_bool_cast_or_errors() {
    let files = generate_plain("oas31_comprehensive.yaml", "App\\Oas31");
    let client = &files[&PathBuf::from("Client/ApiClient.php")];

    assert!(
        !client.contains("is_bool($v)"),
        "OAS 3.1 comprehensive fixture has no bool params → no is_bool dispatch:\n{client}"
    );
    // listItems must still build a query string for the nullable cursor param.
    assert!(
        client.contains("array_filter(["),
        "optional cursor param must flow through array_filter:\n{client}"
    );
}

/// JSON Schema 2020-12 `const: "item"` combined with `type: string` must
/// resolve to a plain `string` property. The `const` literal is absorbed
/// through the type annotation — the current generator does not elevate it
/// to a single-variant enum. This test pins that behavior so a regression
/// (e.g. accidental phantom enum generation) is caught.
#[test]
fn const_keyword_resolves_through_string_type() {
    let files = generate_plain("oas31_comprehensive.yaml", "App\\Oas31");
    let item = &files[&PathBuf::from("Models/Item.php")];

    assert!(
        item.contains("public string $kind"),
        "const-annotated property must emit as plain string:\n{item}"
    );
    assert!(
        !files.contains_key(&PathBuf::from("Models/Kind.php")),
        "const must not create a phantom enum file"
    );
}

/// Top-level `webhooks:` from OAS 3.1 must not contribute to the endpoint
/// list (we don't generate webhook handlers yet) and must not crash the
/// resolver.
#[test]
fn webhooks_are_tolerated_but_not_emitted_as_endpoints() {
    let spec = parser::load_and_resolve(&fixture("oas31_comprehensive.yaml")).unwrap();
    assert!(
        spec.endpoints.iter().all(|e| e.operation_id != "onItemCreated"),
        "webhook operationIds must not leak into the client"
    );
}
