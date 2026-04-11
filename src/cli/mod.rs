use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueEnum};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use crate::config::{CliOverrides, Config, Framework, PhpVersion};

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
            } => {
                // Load config file (silently ignored if not found), then merge CLI flags on top.
                let config = Config::load(&std::env::current_dir()?)?;

                let cli_framework = framework.as_deref().map(Framework::parse).transpose()?;
                let cli_php_version = php_version.as_deref().map(PhpVersion::parse).transpose()?;

                let merged = config.merge_cli(CliOverrides {
                    namespace,
                    output,
                    framework: cli_framework,
                    php_version: cli_php_version,
                    templates,
                    input,
                });

                let input_path = merged.input.ok_or_else(|| {
                    anyhow!(
                        "No input file specified. Use --input <path> or set [input] path in openapi-php.toml"
                    )
                })?;
                let output_path = merged.output.unwrap_or_else(|| PathBuf::from("generated"));
                let namespace = merged.namespace;

                let options = GenerateOptions {
                    input_path,
                    output_path,
                    namespace,
                    mode,
                    framework: merged.framework,
                    templates_dir: merged.templates,
                    dry_run,
                    diff,
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
    templates_dir: Option<PathBuf>,
    dry_run: bool,
    diff: bool,
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
