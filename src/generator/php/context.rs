//! Template context structs and builder functions.
//!
//! Each `*Ctx` struct is `Serialize` and passed directly to a minijinja template.
//! Builder functions (`build_model_ctx`, `build_enum_ctx`, `build_client_ctx`)
//! transform IR types into these template-ready structures.

use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, ResolvedSchema, ResolvedSpec, SecuritySchemeType,
    UnionSchema,
};
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::BTreeSet;

use super::helpers::{
    ReturnKind, build_path_expr, collect_refs, escape_reserved, from_array_expr, items_type_name,
    resolve_return, sanitize_php_ident, sanitize_php_string_literal, sanitize_phpdoc,
    schema_to_php_type, to_array_expr, to_camel_case, to_pascal_case,
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
    /// PHPStan wire type for array items (used in `list<T>` shapes).
    /// For enum items this is the backing scalar (`"string"` or `"int"`),
    /// for DTO items it is `"array<string, mixed>"`,
    /// for primitive items it is the primitive type.
    /// Empty string when `is_array` is false.
    pub items_phpstan_type: String,
    pub description: Option<String>,
    /// OpenAPI `format` value for primitive properties (e.g. "uuid", "email", "uri").
    /// `None` for non-primitive properties or primitives without a declared format.
    /// `date-time` is excluded here because it already maps to `\DateTimeImmutable`.
    pub format: Option<String>,
    pub from_array_expr: String,
    pub to_array_expr: String,
}

#[derive(Debug, Serialize)]
pub struct EnumCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    /// "string" | "int"
    pub backing_type: String,
    pub variants: Vec<VariantCtx>,
}

#[derive(Debug, Serialize)]
pub struct VariantCtx {
    pub name: String,
    /// already PHP-formatted: "'active'" or "1"
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ClientCtx {
    pub namespace: String,
    pub title: String,
    pub base_url: String,
    pub needs_stream_factory: bool,
    pub model_refs: Vec<String>,
    pub endpoints: Vec<EndpointCtx>,
    pub has_exceptions: bool,
    pub auth_schemes: Vec<AuthSchemeCtx>,
    pub has_bearer_auth: bool,
    pub has_api_key_header_auth: bool,
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
    pub query_params: Vec<QueryParamCtx>,
    pub has_query_params: bool,
    pub header_params: Vec<QueryParamCtx>,
    pub has_header_params: bool,
    pub path_expr: String,
    pub has_request_body: bool,
    pub request_body_is_dto: bool,
    /// "json" | "multipart" | "binary" | "text"
    pub request_body_mode: String,
    pub return_void: bool,
    /// Model class name for `Name::fromArray(...)` return
    pub return_ref: Option<String>,
    pub return_array: bool,
    pub error_cases: Vec<ErrorCaseCtx>,
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
    /// PHP expression to construct the typed error, e.g. "Error::fromArray($body)"
    /// None when there is no schema
    pub error_expr: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryParamCtx {
    pub name: String,
    pub php_name: String,
    pub required: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthSchemeCtx {
    /// PHP property name without `$`, e.g. `"bearerToken"`
    pub prop_name: String,
    /// Constructor parameter declaration, e.g. `"private readonly ?string $bearerToken = null"`
    pub constructor_param: String,
    /// HTTP header name, e.g. `"Authorization"` or `"X-Api-Key"`
    pub header_name: String,
    /// Value prefix before the token, e.g. `"Bearer "` or `""`
    pub header_prefix: String,
}

#[derive(Debug, Serialize)]
pub struct UnionCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    /// discriminator.propertyName, e.g. "type"
    pub discriminator: String,
    /// PHP union type string, e.g. "Dog|Cat"
    pub variant_type: String,
    pub variants: Vec<UnionVariantCtx>,
    /// Other class names that need `use` imports
    pub use_imports: Vec<String>,
    /// true → emit `final readonly class` (PHP 8.2+); false → per-property `readonly`
    pub use_readonly_class: bool,
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
            let (items_type, items_phpstan_type) =
                if let ResolvedSchema::Array(arr) = &prop.schema {
                    let itype = items_type_name(&arr.items);
                    let phpstan = phpstan_items_wire_type(&arr.items, schemas);
                    (itype, phpstan)
                } else {
                    (String::new(), String::new())
                };
            let (from_expr, to_expr) = match &prop.schema {
                ResolvedSchema::Ref(ref_name) => {
                    let is_enum = schemas
                        .get(ref_name.as_ref())
                        .map(|s| matches!(s, ResolvedSchema::Enum(_)))
                        .unwrap_or(false);
                    if is_enum {
                        let from_e = if nullable {
                            format!(
                                "isset($data['{prop_name}']) ? {ref_name}::from($data['{prop_name}']) : null"
                            )
                        } else {
                            format!("{ref_name}::from($data['{prop_name}'])")
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
                    let from_e = match arr.items.as_ref() {
                        ResolvedSchema::Ref(rname) => {
                            let is_enum = schemas
                                .get(rname.as_ref())
                                .map(|s| matches!(s, ResolvedSchema::Enum(_)))
                                .unwrap_or(false);
                            if is_enum {
                                if nullable {
                                    format!(
                                        "isset($data['{prop_name}']) ? array_map(fn($item) => {rname}::from($item), $data['{prop_name}']) : null"
                                    )
                                } else {
                                    format!(
                                        "array_map(fn($item) => {rname}::from($item), $data['{prop_name}'] ?? [])"
                                    )
                                }
                            } else {
                                from_array_expr(prop_name, &prop.schema, nullable)
                            }
                        }
                        _ => from_array_expr(prop_name, &prop.schema, nullable),
                    };
                    let to_e = to_array_expr(prop_name, &camel, &prop.schema, nullable);
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
                items_phpstan_type,
                description: prop.description.as_deref().map(sanitize_phpdoc),
                format,
                from_array_expr: from_expr,
                to_array_expr: to_expr,
            }
        })
        .collect();

    let phpstan_from_shape = properties.iter().map(phpstan_from_entry).collect();
    let phpstan_to_shape = properties.iter().map(phpstan_to_entry).collect();

    ModelCtx {
        name: sanitize_php_ident(name),
        namespace: namespace.to_string(),
        description: schema.description.as_deref().map(sanitize_phpdoc),
        use_imports,
        properties,
        use_readonly_class,
        phpstan_from_shape,
        phpstan_to_shape,
    }
}

pub fn build_enum_ctx(name: &str, schema: &EnumSchema, namespace: &str) -> EnumCtx {
    let backing_type = match schema.backing_type {
        EnumBackingType::String => "string",
        EnumBackingType::Int => "int",
    };
    let variants = schema
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
            }
        })
        .collect();

