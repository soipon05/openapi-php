// src/main.rs — thin entry-point; all logic lives in the library crate (src/lib.rs).
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = openapi_php::cli::Args::parse();
    args.run()
}
