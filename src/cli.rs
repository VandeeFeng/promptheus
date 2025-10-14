use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

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

    /// Edit configuration
    Configure,

    /// Sync prompts with remote service
    Sync(SyncArgs),

    /// Import prompts from file
    Import(ImportArgs),

    /// Export prompts to file
    Export(ExportArgs),

    /// Show all available tags
    Tags,

    /// Show all available categories
    Categories,
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
    #[arg(help = "Prompt ID or title")]
    pub identifier: String,

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