    EnumCtx {
        name: sanitize_php_ident(name),
        namespace: namespace.to_string(),
        description: schema.description.as_deref().map(sanitize_phpdoc),
        backing_type: backing_type.to_string(),
        variants,
    }
}

/// Returns `None` when the union cannot be code-generated as a discriminated container:
/// - No discriminator declared, OR
/// - Any variant is not a named `$ref` (inline / primitive variants are unsupported)
pub fn build_union_ctx(
    name: &str,
    schema: &UnionSchema,
    namespace: &str,
    use_readonly_class: bool,
) -> Option<UnionCtx> {
    let disc = schema.discriminator.as_ref()?;

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

    let variants: Vec<UnionVariantCtx> = class_names
        .iter()
        .map(|class_name| {
            // If mapping is present, find the key whose value is this class name.
            // If absent, OAS spec says use the schema name as-is.
            let match_key = if schema.discriminator_mapping.is_empty() {
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
        discriminator: sanitize_php_string_literal(disc),
        variant_type,
        variants,
        use_imports,
        use_readonly_class,
    })
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

pub fn build_client_ctx(spec: &ResolvedSpec, namespace: &str) -> ClientCtx {
    let needs_stream_factory = spec.endpoints.iter().any(|ep| ep.request_body.is_some());

    let mut model_refs: Vec<String> = spec
        .endpoints
        .iter()
        .flat_map(|ep| {
            let mut refs = Vec::new();
            if let Some(ResolvedSchema::Ref(n)) = &ep.response {
                refs.push(sanitize_php_ident(n));
            }
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
            refs
        })
        .collect();
    model_refs.sort();
    model_refs.dedup();

    let endpoints = spec
        .endpoints
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
            let query_params: Vec<QueryParamCtx> = ep
                .query_params
                .iter()
                .map(|p| QueryParamCtx {
                    name: sanitize_php_string_literal(&p.name),
                    php_name: sanitize_php_ident(&p.php_name),
                    required: p.required,
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

            let (return_void, return_ref, return_array) = match &rk {
                ReturnKind::Void => (true, None, false),
                ReturnKind::Ref(n) => (false, Some(n.clone()), false),
                ReturnKind::Array => (false, None, true),
            };

            let operation_pascal = sanitize_php_ident(&to_pascal_case(&ep.operation_id));
            let error_cases: Vec<ErrorCaseCtx> = ep
                .error_responses
                .iter()
                .map(|er| {
                    let suffix = status_code_suffix(er.status_code);
                    let exception_class =
                        sanitize_php_ident(&format!("{operation_pascal}{suffix}"));
                    let error_expr = match &er.schema {
                        Some(ResolvedSchema::Ref(n)) => {
                            let is_enum = spec
                                .schemas
                                .get(n.as_ref())
                                .is_some_and(|s| matches!(s, ResolvedSchema::Enum(_)));
                            if is_enum {
                                None
                            } else {
                                let safe_n = sanitize_php_ident(n);
                                Some(format!("{safe_n}::fromArray($errorBody)"))
                            }
                        }
                        _ => None,
                    };
                    ErrorCaseCtx {
                        status_code: er.status_code,
                        exception_class,
                        error_expr,
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
                has_query_params: !ep.query_params.is_empty(),
                query_params,
                has_header_params: !ep.header_params.is_empty(),
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
                        "multipart/form-data" => "multipart",
                        "application/octet-stream" => "binary",
                        _ => "text",
                    })
                    .unwrap_or("json")
                    .to_string(),
                return_void,
                return_ref,
                return_array,
                error_cases,
            }
        })
        .collect::<Vec<_>>();

    let has_exceptions = endpoints.iter().any(|ep| !ep.error_cases.is_empty());

    let auth_schemes: Vec<AuthSchemeCtx> = spec
        .security_schemes
        .iter()
        .filter_map(|s| match &s.scheme_type {
            SecuritySchemeType::Http { scheme } if scheme == "bearer" => Some(AuthSchemeCtx {
                prop_name: "bearerToken".to_string(),
                constructor_param: "private readonly ?string $bearerToken = null".to_string(),
                header_name: "Authorization".to_string(),
                header_prefix: "Bearer ".to_string(),
            }),
            SecuritySchemeType::ApiKey { in_, name } if in_ == "header" => {
                let prop = sanitize_php_ident(&format!("{}ApiKey", to_camel_case(name)));
                Some(AuthSchemeCtx {
                    prop_name: prop.clone(),
                    constructor_param: format!("private readonly ?string ${prop} = null"),
                    header_name: sanitize_php_string_literal(name),
                    header_prefix: String::new(),
                })
            }
            _ => None,
        })
        .collect();

    let has_bearer_auth = spec.security_schemes.iter().any(
        |s| matches!(&s.scheme_type, SecuritySchemeType::Http { scheme } if scheme == "bearer"),
    );
    let has_api_key_header_auth = spec.security_schemes.iter().any(
        |s| matches!(&s.scheme_type, SecuritySchemeType::ApiKey { in_, .. } if in_ == "header"),
    );

    ClientCtx {
        namespace: namespace.to_string(),
        title: sanitize_phpdoc(&spec.title),
        base_url: sanitize_php_string_literal(&spec.base_url),
        needs_stream_factory,
        model_refs,
        endpoints,
        has_exceptions,
        auth_schemes,
        has_bearer_auth,
        has_api_key_header_auth,
    }
}

// ─── PHPStan array-shape helpers ──────────────────────────────────────────────

/// Map a PHP constructor type to its JSON/array-data equivalent for PHPStan shapes.
///
/// `\DateTimeImmutable` properties are carried as `string` in the raw array.
/// Named DTO classes (`Foo`) appear as `array<string, mixed>` in the raw array.
/// Nullable leading `?` is stripped — key optionality is expressed in the key suffix.
fn phpstan_scalar_type(php_type: &str) -> String {
    let base = php_type.trim_start_matches('?');
    match base {
        "\\DateTimeImmutable" => "string".to_string(),
        "mixed" => "mixed".to_string(),
        b if b.starts_with("array") => "array<string, mixed>".to_string(),
        // Uppercase first char → PHP class name (DTO) → array on the wire
        b if b.chars().next().is_some_and(|c| c.is_uppercase()) => {
            "array<string, mixed>".to_string()
        }
        b => b.to_string(),
    }
}

/// Build a single PHPStan shape entry for the `fromArray @param` annotation.
///
/// - Required non-nullable → `'key': type`  (key must exist, value is non-null)
/// - Optional or nullable  → `'key'?: type|null` (key may be absent or value may be null)
/// - Array properties      → `list<T>` using the items element type
fn phpstan_from_entry(p: &PropertyCtx) -> String {
    let key_required = p.required && !p.php_type.starts_with('?');
    let key_opt = if key_required { "" } else { "?" };
    let base_type = phpstan_wire_type(p);
    let full_type = if p.php_type.starts_with('?') {
        format!("{base_type}|null")
    } else {
        base_type
    };
    format!("'{}'{}: {}", p.name, key_opt, full_type)
}

/// Build a single PHPStan shape entry for the `toArray @return` annotation.
///
/// `array_filter` removes null values from the output, so:
/// - Required non-nullable → `'key': type`  (always present, always non-null)
/// - Nullable or optional  → `'key'?: type` (may be absent; if present it is non-null)
/// - Array properties      → `list<T>` using the items element type
fn phpstan_to_entry(p: &PropertyCtx) -> String {
    let always_present = p.required && !p.php_type.starts_with('?');
    let key_opt = if always_present { "" } else { "?" };
    let base_type = phpstan_wire_type(p);
    format!("'{}'{}: {}", p.name, key_opt, base_type)
}

/// Resolve the PHPStan wire type for a property.
///
/// - Array properties (`is_array=true`): `list<items_phpstan_type>`
/// - Non-array: delegate to `phpstan_scalar_type`
fn phpstan_wire_type(p: &PropertyCtx) -> String {
    if p.is_array {
        format!("list<{}>", p.items_phpstan_type)
    } else {
        phpstan_scalar_type(&p.php_type)
    }
}

/// Resolve the PHPStan wire type for array items, with full schema context.
///
/// - Enum ref    → backing scalar type (`"string"` or `"int"`)
/// - DTO ref     → `"array<string, mixed>"`
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
            _ => "array<string, mixed>".to_string(),
        },
        other => phpstan_scalar_type(&items_type_name(other)),
    }
}
