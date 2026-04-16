//! parse_test.rs
//!
//! Verifies that every fixture file loads into a `RawOpenApi` value without
//! panicking, and that the high-level structure (paths, schemas, compositions)
//! matches what the YAML declares.
//!
//! Uses `parser::load` (raw parsing only), NOT the resolver, so tests stay
//! fast and independent of resolver logic.

use openapi_php::parser;
use openapi_php::parser::raw::RawOrRef;
use std::path::{Path, PathBuf};

// ─── Helper ───────────────────────────────────────────────────────────────────

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ─── Load / round-trip tests ──────────────────────────────────────────────────

/// The minimal fixture must deserialise without error.
#[test]
fn simple_fixture_loads() {
    let result = parser::load(&fixture("simple.yaml"));
    assert!(
        result.is_ok(),
        "simple.yaml failed to parse: {:?}",
        result.err()
    );
}

/// The comprehensive petstore fixture must deserialise without error.
#[test]
fn petstore_fixture_loads() {
    let result = parser::load(&fixture("petstore.yaml"));
    assert!(
        result.is_ok(),
        "petstore.yaml failed to parse: {:?}",
        result.err()
    );
}

// ─── simple.yaml structural assertions ───────────────────────────────────────

/// The OpenAPI envelope version must be "3.0.3".
#[test]
fn simple_fixture_openapi_version() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    assert_eq!(spec.openapi, "3.0.3");
}

/// components.schemas must contain exactly Item, ItemStatus, CreateItemRequest.
#[test]
fn simple_fixture_has_expected_schemas() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    // In the new raw types, Components.schemas is IndexMap (non-optional).
    let schemas = spec
        .components
        .as_ref()
        .map(|c| &c.schemas)
        .expect("simple.yaml must have components.schemas");

    for expected in ["Item", "ItemStatus", "CreateItemRequest"] {
        assert!(
            schemas.contains_key(expected),
            "simple.yaml is missing schema: {expected}"
        );
    }
    assert_eq!(
        schemas.len(),
        3,
        "simple.yaml should have exactly 3 schemas, found {}",
        schemas.len()
    );
}

/// paths must contain both /items and /items/{id}.
/// In the new raw types, `paths` is a plain IndexMap — never None.
#[test]
fn simple_fixture_has_expected_paths() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    let paths = &spec.paths; // IndexMap — no Option unwrap needed

    for expected in ["/items", "/items/{id}"] {
        assert!(
            paths.contains_key(expected),
            "simple.yaml is missing path: {expected}"
        );
    }
    assert_eq!(
        paths.len(),
        2,
        "simple.yaml should have exactly 2 paths, found {}",
        paths.len()
    );
}

/// /items must expose both a GET and a POST operation.
#[test]
fn simple_fixture_items_path_has_get_and_post() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    let items = spec
        .paths
        .get("/items")
        .expect("simple.yaml must have /items");
    assert!(items.get.is_some(), "/items must have a GET (listItems)");
    assert!(items.post.is_some(), "/items must have a POST (createItem)");
}

/// /items/{id} must expose GET and DELETE but not POST.
#[test]
fn simple_fixture_item_by_id_path_has_get_and_delete() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    let by_id = spec
        .paths
        .get("/items/{id}")
        .expect("simple.yaml must have /items/{id}");

    assert!(by_id.get.is_some(), "/items/{{id}} must have GET");
    assert!(by_id.delete.is_some(), "/items/{{id}} must have DELETE");
    assert!(by_id.post.is_none(), "/items/{{id}} must NOT have POST");
}

/// The Item schema must list id and name as required.
/// In the new raw types, `required` is Vec<String> (never None).
#[test]
fn simple_fixture_item_schema_required_fields() {
    let spec = parser::load(&fixture("simple.yaml")).unwrap();
    let schemas = spec.components.as_ref().map(|c| &c.schemas).unwrap();

    let item = match schemas.get("Item").expect("Item schema must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => {
            panic!("Item must be an inline schema, got $ref: {ref_path}")
        }
    };
    // `required` is now Vec<String>, not Option<Vec<String>>.
    assert!(
        item.required.contains(&"id".to_string()),
        "Item.required must include 'id'"
    );
    assert!(
        item.required.contains(&"name".to_string()),
        "Item.required must include 'name'"
    );
}

