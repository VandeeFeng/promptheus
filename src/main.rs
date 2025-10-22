// Binary entry point - import modules directly
mod cli;
mod manager;
mod config;
mod core;
mod sync;
mod utils;

use clap::Parser;
use cli::Cli;
use config::Config;
use utils::error::report_error;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Ensure configuration exists and load it
    if let Err(e) = if cli.config.is_none() {
        Config::ensure_config_exists()
    } else {
        Ok(())
    } {
        report_error(&e);
        std::process::exit(1);
    }

    let config = match if let Some(config_path) = &cli.config {
        Config::load_custom(config_path)
    } else {
        Config::load()
    } {
        Ok(config) => config,
        Err(e) => {
            report_error(&e);
            std::process::exit(1);
        }
    };

    // Execute command
    if let Err(e) = cli.command.execute(config).await {
        report_error(&e);
        std::process::exit(1);
    }
}
