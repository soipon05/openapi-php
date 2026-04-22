//! Template context structs and builder functions.
//!
//! Each `*Ctx` struct is `Serialize` and passed directly to a minijinja template.
//! Builder functions (`build_model_ctx`, `build_enum_ctx`, `build_client_ctx`)
//! transform IR types into these template-ready structures.

use crate::config::PhpVersion;
use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, PhpPrimitive, ResolvedSchema, ResolvedSpec,
    UnionSchema,
};
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::BTreeSet;

use super::helpers::{
    ReturnKind, build_path_expr, collect_refs, escape_reserved, from_array_expr, items_type_name,
    map_phpstan_value_type, resolve_return, sanitize_php_ident, sanitize_php_string_literal,
    sanitize_phpdoc, schema_to_php_type, to_array_expr, to_camel_case, to_pascal_case,
};

// ─── Context structs (Serialize → minijinja) ───────────────────────────────

#[derive(Debug, Serialize)]
pub struct ModelCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub use_imports: Vec<String>,
    pub properties: Vec<PropertyCtx>,
    /// true → emit `readonly final class` (PHP 8.2+); false → per-property `readonly`
    pub use_readonly_class: bool,
    /// PHPStan array-shape entries for the `fromArray(@param)` annotation.
    /// Each entry is a string like `'id'?: string|null` or `'name': string`.
    /// Empty when the model has no properties.
    pub phpstan_from_shape: Vec<String>,
    /// PHPStan array-shape entries for the `toArray(@return)` annotation.
    /// array_filter removes null values, so nullable keys are optional (`?`) but their
    /// value type is the non-null base type.
    pub phpstan_to_shape: Vec<String>,
    /// true when at least one property (or array of properties) uses `\DateTimeImmutable`.
    /// Used to emit `@throws` on `fromArray` (constructor can throw on invalid date strings).
    pub has_datetime_prop: bool,
    /// The exception class name for `@throws` on `fromArray`.
    /// `\DateMalformedStringException` for PHP 8.3+, `\Exception` for older versions.
    pub datetime_throws: String,
    /// Named PHPStan type alias, e.g. `PetData`.
    /// Emitted as `@phpstan-type PetData array{...}` before the class declaration.
    /// Empty string when the model has no properties (no shape to alias).
    pub type_alias_name: String,
    /// Cross-file DTO class names (Object refs) whose `{name}Data` alias is referenced
    /// inside this model's shape. Emitted as `@phpstan-import-type {name}Data from {name}`.
    pub phpstan_import_types: Vec<String>,
    /// true when at least one property is optional or nullable.
    /// Controls whether `toArray()` wraps output in `array_filter(..., fn($v) => $v !== null)`.
    /// When false, all keys are always present and the filter is a no-op that triggers
    /// PHPStan's `notIdentical.alwaysTrue` at level 9.
    pub has_nullable_prop: bool,
}

#[derive(Debug, Serialize)]
pub struct PropertyCtx {
    /// original JSON key (used as toArray key)
    pub name: String,
    /// camelCase PHP var name
    pub camel: String,
    pub php_type: String,
    pub required: bool,
    pub is_array: bool,
    pub items_type: String,
    /// Full PHPStan wire type for this property's value in `fromArray`/`toArray` shapes.
    ///
    /// Computed with full schema context so enum refs resolve to their backing scalar
    /// and array properties are expressed as `list<T>`. Examples:
    /// - `int`, `string`, `bool`
    /// - `string`  (for a `PetStatus` string-backed enum ref)
    /// - `CategoryData`          (for a DTO ref — uses named PHPStan type alias)
    /// - `list<string>`          (for `array<string>`)
    /// - `list<TagData>`         (for `array<SomeDto>` — uses named PHPStan type alias)
    pub phpstan_wire: String,
    pub description: Option<String>,
    /// OpenAPI `format` value for primitive properties (e.g. "uuid", "email", "uri").
    /// `None` for non-primitive properties or primitives without a declared format.
    /// `date-time` is excluded here because it already maps to `\DateTimeImmutable`.
    pub format: Option<String>,
    pub from_array_expr: String,
    pub to_array_expr: String,
    pub deprecated: bool,
}

#[derive(Debug, Serialize)]
pub struct EnumCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    /// "string" | "int"
    pub backing_type: String,
    pub variants: Vec<VariantCtx>,
    /// true when at least one variant has a label (from `x-enum-descriptions`)
    pub has_labels: bool,
}

