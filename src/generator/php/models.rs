use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, PhpPrimitive, ResolvedSchema, ResolvedSpec,
};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

pub fn generate(spec: &ResolvedSpec, output: &Path, namespace: &str) -> Result<()> {
    if spec.schemas.is_empty() {
        return Ok(());
    }
    let models_dir = output.join("Models");
    std::fs::create_dir_all(&models_dir)?;

    for (name, schema) in &spec.schemas {
        let php = match schema {
            ResolvedSchema::Object(obj) => render_class(name, obj, namespace),
            ResolvedSchema::Enum(e) => render_enum(name, e, namespace),
            _ => continue,
        };
        let path = models_dir.join(format!("{name}.php"));
        std::fs::write(&path, php)?;
        println!("  📄 Models/{name}.php");
    }

    Ok(())
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
            PhpPrimitive::DateTime => format!("new \\DateTimeImmutable($data['{key}'])"),
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

fn render_class(name: &str, schema: &ObjectSchema, namespace: &str) -> String {
    let mut out = String::new();
    out.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Models;\n\n"));

    // Fix 7: collect refs and emit sorted use statements
    let mut refs = BTreeSet::new();
    for (_, prop) in &schema.properties {
        collect_refs(&prop.schema, &mut refs);
    }
    let mut emitted_uses = false;
    for r in &refs {
        if r.as_str() != name {
            out.push_str(&format!("use {namespace}\\Models\\{r};\n"));
            emitted_uses = true;
        }
    }
    if emitted_uses {
        out.push('\n');
    }

    if let Some(desc) = &schema.description {
        out.push_str(&format!("/**\n * {desc}\n */\n"));
    }

    out.push_str(&format!("final class {name}\n{{\n"));

    if !schema.properties.is_empty() {
        out.push_str("    public function __construct(\n");
        for (prop_name, prop) in &schema.properties {
            let nullable = !prop.required || prop.nullable;
            let php_type = schema_to_php_type(&prop.schema, nullable);
            let camel = escape_reserved(&to_camel_case(prop_name));

            // Fix 3 & 4: clean docblock with @var list<T> for arrays
            let is_array = matches!(prop.schema, ResolvedSchema::Array(_));
            if is_array {
                let items_type = if let ResolvedSchema::Array(arr) = &prop.schema {
                    items_type_name(&arr.items)
                } else {
                    "mixed".to_string()
                };
                if let Some(desc) = &prop.description {
                    out.push_str(&format!(
                        "        /**\n         * {desc}\n         * @var list<{items_type}>\n         */\n"
                    ));
                } else {
                    out.push_str(&format!(
                        "        /**\n         * @var list<{items_type}>\n         */\n"
                    ));
                }
            } else if let Some(desc) = &prop.description {
                out.push_str(&format!("        /**\n         * {desc}\n         */\n"));
            }

            out.push_str(&format!("        public readonly {php_type} ${camel}"));
            if !prop.required {
                out.push_str(" = null");
            }
            out.push_str(",\n");
        }
        out.push_str("    ) {}\n");

        // Fix 5: fromArray() static constructor
        out.push_str("\n    /** @param array<string, mixed> $data */\n");
        out.push_str("    public static function fromArray(array $data): self\n");
        out.push_str("    {\n");
        out.push_str("        return new self(\n");
        for (prop_name, prop) in &schema.properties {
            let nullable = !prop.required || prop.nullable;
            let camel = escape_reserved(&to_camel_case(prop_name));
            let expr = from_array_expr(prop_name, &prop.schema, nullable);
            out.push_str(&format!("            {camel}: {expr},\n"));
        }
        out.push_str("        );\n");
        out.push_str("    }\n");

        // Fix 6: toArray() method
        out.push_str("\n    /** @return array<string, mixed> */\n");
        out.push_str("    public function toArray(): array\n");
        out.push_str("    {\n");
        out.push_str("        return array_filter([\n");
        for (prop_name, prop) in &schema.properties {
            let nullable = !prop.required || prop.nullable;
            let camel = escape_reserved(&to_camel_case(prop_name));
            let expr = to_array_expr(prop_name, &camel, &prop.schema, nullable);
            out.push_str(&format!("            '{prop_name}' => {expr},\n"));
        }
        out.push_str("        ], fn($v) => $v !== null);\n");
        out.push_str("    }\n");
    }

    out.push_str("}\n");
    out
}

fn render_enum(name: &str, schema: &EnumSchema, namespace: &str) -> String {
    let mut out = String::new();
    out.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Models;\n\n"));

    if let Some(desc) = &schema.description {
        out.push_str(&format!("/**\n * {desc}\n */\n"));
    }

    let backing = match schema.backing_type {
        EnumBackingType::String => "string",
        EnumBackingType::Int => "int",
    };
    out.push_str(&format!("enum {name}: {backing}\n{{\n"));

    for variant in &schema.variants {
        let value = match schema.backing_type {
            EnumBackingType::String => format!("'{}'", variant.value),
            EnumBackingType::Int => variant.value.clone(),
        };
        out.push_str(&format!("    case {} = {};\n", variant.name, value));
    }

    out.push_str("}\n");
    out
}

pub fn schema_to_php_type(schema: &ResolvedSchema, nullable: bool) -> String {
    let (base, schema_nullable) = match schema {
        ResolvedSchema::Primitive(p) => {
            // Fix 1: DateTime → \DateTimeImmutable
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
        // Fix 2: Enum backing type determines PHP type
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
