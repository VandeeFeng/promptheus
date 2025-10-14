use anyhow::Result;
use std::fs;

use crate::cli::EditArgs;
use crate::config::Config;
use crate::storage::Storage;
use crate::utils;

pub async fn handle_edit_command(
    config: Config,
    args: &EditArgs,
    _interactive: bool,
) -> Result<()> {
    let storage = Storage::new(config.clone());
    let prompts = storage.search_prompts(args.tag.as_deref(), args.category.as_deref())?;

    let file_to_edit = config.general.prompt_file.clone();
    let line_number = if let Some(identifier) = args.identifier.as_ref().or(args.id.as_ref()) {
        // Find by identifier
        prompts
            .iter()
            .find(|p| p.id.as_ref() == Some(identifier) || p.description.to_lowercase().contains(&identifier.to_lowercase()))
            .and_then(|p| find_line_number_of_prompt(&file_to_edit, &p.description).ok())
    } else {
        // Interactive selection
        let display_strings: Vec<String> = prompts
            .iter()
            .map(|p| {
                let tags = p.tag.as_deref().map(|t| format!(" #{}", t.join(" #"))).unwrap_or_default();
                let category = p.category.as_deref().map(|c| format!(" [{}]", c)).unwrap_or_default();
                format!("{}{}{}: {}", p.description, category, tags, p.content.lines().next().unwrap_or("").chars().take(50).collect::<String>())
            })
            .collect();

        if let Some(selected_line) = utils::interactive_search_with_external_tool(&display_strings, &config.general.select_cmd, None)? {
            let selected_index = display_strings.iter().position(|d| d == &selected_line);
            selected_index.and_then(|i| find_line_number_of_prompt(&file_to_edit, &prompts[i].description).ok())
        } else {
            None
        }
    };

    utils::edit_file_direct(&file_to_edit, line_number.map(|l| l as u32), args.editor.as_deref())?;

    println!("✓ Prompt file opened for editing.");

    // Auto-sync if enabled
    if let Err(e) = crate::commands::sync::auto_sync_if_enabled(&config).await {
        eprintln!("⚠️  Auto-sync failed: {}", e);
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
        if line.trim() == search_str {
            if let Some(header_line) = last_header_line {
                return Ok(header_line);
            }
        }
    }

    Err(anyhow::anyhow!("Prompt not found in TOML"))
}

