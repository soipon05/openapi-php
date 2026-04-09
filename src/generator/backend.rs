use crate::ir::ResolvedSchema;
use crate::ir::ResolvedSpec;
use anyhow::Result;
use minijinja::{Environment, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::php::context::{build_client_ctx, build_enum_ctx, build_model_ctx};

// ─── Core types ────────────────────────────────────────────────────────────

pub struct CodegenContext<'a> {
    pub spec: &'a ResolvedSpec,
    pub namespace: &'a str,
}

pub struct RenderedFile {
    pub rel_path: PathBuf,
    pub content: String,
}

// ─── CodegenBackend trait ──────────────────────────────────────────────────

pub trait CodegenBackend {
    /// Render all files into memory.
    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>>;

    /// Like `render`, but returns a sorted map of relative-path → content
    /// without touching the filesystem.  Useful for `--dry-run` and testing.
    fn run_dry(&self, ctx: &CodegenContext<'_>) -> Result<BTreeMap<PathBuf, String>> {
        let files = self.render(ctx)?;
        Ok(files.into_iter().map(|f| (f.rel_path, f.content)).collect())
    }

    /// Render and write all files to `output`.
    fn run(&self, ctx: &CodegenContext<'_>, output: &Path) -> Result<()> {
        let files = self.render(ctx)?;
        for file in &files {
            let full_path = output.join(&file.rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &file.content)?;
            println!("  📄 {}", file.rel_path.display());
        }
        println!("✅ Done → {}", output.display());
        Ok(())
    }
}

// ─── PlainPhpBackend ───────────────────────────────────────────────────────

/// Generates plain PHP 8.1+ code (PSR-18 client, readonly DTOs, BackedEnums).
pub struct PlainPhpBackend {
    env: Environment<'static>,
}

impl PlainPhpBackend {
    pub fn new() -> Self {
        let mut env = Environment::new();
        // trim_blocks: newline after {% %} tags consumed
        // lstrip_blocks: leading whitespace before {% %} / {{ }} stripped
        env.set_trim_blocks(true);
        env.set_lstrip_blocks(true);
        env.add_template(
            "model",
            include_str!("templates/php/model.php.j2"),
        )
        .expect("model template is valid");
        env.add_template(
            "enum",
            include_str!("templates/php/enum.php.j2"),
        )
        .expect("enum template is valid");
        env.add_template(
            "client",
            include_str!("templates/php/client.php.j2"),
        )
        .expect("client template is valid");
        Self { env }
    }
}

impl Default for PlainPhpBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CodegenBackend for PlainPhpBackend {
    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>> {
        let mut files: Vec<RenderedFile> = Vec::new();

        // Models
        for (name, schema) in &ctx.spec.schemas {
            match schema {
                ResolvedSchema::Object(obj) => {
                    let model_ctx = build_model_ctx(name, obj, ctx.namespace);
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
                _ => {}
            }
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
