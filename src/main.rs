mod cli;
mod config;
mod prompt;
mod storage;
mod utils;
mod commands;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};
use config::Config;
use commands::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        Config::load_custom(config_path)?
    } else {
        Config::load()?
    };

    // Handle commands
    match cli.command {
        Commands::New(ref args) => {
            new::handle_new_command(config, args, cli.interactive)?;
        }
        Commands::List(ref args) => {
            list::handle_list_command(config, args)?;
        }
        Commands::Search(ref args) => {
            search::handle_search_command(config, args)?;
        }
        Commands::Exec(ref args) => {
            exec::handle_exec_command(config, args)?;
        }
        Commands::Edit(ref args) => {
            edit::handle_edit_command(config, args, cli.interactive)?;
        }
        Commands::Configure => {
            configure::handle_configure_command(config)?;
        }
        Commands::Show(ref args) => {
            show::handle_show_command(config, args)?;
        }
        Commands::Delete(ref args) => {
            delete::handle_delete_command(config, args, cli.interactive)?;
        }
        Commands::Tags => {
            handle_tags_command(config)?;
        }
        Commands::Categories => {
            handle_categories_command(config)?;
        }
        Commands::Sync(_) => {
            println!("⚠️  Sync command not yet implemented");
        }
        Commands::Import(_) => {
            println!("⚠️  Import command not yet implemented");
        }
        Commands::Export(_) => {
            println!("⚠️  Export command not yet implemented");
        }
    }

    Ok(())
}

fn handle_tags_command(config: Config) -> Result<()> {
    let storage = storage::Storage::new(config);
    let tags = storage.get_all_tags()?;

    if tags.is_empty() {
        println!("No tags found.");
        return Ok(());
    }

    println!("🏷️  Available Tags ({})", tags.len());
    println!("====================");
    for tag in tags {
        println!("  {}", tag);
    }

    Ok(())
}

fn handle_categories_command(config: Config) -> Result<()> {
    let storage = storage::Storage::new(config);
    let categories = storage.get_categories()?;

    if categories.is_empty() {
        println!("No categories found.");
        return Ok(());
    }

    println!("📁 Available Categories ({})", categories.len());
    println!("=======================");
    for category in categories {
        println!("  {}", category);
    }

    Ok(())
}
