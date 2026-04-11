//! union_test.rs
//!
//! Integration tests for discriminated-union code generation.
//! Uses the `discriminated_union.yaml` fixture.

use openapi_php::config::PhpVersion;
use openapi_php::generator::{CodegenBackend, CodegenContext, LaravelPhpBackend, PlainPhpBackend};
use openapi_php::parser;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn spec() -> openapi_php::ir::ResolvedSpec {
    parser::load_and_resolve(&fixture("discriminated_union.yaml")).unwrap()
}

// ─── IR sanity checks ─────────────────────────────────────────────────────────

#[test]
fn union_variants_are_refs() {
    use openapi_php::ir::ResolvedSchema;

    let spec = spec();
    let pet = spec.schemas.get("Pet").expect("Pet schema missing");
    let ResolvedSchema::Union(u) = pet else {
        panic!("Pet should be Union, got {pet:?}");
    };
    assert_eq!(u.variants.len(), 2);
    for v in &u.variants {
        assert!(
            matches!(v, ResolvedSchema::Ref(_)),
            "union variant should be Ref, got {v:?}"
        );
    }
}

#[test]
fn discriminator_mapping_captured() {
    use openapi_php::ir::ResolvedSchema;

    let spec = spec();
    let pet = spec.schemas.get("Pet").unwrap();
    let ResolvedSchema::Union(u) = pet else { panic!() };

    assert_eq!(u.discriminator.as_deref(), Some("type"));
    assert_eq!(u.discriminator_mapping.get("dog").map(|s| s.as_str()), Some("Dog"));
    assert_eq!(u.discriminator_mapping.get("cat").map(|s| s.as_str()), Some("Cat"));
}

#[test]
fn no_mapping_union_has_empty_mapping() {
    use openapi_php::ir::ResolvedSchema;

    let spec = spec();
    let pet = spec.schemas.get("PetNoMapping").unwrap();
    let ResolvedSchema::Union(u) = pet else { panic!() };

    assert_eq!(u.discriminator.as_deref(), Some("type"));
    assert!(u.discriminator_mapping.is_empty(), "PetNoMapping should have no mapping");
}

#[test]
fn no_discriminator_union_has_none() {
    use openapi_php::ir::ResolvedSchema;

    let spec = spec();
    let pet = spec.schemas.get("PetAny").unwrap();
    let ResolvedSchema::Union(u) = pet else { panic!() };

    assert!(u.discriminator.is_none(), "PetAny should have no discriminator");
}

// ─── PlainPhpBackend output ───────────────────────────────────────────────────

#[test]
fn plain_generates_union_file_with_mapping() {
    let spec = spec();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let pet_path = PathBuf::from("Models/Pet.php");
    let content = files.get(&pet_path).expect("Models/Pet.php should be generated");

    // final class container (8.2+ uses `final readonly class`)
    assert!(
        content.contains("final readonly class Pet") || content.contains("final class Pet"),
        "should be final class"
    );
    assert!(content.contains("private function __construct"), "should have private __construct");
    assert!(content.contains("Dog|Cat $value"), "should have union property type");

    // fromArray match with mapping-derived keys
    assert!(content.contains("'dog' => new self(Dog::fromArray($data))"), "mapping key 'dog'");
    assert!(content.contains("'cat' => new self(Cat::fromArray($data))"), "mapping key 'cat'");

    // toArray delegates
    assert!(content.contains("return $this->value->toArray()"), "toArray delegate");

    // use imports
    assert!(content.contains("use App\\Test\\Models\\Dog;"), "Dog import");
    assert!(content.contains("use App\\Test\\Models\\Cat;"), "Cat import");
}

#[test]
fn plain_generates_union_file_no_mapping_uses_schema_name() {
    let spec = spec();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let path = PathBuf::from("Models/PetNoMapping.php");
    let content = files.get(&path).expect("Models/PetNoMapping.php should be generated");

    // Without mapping, OAS spec says match key = schema name as-is
    assert!(content.contains("'Dog' => new self(Dog::fromArray($data))"), "schema-name key 'Dog'");
    assert!(content.contains("'Cat' => new self(Cat::fromArray($data))"), "schema-name key 'Cat'");
}

#[test]
fn plain_does_not_generate_file_for_union_without_discriminator() {
    let spec = spec();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    assert!(
        !files.contains_key(&PathBuf::from("Models/PetAny.php")),
        "union without discriminator should produce no file"
    );
}

// ─── LaravelPhpBackend output ─────────────────────────────────────────────────

#[test]
fn laravel_generates_union_dto_no_form_request_or_resource() {
    let spec = spec();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Test",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    // Union DTO generated
    assert!(
        files.contains_key(&PathBuf::from("Models/Pet.php")),
        "Models/Pet.php should be generated"
    );
    // FormRequest and Resource should NOT be generated for union types
    assert!(
        !files.contains_key(&PathBuf::from("Http/Requests/PetRequest.php")),
        "No FormRequest for union types"
    );
    assert!(
        !files.contains_key(&PathBuf::from("Http/Resources/PetResource.php")),
        "No JsonResource for union types"
    );
}
