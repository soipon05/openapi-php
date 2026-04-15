//! laravel_test.rs
//!
//! Integration tests for LaravelPhpBackend.
//! Uses `run_dry()` to generate PHP into memory without touching the filesystem.

use openapi_php::config::PhpVersion;
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
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
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
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
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
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
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
    assert!(
        content.contains("Route::delete("),
        "Expected Route::delete()"
    );
}

#[test]
fn laravel_still_generates_models() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
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

// ─── Controller tests ─────────────────────────────────────────────────────────

#[test]
fn laravel_generates_controller_file() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let backend = LaravelPhpBackend::new(None).unwrap();
    let files = backend.run_dry(&ctx).unwrap();

    let key = PathBuf::from("Http/Controllers/ItemController.php");
    assert!(
        files.contains_key(&key),
        "Expected Http/Controllers/ItemController.php; got: {:?}",
        files.keys().collect::<Vec<_>>()
    );
}

#[test]
fn laravel_controller_has_correct_namespace_and_class() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    assert!(content.contains("declare(strict_types=1)"));
    assert!(
        content.contains("namespace App\\Generated\\Http\\Controllers"),
        "Expected correct namespace"
    );
    assert!(
        content.contains("class ItemController"),
        "Expected class definition"
    );
    // Laravel 12+ controllers do not extend a base Controller class
    assert!(
        !content.contains("extends Controller"),
        "Laravel 12+ controllers must not extend Controller"
    );
}

#[test]
fn laravel_controller_has_index_and_destroy_returning_json_response() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    assert!(
        content.contains("public function index(): JsonResponse"),
        "Expected index() method returning JsonResponse"
    );
    assert!(
        content.contains("public function destroy(int $id): JsonResponse"),
        "Expected destroy(int $id) method"
    );
    assert!(
        content.contains("use Illuminate\\Http\\JsonResponse"),
        "Expected JsonResponse import"
    );
}

#[test]
fn laravel_controller_has_show_with_resource_return() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    assert!(
        content.contains("public function show(int $id): ItemResource"),
        "Expected show(int $id): ItemResource"
    );
    assert!(
        content.contains("use App\\Generated\\Http\\Resources\\ItemResource"),
        "Expected ItemResource import"
    );
}

#[test]
fn laravel_controller_store_has_form_request_param() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    // POST /items uses CreateItemRequest body → CreateItemRequestRequest
    assert!(
        content.contains("CreateItemRequestRequest $request"),
        "Expected FormRequest type hint in store()"
    );
    assert!(
        content.contains("use App\\Generated\\Http\\Requests\\CreateItemRequestRequest"),
        "Expected FormRequest import"
    );
}

#[test]
fn laravel_controller_has_phpdoc_comments() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    assert!(
        content.contains("@return JsonResponse"),
        "Expected @return in PHPDoc"
    );
    assert!(
        content.contains("@return ItemResource"),
        "Expected @return ItemResource in PHPDoc"
    );
    assert!(content.contains("// TODO: implement"), "Expected TODO stub");
}

#[test]
fn laravel_petstore_controller_has_all_crud_methods() {
    let spec = parser::load_and_resolve(&fixture("petstore.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/PetController.php")];

    assert!(content.contains("public function index(): JsonResponse"));
    assert!(content.contains("public function store(NewPetRequest $request): PetResource"));
    assert!(content.contains("public function show(int $petId): PetResource"));
    assert!(
        content.contains("public function update(NewPetRequest $request, int $petId): PetResource")
    );
    assert!(content.contains("public function destroy(int $petId): JsonResponse"));
}

// ─── ISSUE-5: controller must import Illuminate\Routing\Controller ────────────

#[test]
fn laravel_controller_does_not_extend_base_controller() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Generated",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("Http/Controllers/ItemController.php")];

    // Laravel 12+ does not use a base Controller class
    assert!(
        !content.contains("use Illuminate\\Routing\\Controller;"),
        "Laravel 12+ must not import Illuminate\\Routing\\Controller:\n{content}"
    );
    assert!(
        !content.contains("extends Controller"),
        "Laravel 12+ must not extend Controller:\n{content}"
    );
}

// ─── ISSUE-6: routes.php must use the provided namespace, not hardcoded App ──

#[test]
fn laravel_routes_use_provided_namespace() {
    let spec = parser::load_and_resolve(&fixture("simple.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "MyCompany\\Api",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("routes/api.php")];

    // use import must contain FQCN with provided namespace
    assert!(
        content.contains("use MyCompany\\Api\\Http\\Controllers\\ItemController;"),
        "routes/api.php must have use import with provided namespace:\n{content}"
    );
    assert!(
        !content.contains("use App\\Http\\Controllers"),
        "routes/api.php must not hardcode App\\Http\\Controllers:\n{content}"
    );
    // Route:: calls must use short class name, not FQCN inline
    assert!(
        content.contains("[ItemController::class,"),
        "routes/api.php must use short class name in Route:: calls:\n{content}"
    );
    assert!(
        !content.contains("\\MyCompany\\Api\\Http\\Controllers\\ItemController::class"),
        "routes/api.php must not inline FQCN in Route:: calls:\n{content}"
    );
}

// ─── Multi-controller: routes/api.php emits one use import per controller ─────

#[test]
fn laravel_routes_deduplicate_controller_imports() {
    // multi_resource.yaml: GET+POST /pets (→ PetController) and GET /owners (→ OwnerController)
    let spec = parser::load_and_resolve(&fixture("multi_resource.yaml")).unwrap();
    let ctx = CodegenContext {
        php_version: &PhpVersion::Php82,
        spec: &spec,
        namespace: "App\\Api",
    };
    let files = LaravelPhpBackend::new(None).unwrap().run_dry(&ctx).unwrap();
    let content = &files[&PathBuf::from("routes/api.php")];

    // Each controller class must have exactly one use import (no duplicates)
    let pet_import = "use App\\Api\\Http\\Controllers\\PetController;";
    let owner_import = "use App\\Api\\Http\\Controllers\\OwnerController;";
    assert!(
        content.contains(pet_import),
        "Expected PetController import:\n{content}"
    );
    assert!(
        content.contains(owner_import),
        "Expected OwnerController import:\n{content}"
    );
    // Imports must appear exactly once (no duplicates despite GET+POST both mapping to PetController)
    assert_eq!(
        content.matches(pet_import).count(),
        1,
        "PetController import must appear exactly once:\n{content}"
    );
    // Route:: calls use short class names
    assert!(
        content.contains("[PetController::class,"),
        "PetController routes must use short class name:\n{content}"
    );
    assert!(
        content.contains("[OwnerController::class,"),
        "OwnerController routes must use short class name:\n{content}"
    );
}
