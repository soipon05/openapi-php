use crate::generator::backend::{CodegenBackend, CodegenContext, RenderedFile};
use crate::generator::php::context::{build_enum_ctx, build_model_ctx, to_camel_case};
use crate::ir::{
    EnumBackingType, HttpMethod, ObjectSchema, PhpPrimitive, ResolvedParam, ResolvedSchema,
    ResolvedSpec,
};
use anyhow::Result;
use indexmap::IndexMap;
use minijinja::{Environment, Value};
use serde::Serialize;
use std::path::PathBuf;

// ─── Context structs ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct FormRequestCtx {
    namespace: String,
    name: String,
    fields: Vec<FormRequestFieldCtx>,
}

#[derive(Debug, Serialize)]
struct FormRequestFieldCtx {
    name: String,
    rules_str: String,
}

#[derive(Debug, Serialize)]
struct ResourceCtx {
    namespace: String,
    name: String,
    fields: Vec<ResourceFieldCtx>,
}

#[derive(Debug, Serialize)]
struct ResourceFieldCtx {
    key: String,
    expr: String,
}

#[derive(Debug, Serialize)]
struct RoutesCtx {
    routes: Vec<RouteCtx>,
}

#[derive(Debug, Serialize)]
struct RouteCtx {
    method: String,
    path: String,
    controller: String,
    action: String,
    comment: String,
}

// ─── LaravelPhpBackend ────────────────────────────────────────────────────────

/// Generates Laravel-specific PHP code: DTOs, FormRequests, JsonResources, routes.
pub struct LaravelPhpBackend {
    env: Environment<'static>,
}

impl LaravelPhpBackend {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.set_trim_blocks(true);
        env.set_lstrip_blocks(true);
        env.add_template("model", include_str!("../templates/php/model.php.j2"))
            .expect("model template is valid");
        env.add_template("enum", include_str!("../templates/php/enum.php.j2"))
            .expect("enum template is valid");
        env.add_template(
            "form_request",
            include_str!("../templates/php/laravel/form_request.php.j2"),
        )
        .expect("form_request template is valid");
        env.add_template(
            "resource",
            include_str!("../templates/php/laravel/resource.php.j2"),
        )
        .expect("resource template is valid");
        env.add_template(
            "routes",
            include_str!("../templates/php/laravel/routes.php.j2"),
        )
        .expect("routes template is valid");
        Self { env }
    }
}

impl Default for LaravelPhpBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CodegenBackend for LaravelPhpBackend {
    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>> {
        let mut files: Vec<RenderedFile> = Vec::new();

        for (name, schema) in &ctx.spec.schemas {
            match schema {
                ResolvedSchema::Object(obj) => {
                    // DTO — reuse existing model template
                    let model_ctx = build_model_ctx(name, obj, ctx.namespace);
                    let content = self
                        .env
                        .get_template("model")?
                        .render(Value::from_serialize(&model_ctx))?;
                    files.push(RenderedFile {
                        rel_path: PathBuf::from(format!("Models/{name}.php")),
                        content,
                    });

                    // FormRequest
                    let req_ctx =
                        build_form_request_ctx(name, obj, ctx.namespace, &ctx.spec.schemas);
                    let content = self
                        .env
                        .get_template("form_request")?
                        .render(Value::from_serialize(&req_ctx))?;
                    files.push(RenderedFile {
                        rel_path: PathBuf::from(format!("Http/Requests/{name}Request.php")),
                        content,
                    });

                    // JsonResource
                    let res_ctx =
                        build_resource_ctx(name, obj, ctx.namespace, &ctx.spec.schemas);
                    let content = self
                        .env
                        .get_template("resource")?
                        .render(Value::from_serialize(&res_ctx))?;
                    files.push(RenderedFile {
                        rel_path: PathBuf::from(format!("Http/Resources/{name}Resource.php")),
                        content,
                    });
                }
                ResolvedSchema::Enum(e) => {
                    // Enum DTO — reuse existing enum template
                    let enum_ctx = build_enum_ctx(name, e, ctx.namespace);
                    let content = self
                        .env
                        .get_template("enum")?
                        .render(Value::from_serialize(&enum_ctx))?;
                    files.push(RenderedFile {
                        rel_path: PathBuf::from(format!("Models/{name}.php")),
                        content,
                    });
                }
                _ => {}
            }
        }

        // Routes stub
        let routes_ctx = build_routes_ctx(ctx.spec);
        let content = self
            .env
            .get_template("routes")?
            .render(Value::from_serialize(&routes_ctx))?;
        files.push(RenderedFile {
            rel_path: PathBuf::from("routes/api.php"),
            content,
        });

        Ok(files)
    }
}

// ─── Context builders ─────────────────────────────────────────────────────────

