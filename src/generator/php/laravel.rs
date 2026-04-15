//! LaravelPhpBackend — generates Laravel-specific PHP code.
//!
//! Output: readonly DTO classes, BackedEnum enums, FormRequest validators,
//! JsonResource transformers, and a routes/api.php stub.

use crate::cli::GenerateMode;
use crate::generator::backend::{CodegenBackend, CodegenContext, RenderedFile};
use crate::generator::php::context::{build_enum_ctx, build_model_ctx, build_union_ctx};
use crate::ir::{
    EnumBackingType, HttpMethod, ObjectSchema, PhpPrimitive, ResolvedEndpoint, ResolvedParam,
    ResolvedSchema, ResolvedSpec,
};
use crate::php_utils::{to_camel_case, to_pascal_case};
use anyhow::Result;
use indexmap::IndexMap;
use minijinja::{Environment, Value};
use serde::Serialize;
use std::path::{Path, PathBuf};

use super::helpers::{sanitize_php_ident, sanitize_php_string_literal, sanitize_phpdoc};
use super::templates::add_template_with_override;

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
struct ControllerCtx {
    namespace: String,
    name: String,
    use_statements: Vec<String>,
    methods: Vec<ControllerMethodCtx>,
}

#[derive(Debug, Serialize)]
struct ControllerMethodCtx {
    action: String,
    summary: Option<String>,
    params_str: String,
    return_type: String,
    phpdoc_params: Vec<String>,
    phpdoc_return: String,
}

#[derive(Debug, Serialize)]
struct RoutesCtx {
    controller_imports: Vec<String>,
    routes: Vec<RouteCtx>,
}

#[derive(Debug, Serialize)]
struct RouteCtx {
    method: String,
    path: String,
    controller_short: String,
    action: String,
    comment: String,
}

// ─── LaravelPhpBackend ────────────────────────────────────────────────────────

/// Generates Laravel-specific PHP code: DTOs, FormRequests, JsonResources, routes.
pub struct LaravelPhpBackend {
    env: Environment<'static>,
}

