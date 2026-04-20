//! PHP code-generation helpers used by context builders and backends.
//!
//! This module contains functions that translate IR types into PHP code
//! fragments (type annotations, expressions, etc.).  Naming utilities
//! (`to_camel_case`, `escape_reserved`, …) live in [`crate::php_utils`] and
//! are re-exported here for convenience.

pub use crate::php_utils::{
    PHP_RESERVED, escape_reserved, sanitize_php_ident, sanitize_php_string_literal,
    sanitize_phpdoc, to_camel_case, to_pascal_case,
};

use crate::ir::{
    EnumBackingType, MapSchema, PhpPrimitive, ResolvedParam, ResolvedSchema, UnionSchema,
};
use std::collections::BTreeSet;

// ─── Nullable-ref union detection ────────────────────────────────────────

pub fn nullable_ref_name(schema: &UnionSchema) -> Option<&str> {
    let mut ref_name: Option<&str> = None;
    let mut has_null_sentinel = false;

    for variant in &schema.variants {
        match variant {
            ResolvedSchema::Ref(n) => {
                if ref_name.is_some() {
                    return None; // multiple Refs → genuine union, not nullable
                }
                ref_name = Some(n.as_ref());
            }
            ResolvedSchema::Primitive(_) => {
                has_null_sentinel = true;
            }
            _ => return None, // Object / Array / Map / Enum / nested Union → not a nullable ref
        }
    }

    if ref_name.is_some() && has_null_sentinel {
        ref_name
    } else {
        None
    }
}

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
        ResolvedSchema::Object(_) | ResolvedSchema::Array(_) | ResolvedSchema::Map(_) => {
            ("array".to_string(), false)
        }
        ResolvedSchema::Enum(e) => match e.backing_type {
            EnumBackingType::String => ("string".to_string(), false),
            EnumBackingType::Int => ("int".to_string(), false),
        },
        ResolvedSchema::Union(u) => {
            if let Some(name) = nullable_ref_name(u) {
                return format!("?{}", sanitize_php_ident(name));
            }
            ("mixed".to_string(), false)
        }
        ResolvedSchema::Ref(name) => (sanitize_php_ident(name), false),
    };
    let is_nullable = nullable || schema_nullable;
    if is_nullable && base != "mixed" {
        format!("?{base}")
    } else {
        base
    }
}

/// Returns the PHPStan `array<string, V>` representation for a [`MapSchema`].
pub fn map_phpstan_value_type(m: &MapSchema) -> String {
    let value = match m.value_type.as_ref() {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::String | PhpPrimitive::Mixed => "string".to_string(),
            PhpPrimitive::Int => "int".to_string(),
            PhpPrimitive::Float => "float".to_string(),
            PhpPrimitive::Bool => "bool".to_string(),
            PhpPrimitive::DateTime => "string".to_string(), // date-time serialised as string
        },
        ResolvedSchema::Ref(name) => format!("{}Data", sanitize_php_ident(name)),
        _ => "mixed".to_string(),
    };
    format!("array<string, {value}>")
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
        ResolvedSchema::Ref(name) => sanitize_php_ident(name),
        ResolvedSchema::Object(_) => "array<string, mixed>".to_string(),
        ResolvedSchema::Map(m) => map_phpstan_value_type(m),
        _ => "mixed".to_string(),
    }
}

// ─── fromArray / toArray expression builders ──────────────────────────────

