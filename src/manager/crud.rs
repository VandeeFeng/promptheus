// CRUD operations - Create, Read, Update, Delete
// Consolidated from new.rs, show.rs, edit.rs, delete.rs

use anyhow::Result;
use std::fs;

use crate::cli::{NewArgs, ShowArgs, EditArgs, DeleteArgs};
use crate::config::Config;
use crate::core::data::Prompt;
use crate::core::traits::{PromptSearch, PromptInteraction, PromptCrud};
use crate::core::operations::PromptOperations;
use crate::utils::{self, print_sync_warning, handle_not_found, OutputStyle, print_cancelled, print_system_error, print_empty_result};

// Create operations
pub async fn handle_new_command(
    config: Config,
    args: &NewArgs,
) -> Result<()> {
    let storage = PromptOperations::new(config.clone());

    let description = match &args.description {
        Some(d) => d.clone(),
        None => match utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Description")), &[]) {
            Some(desc) => desc,
            None => return Ok(()),
        },
    };

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if args.editor {
        utils::open_editor_custom(None, None, Some(&config.general.editor))?
    } else {
        match utils::prompt_multiline(&format!("{}:", OutputStyle::label("Prompt content"))) {
            Some(content) => content,
            None => return Ok(()),
        }
    };

    let mut prompt = Prompt::new(description.clone(), content);

    // Handle tags interactively if not specified
    if let Some(tag_str) = &args.tag {
        let tags: Vec<String> = tag_str.split_whitespace().map(|t| t.to_string()).collect();
        for tag in tags {
            prompt.add_tag(tag);
        }
    } else {
        let existing_tags = storage.get_all_tags()?;
        loop {
            let custom_tag = match utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Tag")), &existing_tags) {
                Some(tag) => tag,
                None => return Ok(()), // ESC to cancel
            };
            if custom_tag.is_empty() {
                break; // Empty input to finish adding tags
            }
            prompt.add_tag(custom_tag);
        }
    }

    // Add default tags from config
    for tag in &config.general.default_tags {
        prompt.add_tag(tag.clone());
    }

    // Handle category interactively if not specified
    if let Some(category) = &args.category {
        prompt.category = Some(category.clone());
    } else {
        let existing_categories = storage.get_categories()?;

        let custom_category = match utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Category")), &existing_categories) {
            Some(category) => category,
            None => return Ok(()),
        };
        if !custom_category.is_empty() {
            prompt.category = Some(custom_category);
        }
    }

    storage.add_prompt(prompt)?;
    utils::output::print_success(&format!("Prompt '{}' saved successfully!", description));

    // Auto-sync if enabled
    if let Err(e) = crate::manager::sync::auto_sync_if_enabled(&config).await {
        print_sync_warning(&e.to_string());
    }

    Ok(())
}

// Read operations
pub fn handle_show_command(
    config: Config,
    args: &ShowArgs,
) -> Result<()> {
    let manager = PromptOperations::new(config);

    if let Some(prompt) = manager.find_prompt(&args.identifier)? {
        OutputStyle::print_prompt_detailed(&prompt);
    } else {
        handle_not_found("Prompt", &args.identifier);
    }

    Ok(())
}

// Update operations
pub async fn handle_edit_command(
    config: Config,
    args: &EditArgs,
) -> Result<()> {
    let storage = PromptOperations::new(config.clone());
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
        // Interactive selection
        if let Some(selected_prompt) = storage.select_interactive_prompts(prompts)? {
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
    if let Err(e) = crate::manager::sync::auto_sync_if_enabled(&config).await {
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

// Delete operations
pub fn handle_delete_command(
    config: Config,
    args: &DeleteArgs,
) -> Result<()> {
    let manager = PromptOperations::new(config.clone());

    // Find prompt by ID or description
    let prompt = if let Some(found) = manager.find_prompt(&args.identifier)? {
        found
    } else {
        // If not found, try interactive selection
        let prompts = manager.search_prompts(None, None)?;

        if prompts.is_empty() {
            print_empty_result("prompts");
            return Ok(());
        }

        // Use interactive selection
        if let Some(selected_prompt) = manager.select_interactive_prompts(prompts)? {
            selected_prompt
        } else {
            print_cancelled("Prompt selection cancelled");
            return Ok(());
        }
    };

    println!("Prompt to delete:");
    OutputStyle::print_prompt_basic(&prompt);

    if !args.force
        && !utils::prompt_yes_no("\nAre you sure you want to delete this prompt?")? {
            print_cancelled("Prompt not deleted");
            return Ok(());
        }

    if let Some(id) = &prompt.id {
        manager.delete_prompt(id)?;
    } else {
        print_system_error("Cannot delete prompt: missing ID");
        return Err(anyhow::anyhow!("Cannot delete prompt: missing ID"));
    }
    println!("✓ Prompt '{}' deleted successfully!", prompt.description);

    Ok(())
}