#[derive(Debug, Serialize)]
pub struct VariantCtx {
    pub name: String,
    /// already PHP-formatted: "'active'" or "1"
    pub value: String,
    /// Human-readable label from `x-enum-descriptions`; `None` when not set.
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClientCtx {
    pub namespace: String,
    pub class_name: String,
    pub title: String,
    pub base_url: String,
    pub needs_stream_factory: bool,
    pub model_refs: Vec<String>,
    pub endpoints: Vec<EndpointCtx>,
    pub has_exceptions: bool,
    /// Model refs (`return_array_of`) whose PHPStan type alias is referenced in
    /// `@var list<FooData>` comments inside the client. Emitted as
    /// `@phpstan-import-type FooData from \Namespace\Models\Foo` on the class docblock.
    pub phpstan_import_types: Vec<String>,
    /// Set of private helper methods this client needs. The template iterates
    /// and emits only the helpers actually referenced by the endpoints, so new
    /// helper kinds (xml decode, form-urlencoded builder, ...) are added by
    /// extending `ClientHelper` — no new `needs_*` bool per helper.
    pub helpers: Vec<ClientHelper>,
}

/// Private helper methods that may be emitted onto the generated Client class.
///
/// The template uses minijinja's `in` operator (`{% if "decode_json" in helpers %}`)
/// so the `snake_case` serde rename must line up with the template's string literals.
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ClientHelper {
    /// `decodeJson()` — required when any endpoint returns a single DTO.
    DecodeJson,
    /// `decodeJsonList()` — required when any endpoint returns `list<Foo>` or a bare array.
    DecodeJsonList,
    /// `buildMultipartBody()` — required when any endpoint uses `multipart/form-data`.
    MultipartBody,
    /// `assertSuccessful()` — required when at least one endpoint has no typed
    /// error cases (the fallback path throws a plain `\RuntimeException`).
    AssertSuccessful,
}

#[derive(Debug, Serialize)]
pub struct EndpointCtx {
    pub fn_name: String,
    pub method_str: String,
    pub path: String,
    pub params_str: String,
    pub return_type: String,
    pub has_json: bool,
    pub summary: Option<String>,
    pub deprecated: bool,
    /// Scalar (non-array) query parameters — serialised via `http_build_query`.
    /// Exposed as a Vec so the template can introspect param names (e.g. for
    /// snapshot tests); the actual PHP is pre-rendered into `query_string_block`.
    pub query_params: Vec<QueryParamCtx>,
    /// Pre-rendered PHP block that assigns `$queryStr` for this endpoint.
    /// Built in Rust so the template doesn't have to reason about the
    /// `optional × bool-cast` matrix — the template just drops it verbatim.
    /// Empty string when the endpoint has no scalar query params.
    pub query_string_block: String,
    /// Array-type query parameters — serialised as repeated keys or comma-joined.
    pub array_query_params: Vec<ArrayQueryParamCtx>,
    pub header_params: Vec<QueryParamCtx>,
    pub path_expr: String,
    pub has_request_body: bool,
    pub request_body_is_dto: bool,
    /// "json" | "multipart" | "binary" | "text"
    pub request_body_mode: String,
    pub return_void: bool,
    /// Model class name for `Name::fromArray(...)` return
    pub return_ref: Option<String>,
    pub return_array: bool,
    /// Model class name when the response is `list<Name>` — drives `array_map` + `@return list<T>`
    pub return_array_of: Option<String>,
    pub error_cases: Vec<ErrorCaseCtx>,
    /// `true` when the OpenAPI operation declares at least one security requirement.
    pub requires_auth: bool,
}

#[derive(Debug, Serialize)]
pub struct ExceptionCtx {
    pub class_name: String,
    pub namespace: String,
    pub status_code: u16,
    /// PHP class name of the error model, e.g. "Error" — None if no schema
    pub error_model: Option<String>,
    /// Full use-import path, e.g. "App\\Generated\\Models\\Error"
    pub error_use: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ErrorCaseCtx {
    pub status_code: u16,
    /// short exception class name, e.g. "GetUserInfoNotFoundException"
    pub exception_class: String,
    /// Short class name of the error DTO (e.g. "Error"). When `Some`, the
    /// template narrows the response body to `{name}Data` before calling
    /// `{name}::fromArray(...)`. When `None`, the exception is constructed
    /// from the raw body string instead.
    pub error_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryParamCtx {
    pub name: String,
    pub php_name: String,
    pub required: bool,
}

#[derive(Debug, Serialize)]
pub struct ArrayQueryParamCtx {
    pub name: String,
    pub php_name: String,
    pub required: bool,
    /// true  → repeated keys: `?tags=a&tags=b`
    /// false → comma-joined:  `?tags=a,b`
    pub explode: bool,
}

#[derive(Debug, Serialize)]
pub struct UnionCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    /// true when the OpenAPI schema declares a `discriminator.propertyName`.
    /// When false, `fromArray()` tries each variant in order (catch-and-retry);
    /// when true, it dispatches on the discriminator field.
    pub has_discriminator: bool,
    /// discriminator.propertyName, e.g. "type" (empty when `has_discriminator` is false)
    pub discriminator: String,
    /// PHP union type string, e.g. "Dog|Cat"
    pub variant_type: String,
    pub variants: Vec<UnionVariantCtx>,
    /// Other class names that need `use` imports
    pub use_imports: Vec<String>,
    /// true → emit `final readonly class` (PHP 8.2+); false → per-property `readonly`
    pub use_readonly_class: bool,
    /// PHPStan `@param` type for `fromArray()`, e.g. "DogData|CatData"
    pub phpstan_param_type: String,
}

#[derive(Debug, Serialize)]
pub struct UnionVariantCtx {
    /// The discriminator value to match on, e.g. "dog" (from mapping) or "Dog" (schema name)
    pub match_key: String,
    /// PHP class name, e.g. "Dog"
    pub class_name: String,
}

// ─── Context builders ──────────────────────────────────────────────────────

pub fn build_model_ctx(
    name: &str,
    schema: &ObjectSchema,
    namespace: &str,
    schemas: &IndexMap<String, ResolvedSchema>,
    use_readonly_class: bool,
    php_version: &PhpVersion,
) -> ModelCtx {
    let mut refs = BTreeSet::new();
    for (_, prop) in &schema.properties {
        collect_refs(&prop.schema, &mut refs);
    }
    let use_imports: Vec<String> = refs
        .into_iter()
        .filter(|r| r.as_str() != name)
        .map(|r| sanitize_php_ident(&r))
        .collect();

    let properties: Vec<PropertyCtx> = schema
        .properties
        .iter()
        .map(|(prop_name, prop)| {
            let nullable = !prop.required || prop.nullable;
            let php_type = schema_to_php_type(&prop.schema, nullable);
            let camel = sanitize_php_ident(&escape_reserved(&to_camel_case(prop_name)));
            let is_array = matches!(prop.schema, ResolvedSchema::Array(_));
            let items_type = if let ResolvedSchema::Array(arr) = &prop.schema {
                items_type_name(&arr.items)
            } else {
                String::new()
            };
            let phpstan_wire = compute_phpstan_wire(&prop.schema, schemas, &php_type);
            let (from_expr, to_expr) = match &prop.schema {
                ResolvedSchema::Ref(ref_name) => {
                    let enum_backing = schemas.get(ref_name.as_ref()).and_then(|s| match s {
                        ResolvedSchema::Enum(e) => Some(e.backing_type),
                        _ => None,
                    });
                    if let Some(backing) = enum_backing {
                        let assert_call = enum_backing_require_expr(backing, prop_name);
                        let from_e = if nullable {
                            format!(
                                "isset($data['{prop_name}']) ? {ref_name}::from({assert_call}) : null"
                            )
                        } else {
                            format!("{ref_name}::from({assert_call})")
                        };
                        let to_e = if nullable {
                            format!("$this->{camel}?->value")
                        } else {
                            format!("$this->{camel}->value")
                        };
                        (from_e, to_e)
                    } else {
                        (
                            from_array_expr(prop_name, &prop.schema, nullable),
                            to_array_expr(prop_name, &camel, &prop.schema, nullable),
                        )
                    }
                }
                ResolvedSchema::Array(arr) => {
                    let (from_e, to_e) = match arr.items.as_ref() {
                        ResolvedSchema::Ref(rname) => {
                            let enum_backing = schemas.get(rname.as_ref()).and_then(|s| match s {
                                ResolvedSchema::Enum(e) => Some(e.backing_type),
                                _ => None,
                            });
                            if let Some(backing) = enum_backing {
                                let item_check = enum_backing_item_check(backing, prop_name);
                                let from_e = if nullable {
                                    format!(
                                        "isset($data['{prop_name}']) ? array_map(fn($item) => {rname}::from({item_check}), TypeAssert::requireList($data, '{prop_name}')) : null"
                                    )
                                } else {
                                    format!(
                                        "array_map(fn($item) => {rname}::from({item_check}), TypeAssert::requireList($data, '{prop_name}'))"
                                    )
                                };
                                let to_e = if nullable {
                                    format!("$this->{camel} !== null ? array_map(fn($item) => $item->value, $this->{camel}) : null")
                                } else {
                                    format!("array_map(fn($item) => $item->value, $this->{camel})")
                                };
                                (from_e, to_e)
                            } else {
                                let from_e =
                                    from_array_expr(prop_name, &prop.schema, nullable);
                                let to_e = if nullable {
                                    format!("$this->{camel} !== null ? array_map(fn($item) => $item->toArray(), $this->{camel}) : null")
                                } else {
                                    format!("array_map(fn($item) => $item->toArray(), $this->{camel})")
                                };
                                (from_e, to_e)
                            }
                        }
                        _ => (
                            from_array_expr(prop_name, &prop.schema, nullable),
                            to_array_expr(prop_name, &camel, &prop.schema, nullable),
                        ),
                    };
                    (from_e, to_e)
                }
                _ => (
                    from_array_expr(prop_name, &prop.schema, nullable),
                    to_array_expr(prop_name, &camel, &prop.schema, nullable),
                ),
            };
            // Surface the OpenAPI format for primitive properties in PHPDoc.
            // Excludes date-time (already expressed as \DateTimeImmutable in php_type).
            let format = match &prop.schema {
                ResolvedSchema::Primitive(p)
                    if p.format.as_deref() != Some("date-time")
                        && p.format.as_deref() != Some("date") =>
                {
                    p.format.clone()
                }
                _ => None,
            };

            PropertyCtx {
                name: sanitize_php_string_literal(prop_name),
                camel: camel.clone(),
                php_type,
                required: prop.required && !prop.nullable,
                is_array,
                items_type,
                phpstan_wire,
                description: prop.description.as_deref().map(sanitize_phpdoc),
                format,
                from_array_expr: from_expr,
                to_array_expr: to_expr,
                deprecated: prop.deprecated,
            }
        })
        .collect();

    // PHP deprecates (and will later reject) required parameters declared after
    // optional ones. OpenAPI `properties` preserve declaration order — which
    // authors often write with optionals before requireds — so we re-sort the
    // constructor arg list here: required first, defaulted last.
    // Stable sort preserves the author's original ordering within each group.
    // (`PropertyCtx.required` is already `prop.required && !prop.nullable`, so
    // nullable props are never in the required group — no extra check needed.)
    let mut properties = properties;
    properties.sort_by_key(|p| !p.required);

    let phpstan_from_shape: Vec<String> = properties.iter().map(phpstan_from_entry).collect();
    let phpstan_to_shape: Vec<String> = properties.iter().map(phpstan_to_entry).collect();
    let has_datetime_prop = schema
        .properties
        .values()
        .any(|prop| has_datetime_schema(&prop.schema));
    let datetime_throws = match php_version {
        PhpVersion::Php83 | PhpVersion::Php84 => "\\DateMalformedStringException".to_string(),
        _ => "\\Exception".to_string(),
    };

    let type_alias_name = if phpstan_from_shape.is_empty() {
        String::new()
    } else {
        format!("{}Data", sanitize_php_ident(name))
    };

    // Collect DTO refs referenced inside the phpstan shape (as `{name}Data`).
    // Object and Union refs both surface as `{name}Data` in shapes and need importing.
    // Enum refs resolve to their backing scalar and do not need an import.
    let phpstan_import_types: Vec<String> = use_imports
        .iter()
        .filter(|imp| {
            matches!(
                schemas.get(imp.as_str()),
                Some(ResolvedSchema::Object(_)) | Some(ResolvedSchema::Union(_))
            )
        })
        .cloned()
        .collect();

    let has_nullable_prop = properties
        .iter()
        .any(|p| !p.required || p.php_type.starts_with('?'));

    ModelCtx {
        name: sanitize_php_ident(name),
        namespace: namespace.to_string(),
        description: schema.description.as_deref().map(sanitize_phpdoc),
        use_imports,
        properties,
        use_readonly_class,
        phpstan_from_shape,
        phpstan_to_shape,
        has_datetime_prop,
        datetime_throws,
        type_alias_name,
        phpstan_import_types,
        has_nullable_prop,
    }
}

pub fn build_enum_ctx(name: &str, schema: &EnumSchema, namespace: &str) -> EnumCtx {
    let backing_type = match schema.backing_type {
        EnumBackingType::String => "string",
        EnumBackingType::Int => "int",
    };
    let variants: Vec<VariantCtx> = schema
        .variants
        .iter()
        .map(|v| {
            let value = match schema.backing_type {
                EnumBackingType::String => {
                    format!("'{}'", sanitize_php_string_literal(&v.value))
                }
                EnumBackingType::Int => {
                    // Keep only digits and a leading minus so the literal is always valid.
                    let safe: String = v
                        .value
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '-')
                        .collect();
                    if safe.is_empty() {
                        "0".to_string()
                    } else {
                        safe
                    }
                }
            };
            VariantCtx {
                name: sanitize_php_ident(&v.name),
                value,
                label: v.label.clone(),
            }
        })
        .collect();

