//! Configuration loading and merging.
//!
//! Config is sourced from `openapi-php.toml` (searched upward from CWD until the
//! git repo root or the home directory), then overridden by CLI flags via
//! [`Config::merge_cli`]. CLI always wins over the file.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};

// ─── Public enums ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Framework {
    #[default]
    Plain,
    Laravel,
    Symfony,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PhpVersion {
    Php81,
    #[default]
    Php82,
    Php83,
    Php84,
}

// ─── Config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub namespace: String,
    pub output: Option<PathBuf>,
    pub framework: Framework,
    pub php_version: PhpVersion,
    pub templates: Option<PathBuf>,
    pub input: Option<PathBuf>,
    pub split_by_tag: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            namespace: "App\\Generated".to_string(),
            output: None,
            framework: Framework::default(),
            php_version: PhpVersion::default(),
            templates: None,
            input: None,
            split_by_tag: false,
        }
    }
}

// ─── CLI override bag (populated from parsed CLI args) ─────────────────────

pub struct CliOverrides {
    pub namespace: Option<String>,
    pub output: Option<PathBuf>,
    pub framework: Option<Framework>,
    pub php_version: Option<PhpVersion>,
    pub templates: Option<PathBuf>,
    pub input: Option<PathBuf>,
    pub split_by_tag: Option<bool>,
}

// ─── Raw TOML deserialization ──────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
struct RawToml {
    #[serde(default)]
    generator: RawGenerator,
    #[serde(default)]
    input: RawInput,
}

#[derive(Debug, Deserialize, Default)]
struct RawGenerator {
    namespace: Option<String>,
    output: Option<String>,
    framework: Option<String>,
    php_version: Option<String>,
    templates: Option<String>,
    split_by_tag: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
struct RawInput {
    path: Option<String>,
}

// ─── Config impl ───────────────────────────────────────────────────────────

impl Config {
    /// Load config by searching upward from `start_dir`.
    /// Returns `Config::default()` if no file is found.
    pub fn load(start_dir: &Path) -> Result<Config> {
        match Self::find_config_file(start_dir) {
            Some(path) => {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                Self::from_toml_str(&content)
            }
            None => Ok(Config::default()),
        }
    }

    /// Parse a TOML string directly (useful for testing).
    pub fn from_toml_str(content: &str) -> Result<Config> {
        let raw: RawToml = toml::from_str(content).context("Failed to parse openapi-php.toml")?;

        let framework = match raw.generator.framework.as_deref() {
            Some("plain") | None => Framework::Plain,
            Some("laravel") => Framework::Laravel,
            Some("symfony") => Framework::Symfony,
            Some(other) => {
                bail!("Unknown framework '{other}'. Valid values: plain, laravel, symfony")
            }
        };

        let php_version = match raw.generator.php_version.as_deref() {
            Some("8.1") => PhpVersion::Php81,
            Some("8.2") | None => PhpVersion::Php82,
            Some("8.3") => PhpVersion::Php83,
            Some("8.4") => PhpVersion::Php84,
            Some(other) => bail!("Unknown php_version '{other}'. Valid values: 8.1, 8.2, 8.3, 8.4"),
        };

        Ok(Config {
            namespace: raw
                .generator
                .namespace
                .unwrap_or_else(|| "App\\Generated".to_string()),
            output: raw.generator.output.map(PathBuf::from),
            framework,
            php_version,
            templates: raw.generator.templates.map(PathBuf::from),
            input: raw.input.path.map(PathBuf::from),
            split_by_tag: raw.generator.split_by_tag.unwrap_or(false),
        })
    }

    /// Merge CLI overrides into this config. CLI values win.
    pub fn merge_cli(self, cli: CliOverrides) -> Config {
        Config {
            namespace: cli.namespace.unwrap_or(self.namespace),
            output: cli.output.or(self.output),
            framework: cli.framework.unwrap_or(self.framework),
            php_version: cli.php_version.unwrap_or(self.php_version),
            templates: cli.templates.or(self.templates),
            input: cli.input.or(self.input),
            split_by_tag: cli.split_by_tag.unwrap_or(self.split_by_tag),
        }
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    /// Walk parent directories looking for `openapi-php.toml`.
    /// Stops at the git repo root (first `.git` ancestor), the home dir, or the fs root.
    fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let mut dir = start_dir.to_path_buf();

        loop {
            let candidate = dir.join("openapi-php.toml");
            if candidate.exists() {
                return Some(candidate);
            }

            // Stop after checking the git repo root
            if dir.join(".git").exists() {
                break;
            }

            // Stop after checking home dir
            if let Some(ref h) = home
                && dir == *h
            {
                break;
            }

            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => break,
            }
        }

        None
    }
}

// ─── String → enum helpers (used by CLI parser) ───────────────────────────

impl Framework {
    pub fn parse(s: &str) -> Result<Framework> {
        match s {
            "plain" => Ok(Framework::Plain),
            "laravel" => Ok(Framework::Laravel),
            "symfony" => Ok(Framework::Symfony),
            other => bail!("Unknown framework '{other}'. Valid values: plain, laravel, symfony"),
        }
    }
}

impl PhpVersion {
    pub fn parse(s: &str) -> Result<PhpVersion> {
        match s {
            "8.1" => Ok(PhpVersion::Php81),
            "8.2" => Ok(PhpVersion::Php82),
            "8.3" => Ok(PhpVersion::Php83),
            "8.4" => Ok(PhpVersion::Php84),
            other => bail!("Unknown php_version '{other}'. Valid values: 8.1, 8.2, 8.3, 8.4"),
        }
    }

    /// Returns `true` when the version supports `readonly class` (PHP 8.2+).
    pub fn supports_readonly_class(&self) -> bool {
        matches!(
            self,
            PhpVersion::Php82 | PhpVersion::Php83 | PhpVersion::Php84
        )
    }
}
