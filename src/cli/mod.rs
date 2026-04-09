use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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
        input: PathBuf,

        /// Output directory for generated PHP files
        #[arg(short, long, default_value = "generated")]
        output: PathBuf,

        /// PHP namespace (e.g. "App\\Generated")
        #[arg(short, long, default_value = "App\\Generated")]
        namespace: String,

        /// What to generate
        #[arg(short, long, default_value = "all")]
        mode: GenerateMode,
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
                let spec = crate::parser::load(&input)?;
                println!("✅ Valid OpenAPI {}", spec.openapi);
                println!("   title: {}", spec.info.title);
                println!("   paths: {}", spec.paths.as_ref().map_or(0, |p| p.len()));
                println!("   schemas: {}", spec.components.as_ref()
                    .and_then(|c| c.schemas.as_ref())
                    .map_or(0, |s| s.len()));
            }

            Command::Generate { input, output, namespace, mode } => {
                let spec = crate::parser::load(&input)?;
                println!("🔧 Generating PHP from: {}", input.display());
                crate::generator::run(&spec, &output, &namespace, mode)?;
            }
        }
        Ok(())
    }
}