pub fn inner_from_array(key: &str, schema: &ResolvedSchema) -> String {
    // `key` is expected to be pre-sanitized by the caller (from_array_expr).
    // The `isset($data['key'])` guard at the caller ensures presence; here we
    // only need runtime *type* validation via `TypeAssert::require*`.
    match schema {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::Int => format!("TypeAssert::requireInt($data, '{key}')"),
            PhpPrimitive::Float => format!("TypeAssert::requireFloat($data, '{key}')"),
            PhpPrimitive::Bool => format!("TypeAssert::requireBool($data, '{key}')"),
            PhpPrimitive::DateTime => {
                format!("new \\DateTimeImmutable(TypeAssert::requireString($data, '{key}'))")
            }
            PhpPrimitive::String | PhpPrimitive::Mixed => {
                format!("TypeAssert::requireString($data, '{key}')")
            }
        },
        ResolvedSchema::Ref(name) => {
            let safe = sanitize_php_ident(name);
            format!("{safe}::fromArray(TypeAssert::requireArray($data, '{key}'))")
        }
        ResolvedSchema::Array(arr) => array_from_list_expr(key, &arr.items),
        ResolvedSchema::Object(_) | ResolvedSchema::Map(_) => {
            format!("TypeAssert::requireArray($data, '{key}')")
        }
        _ => format!("$data['{key}']"),
    }
}

/// Build the `array_map(..., TypeAssert::requireList(...))` expression for a
/// list-typed property whose items have `items` schema.
fn array_from_list_expr(key: &str, items: &ResolvedSchema) -> String {
    let list = format!("TypeAssert::requireList($data, '{key}')");
    match items {
        ResolvedSchema::Ref(rname) => {
            let safe = sanitize_php_ident(rname);
            // Ref items: map each item through its fromArray, narrowing mixed→array first.
            format!(
                "array_map(fn($item) => {safe}::fromArray(is_array($item) ? $item : throw new \\UnexpectedValueException(\"Field '{key}' items must be array, got \" . get_debug_type($item))), {list})"
            )
        }
        ResolvedSchema::Primitive(p) => {
            let (predicate, type_name, transform) = primitive_item_check(p.php_type);
            if transform.is_empty() {
                // String/Int/Float/Bool/Mixed: narrow the item, return as-is.
                format!(
                    "array_map(fn($item) => {predicate}($item) ? $item : throw new \\UnexpectedValueException(\"Field '{key}' items must be {type_name}, got \" . get_debug_type($item)), {list})"
                )
            } else {
                // DateTime: narrow to string, then construct.
                format!(
                    "array_map(fn($item) => {transform}({predicate}($item) ? $item : throw new \\UnexpectedValueException(\"Field '{key}' items must be {type_name}, got \" . get_debug_type($item))), {list})"
                )
            }
        }
        _ => list,
    }
}

/// Returns `(is_predicate, php_type_name, transform_expr)` for per-item list validation.
/// `transform_expr` is empty when the narrowed value is returned directly.
fn primitive_item_check(ty: PhpPrimitive) -> (&'static str, &'static str, &'static str) {
    match ty {
        PhpPrimitive::Int => ("is_int", "int", ""),
        PhpPrimitive::Float => ("is_float", "float", ""),
        PhpPrimitive::Bool => ("is_bool", "bool", ""),
        PhpPrimitive::String | PhpPrimitive::Mixed => ("is_string", "string", ""),
        PhpPrimitive::DateTime => ("is_string", "string", "new \\DateTimeImmutable"),
    }
}

/// Build a `fromArray` expression for a **required** (non-nullable) field.
///
/// Delegates to `TypeAssert::require*` helpers that validate both presence and
/// runtime type, throwing `\UnexpectedValueException` on either mismatch. This
/// replaces earlier silent-cast behaviour (`(int) $data['x']`) so that malformed
/// input fails loudly at the DTO boundary.
///
/// Array schemas throw on missing; callers relying on "absent = empty" must
/// mark the property optional in the schema.
fn required_from_array_expr(key: &str, schema: &ResolvedSchema) -> String {
    match schema {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::Int => format!("TypeAssert::requireInt($data, '{key}')"),
            PhpPrimitive::Float => format!("TypeAssert::requireFloat($data, '{key}')"),
            PhpPrimitive::Bool => format!("TypeAssert::requireBool($data, '{key}')"),
            PhpPrimitive::DateTime => {
                format!("new \\DateTimeImmutable(TypeAssert::requireString($data, '{key}'))")
            }
            PhpPrimitive::String | PhpPrimitive::Mixed => {
                format!("TypeAssert::requireString($data, '{key}')")
            }
        },
        ResolvedSchema::Ref(name) => {
            let safe = sanitize_php_ident(name);
            format!("{safe}::fromArray(TypeAssert::requireArray($data, '{key}'))")
        }
        ResolvedSchema::Array(arr) => array_from_list_expr(key, &arr.items),
        ResolvedSchema::Object(_) | ResolvedSchema::Map(_) => {
            format!("TypeAssert::requireArray($data, '{key}')")
        }
        _ => {
            let throw =
                format!("throw new \\UnexpectedValueException(\"Missing required field '{key}'\")");
            format!("($data['{key}'] ?? {throw})")
        }
    }
}

