use crate::cli::GenerateMode;
use crate::ir::ResolvedSpec;
use anyhow::Result;
use std::path::Path;

pub mod backend;
pub mod php;

pub use backend::{CodegenBackend, CodegenContext, PlainPhpBackend, RenderedFile};

pub fn run(spec: &ResolvedSpec, output: &Path, namespace: &str, mode: GenerateMode) -> Result<()> {
    std::fs::create_dir_all(output)?;

    let ctx = CodegenContext { spec, namespace };
    let backend = PlainPhpBackend::new();
    let files = backend.render(&ctx)?;

    for file in &files {
        let include = match mode {
            GenerateMode::Models => file.rel_path.starts_with("Models"),
            GenerateMode::Client => file.rel_path.starts_with("Client"),
            GenerateMode::All => true,
        };
        if include {
            let full_path = output.join(&file.rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &file.content)?;
            println!("  📄 {}", file.rel_path.display());
        }
    }

    println!("✅ Done → {}", output.display());
    Ok(())
}
