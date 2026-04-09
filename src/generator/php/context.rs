use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, PhpPrimitive, ResolvedParam, ResolvedSchema,
    ResolvedSpec,
};
use serde::Serialize;
use std::collections::BTreeSet;

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

pub fn build_model_ctx(name: &str, schema: &ObjectSchema, namespace: &str) -> ModelCtx {
    let mut refs = BTreeSet::new();
    for (_, prop) in &schema.properties {
        collect_refs(&prop.schema, &mut refs);
    }
    let use_imports: Vec<String> = refs
        .into_iter()
        .filter(|r| r.as_str() != name)
        .collect();

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
            PropertyCtx {
                name: prop_name.clone(),
                camel: camel.clone(),
                php_type,
                required: prop.required && !prop.nullable,
                is_array,
                items_type,
                description: prop.description.clone(),
                from_array_expr: from_array_expr(prop_name, &prop.schema, nullable),
                to_array_expr: to_array_expr(prop_name, &camel, &prop.schema, nullable),
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

// ─── PHP helper functions ──────────────────────────────────────────────────

pub fn schema_to_php_type(schema: &ResolvedSchema, nullable: bool) -> String {
    let (base, schema_nullable) = match schema {
        ResolvedSchema::Primitive(p) => {
            let base = match p.php_type {
                PhpPrimitive::DateTime => "\\DateTimeImmutable",
                PhpPrimitive::String => "string",
                PhpPrimitive::Int => "int",
                PhpPrimitive::Float => "float",
                PhpPrimitive::Bool => "bool",
                PhpPrimitive::Mixed => "mixed",
            };
            (base.to_string(), p.nullable)
        }
        ResolvedSchema::Object(_) | ResolvedSchema::Array(_) => ("array".to_string(), false),
        ResolvedSchema::Enum(e) => match e.backing_type {
            EnumBackingType::String => ("string".to_string(), false),
            EnumBackingType::Int => ("int".to_string(), false),
        },
        ResolvedSchema::Union(_) => ("mixed".to_string(), false),
        ResolvedSchema::Ref(name) => (name.to_string(), false),
    };
    let is_nullable = nullable || schema_nullable;
    if is_nullable && base != "mixed" {
        format!("?{base}")
    } else {
        base
    }
}

pub fn to_camel_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap_next = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '_' || ch == '-' {
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase());
            cap_next = false;
        } else if i == 0 {
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

const PHP_RESERVED: &[&str] = &[
    "array", "list", "string", "int", "float", "bool", "null", "true", "false", "match", "fn",
    "class", "interface", "enum", "default", "return", "echo", "print", "unset", "isset", "empty",
];

pub fn escape_reserved(name: &str) -> String {
    if PHP_RESERVED.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

fn collect_refs(schema: &ResolvedSchema, refs: &mut BTreeSet<String>) {
    match schema {
        ResolvedSchema::Ref(name) => {
            refs.insert(name.to_string());
        }
        ResolvedSchema::Array(a) => collect_refs(&a.items, refs),
        ResolvedSchema::Object(o) => {
            for (_, prop) in &o.properties {
                collect_refs(&prop.schema, refs);
            }
        }
        _ => {}
    }
}

fn items_type_name(schema: &ResolvedSchema) -> String {
    match schema {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::DateTime => "\\DateTimeImmutable".to_string(),
            PhpPrimitive::String => "string".to_string(),
            PhpPrimitive::Int => "int".to_string(),
            PhpPrimitive::Float => "float".to_string(),
            PhpPrimitive::Bool => "bool".to_string(),
            PhpPrimitive::Mixed => "mixed".to_string(),
        },
        ResolvedSchema::Ref(name) => name.to_string(),
        ResolvedSchema::Object(_) => "array<string, mixed>".to_string(),
        _ => "mixed".to_string(),
    }
}

fn inner_from_array(key: &str, schema: &ResolvedSchema) -> String {
    match schema {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::Int => format!("(int) $data['{key}']"),
            PhpPrimitive::Float => format!("(float) $data['{key}']"),
            PhpPrimitive::Bool => format!("(bool) $data['{key}']"),
            PhpPrimitive::DateTime => {
                format!("new \\DateTimeImmutable($data['{key}'])")
            }
            PhpPrimitive::String | PhpPrimitive::Mixed => format!("(string) $data['{key}']"),
        },
        ResolvedSchema::Ref(name) => format!("{name}::fromArray($data['{key}'])"),
        ResolvedSchema::Array(arr) => match arr.items.as_ref() {
            ResolvedSchema::Ref(rname) => {
                format!("array_map(fn($item) => {rname}::fromArray($item), $data['{key}'])")
            }
            _ => format!("(array) $data['{key}']"),
        },
        ResolvedSchema::Object(_) => format!("(array) $data['{key}']"),
        _ => format!("$data['{key}']"),
    }
}

fn from_array_expr(key: &str, schema: &ResolvedSchema, nullable: bool) -> String {
    if nullable {
        let inner = inner_from_array(key, schema);
        format!("isset($data['{key}']) ? {inner} : null")
    } else {
        match schema {
            ResolvedSchema::Array(arr) => match arr.items.as_ref() {
                ResolvedSchema::Ref(rname) => {
                    format!(
                        "array_map(fn($item) => {rname}::fromArray($item), $data['{key}'] ?? [])"
                    )
                }
                _ => format!("(array) ($data['{key}'] ?? [])"),
            },
            _ => inner_from_array(key, schema),
        }
    }
}

fn to_array_expr(_key: &str, camel: &str, schema: &ResolvedSchema, nullable: bool) -> String {
    match schema {
        ResolvedSchema::Primitive(p) if p.php_type == PhpPrimitive::DateTime => {
            if nullable {
                format!("$this->{camel}?->format(\\DateTimeInterface::RFC3339)")
            } else {
                format!("$this->{camel}->format(\\DateTimeInterface::RFC3339)")
            }
        }
        ResolvedSchema::Ref(_) => {
            if nullable {
                format!("$this->{camel}?->toArray()")
            } else {
                format!("$this->{camel}->toArray()")
            }
        }
        ResolvedSchema::Enum(_) => {
            if nullable {
                format!("$this->{camel}?->value")
            } else {
                format!("$this->{camel}->value")
            }
        }
        _ => format!("$this->{camel}"),
    }
}

// ─── Return kind helpers (shared with client context building) ─────────────

enum ReturnKind {
    Void,
    Ref(String),
    Array,
}

fn resolve_return(response: &Option<ResolvedSchema>) -> (String, ReturnKind) {
    match response {
        None => ("void".to_string(), ReturnKind::Void),
        Some(ResolvedSchema::Ref(name)) => (name.to_string(), ReturnKind::Ref(name.to_string())),
        Some(schema) => {
            let php_type = schema_to_php_type(schema, false);
            (php_type, ReturnKind::Array)
        }
    }
}

pub fn build_path_expr(path: &str, path_params: &[ResolvedParam]) -> String {
    if path_params.is_empty() {
        return format!("'{path}'");
    }
    let fmt = path
        .split('/')
        .map(|seg| {
            if seg.starts_with('{') && seg.ends_with('}') {
                "%s".to_string()
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    let args: String = path_params
        .iter()
        .map(|p| format!("${}", p.php_name))
        .collect::<Vec<_>>()
        .join(", ");

    format!("sprintf('{fmt}', {args})")
}
