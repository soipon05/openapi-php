use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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

        /// Print what would be generated without writing any files
        #[arg(long)]
        dry_run: bool,

        /// Show diff of generated vs existing files; exits 1 if any change
        #[arg(long)]
        diff: bool,
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
                dry_run,
                diff,
            } => {
                // Load config file (silently ignored if not found), then merge CLI flags on top.
                let config = Config::load(&std::env::current_dir()?)?;

                let cli_framework = framework
                    .as_deref()
                    .map(Framework::parse)
                    .transpose()?;
                let cli_php_version = php_version
                    .as_deref()
                    .map(PhpVersion::parse)
                    .transpose()?;

                let merged = config.merge_cli(CliOverrides {
                    namespace,
                    output,
                    framework: cli_framework,
                    php_version: cli_php_version,
                    input,
                });

                let input_path = merged.input.ok_or_else(|| {
                    anyhow!(
                        "No input file specified. Use --input <path> or set [input] path in openapi-php.toml"
                    )
                })?;
                let output_path = merged.output.unwrap_or_else(|| PathBuf::from("generated"));
                let namespace = merged.namespace;

                let spec = crate::parser::load_and_resolve(&input_path)?;

                if dry_run {
                    crate::generator::run_dry_print(&spec, &namespace, mode, merged.framework)?;
                } else if diff {
                    let has_changes = crate::generator::run_diff(
                        &spec,
                        &output_path,
                        &namespace,
                        mode,
                        merged.framework,
                    )?;
                    if has_changes {
                        std::process::exit(1);
                    }
                } else {
                    println!("🔧 Generating PHP from: {}", input_path.display());
                    crate::generator::run(&spec, &output_path, &namespace, mode, merged.framework)?;
                }
            }
        }
        Ok(())
    }
}
