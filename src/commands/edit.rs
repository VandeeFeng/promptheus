use anyhow::Result;
use std::fs;

use crate::cli::EditArgs;
use crate::config::Config;
use crate::commands::handlers::InteractiveSelector;
use crate::utils::{self, print_sync_warning, handle_not_found};

pub async fn handle_edit_command(
    config: Config,
    args: &EditArgs,
) -> Result<()> {
    let storage = crate::manager::Manager::new(config.clone());
    let prompts = storage.search_prompts(None, args.tag.as_deref())?;

    let file_to_edit = config.general.prompt_file.clone();
    let line_number = if let Some(identifier) = args.identifier.as_ref().or(args.id.as_ref()) {
        // Find by identifier
        if let Some(prompt) = prompts.iter().find(|p| p.id.as_ref() == Some(identifier) || p.description.to_lowercase().contains(&identifier.to_lowercase())) {
                match find_line_number_of_prompt(&file_to_edit, &prompt.description) {
                    Ok(line_num) => Some(line_num),
                    Err(_) => {
                        handle_not_found("Prompt in TOML file", &prompt.description);
                        return Ok(());
                    }
                }
            } else {
                None
            }
    } else {
        // Interactive selection using unified trait interface
        if let Some(selected_prompt) = storage.select_interactive_prompts(prompts, &config)? {
            match find_line_number_of_prompt(&file_to_edit, &selected_prompt.description) {
                Ok(line_num) => Some(line_num),
                Err(_) => {
                    handle_not_found("Prompt in TOML file", &selected_prompt.description);
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        }
    };

    utils::edit_file_direct(&file_to_edit, line_number.map(|l| l as u32), args.editor.as_deref())?;

    // Auto-sync if enabled
    if let Err(e) = crate::commands::sync::auto_sync_if_enabled(&config).await {
        print_sync_warning(&e.to_string());
    }

    Ok(())
}

fn find_line_number_of_prompt(file_path: &std::path::Path, prompt_description: &str) -> Result<usize> {
    let content = fs::read_to_string(file_path)?;
    let mut last_header_line = None;
    // Construct the exact line to search for, including quotes.
    let search_str = format!(r#"Description = "{}""#, prompt_description);

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        if line.trim() == "[[prompts]]" {
            last_header_line = Some(line_num);
        }
        // Check if the trimmed line is the exact description line we're looking for.
        if line.trim() == search_str
            && let Some(header_line) = last_header_line {
                return Ok(header_line);
            }
    }

    Err(anyhow::anyhow!("Prompt not found in TOML"))
}

