//! Template context structs and builder functions.
//!
//! Each `*Ctx` struct is `Serialize` and passed directly to a minijinja template.
//! Builder functions (`build_model_ctx`, `build_enum_ctx`, `build_client_ctx`)
//! transform IR types into these template-ready structures.

use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, ResolvedSchema, ResolvedSpec, UnionSchema,
};
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::BTreeSet;

use super::helpers::{
    ReturnKind, build_path_expr, collect_refs, escape_reserved, from_array_expr, items_type_name,
    resolve_return, schema_to_php_type, to_array_expr, to_camel_case, to_pascal_case,
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
    pub description: Option<String>,
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
    pub path_expr: String,
    pub has_request_body: bool,
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
    let use_imports: Vec<String> = refs.into_iter().filter(|r| r.as_str() != name).collect();

    let properties = schema
        .properties
        .iter()
        .map(|(prop_name, prop)| {
            let nullable = !prop.required || prop.nullable;
            let php_type = schema_to_php_type(&prop.schema, nullable);
            let camel = escape_reserved(&to_camel_case(prop_name));
            let is_array = matches!(prop.schema, ResolvedSchema::Array(_));
            let items_type = if let ResolvedSchema::Array(arr) = &prop.schema {
                items_type_name(&arr.items)
            } else {
                String::new()
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
            PropertyCtx {
                name: prop_name.clone(),
                camel: camel.clone(),
                php_type,
                required: prop.required && !prop.nullable,
                is_array,
                items_type,
                description: prop.description.clone(),
                from_array_expr: from_expr,
                to_array_expr: to_expr,
            }
        })
        .collect();

    ModelCtx {
        name: name.to_string(),
        namespace: namespace.to_string(),
        description: schema.description.clone(),
        use_imports,
        properties,
        use_readonly_class,
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
                EnumBackingType::String => format!("'{}'", v.value),
                EnumBackingType::Int => v.value.clone(),
            };
            VariantCtx {
                name: v.name.clone(),
                value,
            }
        })
        .collect();

    EnumCtx {
        name: name.to_string(),
        namespace: namespace.to_string(),
        description: schema.description.clone(),
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
                Some(n.to_string())
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
                class_name.clone()
            } else {
                schema
                    .discriminator_mapping
                    .iter()
                    .find(|(_, v)| v.as_str() == class_name.as_str())
                    .map(|(k, _)| k.clone())
                    .unwrap_or_else(|| class_name.clone())
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
        if cn != name {
            refs.insert(cn.clone());
        }
    }
    let use_imports: Vec<String> = refs.into_iter().collect();

    Some(UnionCtx {
        name: name.to_string(),
        namespace: namespace.to_string(),
        description: schema.description.clone(),
        discriminator: disc.clone(),
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
            let class_name = format!("{operation_pascal}{suffix}");

            if seen.contains(&class_name) {
                continue;
            }
            seen.insert(class_name.clone());

            let (error_model, error_use) = match &er.schema {
                Some(ResolvedSchema::Ref(n)) => {
                    let model = n.to_string();
                    let use_path = format!("{namespace}\\Models\\{model}");
                    (Some(model), Some(use_path))
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
                refs.push(n.to_string());
            }
            for er in &ep.error_responses {
                if let Some(ResolvedSchema::Ref(n)) = &er.schema {
                    refs.push(n.to_string());
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
            let fn_name = escape_reserved(&to_camel_case(&ep.operation_id));
            let method_str = ep.method.as_str().to_string();

            let mut params: Vec<String> = Vec::new();
            for p in &ep.path_params {
                let t = schema_to_php_type(&p.schema, !p.required);
                params.push(format!("{t} ${}", p.php_name));
            }
            for p in &ep.query_params {
                let t = schema_to_php_type(&p.schema, !p.required);
                params.push(format!("{t} ${}", p.php_name));
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
                    name: p.name.clone(),
                    php_name: p.php_name.clone(),
                })
                .collect();

            let (return_void, return_ref, return_array) = match &rk {
                ReturnKind::Void => (true, None, false),
                ReturnKind::Ref(n) => (false, Some(n.clone()), false),
                ReturnKind::Array => (false, None, true),
            };

            let operation_pascal = to_pascal_case(&ep.operation_id);
            let error_cases: Vec<ErrorCaseCtx> = ep
                .error_responses
                .iter()
                .map(|er| {
                    let suffix = status_code_suffix(er.status_code);
                    let exception_class = format!("{operation_pascal}{suffix}");
                    let error_expr = match &er.schema {
                        Some(ResolvedSchema::Ref(n)) => {
                            Some(format!("{n}::fromArray($body)"))
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
                path: ep.path.clone(),
                params_str: params.join(", "),
                return_type,
                has_json,
                summary: ep.summary.clone(),
                deprecated: ep.deprecated,
                has_query_params: !ep.query_params.is_empty(),
                query_params,
                path_expr,
                has_request_body: ep.request_body.is_some(),
                return_void,
                return_ref,
                return_array,
                error_cases,
            }
        })
        .collect::<Vec<_>>();

    let has_exceptions = endpoints.iter().any(|ep| !ep.error_cases.is_empty());

    ClientCtx {
        namespace: namespace.to_string(),
        title: spec.title.clone(),
        base_url: spec.base_url.clone(),
        needs_stream_factory,
        model_refs,
        endpoints,
        has_exceptions,
    }
}