    let has_labels = variants.iter().any(|v| v.label.is_some());

    EnumCtx {
        name: sanitize_php_ident(name),
        namespace: namespace.to_string(),
        description: schema.description.as_deref().map(sanitize_phpdoc),
        backing_type: backing_type.to_string(),
        variants,
        has_labels,
    }
}

/// Returns `None` when the union cannot be code-generated:
/// - Any variant is not a named `$ref` (inline / primitive variants are unsupported)
///
/// With a discriminator, `fromArray()` dispatches on the declared property.
/// Without one, `fromArray()` tries each variant in order and catches
/// `\UnexpectedValueException` from `TypeAssert` to fall through.
pub fn build_union_ctx(
    name: &str,
    schema: &UnionSchema,
    namespace: &str,
    use_readonly_class: bool,
) -> Option<UnionCtx> {
    // All variants must be named $ref
    let class_names: Vec<String> = schema
        .variants
        .iter()
        .filter_map(|v| {
            if let ResolvedSchema::Ref(n) = v {
                Some(sanitize_php_ident(n))
            } else {
                None
            }
        })
        .collect();
    if class_names.len() != schema.variants.len() || class_names.is_empty() {
        return None;
    }

    let has_discriminator = schema.discriminator.is_some();

    let variants: Vec<UnionVariantCtx> = class_names
        .iter()
        .map(|class_name| {
            // Match key is only meaningful when a discriminator is declared.
            // With no discriminator, the template iterates variants for try/catch.
            let match_key = if !has_discriminator {
                String::new()
            } else if schema.discriminator_mapping.is_empty() {
                sanitize_php_string_literal(class_name)
            } else {
                schema
                    .discriminator_mapping
                    .iter()
                    .find(|(_, v)| v.as_str() == class_name.as_str())
                    .map(|(k, _)| sanitize_php_string_literal(k))
                    .unwrap_or_else(|| sanitize_php_string_literal(class_name))
            };
            UnionVariantCtx {
                match_key,
                class_name: class_name.clone(),
            }
        })
        .collect();

    let variant_type = class_names.join("|");
    let phpstan_param_type = class_names
        .iter()
        .map(|cn| format!("{}Data", cn))
        .collect::<Vec<_>>()
        .join("|");

    let mut refs = BTreeSet::new();
    for cn in &class_names {
        if cn.as_str() != sanitize_php_ident(name).as_str() {
            refs.insert(cn.clone());
        }
    }
    let use_imports: Vec<String> = refs.into_iter().collect();

    Some(UnionCtx {
        name: sanitize_php_ident(name),
        namespace: namespace.to_string(),
        description: schema.description.as_deref().map(sanitize_phpdoc),
        has_discriminator,
        discriminator: schema
            .discriminator
            .as_deref()
            .map(sanitize_php_string_literal)
            .unwrap_or_default(),
        variant_type,
        variants,
        use_imports,
        use_readonly_class,
        phpstan_param_type,
    })
}

/// Render the `$queryStr = ...` PHP block for an endpoint, handling the
/// full `optional × bool-cast` matrix. Keeping this in Rust lets the template
/// collapse three near-duplicate branches to a single `{{ ep.query_string_block | safe }}`
/// and keeps the same `for qp in params` loop out of Jinja entirely.
///
/// Returns an empty string when there are no scalar query params (the caller
/// falls back to the array-param-only path).
fn build_query_string_block(
    query_params: &[QueryParamCtx],
    has_optional: bool,
    has_bool: bool,
) -> String {
    if query_params.is_empty() {
        return String::new();
    }
    let entries: String = query_params
        .iter()
        .map(|qp| format!("            '{}' => ${},\n", qp.name, qp.php_name))
        .collect();

    let mut out = String::new();
    if has_optional {
        // Filter nulls first, then optionally cast bools, then http_build_query.
        out.push_str("        $queryParams = array_filter([\n");
        out.push_str(&entries);
        out.push_str("        ], fn($v) => $v !== null);\n");
        if has_bool {
            out.push_str(
                "        $queryParams = array_map(fn($v) => is_bool($v) ? ($v ? 'true' : 'false') : $v, $queryParams);\n",
            );
        }
        out.push_str(
            "        $queryStr = count($queryParams) > 0 ? http_build_query($queryParams) : '';",
        );
    } else if has_bool {
        // All required; cast bools inline inside http_build_query's input.
        out.push_str(
            "        $queryParams = array_map(fn($v) => is_bool($v) ? ($v ? 'true' : 'false') : $v, [\n",
        );
        out.push_str(&entries);
        out.push_str("        ]);\n");
        out.push_str("        $queryStr = http_build_query($queryParams);");
    } else {
        // All required, no bool params — plain http_build_query is enough.
        out.push_str("        $queryParams = [\n");
        out.push_str(&entries);
        out.push_str("        ];\n");
        out.push_str("        $queryStr = http_build_query($queryParams);");
    }
    out
}

/// Maps a numeric HTTP status code to a PHP exception class name suffix.
fn status_code_suffix(code: u16) -> &'static str {
    match code {
        400 => "BadRequestException",
        401 => "UnauthorizedException",
        403 => "ForbiddenException",
        404 => "NotFoundException",
        409 => "ConflictException",
        422 => "UnprocessableEntityException",
        429 => "TooManyRequestsException",
        500 => "InternalServerErrorException",
        c if (400..500).contains(&c) => "ClientException",
        c if c >= 500 => "ServerException",
        _ => "Exception",
    }
}