fn build_form_request_ctx(
    name: &str,
    schema: &ObjectSchema,
    namespace: &str,
    schemas: &IndexMap<String, ResolvedSchema>,
) -> FormRequestCtx {
    let fields = schema
        .properties
        .iter()
        .map(|(prop_name, prop)| {
            let required = prop.required && !prop.nullable;
            let rules = derive_validation_rules(&prop.schema, required, schemas);
            let rules_str = format!(
                "[{}]",
                rules
                    .iter()
                    .map(|r| format!("'{r}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            FormRequestFieldCtx {
                name: prop_name.clone(),
                rules_str,
            }
        })
        .collect();

    FormRequestCtx {
        namespace: namespace.to_string(),
        name: name.to_string(),
        fields,
    }
}

fn derive_validation_rules(
    schema: &ResolvedSchema,
    required: bool,
    schemas: &IndexMap<String, ResolvedSchema>,
) -> Vec<String> {
    // Object and non-enum Ref → always nullable array
    match schema {
        ResolvedSchema::Object(_) => {
            return vec!["nullable".to_string(), "array".to_string()];
        }
        ResolvedSchema::Ref(r) => {
            if !matches!(schemas.get(r.as_ref()), Some(ResolvedSchema::Enum(_))) {
                return vec!["nullable".to_string(), "array".to_string()];
            }
        }
        _ => {}
    }

    let presence = if required { "required" } else { "nullable" };
    let mut rules = vec![presence.to_string()];

    match schema {
        ResolvedSchema::Primitive(p) => match p.php_type {
            PhpPrimitive::String | PhpPrimitive::Mixed => {
                rules.push("string".to_string());
                if required {
                    rules.push("max:255".to_string());
                }
            }
            PhpPrimitive::Int => rules.push("integer".to_string()),
            PhpPrimitive::Float => rules.push("numeric".to_string()),
            PhpPrimitive::Bool => rules.push("boolean".to_string()),
            PhpPrimitive::DateTime => rules.push("date".to_string()),
        },
        ResolvedSchema::Array(_) => rules.push("array".to_string()),
        ResolvedSchema::Enum(e) => match e.backing_type {
            EnumBackingType::String => rules.push("string".to_string()),
            EnumBackingType::Int => rules.push("integer".to_string()),
        },
        ResolvedSchema::Ref(r) => {
            // At this point we know it's an enum ref (non-enum handled above)
            if let Some(ResolvedSchema::Enum(e)) = schemas.get(r.as_ref()) {
                match e.backing_type {
                    EnumBackingType::String => rules.push("string".to_string()),
                    EnumBackingType::Int => rules.push("integer".to_string()),
                }
            }
        }
        // Object already handled; Union → no extra rules
        _ => {}
    }

    rules
}

fn build_resource_ctx(
    name: &str,
    schema: &ObjectSchema,
    namespace: &str,
    schemas: &IndexMap<String, ResolvedSchema>,
) -> ResourceCtx {
    let fields = schema
        .properties
        .iter()
        .map(|(prop_name, prop)| {
            let camel = to_camel_case(prop_name);
            let nullable = !prop.required || prop.nullable;
            let expr = resource_field_expr(&camel, &prop.schema, nullable, schemas);
            ResourceFieldCtx {
                key: prop_name.clone(),
                expr,
            }
        })
        .collect();

    ResourceCtx {
        namespace: namespace.to_string(),
        name: name.to_string(),
        fields,
    }
}

fn resource_field_expr(
    camel: &str,
    schema: &ResolvedSchema,
    nullable: bool,
    schemas: &IndexMap<String, ResolvedSchema>,
) -> String {
    match schema {
        ResolvedSchema::Primitive(p) if p.php_type == PhpPrimitive::DateTime => {
            if nullable {
                format!("$this->{camel}?->format(\\DateTimeInterface::RFC3339)")
            } else {
                format!("$this->{camel}->format(\\DateTimeInterface::RFC3339)")
            }
        }
        ResolvedSchema::Enum(_) => {
            if nullable {
                format!("$this->{camel}?->value")
            } else {
                format!("$this->{camel}->value")
            }
        }
        ResolvedSchema::Ref(r) => {
            if matches!(schemas.get(r.as_ref()), Some(ResolvedSchema::Enum(_))) {
                if nullable {
                    format!("$this->{camel}?->value")
                } else {
                    format!("$this->{camel}->value")
                }
            } else {
                format!("$this->{camel}")
            }
        }
        _ => format!("$this->{camel}"),
    }
}

fn build_routes_ctx(spec: &ResolvedSpec) -> RoutesCtx {
    let routes = spec
        .endpoints
        .iter()
        .map(|ep| {
            let method = ep.method.as_str().to_lowercase();
            let path = ep.path.clone();

            // Controller name derived from first tag or first non-param path segment
            let tag = ep.tags.first().cloned().unwrap_or_else(|| {
                path.split('/')
                    .find(|s| !s.is_empty() && !s.starts_with('{'))
                    .unwrap_or("api")
                    .to_string()
            });
            let controller_base = to_pascal_case(&singularize(&tag));
            let controller = format!("App\\Http\\Controllers\\{controller_base}Controller");

            let action = derive_action(&ep.method, &ep.path_params);
            let comment = format!(
                "{} {} → \\{}@{}",
                ep.method.as_str(),
                path,
                controller,
                action
            );

            RouteCtx {
                method,
                path,
                controller,
                action,
                comment,
            }
        })
        .collect();

    RoutesCtx { routes }
}

fn to_pascal_case(s: &str) -> String {
    let camel = to_camel_case(s);
    let mut chars = camel.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn singularize(s: &str) -> String {
    if s.len() > 3 && s.ends_with("ies") {
        format!("{}y", &s[..s.len() - 3])
    } else if s.len() > 1
        && s.ends_with('s')
        && !s.ends_with("ss")
        && !s.ends_with("us")
        && !s.ends_with("is")
    {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn derive_action(method: &HttpMethod, path_params: &[ResolvedParam]) -> String {
    let has_params = !path_params.is_empty();
    match method {
        HttpMethod::Get => {
            if has_params {
                "show".to_string()
            } else {
                "index".to_string()
            }
        }
        HttpMethod::Post => "store".to_string(),
        HttpMethod::Put | HttpMethod::Patch => "update".to_string(),
        HttpMethod::Delete => "destroy".to_string(),
        _ => "index".to_string(),
    }
}
