use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use crate::config::{CliOverrides, Config, Framework, PhpVersion};
use crate::php_utils::{to_pascal_case, validate_namespace};

#[derive(Parser)]
#[command(name = "openapi-php")]
#[command(about = "Generate PHP code from OpenAPI 3.x specs", long_about = None)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate PHP code from an OpenAPI spec
    Generate {
        /// Path to the OpenAPI spec file (YAML or JSON)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output directory for generated PHP files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// PHP namespace (e.g. "App\\Generated")
        #[arg(short, long)]
        namespace: Option<String>,

        /// What to generate
        #[arg(short, long, default_value = "all")]
        mode: GenerateMode,

        /// Target framework: plain | laravel | symfony (overrides config file)
        #[arg(long)]
        framework: Option<String>,

        /// Target PHP version: 8.1 | 8.2 | 8.3 (overrides config file)
        #[arg(long)]
        php_version: Option<String>,

        /// Directory containing template overrides
        #[arg(long)]
        templates: Option<PathBuf>,

        /// Print what would be generated without writing any files
        #[arg(long)]
        dry_run: bool,

        /// Show diff of generated vs existing files; exits 1 if any change
        #[arg(long)]
        diff: bool,

        /// Re-run generation when the input file changes
        #[arg(long)]
        watch: bool,

        /// Glob pattern(s) for multiple spec files (alternative to --input).
        /// Use with --namespace-prefix and --output for zero-config multi-spec generation.
        /// Example: --inputs "specs/*.yml"
        #[arg(long, conflicts_with = "input")]
        inputs: Option<String>,

        /// Namespace prefix used when deriving per-spec namespaces from filenames.
        /// Required when --inputs is used. Example: --namespace-prefix "SaaSus\\"
        #[arg(long, conflicts_with = "namespace")]
        namespace_prefix: Option<String>,

        /// Generate one Client file per OpenAPI tag instead of a single ApiClient.php
        #[arg(long)]
        split_by_tag: bool,
    },

    /// Validate an OpenAPI spec file
    Validate {
        /// Path to the OpenAPI spec file
        #[arg(short, long)]
        input: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum GenerateMode {
    /// Models (DTOs) only
    Models,
    /// API client only
    Client,
    /// Both models and client
    All,
}

impl Args {
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Validate { input } => {
                let spec = crate::parser::load_and_resolve(&input)?;
                println!("✅ Valid API v{}", spec.version);
                println!("   title:     {}", spec.title);
                println!("   endpoints: {}", spec.endpoints.len());
                println!("   schemas:   {}", spec.schemas.len());
            }

            Command::Generate {
                input,
                output,
                namespace,
                mode,
                framework,
                php_version,
                templates,
                dry_run,
                diff,
                watch,
                inputs,
                namespace_prefix,
                split_by_tag,
            } => {
                // Load config file (silently ignored if not found), then merge CLI flags on top.
                let config = Config::load(&std::env::current_dir()?)?;

                // ── Multi-spec path (--inputs) ──────────────────────────────────────
                if let Some(glob_pattern) = inputs {
                    if watch {
                        bail!("--watch is not supported with --inputs");
                    }

                    let prefix = namespace_prefix.unwrap_or_else(|| "App\\Generated".to_string());
                    validate_namespace(&prefix)?;
                    let output_base = output.unwrap_or_else(|| PathBuf::from("generated"));

                    let cli_framework = framework.as_deref().map(Framework::parse).transpose()?;
                    let cli_php_version =
                        php_version.as_deref().map(PhpVersion::parse).transpose()?;
                    let framework_val = cli_framework.unwrap_or_else(|| config.framework.clone());
                    let php_ver = cli_php_version.unwrap_or_else(|| config.php_version.clone());

                    let mut paths = glob_expand(&glob_pattern)?;
                    paths.sort();

                    if paths.is_empty() {
                        bail!("No files matched the pattern: {glob_pattern}");
                    }

                    let mut any_diff = false;
                    for path in &paths {
                        let derived = derive_namespace_fragment(path)?;
                        let full_namespace = format!("{prefix}\\{derived}");
                        let full_output = output_base.join(&derived);

                        let opts = GenerateOptions {
                            input_path: path.clone(),
                            output_path: full_output,
                            namespace: full_namespace,
                            mode: mode.clone(),
                            framework: framework_val.clone(),
                            php_version: php_ver.clone(),
                            templates_dir: templates.clone().or_else(|| config.templates.clone()),
                            dry_run,
                            diff,
                            split_by_tag: split_by_tag || config.split_by_tag,
                        };
                        let had = run_generate_once(&opts)?;
                        any_diff |= had;
                    }
                    if any_diff {
                        std::process::exit(1);
                    }

                    return Ok(());
                }

                // ── Single-spec path (--input) ──────────────────────────────────────
                let cli_framework = framework.as_deref().map(Framework::parse).transpose()?;
                let cli_php_version = php_version.as_deref().map(PhpVersion::parse).transpose()?;

                let merged = config.merge_cli(CliOverrides {
                    namespace,
                    output,
                    framework: cli_framework,
                    php_version: cli_php_version,
                    templates,
                    input,
                    split_by_tag: if split_by_tag { Some(true) } else { None },
                });

                let input_path = merged.input.ok_or_else(|| {
                    anyhow!(
                        "No input file specified. Use --input <path> or set [input] path in openapi-php.toml"
                    )
                })?;
                let output_path = merged.output.unwrap_or_else(|| PathBuf::from("generated"));
                let namespace = merged.namespace;
                validate_namespace(&namespace)?;

                let options = GenerateOptions {
                    input_path,
                    output_path,
                    namespace,
                    mode,
                    framework: merged.framework,
                    php_version: merged.php_version,
                    templates_dir: merged.templates,
                    dry_run,
                    diff,
                    split_by_tag: merged.split_by_tag,
                };

                if watch {
                    run_generate_watch(options)?;
                } else if run_generate_once(&options)? {
                    std::process::exit(1);
                }
            }
        }
        Ok(())
    }
}

