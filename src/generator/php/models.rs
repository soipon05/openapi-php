use crate::ir::{
    EnumBackingType, EnumSchema, ObjectSchema, PhpPrimitive, ResolvedSchema, ResolvedSpec,
};
use anyhow::Result;
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

fn render_class(name: &str, schema: &ObjectSchema, namespace: &str) -> String {
    let mut out = String::new();
    out.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
    out.push_str(&format!("namespace {namespace}\\Models;\n\n"));

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

            if let Some(desc) = &prop.description {
                out.push_str(&format!("        /** @var {php_type} {desc} */\n"));
            }
            out.push_str(&format!("        public readonly {php_type} ${camel}"));
            if !prop.required {
                out.push_str(" = null");
            }
            out.push_str(",\n");
        }
        out.push_str("    ) {}\n");
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
            let base = match p.php_type {
                PhpPrimitive::String | PhpPrimitive::DateTime => "string",
                PhpPrimitive::Int => "int",
                PhpPrimitive::Float => "float",
                PhpPrimitive::Bool => "bool",
                PhpPrimitive::Mixed => "mixed",
            };
            (base.to_string(), p.nullable)
        }
        ResolvedSchema::Object(_) | ResolvedSchema::Array(_) => ("array".to_string(), false),
        ResolvedSchema::Enum(_) => ("string".to_string(), false),
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
