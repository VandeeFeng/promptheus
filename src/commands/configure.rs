use crate::config::Config;
use crate::utils;
use anyhow::Result;

pub fn handle_configure_command(
    mut config: Config,
) -> Result<()> {
    println!("⚙️  Promptheus Configuration");
    println!("==========================");

    println!("Current configuration:");
    println!("  Prompt file: {}", config.general.prompt_file.display());
    println!("  Editor: {}", config.general.editor);
    println!("  Select command: {}", config.general.select_cmd);
    println!("  Default tags: {}", config.general.default_tags.join(", "));
    println!("  Auto sync: {}", config.general.auto_sync);
    println!("  Sort by: {:?}", config.general.sort_by);
    println!("  Color: {}", config.general.color);

    if utils::prompt_yes_no("\nEdit configuration?")? {
        // Edit prompt file path
        let new_path = utils::prompt_input(&format!("Prompt file [{}]: ", config.general.prompt_file.display()))?;
        if !new_path.is_empty() {
            config.general.prompt_file = new_path.into();
        }

        // Edit editor
        let new_editor = utils::prompt_input(&format!("Editor [{}]: ", config.general.editor))?;
        if !new_editor.is_empty() {
            config.general.editor = new_editor;
        }

        // Edit select command
        let new_select = utils::prompt_input(&format!("Select command [{}]: ", config.general.select_cmd))?;
        if !new_select.is_empty() {
            config.general.select_cmd = new_select;
        }

        // Edit default tags
        let new_tags = utils::prompt_input(&format!("Default tags [{}]: ", config.general.default_tags.join(", ")))?;
        if !new_tags.is_empty() {
            config.general.default_tags = new_tags.split_whitespace().map(|t| t.to_string()).collect();
        }

        // Edit auto sync
        config.general.auto_sync = utils::prompt_yes_no(&format!("Auto sync [{}]? ", config.general.auto_sync))?;

        config.save()?;
        println!("✓ Configuration saved!");
    } else {
        println!("Opening configuration file in editor...");
        let config_path = Config::config_file_path();
        utils::open_editor(Some(&config_path.to_string_lossy()))?;
    }

    Ok(())
}