pub fn from_array_expr(key: &str, schema: &ResolvedSchema, nullable: bool) -> String {
    // Sanitize the JSON key for use inside PHP single-quoted string literals.
    let key = &sanitize_php_string_literal(key);

    // Nullable-ref union: always use the isset pattern regardless of prop.required,
    // because the union itself declares that null is a valid value.
    if let ResolvedSchema::Union(u) = schema
        && let Some(name) = nullable_ref_name(u)
    {
        let safe = sanitize_php_ident(name);
        return format!(
            "isset($data['{key}']) ? {safe}::fromArray(TypeAssert::requireArray($data, '{key}')) : null"
        );
    }

    if nullable {
        let inner = inner_from_array(key, schema);
        format!("isset($data['{key}']) ? {inner} : null")
    } else {
        required_from_array_expr(key, schema)
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
        ResolvedSchema::Union(u) if nullable_ref_name(u).is_some() => {
            // Nullable-ref union always uses ?->toArray() because null is a valid value.
            format!("$this->{camel}?->toArray()")
        }
        ResolvedSchema::Array(arr) => match arr.items.as_ref() {
            ResolvedSchema::Ref(_) => {
                format!("array_map(fn($item) => $item->toArray(), $this->{camel})")
            }
            _ => format!("$this->{camel}"),
        },
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
        ResolvedSchema::Map(m) => collect_refs(&m.value_type, refs),
        ResolvedSchema::Object(o) => {
            for (_, prop) in &o.properties {
                collect_refs(&prop.schema, refs);
            }
        }
        ResolvedSchema::Union(u) => {
            for variant in &u.variants {
                collect_refs(variant, refs);
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
        .map(|p| format!("${}", sanitize_php_ident(&p.php_name)))
        .collect::<Vec<_>>()
        .join(", ");

    format!("sprintf('{fmt}', {args})")
}

// ─── Return kind helpers ──────────────────────────────────────────────────

pub enum ReturnKind {
    Void,
    Ref(String),
    /// Array of an untyped/primitive value (e.g. `array<string, mixed>`).
    Array,
    /// Array of a named DTO — response is `list<T>` that must be mapped via `T::fromArray`.
    ArrayOf(String),
}

pub fn resolve_return(response: &Option<ResolvedSchema>) -> (String, ReturnKind) {
    match response {
        None => ("void".to_string(), ReturnKind::Void),
        Some(ResolvedSchema::Ref(name)) => {
            let safe = sanitize_php_ident(name);
            (safe.clone(), ReturnKind::Ref(safe))
        }
        Some(ResolvedSchema::Array(arr)) => {
            if let ResolvedSchema::Ref(name) = arr.items.as_ref() {
                let safe = sanitize_php_ident(name);
                ("array".to_string(), ReturnKind::ArrayOf(safe))
            } else {
                ("array".to_string(), ReturnKind::Array)
            }
        }
        Some(schema) => {
            let php_type = schema_to_php_type(schema, false);
            (php_type, ReturnKind::Array)
        }
    }
}
