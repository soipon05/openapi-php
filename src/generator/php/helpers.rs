//! PHP code-generation helpers used by context builders and backends.
//!
//! This module contains functions that translate IR types into PHP code
//! fragments (type annotations, expressions, etc.).  Naming utilities
//! (`to_camel_case`, `escape_reserved`, …) live in [`crate::php_utils`] and
//! are re-exported here for convenience.

pub use crate::php_utils::{PHP_RESERVED, escape_reserved, to_camel_case, to_pascal_case};

use crate::ir::{EnumBackingType, PhpPrimitive, ResolvedParam, ResolvedSchema};
use std::collections::BTreeSet;

// ─── PHP type mapping ─────────────────────────────────────────────────────

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

pub fn items_type_name(schema: &ResolvedSchema) -> String {
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

// ─── fromArray / toArray expression builders ──────────────────────────────

pub fn inner_from_array(key: &str, schema: &ResolvedSchema) -> String {
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

pub fn from_array_expr(key: &str, schema: &ResolvedSchema, nullable: bool) -> String {
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

pub fn to_array_expr(_key: &str, camel: &str, schema: &ResolvedSchema, nullable: bool) -> String {
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

// ─── Ref collection ───────────────────────────────────────────────────────

pub fn collect_refs(schema: &ResolvedSchema, refs: &mut BTreeSet<String>) {
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

// ─── Path expression builder ──────────────────────────────────────────────

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

// ─── Return kind helpers ──────────────────────────────────────────────────

pub enum ReturnKind {
    Void,
    Ref(String),
    Array,
}

pub fn resolve_return(response: &Option<ResolvedSchema>) -> (String, ReturnKind) {
    match response {
        None => ("void".to_string(), ReturnKind::Void),
        Some(ResolvedSchema::Ref(name)) => (name.to_string(), ReturnKind::Ref(name.to_string())),
        Some(schema) => {
            let php_type = schema_to_php_type(schema, false);
            (php_type, ReturnKind::Array)
        }
    }
}
