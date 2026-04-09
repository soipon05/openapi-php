use crate::parser::types::{OpenApi, Schema, SchemaType};
use anyhow::Result;
use std::path::Path;

pub fn generate(spec: &OpenApi, output: &Path, namespace: &str) -> Result<()> {
    let schemas = match spec.components.as_ref().and_then(|c| c.schemas.as_ref()) {
        Some(s) => s,
        None => return Ok(()),
    };

    let models_dir = output.join("Models");
    std::fs::create_dir_all(&models_dir)?;

    for (name, schema) in schemas {
        let php = render_model(name, schema, namespace);
        let path = models_dir.join(format!("{name}.php"));
        std::fs::write(&path, php)?;
        println!("  📄 Models/{name}.php");
    }

    Ok(())
}

fn render_model(name: &str, schema: &Schema, namespace: &str) -> String {
    let mut out = String::new();

    out.push_str("<?php\n\n");
    out.push_str(&format!("declare(strict_types=1);\n\n"));
    out.push_str(&format!("namespace {namespace}\\Models;\n\n"));

    if let Some(desc) = &schema.description {
        out.push_str(&format!("/**\n * {desc}\n */\n"));
    }

    out.push_str(&format!("final class {name}\n{{\n"));

    if let Some(props) = &schema.properties {
        let required = schema.required.as_deref().unwrap_or(&[]);

        // Constructor
        out.push_str("    public function __construct(\n");
        for (prop_name, prop_schema) in props {
            let is_required = required.contains(prop_name);
            let php_type = to_php_type(prop_schema, !is_required);
            let camel = to_camel_case(prop_name);

            if let Some(desc) = &prop_schema.description {
                out.push_str(&format!("        /** @var {php_type} {desc} */\n"));
            }

            out.push_str(&format!("        public readonly {php_type} ${camel}"));
            if !is_required {
                out.push_str(" = null");
            }
            out.push_str(",\n");
        }
        out.push_str("    ) {}\n");
    }

    out.push_str("}\n");
    out
}

pub fn to_php_type(schema: &Schema, nullable: bool) -> String {
    let base = if let Some(ref_path) = &schema.ref_path {
        ref_path.split('/').last().unwrap_or("mixed").to_string()
    } else {
        match schema.schema_type.as_ref() {
            Some(SchemaType::String) => match schema.format.as_deref() {
                Some("date") | Some("date-time") => "string".to_string(),
                _ => "string".to_string(),
            },
            Some(SchemaType::Integer) => "int".to_string(),
            Some(SchemaType::Number) => "float".to_string(),
            Some(SchemaType::Boolean) => "bool".to_string(),
            Some(SchemaType::Array) => "array".to_string(),
            Some(SchemaType::Object) => "array".to_string(),
            None => "mixed".to_string(),
        }
    };

    let is_nullable = nullable || schema.nullable.unwrap_or(false);
    if is_nullable && base != "mixed" {
        format!("?{base}")
    } else {
        base
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}