/// Build exception context structs for all error responses across all endpoints.
pub fn build_exception_ctxs(spec: &ResolvedSpec, namespace: &str) -> Vec<ExceptionCtx> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut result = Vec::new();

    for ep in &spec.endpoints {
        let operation_pascal = to_pascal_case(&ep.operation_id);
        for er in &ep.error_responses {
            let suffix = status_code_suffix(er.status_code);
            let class_name = sanitize_php_ident(&format!("{operation_pascal}{suffix}"));

            if seen.contains(&class_name) {
                continue;
            }
            seen.insert(class_name.clone());

            let (error_model, error_use) = match &er.schema {
                Some(ResolvedSchema::Ref(n)) => {
                    let is_enum = spec
                        .schemas
                        .get(n.as_ref())
                        .is_some_and(|s| matches!(s, ResolvedSchema::Enum(_)));
                    if is_enum {
                        (None, None)
                    } else {
                        let model = sanitize_php_ident(n);
                        let use_path = format!("{namespace}\\Models\\{model}");
                        (Some(model), Some(use_path))
                    }
                }
                _ => (None, None),
            };

            result.push(ExceptionCtx {
                class_name,
                namespace: namespace.to_string(),
                status_code: er.status_code,
                error_model,
                error_use,
            });
        }
    }

    result
}