// ─── petstore.yaml structural assertions ─────────────────────────────────────

/// Must contain all nine declared schemas.
#[test]
fn petstore_fixture_has_expected_schemas() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let schemas = spec
        .components
        .as_ref()
        .map(|c| &c.schemas)
        .expect("petstore.yaml must have components.schemas");

    for expected in [
        "Pet",
        "NewPet",
        "Category",
        "Tag",
        "PetStatus",
        "DomesticPet",
        "PetOrError",
        "ApiResponse",
        "Error",
    ] {
        assert!(
            schemas.contains_key(expected),
            "petstore.yaml is missing schema: {expected}"
        );
    }
}

/// Must expose /pets and /pets/{petId}.
#[test]
fn petstore_fixture_has_expected_paths() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let paths = &spec.paths;
    for expected in ["/pets", "/pets/{petId}"] {
        assert!(
            paths.contains_key(expected),
            "petstore.yaml is missing path: {expected}"
        );
    }
}

/// /pets/{petId} must declare GET, PUT, and DELETE.
#[test]
fn petstore_pet_by_id_path_has_all_verbs() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let by_id = spec
        .paths
        .get("/pets/{petId}")
        .expect("petstore.yaml must have /pets/{petId}");

    assert!(by_id.get.is_some(), "/pets/{{petId}} must have GET");
    assert!(by_id.put.is_some(), "/pets/{{petId}} must have PUT");
    assert!(by_id.delete.is_some(), "/pets/{{petId}} must have DELETE");
}

/// DomesticPet must use allOf composition whose first entry is $ref Pet.
/// In the new raw types, `all_of` is Vec<RawOrRef<Schema>>.
#[test]
fn petstore_domestic_pet_uses_all_of() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let schemas = spec.components.as_ref().map(|c| &c.schemas).unwrap();

    let domestic = match schemas.get("DomesticPet").expect("DomesticPet must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => {
            panic!("DomesticPet must be inline, got $ref: {ref_path}")
        }
    };
    assert!(
        !domestic.all_of.is_empty(),
        "DomesticPet must have a non-empty allOf list"
    );
    match &domestic.all_of[0] {
        RawOrRef::Ref { ref_path } => assert_eq!(
            ref_path, "#/components/schemas/Pet",
            "DomesticPet.allOf[0] must reference Pet"
        ),
        RawOrRef::Value(_) => {
            panic!("DomesticPet.allOf[0] must be a $ref, not an inline schema")
        }
    }
}

/// PetOrError must use oneOf with exactly two variants.
#[test]
fn petstore_pet_or_error_uses_one_of() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let schemas = spec.components.as_ref().map(|c| &c.schemas).unwrap();

    let schema = match schemas.get("PetOrError").expect("PetOrError must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => {
            panic!("PetOrError must be inline, got $ref: {ref_path}")
        }
    };
    assert_eq!(
        schema.one_of.len(),
        2,
        "PetOrError.oneOf must have exactly 2 variants"
    );
}

/// Pet.tags must be an array whose items is $ref Tag.
#[test]
fn petstore_pet_tags_field_uses_ref_items() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let schemas = spec.components.as_ref().map(|c| &c.schemas).unwrap();

    // Pet schema
    let pet = match schemas.get("Pet").expect("Pet must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => panic!("Pet must be inline, got $ref: {ref_path}"),
    };
    // Pet.tags property (properties is IndexMap<String, RawOrRef<Schema>>)
    let tags_schema = match pet.properties.get("tags").expect("Pet.tags must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => panic!("Pet.tags must be inline, got $ref: {ref_path}"),
    };
    // tags.items (Option<Box<RawOrRef<Schema>>>)
    let items = tags_schema
        .items
        .as_ref()
        .expect("Pet.tags must have an items schema");

    match items.as_ref() {
        RawOrRef::Ref { ref_path } => assert_eq!(
            ref_path, "#/components/schemas/Tag",
            "Pet.tags.items must $ref Tag"
        ),
        RawOrRef::Value(_) => panic!("Pet.tags.items must be a $ref, not an inline schema"),
    }
}

