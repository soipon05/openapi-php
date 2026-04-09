use crate::cli::GenerateMode;
use crate::parser::types::OpenApi;
use anyhow::Result;
use std::path::Path;

pub mod php;

pub fn run(spec: &OpenApi, output: &Path, namespace: &str, mode: GenerateMode) -> Result<()> {
    std::fs::create_dir_all(output)?;

    match mode {
        GenerateMode::Models => {
            php::models::generate(spec, output, namespace)?;
        }
        GenerateMode::Client => {
            php::client::generate(spec, output, namespace)?;
        }
        GenerateMode::All => {
            php::models::generate(spec, output, namespace)?;
            php::client::generate(spec, output, namespace)?;
        }
    }

    println!("✅ Done → {}", output.display());
    Ok(())
}