/// Filter mode for `build_client_ctx`.
pub enum TagFilter<'a> {
    /// Include all endpoints (default `ApiClient`).
    All,
    /// Include only endpoints that have the given tag.
    Tag(&'a str),
    /// Include only endpoints that have no tags at all.
    Untagged,
}

pub fn build_client_ctx(
    spec: &ResolvedSpec,
    namespace: &str,
    tag_filter: TagFilter<'_>,
) -> ClientCtx {
    // Determine class name from filter
    let class_name = match &tag_filter {
        TagFilter::All => "ApiClient".to_string(),
        TagFilter::Tag(tag) => format!("{}Client", to_pascal_case(tag)),
        TagFilter::Untagged => "DefaultClient".to_string(),
    };

    // Filter endpoints according to tag_filter
    let filtered_endpoints: Vec<_> = spec
        .endpoints
        .iter()
        .filter(|ep| match &tag_filter {
            TagFilter::All => true,
            TagFilter::Tag(tag) => ep.tags.iter().any(|t| t == *tag),
            TagFilter::Untagged => ep.tags.is_empty(),
        })
        .collect();

    let needs_stream_factory = filtered_endpoints
        .iter()
        .any(|ep| ep.request_body.is_some());

    let mut model_refs: Vec<String> = filtered_endpoints
        .iter()
        .flat_map(|ep| {
            let mut refs = Vec::new();
            // Response DTO — single ref *or* array-of-ref. Array responses
            // surface as `ResolvedSchema::Array { items: Ref(n) }` and must
            // emit the same `use` import so `Foo::fromArray(...)` resolves.
            match &ep.response {
                Some(ResolvedSchema::Ref(n)) => refs.push(sanitize_php_ident(n)),
                Some(ResolvedSchema::Array(arr)) => {
                    if let ResolvedSchema::Ref(n) = arr.items.as_ref() {
                        refs.push(sanitize_php_ident(n));
                    }
                }
                _ => {}
            }
            // Error response DTOs
            for er in &ep.error_responses {
                if let Some(ResolvedSchema::Ref(n)) = &er.schema {
                    let is_enum = spec
                        .schemas
                        .get(n.as_ref())
                        .is_some_and(|s| matches!(s, ResolvedSchema::Enum(_)));
                    if !is_enum {
                        refs.push(sanitize_php_ident(n));
                    }
                }
            }
            // Request body DTO
            if let Some(ResolvedSchema::Ref(n)) = ep.request_body.as_ref().map(|rb| &rb.schema) {
                let is_enum = spec
                    .schemas
                    .get(n.as_ref())
                    .is_some_and(|s| matches!(s, ResolvedSchema::Enum(_)));
                if !is_enum {
                    refs.push(sanitize_php_ident(n));
                }
            }
            refs
        })
        .collect();
    model_refs.sort();
    model_refs.dedup();

    let endpoints = filtered_endpoints
        .iter()
        .map(|ep| {
            let fn_name = sanitize_php_ident(&escape_reserved(&to_camel_case(&ep.operation_id)));
            let method_str = ep.method.as_str().to_string();

            let mut params: Vec<String> = Vec::new();
            for p in &ep.path_params {
                let t = schema_to_php_type(&p.schema, !p.required);
                params.push(format!("{t} ${}", sanitize_php_ident(&p.php_name)));
            }
            for p in &ep.query_params {
                let t = schema_to_php_type(&p.schema, !p.required);
                params.push(format!("{t} ${}", sanitize_php_ident(&p.php_name)));
            }
            for p in &ep.header_params {
                let t = schema_to_php_type(&p.schema, !p.required);
                params.push(format!("{t} ${}", sanitize_php_ident(&p.php_name)));
            }
            if let Some(rb) = &ep.request_body {
                let t = schema_to_php_type(&rb.schema, !rb.required);
                params.push(format!("{t} $body"));
            }

            let (return_type, rk) = resolve_return(&ep.response);
            let has_json = !matches!(rk, ReturnKind::Void) || ep.request_body.is_some();
            let path_expr = build_path_expr(&ep.path, &ep.path_params);
            // Split query params into scalar vs array-type based on whether explode is set.
            let query_params: Vec<QueryParamCtx> = ep
                .query_params
                .iter()
                .filter(|p| p.explode.is_none()) // scalar params
                .map(|p| QueryParamCtx {
                    name: sanitize_php_string_literal(&p.name),
                    php_name: sanitize_php_ident(&p.php_name),
                    required: p.required,
                })
                .collect();

            let has_bool_query_param = ep.query_params.iter().any(|p| {
                matches!(
                    &p.schema,
                    ResolvedSchema::Primitive(prim) if prim.php_type == PhpPrimitive::Bool
                )
            });
            let has_optional_query_params = query_params.iter().any(|p| !p.required);
            let query_string_block = build_query_string_block(
                &query_params,
                has_optional_query_params,
                has_bool_query_param,
            );

            let array_query_params: Vec<ArrayQueryParamCtx> = ep
                .query_params
                .iter()
                .filter_map(|p| {
                    p.explode.map(|explode| ArrayQueryParamCtx {
                        name: sanitize_php_string_literal(&p.name),
                        php_name: sanitize_php_ident(&p.php_name),
                        required: p.required,
                        explode,
                    })
                })
                .collect();

            let header_params: Vec<QueryParamCtx> = ep
                .header_params
                .iter()
                .map(|p| QueryParamCtx {
                    name: sanitize_php_string_literal(&p.name),
                    php_name: sanitize_php_ident(&p.php_name),
                    required: p.required,
                })
                .collect();

            let (return_void, return_ref, return_array, return_array_of) = match &rk {
                ReturnKind::Void => (true, None, false, None),
                ReturnKind::Ref(n) => (false, Some(n.clone()), false, None),
                ReturnKind::Array => (false, None, true, None),
                ReturnKind::ArrayOf(n) => (false, None, false, Some(n.clone())),
            };

            let operation_pascal = sanitize_php_ident(&to_pascal_case(&ep.operation_id));
            let error_cases: Vec<ErrorCaseCtx> = ep
                .error_responses
                .iter()
                .map(|er| {
                    let suffix = status_code_suffix(er.status_code);
                    let exception_class =
                        sanitize_php_ident(&format!("{operation_pascal}{suffix}"));
                    let error_model = match &er.schema {
                        Some(ResolvedSchema::Ref(n)) => {
                            let is_enum = spec
                                .schemas
                                .get(n.as_ref())
                                .is_some_and(|s| matches!(s, ResolvedSchema::Enum(_)));
                            if is_enum {
                                None
                            } else {
                                Some(sanitize_php_ident(n))
                            }
                        }
                        _ => None,
                    };
                    ErrorCaseCtx {
                        status_code: er.status_code,
                        exception_class,
                        error_model,
                    }
                })
                .collect();

            EndpointCtx {
                fn_name,
                method_str,
                path: sanitize_php_string_literal(&ep.path),
                params_str: params.join(", "),
                return_type,
                has_json,
                summary: ep.summary.as_deref().map(sanitize_phpdoc),
                deprecated: ep.deprecated,
                query_string_block,
                query_params,
                array_query_params,
                header_params,
                path_expr,
                has_request_body: ep.request_body.is_some(),
                request_body_is_dto: matches!(
                    ep.request_body.as_ref().map(|rb| &rb.schema),
                    Some(ResolvedSchema::Ref(_))
                ),
                request_body_mode: ep
                    .request_body
                    .as_ref()
                    .map(|rb| match rb.content_type.as_str() {
                        "application/json" => "json",
                        "application/x-www-form-urlencoded" => "form",
                        "multipart/form-data" => "multipart",
                        "application/octet-stream" => "binary",
                        _ => "text",
                    })
                    .unwrap_or("json")
                    .to_string(),
                return_void,
                return_ref,
                return_array,
                return_array_of,
                error_cases,
                requires_auth: ep.requires_auth,
            }
        })
        .collect::<Vec<_>>();

    let has_exceptions = endpoints.iter().any(|ep| !ep.error_cases.is_empty());
    let mut helpers: Vec<ClientHelper> = Vec::new();
    if endpoints.iter().any(|ep| ep.return_ref.is_some()) {
        helpers.push(ClientHelper::DecodeJson);
    }
    if endpoints
        .iter()
        .any(|ep| ep.return_array_of.is_some() || ep.return_array)
    {
        helpers.push(ClientHelper::DecodeJsonList);
    }
    if endpoints
        .iter()
        .any(|ep| ep.request_body_mode == "multipart")
    {
        helpers.push(ClientHelper::MultipartBody);
    }
    // `assertSuccessful()` is the fallback for endpoints without typed error cases.
    // Skip emitting it entirely if every endpoint declares error cases (the typed
    // dispatch already throws on every non-2xx path).
    if endpoints.iter().any(|ep| ep.error_cases.is_empty()) {
        helpers.push(ClientHelper::AssertSuccessful);
    }

    // Models whose PHPStan `{Name}Data` alias is used by the client: narrowed
    // with `/** @var FooData */` for single DTOs and `/** @var list<FooData> */`
    // for list endpoints. Error-response DTOs that appear as `Foo::fromArray(...)`
    // in the typed-error dispatch also need the alias imported.
    let mut phpstan_import_types: Vec<String> = Vec::new();
    for ep in &endpoints {
        if let Some(r) = &ep.return_ref {
            phpstan_import_types.push(r.clone());
        }
        if let Some(r) = &ep.return_array_of {
            phpstan_import_types.push(r.clone());
        }
        for ec in &ep.error_cases {
            if let Some(m) = &ec.error_model {
                phpstan_import_types.push(m.clone());
            }
        }
    }
    phpstan_import_types.sort();
    phpstan_import_types.dedup();

    ClientCtx {
        namespace: namespace.to_string(),
        class_name,
        title: sanitize_phpdoc(&spec.title),
        base_url: sanitize_php_string_literal(&spec.base_url),
        needs_stream_factory,
        model_refs,
        endpoints,
        has_exceptions,
        phpstan_import_types,
        helpers,
    }
}

