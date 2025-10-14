use crate::config::Config;
use crate::utils;
use crate::cli::ConfigCommands;
use anyhow::Result;

pub fn handle_config_command(
    mut config: Config,
    command: Option<ConfigCommands>,
) -> Result<()> {
    match command {
        Some(ConfigCommands::Show) => handle_show_command(&config),
        Some(ConfigCommands::Open) => handle_open_command(),
        Some(ConfigCommands::Reset) => handle_reset_command(&mut config),
        None => handle_config_help(),
    }
}

fn handle_show_command(config: &Config) -> Result<()> {
    println!("⚙️  Promptheus Configuration");
    println!("==========================");

    println!("General:");
    println!("  Prompt file: {}", config.general.prompt_file.display());
    if !config.general.prompt_dirs.is_empty() {
        println!("  Prompt dirs: {}", config.general.prompt_dirs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
    }
    println!("  Editor: {}", config.general.editor);
    println!("  Select command: {}", config.general.select_cmd);
    if !config.general.default_tags.is_empty() {
        println!("  Default tags: {}", config.general.default_tags.join(", "));
    }
    println!("  Auto sync: {}", config.general.auto_sync);
    println!("  Sort by: {:?}", config.general.sort_by);
    println!("  Color: {}", config.general.color);
    println!("  Content preview: {}", config.general.content_preview);
    println!("  Search case sensitive: {}", config.general.search_case_sensitive);
    if let Some(format) = &config.general.format {
        println!("  Default format: {}", format);
    }

    if let Some(gist) = &config.gist {
        println!("Gist:");
        println!("  File name: {}", gist.file_name);
        if gist.access_token.is_some() {
            println!("  Access token: ✓");
        }
        if let Some(gist_id) = &gist.gist_id {
            println!("  Gist ID: {}", gist_id);
        }
        println!("  Public: {}", gist.public);
        println!("  Auto sync: {}", gist.auto_sync);
    }

    if let Some(gitlab) = &config.gitlab {
        println!("GitLab:");
        println!("  File name: {}", gitlab.file_name);
        if gitlab.access_token.is_some() {
            println!("  Access token: ✓");
        }
        println!("  URL: {}", gitlab.url);
        if let Some(id) = gitlab.id {
            println!("  ID: {}", id);
        }
        println!("  Visibility: {}", gitlab.visibility);
        println!("  Auto sync: {}", gitlab.auto_sync);
        println!("  Skip SSL: {}", gitlab.skip_ssl);
    }

    Ok(())
}


fn handle_config_help() -> Result<()> {
    println!("⚙️  Configuration Management");
    println!("==========================");
    println!("Available configuration commands:");
    println!("  promptheus config show    - Show current configuration");
    println!("  promptheus config open    - Open configuration file in editor");
    println!("  promptheus config reset   - Reset configuration to defaults");
    println!();
    println!("Configuration file location: {}", Config::config_file_path().display());
    Ok(())
}

fn handle_open_command() -> Result<()> {
    // Ensure config file exists
    Config::ensure_config_exists()?;

    println!("Opening configuration file in editor...");
    let config_path = Config::config_file_path();
    println!("File: {}", config_path.display());

    utils::edit_file_direct(&config_path, None, None)?;
    Ok(())
}

fn handle_reset_command(config: &mut Config) -> Result<()> {
    if utils::prompt_yes_no("Are you sure you want to reset configuration to defaults? This will overwrite your current settings.")? {
        *config = Config::default();
        config.save()?;
        println!("✓ Configuration reset to defaults!");
    } else {
        println!("Reset cancelled.");
    }
    Ok(())
}

