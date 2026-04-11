//! Template context structs and builder functions.
//!
//! Each `*Ctx` struct is `Serialize` and passed directly to a minijinja template.
//! Builder functions (`build_model_ctx`, `build_enum_ctx`, `build_client_ctx`)
//! transform IR types into these template-ready structures.

use crate::ir::{EnumBackingType, EnumSchema, ObjectSchema, ResolvedSchema, ResolvedSpec};
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::BTreeSet;

use super::helpers::{
    ReturnKind, build_path_expr, collect_refs, escape_reserved, from_array_expr, items_type_name,
    resolve_return, schema_to_php_type, to_array_expr, to_camel_case,
};

// ─── Context structs (Serialize → minijinja) ───────────────────────────────

#[derive(Debug, Serialize)]
pub struct ModelCtx {
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub use_imports: Vec<String>,
    pub properties: Vec<PropertyCtx>,
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
}

#[derive(Debug, Serialize)]
pub struct QueryParamCtx {
    pub name: String,
    pub php_name: String,
}

// ─── Context builders ──────────────────────────────────────────────────────

pub fn build_model_ctx(
    name: &str,
    schema: &ObjectSchema,
    namespace: &str,
    schemas: &IndexMap<String, ResolvedSchema>,
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

pub fn build_client_ctx(spec: &ResolvedSpec, namespace: &str) -> ClientCtx {
    let needs_stream_factory = spec.endpoints.iter().any(|ep| ep.request_body.is_some());

    let mut model_refs: Vec<String> = spec
        .endpoints
        .iter()
        .filter_map(|ep| {
            if let Some(ResolvedSchema::Ref(n)) = &ep.response {
                Some(n.to_string())
            } else {
                None
            }
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
            }
        })
        .collect();

    ClientCtx {
        namespace: namespace.to_string(),
        title: spec.title.clone(),
        base_url: spec.base_url.clone(),
        needs_stream_factory,
        model_refs,
        endpoints,
    }
}
