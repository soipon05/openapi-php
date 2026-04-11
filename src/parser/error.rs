use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Cannot read file `{path}`: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Invalid YAML in `{path}`: {source}")]
    Yaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("Invalid JSON in `{path}`: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("Unsupported file extension `{ext}` in `{path}` (expected .yaml, .yml, or .json)")]
    UnsupportedExtension { path: PathBuf, ext: String },
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("Unknown $ref `{ref_path}` — schema `{name}` not found in components/schemas")]
    UnknownRef { ref_path: String, name: String },

    #[error("Circular $ref detected: `{cycle}`")]
    CircularRef { cycle: String },

    #[error("$ref `{ref_path}` has invalid format (expected #/components/schemas/Name)")]
    InvalidRefFormat { ref_path: String },
}