// ─── PHPStan array-shape helpers ──────────────────────────────────────────────

/// Runtime-assert call for a top-level enum property keyed by `prop_name`.
/// Returns a string like `TypeAssert::requireString($data, 'status')`.
fn enum_backing_require_expr(backing: EnumBackingType, prop_name: &str) -> String {
    let method = match backing {
        EnumBackingType::String => "requireString",
        EnumBackingType::Int => "requireInt",
    };
    format!("TypeAssert::{method}($data, '{prop_name}')")
}

/// Per-item runtime check inside an `array_map` lambda for enum arrays.
/// The enum's `::from()` would otherwise accept unvalidated `mixed` items.
fn enum_backing_item_check(backing: EnumBackingType, prop_name: &str) -> String {
    let (predicate, type_name) = match backing {
        EnumBackingType::String => ("is_string", "string"),
        EnumBackingType::Int => ("is_int", "int"),
    };
    format!(
        "{predicate}($item) ? $item : throw new \\UnexpectedValueException(\"Field '{prop_name}' items must be {type_name}, got \" . get_debug_type($item))"
    )
}

/// Returns true if the schema is (or contains) a date-time primitive.
/// Used to determine whether `fromArray` can throw on date parsing.
fn has_datetime_schema(schema: &ResolvedSchema) -> bool {
    match schema {
        ResolvedSchema::Primitive(p) => p.php_type == PhpPrimitive::DateTime,
        ResolvedSchema::Array(arr) => has_datetime_schema(&arr.items),
        _ => false,
    }
}

