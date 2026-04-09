pub mod raw;
pub mod resolve;

use anyhow::{Context, Result};
use std::path::Path;
use raw::types::RawOpenApi;

pub fn load(path: &Path) -> Result<RawOpenApi> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let spec: RawOpenApi = match path.extension().and_then(|e| e.to_str()) {
        Some("json") => serde_json::from_str(&content)
            .with_context(|| "Failed to parse JSON")?,
        _ => serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse YAML")?,
    };

    Ok(spec)
}

pub fn load_and_resolve(path: &Path) -> Result<crate::ir::ResolvedSpec> {
    let raw = load(path)?;
    resolve::resolve(&raw)
}
