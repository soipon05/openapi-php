//! CodegenBackend trait and shared types for code generation.
//!
//! The trait defines the `render()` interface that every backend must implement.
//! Concrete implementations live in `php::plain` and `php::laravel`.

use crate::cli::GenerateMode;
use crate::config::PhpVersion;
use crate::ir::ResolvedSpec;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ─── Core types ────────────────────────────────────────────────────────────

pub struct CodegenContext<'a> {
    pub spec: &'a ResolvedSpec,
    pub namespace: &'a str,
    pub php_version: &'a PhpVersion,
    pub split_by_tag: bool,
}

pub struct RenderedFile {
    pub rel_path: PathBuf,
    pub content: String,
}

// ─── CodegenBackend trait ──────────────────────────────────────────────────

pub trait CodegenBackend {
    /// Render all files into memory.
    fn render(&self, ctx: &CodegenContext<'_>) -> Result<Vec<RenderedFile>>;

    /// Like `render`, but returns a sorted map of relative-path → content.
    /// Useful for `--dry-run` and testing.
    fn run_dry(&self, ctx: &CodegenContext<'_>) -> Result<BTreeMap<PathBuf, String>> {
        let files = self.render(ctx)?;
        Ok(files.into_iter().map(|f| (f.rel_path, f.content)).collect())
    }

    /// Decide whether `path` should be included for a given `--mode` flag.
    /// Override in backends whose directory layout differs from the default.
    fn filter_by_mode(&self, path: &Path, mode: &GenerateMode) -> bool {
        match mode {
            GenerateMode::Models => path.starts_with("Models"),
            GenerateMode::Client => path.starts_with("Client") || path.starts_with("Exceptions"),
            GenerateMode::All => true,
        }
    }
}
