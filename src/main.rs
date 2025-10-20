// Binary entry point - import modules directly
mod cli;
mod manager;
mod config;
mod core;
mod sync;
mod utils;

use anyhow::Result;
use clap::Parser;

use cli::Cli;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure configuration exists and load it
    if cli.config.is_none() {
        Config::ensure_config_exists()?;
    }

    let config = if let Some(config_path) = &cli.config {
        Config::load_custom(config_path)?
    } else {
        Config::load()?
    };

    // Execute command
    cli.command.execute(config).await?;

    Ok(())
}
