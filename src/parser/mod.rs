pub mod types;

use anyhow::{Context, Result};
use std::path::Path;
use types::OpenApi;

pub fn load(path: &Path) -> Result<OpenApi> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let spec: OpenApi = match path.extension().and_then(|e| e.to_str()) {
        Some("json") => serde_json::from_str(&content)
            .with_context(|| "Failed to parse JSON")?,
        _ => serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse YAML")?,
    };

    Ok(spec)
}