// ─── OpenAPI 3.1 nullable type array ─────────────────────────────────────────

/// OAS 3.1 `type: ["string", "null"]` must resolve to nullable: true.
#[test]
fn openapi31_nullable_type_array_is_parsed() {
    use openapi_php::ir::ResolvedSchema;
    let spec = openapi_php::parser::load_and_resolve(&fixture("openapi31_nullable.yaml")).unwrap();
    let item = spec.schemas.get("Item").expect("Item schema must exist");
    if let ResolvedSchema::Object(obj) = item {
        let desc_prop = obj
            .properties
            .get("description")
            .expect("description property must exist");
        assert!(
            desc_prop.nullable,
            "description should be nullable (3.1 type array)"
        );

        let score_prop = obj
            .properties
            .get("score")
            .expect("score property must exist");
        assert!(score_prop.nullable, "score should be nullable");

        let rating_prop = obj
            .properties
            .get("rating")
            .expect("rating property must exist");
        assert!(rating_prop.nullable, "rating should be nullable");

        let id_prop = obj.properties.get("id").expect("id property must exist");
        assert!(!id_prop.nullable, "id should not be nullable");

        let name_prop = obj
            .properties
            .get("name")
            .expect("name property must exist");
        assert!(!name_prop.nullable, "name should not be nullable");
    } else {
        panic!("Item should be an object schema");
    }
}

/// Pet.createdAt must carry format: date-time.
#[test]
fn petstore_pet_created_at_has_date_time_format() {
    let spec = parser::load(&fixture("petstore.yaml")).unwrap();
    let schemas = spec.components.as_ref().map(|c| &c.schemas).unwrap();

    let pet = match schemas.get("Pet").expect("Pet must exist") {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => panic!("Pet must be inline, got $ref: {ref_path}"),
    };
    let created_at = match pet
        .properties
        .get("createdAt")
        .expect("Pet.createdAt must exist")
    {
        RawOrRef::Value(s) => s,
        RawOrRef::Ref { ref_path } => {
            panic!("Pet.createdAt must be inline, got $ref: {ref_path}")
        }
    };
    assert_eq!(
        created_at.format.as_deref(),
        Some("date-time"),
        "Pet.createdAt must have format: date-time"
    );
}

// ─── Global security inheritance ─────────────────────────────────────────────

/// An operation without an explicit security field inherits the global security
/// definition and must resolve to `requires_auth: true`.
#[test]
fn global_security_inherited_when_operation_has_none() {
    let spec =
        openapi_php::parser::load_and_resolve(&fixture("global_security.yaml")).unwrap();

    let ep = spec
        .endpoints
        .iter()
        .find(|e| e.operation_id == "getSecure")
        .expect("getSecure endpoint must exist");

    assert!(
        ep.requires_auth,
        "getSecure has no operation-level security, so it must inherit the global definition (requires_auth = true)"
    );
}

/// An operation with `security: []` explicitly overrides global security and
/// must resolve to `requires_auth: false`.
#[test]
fn global_security_overridden_by_empty_operation_security() {
    let spec =
        openapi_php::parser::load_and_resolve(&fixture("global_security.yaml")).unwrap();

    let ep = spec
        .endpoints
        .iter()
        .find(|e| e.operation_id == "getPublic")
        .expect("getPublic endpoint must exist");

    assert!(
        !ep.requires_auth,
        "getPublic has security: [] which must override global security (requires_auth = false)"
    );
}

/// Verify that the raw global security field on the OpenApi struct is parsed
/// correctly (one entry keyed \"ApiKeyAuth\").
#[test]
fn global_security_field_is_parsed_on_raw_spec() {
    let spec = openapi_php::parser::load(&fixture("global_security.yaml")).unwrap();
    assert_eq!(
        spec.security.len(),
        1,
        "global_security.yaml must have exactly 1 global security requirement"
    );
    assert!(
        spec.security[0].contains_key("ApiKeyAuth"),
        "global security entry must be keyed 'ApiKeyAuth'"
    );
}
