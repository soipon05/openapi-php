use openapi_php::config::PhpVersion;
use openapi_php::generator::{CodegenBackend, CodegenContext, PlainPhpBackend};
use openapi_php::parser;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn spec() -> openapi_php::ir::ResolvedSpec {
    parser::load_and_resolve(&fixture("nullable_ref_union.yaml")).unwrap()
}

// ─── IR checks ───────────────────────────────────────────────────────────────

#[test]
fn nullable_ref_union_resolves_as_union() {
    use openapi_php::ir::ResolvedSchema;

    let spec = spec();
    let item = spec.schemas.get("Item").unwrap();
    let ResolvedSchema::Object(obj) = item else { panic!("Item should be Object") };

    let category_prop = &obj.properties["category"];
    assert!(
        matches!(&category_prop.schema, ResolvedSchema::Union(_)),
        "category property should be Union, got {:?}",
        category_prop.schema
    );
}

// ─── Generated PHP checks ────────────────────────────────────────────────────

#[test]
fn nullable_ref_union_property_gets_question_mark_type() {
    let spec = spec();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Test",
        php_version: &PhpVersion::Php82,
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let content = files.get(&PathBuf::from("Models/Item.php"))
        .expect("Models/Item.php should be generated");

    // Property type should be ?Category, not mixed
    assert!(
        content.contains("?Category $category"),
        "category should have ?Category type, got:\n{content}"
    );
    assert!(
        !content.contains("mixed $category"),
        "category should NOT be mixed"
    );
}

#[test]
fn nullable_ref_union_from_array_uses_isset() {
    let spec = spec();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Test",
        php_version: &PhpVersion::Php82,
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let content = files.get(&PathBuf::from("Models/Item.php")).unwrap();

    // fromArray should use isset pattern for nullable ref
    assert!(
        content.contains("isset($data['category']) ? Category::fromArray($data['category']) : null"),
        "fromArray should use isset pattern:\n{content}"
    );
}

#[test]
fn nullable_ref_union_to_array_uses_nullsafe() {
    let spec = spec();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Test",
        php_version: &PhpVersion::Php82,
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let content = files.get(&PathBuf::from("Models/Item.php")).unwrap();

    // toArray should use null-safe operator
    assert!(
        content.contains("$this->category?->toArray()"),
        "toArray should use ?->toArray():\n{content}"
    );
}

#[test]
fn nullable_ref_union_use_import_generated() {
    let spec = spec();
    let ctx = CodegenContext {
        spec: &spec,
        namespace: "App\\Test",
        php_version: &PhpVersion::Php82,
    };
    let backend = PlainPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let content = files.get(&PathBuf::from("Models/Item.php")).unwrap();

    // Category must be imported
    assert!(
        content.contains("use App\\Test\\Models\\Category;"),
        "Category import missing:\n{content}"
    );
}
