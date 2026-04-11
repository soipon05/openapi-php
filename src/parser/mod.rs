pub mod error;
pub mod raw;
pub mod resolve;

pub use error::{ParseError, ResolveError};

use anyhow::Result;
use raw::types::RawOpenApi;
use std::path::Path;

pub fn load(path: &Path) -> Result<RawOpenApi> {
    let content = std::fs::read_to_string(path).map_err(|e| ParseError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let spec: RawOpenApi = match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| ParseError::Yaml {
            path: path.to_path_buf(),
            source: e,
        })?,
        "json" => serde_json::from_str(&content).map_err(|e| ParseError::Json {
            path: path.to_path_buf(),
            source: e,
        })?,
        other => {
            return Err(ParseError::UnsupportedExtension {
                path: path.to_path_buf(),
                ext: other.to_string(),
            }
            .into());
        }
    };

    Ok(spec)
}

pub fn load_and_resolve(path: &Path) -> Result<crate::ir::ResolvedSpec> {
    let raw = load(path)?;
    resolve::resolve(&raw)
}
