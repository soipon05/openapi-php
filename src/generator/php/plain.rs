//! PlainPhpBackend — generates plain PHP 8.1+ code.
//!
//! Output: readonly DTO classes, BackedEnum enums, and a PSR-18 API client.
//! Uses minijinja templates embedded at compile time.

use crate::generator::backend::{CodegenBackend, CodegenContext, RenderedFile};
use crate::ir::ResolvedSchema;
use anyhow::Result;
use minijinja::{Environment, Value};
use std::path::{Path, PathBuf};

use super::context::{
    build_client_ctx, build_enum_ctx, build_exception_ctxs, build_model_ctx, build_union_ctx,
};
use super::templates::add_template_with_override;

pub struct PlainPhpBackend {
    env: Environment<'static>,
}

impl PlainPhpBackend {
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
            "client",
            "client.php.j2",
            include_str!("../templates/php/client.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "union",
            "union.php.j2",
            include_str!("../templates/php/union.php.j2"),
        )?;
        add_template_with_override(
            &mut env,
            templates_dir,
            "exception",
            "exception.php.j2",
            include_str!("../templates/php/exception.php.j2"),
        )?;
        Ok(Self { env })
    }
}

impl CodegenBackend for PlainPhpBackend {
    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>> {
        let mut files: Vec<RenderedFile> = Vec::new();

        // Models
        for (name, schema) in &ctx.spec.schemas {
            match schema {
                ResolvedSchema::Object(obj) => {
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
                }
                ResolvedSchema::Enum(e) => {
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
                    // discriminator absent or non-ref variants → skip (no file generated)
                }
                // Array, Primitive have no standalone PHP file representation
                _ => {}
            }
        }

        // Exception classes
        let exception_ctxs = build_exception_ctxs(ctx.spec, ctx.namespace);
        for exc_ctx in &exception_ctxs {
            let content = self
                .env
                .get_template("exception")?
                .render(Value::from_serialize(exc_ctx))?;
            files.push(RenderedFile {
                rel_path: PathBuf::from(format!("Exceptions/{}.php", exc_ctx.class_name)),
                content,
            });
        }

        // API client
        let client_ctx = build_client_ctx(ctx.spec, ctx.namespace);
        let content = self
            .env
            .get_template("client")?
            .render(Value::from_serialize(&client_ctx))?;
        files.push(RenderedFile {
            rel_path: PathBuf::from("Client/ApiClient.php"),
            content,
        });

        Ok(files)
    }
}
