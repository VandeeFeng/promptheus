use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;
use crate::commands::{new, list, search, exec, edit, configure, show, delete};
use crate::commands::{sync, push};
use crate::utils::print_warning;

#[derive(Parser)]
#[command(name = "promptheus")]
#[command(about = "A Rust-based prompt management tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub debug: bool,

    #[arg(short = 'i', long, help = "Run in interactive mode")]
    pub interactive: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Commands {
    pub async fn execute(self, config: Config, interactive: bool) -> Result<()> {
        match self {
            Commands::New(args) => {
                new::handle_new_command(config, &args, interactive).await?;
            }
            Commands::List(args) => {
                list::handle_list_command(config, &args)?;
            }
            Commands::Search(args) => {
                search::handle_search_command(config, &args)?;
            }
            Commands::Exec(args) => {
                exec::handle_exec_command(config, &args)?;
            }
            Commands::Edit(args) => {
                edit::handle_edit_command(config, &args, interactive).await?;
            }
            Commands::Config(args) => {
                configure::handle_config_command(config, args.command.clone())?;
            }
            Commands::Show(args) => {
                show::handle_show_command(config, &args)?;
            }
            Commands::Delete(args) => {
                delete::handle_delete_command(config, &args, interactive)?;
            }
            Commands::Sync(args) => {
                sync::handle_sync_command(config, &args).await?;
            }
            Commands::Push => {
                push::handle_push_command(config).await?;
            }
            Commands::Import(_) => {
                print_warning("Import command not yet implemented");
            }
            Commands::Export(_) => {
                print_warning("Export command not yet implemented");
            }
        }
        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new prompt
    New(NewArgs),

    /// Edit an existing prompt
    Edit(EditArgs),

    /// List all prompts
    List(ListArgs),

    /// Search prompts interactively
    Search(SearchArgs),

    /// Execute a prompt (copy to clipboard or output)
    Exec(ExecArgs),

    /// Delete a prompt
    Delete(DeleteArgs),

    /// Show prompt details
    Show(ShowArgs),

    /// Configuration management
    Config(ConfigArgs),

    /// Sync prompts with remote service
    Sync(SyncArgs),

    /// Push local prompts to remote service (force upload)
    Push,

    /// Import prompts from file
    Import(ImportArgs),

    /// Export prompts to file
    Export(ExportArgs),

  }

#[derive(Args)]
pub struct NewArgs {
    #[arg(short = 'T', long)]
    pub title: Option<String>,

    #[arg(short = 'd', long)]
    pub description: Option<String>,

    #[arg(short = 't', long)]
    pub tag: Option<String>,

    #[arg(short = 'c', long)]
    pub category: Option<String>,

    #[arg(long)]
    pub editor: bool,

    #[arg(long)]
    pub content: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    #[arg(help = "Prompt ID or title to edit")]
    pub identifier: Option<String>,

    #[arg(long)]
    pub id: Option<String>,

    #[arg(short = 't', long, help = "Filter by tag")]
    pub tag: Option<String>,

    #[arg(short = 'c', long, help = "Filter by category")]
    pub category: Option<String>,

    #[arg(long, help = "Force edit of prompt file directly instead of interactive selection")]
    pub file: bool,

    #[arg(long, help = "Editor command to use (overrides config)")]
    pub editor: Option<String>,

    #[arg(long, help = "Line number to jump to")]
    pub line: Option<u32>,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long)]
    pub tag: Option<String>,

    #[arg(short = 'c', long)]
    pub category: Option<String>,

    #[arg(short, long)]
    pub format: Option<ListFormat>,

    #[arg(long)]
    pub stats: bool,

    #[arg(long, help = "Show all available tags")]
    pub tags: bool,

    #[arg(long, help = "Show all available categories")]
    pub categories: bool,
}

#[derive(Args)]
pub struct SearchArgs {
    #[arg(short, long)]
    pub tag: Option<String>,

    #[arg(short, long)]
    pub category: Option<String>,

    #[arg(short = 'q', long)]
    pub query: Option<String>,

    #[arg(long)]
    pub execute: bool,

    #[arg(long)]
    pub copy: bool,
}

#[derive(Args)]
pub struct ExecArgs {
    #[arg(help = "Prompt ID or description")]
    pub identifier: Option<String>,

    #[arg(long)]
    pub copy: bool,

    #[arg(long)]
    pub output: bool,

    #[arg(long)]
    pub vars: Vec<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    #[arg(help = "Prompt ID or title")]
    pub identifier: String,

    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ShowArgs {
    #[arg(help = "Prompt ID or title")]
    pub identifier: String,

    #[arg(long)]
    pub vars: Vec<String>,
}

#[derive(Args)]
pub struct SyncArgs {
    #[arg(short, long)]
    pub upload: bool,

    #[arg(short, long)]
    pub download: bool,

    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ImportArgs {
    #[arg(help = "File to import from")]
    pub file: PathBuf,

    #[arg(short, long)]
    pub format: Option<ImportFormat>,

    #[arg(long)]
    pub merge: bool,
}

#[derive(Args)]
pub struct ExportArgs {
    #[arg(help = "File to export to")]
    pub file: PathBuf,

    #[arg(short, long)]
    pub format: ExportFormat,

    #[arg(short, long)]
    pub tag: Option<String>,

    #[arg(short = 'c', long)]
    pub category: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ListFormat {
    Simple,
    Detailed,
    Table,
    Json,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ImportFormat {
    Toml,
    Json,
    Yaml,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ExportFormat {
    Toml,
    Json,
    Yaml,
    Markdown,
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Open configuration file in editor
    Open,

    /// Reset configuration to defaults
    Reset,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_args_enhanced() {
        // Test that the enhanced EditArgs structure works correctly
        let args = EditArgs {
            identifier: Some("test-prompt".to_string()),
            id: None,
            tag: Some("test".to_string()),
            category: Some("test-category".to_string()),
            file: false,
            editor: Some("vim".to_string()),
            line: Some(42),
        };

        assert_eq!(args.identifier, Some("test-prompt".to_string()));
        assert_eq!(args.tag, Some("test".to_string()));
        assert_eq!(args.category, Some("test-category".to_string()));
        assert_eq!(args.editor, Some("vim".to_string()));
        assert_eq!(args.line, Some(42));
        assert!(!args.file);
    }

    #[test]
    fn test_edit_args_defaults() {
        // Test default values
        let args = EditArgs {
            identifier: None,
            id: None,
            tag: None,
            category: None,
            file: false,
            editor: None,
            line: None,
        };

        assert!(args.identifier.is_none());
        assert!(args.id.is_none());
        assert!(args.tag.is_none());
        assert!(args.category.is_none());
        assert!(!args.file);
        assert!(args.editor.is_none());
        assert!(args.line.is_none());
    }
}