/// Map a PHP constructor type to its JSON/array-data equivalent for PHPStan shapes.
///
/// `\DateTimeImmutable` properties are carried as `string` in the raw array.
/// Nullable leading `?` is stripped — key optionality is expressed in the key suffix.
/// DTO class names (`Foo`) map to their named PHPStan type alias (`FooData`).
fn phpstan_scalar_type(php_type: &str) -> String {
    let base = php_type.trim_start_matches('?');
    match base {
        "\\DateTimeImmutable" => "string".to_string(),
        "mixed" => "mixed".to_string(),
        b if b.starts_with("array") => "array<string, mixed>".to_string(),
        // Uppercase first char → PHP class name (DTO) → use named PHPStan type alias
        b if b.chars().next().is_some_and(|c| c.is_uppercase()) => {
            format!("{}Data", b)
        }
        b => b.to_string(),
    }
}

/// Build a single PHPStan shape entry for the `fromArray @param` annotation.
///
/// - Required non-nullable → `'key': type`
/// - Optional or nullable  → `'key'?: type|null`
fn phpstan_from_entry(p: &PropertyCtx) -> String {
    let key_required = p.required && !p.php_type.starts_with('?');
    let key_opt = if key_required { "" } else { "?" };
    let full_type = if p.php_type.starts_with('?') {
        format!("{}|null", p.phpstan_wire)
    } else {
        p.phpstan_wire.clone()
    };
    format!("'{}'{}: {}", p.name, key_opt, full_type)
}