struct GenerateOptions {
    input_path: PathBuf,
    output_path: PathBuf,
    namespace: String,
    mode: GenerateMode,
    framework: Framework,
    php_version: PhpVersion,
    templates_dir: Option<PathBuf>,
    dry_run: bool,
    diff: bool,
    split_by_tag: bool,
}

fn run_generate_once(options: &GenerateOptions) -> Result<bool> {
    let spec = crate::parser::load_and_resolve(&options.input_path)?;

    if options.dry_run {
        crate::generator::run_dry_print(
            &spec,
            &options.namespace,
            options.mode.clone(),
            options.framework.clone(),
            options.templates_dir.as_deref(),
            &options.php_version,
            options.split_by_tag,
        )?;
        return Ok(false);
    }

    if options.diff {
        return crate::generator::run_diff(
            &spec,
            &options.output_path,
            &options.namespace,
            options.mode.clone(),
            options.framework.clone(),
            options.templates_dir.as_deref(),
            &options.php_version,
            options.split_by_tag,
        );
    }

    println!("🔧 Generating PHP from: {}", options.input_path.display());
    crate::generator::run(
        &spec,
        &options.output_path,
        &options.namespace,
        options.mode.clone(),
        options.framework.clone(),
        options.templates_dir.as_deref(),
        &options.php_version,
        options.split_by_tag,
    )?;
    Ok(false)
}

fn run_generate_watch(options: GenerateOptions) -> Result<()> {
    let watch_path = options.input_path.clone();
    let watch_dir = watch_path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let debounce = Duration::from_millis(300);

    run_generate_once(&options)?;
    println!("👀 Watching {} (Ctrl+C to stop)", watch_path.display());

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |result| {
            let _ = tx.send(result);
        },
        notify::Config::default(),
    )?;
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

    let mut pending = false;
    let mut last_event_at = Instant::now();

    loop {
        let timeout = if pending {
            debounce.saturating_sub(last_event_at.elapsed())
        } else {
            Duration::from_secs(1)
        };

        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if event_targets_input(&event, &watch_path) {
                    pending = true;
                    last_event_at = Instant::now();
                }
            }
            Ok(Err(err)) => eprintln!("watch error: {err}"),
            Err(RecvTimeoutError::Timeout) if pending => {
                println!("♻️  Change detected: {}", watch_path.display());
                if let Err(err) = run_generate_once(&options) {
                    eprintln!("generation failed: {err:#}");
                }
                pending = false;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(anyhow!("watch channel disconnected"));
            }
        }
    }
}

fn event_targets_input(event: &Event, input_path: &Path) -> bool {
    let target_path = normalize_path(input_path);
    event
        .paths
        .iter()
        .map(|path| normalize_path(path))
        .any(|path| path == target_path)
}

fn normalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

// ─── Multi-spec helpers ────────────────────────────────────────────────────

/// Expand a glob pattern to a sorted list of existing file paths.
fn glob_expand(pattern: &str) -> Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = glob::glob(pattern)
        .with_context(|| format!("Invalid glob pattern: {pattern}"))?
        .filter_map(|r| r.ok())
        .filter(|p| p.is_file())
        .collect();
    Ok(paths)
}

/// Derive a PascalCase namespace fragment from a spec filename.
///
/// Examples:
/// - `ordersapi.yml`     → `"Orders"`
/// - `paymentsapi.yml`   → `"Payments"`
/// - `webhookapi.yml`    → `"Webhook"`
/// - `catalog.yml`       → `"Catalog"`
fn derive_namespace_fragment(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Cannot derive namespace from path: {}", path.display()))?;

    // Strip trailing "api" (case-insensitive) unless the entire stem is "api"
    let lower = stem.to_lowercase();
    let base = if lower.ends_with("api") && lower.len() > 3 {
        &stem[..stem.len() - 3]
    } else {
        stem
    };

    let fragment = to_pascal_case(base);
    if fragment.is_empty() {
        bail!("Derived empty namespace from filename: {}", path.display());
    }
    Ok(fragment)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_namespace_fragment_strips_api_suffix() {
        let cases = vec![
            ("ordersapi.yml", "Orders"),
            ("paymentsapi.yml", "Payments"),
            ("webhookapi.yml", "Webhook"),
            ("notificationsapi.yml", "Notifications"),
            ("catalog.yml", "Catalog"),
            ("api.yml", "Api"), // stem is exactly "api" — no stripping
        ];
        for (filename, expected) in cases {
            let path = std::path::Path::new(filename);
            assert_eq!(
                derive_namespace_fragment(path).unwrap(),
                expected,
                "failed for {filename}"
            );
        }
    }

    #[test]
    fn to_pascal_case_handles_separators() {
        assert_eq!(to_pascal_case("auth"), "Auth");
        assert_eq!(to_pascal_case("my_service"), "MyService");
        assert_eq!(to_pascal_case("my-service"), "MyService");
        assert_eq!(to_pascal_case("apilog"), "Apilog");
    }
}