impl LaravelPhpBackend {
    pub fn new(templates_dir: Option<&Path>) -> Result<Self> {
        let mut env = Environment::new();
        env.set_trim_blocks(true);
        env.set_lstrip_blocks(true);
        add_template_with_override(
            &mut env,
            templates_dir,
            "model",
            "model.php.j2",
            include_str!("../templates/php/model.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "enum",
            "enum.php.j2",
            include_str!("../templates/php/enum.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "form_request",
            "laravel/form_request.php.j2",
            include_str!("../templates/php/laravel/form_request.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "resource",
            "laravel/resource.php.j2",
            include_str!("../templates/php/laravel/resource.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "routes",
            "laravel/routes.php.j2",
            include_str!("../templates/php/laravel/routes.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "controller",
            "laravel/controller.php.j2",
            include_str!("../templates/php/laravel/controller.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "union",
            "union.php.j2",
            include_str!("../templates/php/union.php.j2"),
        )?;
        Ok(Self { env })
    }
}

impl CodegenBackend for LaravelPhpBackend {
    fn filter_by_mode(&self, path: &Path, mode: &GenerateMode) -> bool {
        match mode {
            GenerateMode::Models => path.starts_with("Models") || path.starts_with("Http"),
            GenerateMode::Client => path.starts_with("routes"),
            GenerateMode::All => true,
        }
    }

    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>> {
        let mut files: Vec<RenderedFile> = Vec::new();

        for (name, schema) in &ctx.spec.schemas {
            match schema {
                ResolvedSchema::Object(obj) => {
                    // DTO — reuse existing model template
                    let model_ctx = build_model_ctx(
                        name,
                        obj,
                        ctx.namespace,
                        &ctx.spec.schemas,
                        ctx.php_version.supports_readonly_class(),
                        ctx.php_version,
                    );
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
                    let res_ctx = build_resource_ctx(name, obj, ctx.namespace, &ctx.spec.schemas);
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
                ResolvedSchema::Union(u) => {
                    // Union DTOs are generated as discriminated containers (Models/ only).
                    // No FormRequest or JsonResource is generated for union types.
                    if let Some(union_ctx) = build_union_ctx(
                        name,
                        u,
                        ctx.namespace,
                        ctx.php_version.supports_readonly_class(),
                    ) {
                        let content = self
                            .env
                            .get_template("union")?
                            .render(Value::from_serialize(&union_ctx))?;
                        files.push(RenderedFile {
                            rel_path: PathBuf::from(format!("Models/{name}.php")),
                            content,
                        });
                    }
                    // discriminator absent or non-ref variants → skip
                }
                // Array, Primitive have no standalone PHP file representation
                _ => {}
            }
        }

        // Routes stub
        let routes_ctx = build_routes_ctx(ctx.spec, ctx.namespace);
        let content = self
            .env
            .get_template("routes")?
            .render(Value::from_serialize(&routes_ctx))?;
        files.push(RenderedFile {
            rel_path: PathBuf::from("routes/api.php"),
            content,
        });

        // Controller stubs
        for ctrl_ctx in build_controller_ctxs(ctx.spec, ctx.namespace) {
            let ctrl_name = ctrl_ctx.name.clone();
            let content = self
                .env
                .get_template("controller")?
                .render(Value::from_serialize(&ctrl_ctx))?;
            files.push(RenderedFile {
                rel_path: PathBuf::from(format!("Http/Controllers/{ctrl_name}Controller.php")),
                content,
            });
        }

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
                name: sanitize_php_string_literal(prop_name),
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
                match (p.min_length, p.max_length) {
                    (Some(min), Some(max)) => {
                        rules.push(format!("between:{min},{max}"));
                    }
                    (Some(min), None) => rules.push(format!("min:{min}")),
                    (None, Some(max)) => rules.push(format!("max:{max}")),
                    (None, None) if required => rules.push("max:255".to_string()),
                    _ => {}
                }
                if let Some(pat) = &p.pattern {
                    // Sanitize first (removes ' and \), then escape forward slashes
                    let safe = sanitize_php_string_literal(pat);
                    let escaped = safe.replace('/', "\\/");
                    rules.push(format!("regex:/{escaped}/"));
                }
            }
            PhpPrimitive::Int => {
                rules.push("integer".to_string());
                if let Some(min) = p.minimum {
                    rules.push(format!("min:{}", min as i64));
                }
                if let Some(max) = p.maximum {
                    rules.push(format!("max:{}", max as i64));
                }
            }
            PhpPrimitive::Float => {
                rules.push("numeric".to_string());
                if let Some(min) = p.minimum {
                    rules.push(format!("min:{min}"));
                }
                if let Some(max) = p.maximum {
                    rules.push(format!("max:{max}"));
                }
            }
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
                key: sanitize_php_string_literal(prop_name),
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

fn build_routes_ctx(spec: &ResolvedSpec, namespace: &str) -> RoutesCtx {
    let mut controller_imports: Vec<String> = Vec::new();

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
            let controller_short = format!("{controller_base}Controller");
            let fqcn = format!("{namespace}\\Http\\Controllers\\{controller_short}");

            if !controller_imports.contains(&fqcn) {
                controller_imports.push(fqcn);
            }

            let action = derive_action(&ep.method, &ep.path_params);
            let safe_path = sanitize_php_string_literal(&path);
            // comment は改行を除去（// コメント内への改行インジェクション防止）
            let comment = format!(
                "{} {} → {controller_short}@{action}",
                ep.method.as_str(),
                path.replace(['\r', '\n'], ""),
            );

            RouteCtx {
                method,
                path: safe_path,
                controller_short,
                action,
                comment,
            }
        })
        .collect();

    controller_imports.sort();

    RoutesCtx {
        controller_imports,
        routes,
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

fn build_controller_ctxs(spec: &ResolvedSpec, namespace: &str) -> Vec<ControllerCtx> {
    // Group endpoints by controller name (same derivation as routes)
    let mut groups: IndexMap<String, Vec<&ResolvedEndpoint>> = IndexMap::new();
    for ep in &spec.endpoints {
        let tag = ep.tags.first().cloned().unwrap_or_else(|| {
            ep.path
                .split('/')
                .find(|s| !s.is_empty() && !s.starts_with('{'))
                .unwrap_or("api")
                .to_string()
        });
        let name = sanitize_php_ident(&to_pascal_case(&singularize(&tag)));
        groups.entry(name).or_default().push(ep);
    }

    groups
        .into_iter()
        .map(|(name, endpoints)| {
            let mut use_statements: Vec<String> = Vec::new();
            let mut methods: Vec<ControllerMethodCtx> = Vec::new();

            for ep in endpoints {
                let action = derive_action(&ep.method, &ep.path_params);
                let mut params_parts: Vec<String> = Vec::new();
                let mut phpdoc_params: Vec<String> = Vec::new();

                // Request body parameter (comes first in Laravel convention)
                if let Some(body) = &ep.request_body
                    && let ResolvedSchema::Ref(r) = &body.schema
                {
                    let req_class = format!("{r}Request");
                    let use_stmt = format!("{namespace}\\Http\\Requests\\{req_class}");
                    if !use_statements.contains(&use_stmt) {
                        use_statements.push(use_stmt);
                    }
                    phpdoc_params.push(format!("@param {req_class} $request"));
                    params_parts.push(format!("{req_class} $request"));
                }

                // Path parameters
                for pp in &ep.path_params {
                    let type_hint = match &pp.schema {
                        ResolvedSchema::Primitive(p) => match p.php_type {
                            PhpPrimitive::Int => "int",
                            PhpPrimitive::Float => "float",
                            PhpPrimitive::Bool => "bool",
                            _ => "string",
                        },
                        _ => "string",
                    };
                    phpdoc_params.push(format!("@param {type_hint} ${}", pp.php_name));
                    params_parts.push(format!("{type_hint} ${}", pp.php_name));
                }

                // Return type derived from response schema
                let return_type = match &ep.response {
                    Some(ResolvedSchema::Ref(r)) => {
                        let res_class = format!("{r}Resource");
                        let use_stmt = format!("{namespace}\\Http\\Resources\\{res_class}");
                        if !use_statements.contains(&use_stmt) {
                            use_statements.push(use_stmt);
                        }
                        res_class
                    }
                    _ => "JsonResponse".to_string(),
                };

                methods.push(ControllerMethodCtx {
                    action,
                    summary: ep.summary.as_deref().map(sanitize_phpdoc),
                    params_str: params_parts.join(", "),
                    phpdoc_return: format!("@return {return_type}"),
                    return_type,
                    phpdoc_params,
                });
            }

            use_statements.sort();

            ControllerCtx {
                namespace: namespace.to_string(),
                name,
                use_statements,
                methods,
            }
        })
        .collect()
}