/// Build a single PHPStan shape entry for the `toArray @return` annotation.
///
/// `array_filter` removes null values, so nullable keys are optional (`?`)
/// but their value type is always non-null.
fn phpstan_to_entry(p: &PropertyCtx) -> String {
    let always_present = p.required && !p.php_type.starts_with('?');
    let key_opt = if always_present { "" } else { "?" };
    format!("'{}'{}: {}", p.name, key_opt, p.phpstan_wire)
}

/// Compute the full PHPStan wire type for a property given complete schema context.
///
/// This is stored in `PropertyCtx.phpstan_wire` and used directly in shape entries.
///
/// | Schema kind       | Result                   |
/// |-------------------|--------------------------|
/// | Array of enum     | `list<string\|int>`      |
/// | Array of DTO      | `list<FooData>`          |
/// | Array of prim     | `list<string>` etc.      |
/// | Ref → enum        | backing scalar           |
/// | Ref → DTO         | `FooData`                |
/// | Primitive/DateTime| scalar / `string`        |
fn compute_phpstan_wire(
    schema: &ResolvedSchema,
    schemas: &IndexMap<String, ResolvedSchema>,
    php_type: &str,
) -> String {
    match schema {
        ResolvedSchema::Array(arr) => {
            let item_wire = phpstan_items_wire_type(&arr.items, schemas);
            format!("list<{item_wire}>")
        }
        ResolvedSchema::Ref(name) => match schemas.get(name.as_ref()) {
            Some(ResolvedSchema::Enum(e)) => match e.backing_type {
                EnumBackingType::String => "string".to_string(),
                EnumBackingType::Int => "int".to_string(),
            },
            Some(ResolvedSchema::Object(_)) => format!("{}Data", name),
            _ => "array<string, mixed>".to_string(),
        },
        ResolvedSchema::Map(m) => map_phpstan_value_type(m),
        _ => phpstan_scalar_type(php_type),
    }
}

/// Resolve the PHPStan wire type for array items, with full schema context.
///
/// - Enum ref    → backing scalar type (`"string"` or `"int"`)
/// - DTO ref     → named PHPStan type alias (`"FooData"`)
/// - Primitive   → the primitive type (via `phpstan_scalar_type`)
fn phpstan_items_wire_type(
    items: &ResolvedSchema,
    schemas: &IndexMap<String, ResolvedSchema>,
) -> String {
    match items {
        ResolvedSchema::Ref(name) => match schemas.get(name.as_ref()) {
            Some(ResolvedSchema::Enum(e)) => match e.backing_type {
                EnumBackingType::String => "string".to_string(),
                EnumBackingType::Int => "int".to_string(),
            },
            Some(ResolvedSchema::Object(_)) => format!("{}Data", name),
            _ => "array<string, mixed>".to_string(),
        },
        other => phpstan_scalar_type(&items_type_name(other)),
    }
}
