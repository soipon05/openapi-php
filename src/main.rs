use anyhow::Result;
use clap::Parser;

mod cli;
mod generator;
mod parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    args.run()
}
