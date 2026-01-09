// CRUD operations - Create, Read, Update, Delete
// Consolidated from new.rs, show.rs, edit.rs, delete.rs

use std::fs;

use crate::cli::{DeleteArgs, EditArgs, NewArgs, ShowArgs};
use crate::config::Config;
use crate::core::data::Prompt;
use crate::core::operations::PromptOperations;
use crate::core::traits::{PromptCrud, PromptInteraction, PromptSearch};
use crate::utils::{
    self, OutputStyle,
    error::{AppError, FlowResult},
};

// Create operations
pub async fn handle_new_command(config: Config, args: &NewArgs) -> Result<FlowResult, AppError> {
    let storage = PromptOperations::new(&config);

    let description = prompt_input_default("Description", &args.description, &[], false)?;

    let content = resolve_prompt_content(&storage, args)?;

    let mut prompt = Prompt::new(description.clone(), content);
    add_tags_to_prompt(&storage, &mut prompt, args.tag.as_deref())?;

    // Handle category interactively if not specified
    if let Some(category) = &args.category {
        prompt.category = Some(category.clone());
    } else {
        let existing_categories = storage.get_categories()?;
        let custom_category = prompt_input_labeled("Category", &existing_categories)?;
        if !custom_category.is_empty() {
            prompt.category = Some(custom_category);
        }
    }

    storage.add_prompt(prompt)?;
    utils::output::print_success(&format!("Prompt '{}' saved successfully!", description));

    crate::manager::sync::handle_auto_sync_after_crud(storage.config()).await;

    Ok(FlowResult::Success(
        "Prompt saved successfully!".to_string(),
    ))
}

fn resolve_prompt_content(storage: &PromptOperations, args: &NewArgs) -> Result<String, AppError> {
    if let Some(content) = &args.content {
        return Ok(content.clone());
    }

    if args.editor {
        return Ok(utils::open_editor_custom(
            None,
            None,
            Some(&storage.config().general.editor),
        )?);
    }

    match utils::prompt_multiline(&format!("{}:", OutputStyle::label("Prompt content"))) {
        Some(content) => Ok(content),
        None => Err(AppError::System("Operation cancelled by user".to_string())),
    }
}

fn add_tags_to_prompt(
    storage: &PromptOperations,
    prompt: &mut Prompt,
    tag_arg: Option<&str>,
) -> Result<(), AppError> {
    if let Some(tag_str) = tag_arg {
        let tags: Vec<String> = tag_str.split_whitespace().map(String::from).collect();
        tags.into_iter().for_each(|tag| prompt.add_tag(tag));
    } else {
        let existing_tags = storage.get_all_tags()?;
        loop {
            let custom_tag = prompt_input_labeled("Tag", &existing_tags)?;
            if custom_tag.is_empty() {
                break;
            }
            prompt.add_tag(custom_tag);
        }
    }

    // Add default tags from config
    for tag in &storage.config().general.default_tags {
        prompt.add_tag(tag.clone());
    }

    Ok(())
}

fn prompt_input_default(
    label: &str,
    provided_value: &Option<String>,
    completions: &[String],
    multiline: bool,
) -> Result<String, AppError> {
    if let Some(value) = provided_value {
        return Ok(value.clone());
    }

    let result = if multiline {
        utils::prompt_multiline(&format!("{}:", OutputStyle::label(label)))
    } else {
        utils::prompt_input_with_autocomplete(
            &format!("{}: ", OutputStyle::label(label)),
            completions,
        )
    };

    result.ok_or_else(|| AppError::System("Operation cancelled by user".to_string()))
}

fn prompt_input_labeled(label: &str, completions: &[String]) -> Result<String, AppError> {
    utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label(label)), completions)
        .ok_or_else(|| AppError::System("Operation cancelled by user".to_string()))
}

// Read operations
pub fn handle_show_command(config: Config, args: &ShowArgs) -> Result<FlowResult, AppError> {
    let manager = PromptOperations::new(&config);

    if let Some(prompt) = manager.find_prompt(&args.identifier)? {
        // Display complete prompt with all logic handled internally
        OutputStyle::display_prompt_complete(&prompt)?;
    } else {
        return Ok(FlowResult::NotFound {
            item_type: "Prompt".to_string(),
            search_term: args.identifier.clone(),
        });
    }

    Ok(FlowResult::Success("".to_string()))
}

// Update operations
pub async fn handle_edit_command(config: Config, args: &EditArgs) -> Result<FlowResult, AppError> {
    let storage = PromptOperations::new(&config);
    let prompts = storage.search_prompts(None, args.tag.as_deref())?;

    let prompt = match resolve_prompt_to_edit(&storage, args, prompts) {
        Ok(p) => p,
        Err(FlowResult::NotFound {
            item_type,
            search_term,
        }) => {
            return Ok(FlowResult::NotFound {
                item_type,
                search_term,
            });
        }
        Err(FlowResult::Cancelled(msg)) => {
            return Ok(FlowResult::Cancelled(msg));
        }
        Err(FlowResult::Success(_) | FlowResult::EmptyList { .. }) => {
            return Err(AppError::System("Unexpected flow result".to_string()));
        }
    };

    let line_number =
        match find_prompt_line_number(&storage.config().general.prompt_file, &prompt.description) {
            Ok(num) => num,
            Err(_) => {
                return Ok(FlowResult::NotFound {
                    item_type: "Prompt in TOML file".to_string(),
                    search_term: prompt.description.clone(),
                });
            }
        };

    utils::edit_file_direct(
        &storage.config().general.prompt_file,
        Some(line_number as u32),
        args.editor.as_deref(),
    )?;

    crate::manager::sync::handle_auto_sync_after_crud(storage.config()).await;

    Ok(FlowResult::Success("".to_string()))
}

fn resolve_prompt_to_edit(
    storage: &PromptOperations,
    args: &EditArgs,
    prompts: Vec<Prompt>,
) -> Result<Prompt, FlowResult> {
    if let Some(identifier) = args.identifier.as_ref().or(args.id.as_ref()) {
        prompts
            .iter()
            .find(|p| {
                p.id.as_ref() == Some(identifier)
                    || p.description
                        .to_lowercase()
                        .contains(&identifier.to_lowercase())
            })
            .cloned()
            .ok_or_else(|| FlowResult::NotFound {
                item_type: "Prompt".to_string(),
                search_term: identifier.clone(),
            })
    } else {
        storage
            .select_interactive_prompts(prompts)
            .map_err(|e| match e {
                AppError::System(msg) => FlowResult::Cancelled(msg),
                _ => FlowResult::Cancelled("Failed to select prompt".to_string()),
            })?
            .ok_or_else(|| FlowResult::Cancelled("Operation cancelled by user".to_string()))
    }
}

fn find_prompt_line_number(
    file_path: &std::path::Path,
    prompt_description: &str,
) -> Result<usize, AppError> {
    let content = fs::read_to_string(file_path).map_err(|e| AppError::Io(e.to_string()))?;
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
            && let Some(header_line) = last_header_line
        {
            return Ok(header_line);
        }
    }

    Err(AppError::System("Prompt not found in TOML".to_string()))
}

// Delete operations
pub fn handle_delete_command(config: Config, args: &DeleteArgs) -> Result<FlowResult, AppError> {
    let manager = PromptOperations::new(&config);
    let prompt = resolve_prompt_to_delete(&manager, &args.identifier)?;

    println!("Prompt to delete:");
    OutputStyle::print_prompt_basic(&prompt);

    confirm_delete(&manager, &prompt, args.force)?;

    Ok(FlowResult::Success("".to_string()))
}

fn resolve_prompt_to_delete(
    manager: &PromptOperations,
    identifier: &str,
) -> Result<Prompt, AppError> {
    if let Some(found) = manager.find_prompt(identifier)? {
        return Ok(found);
    }

    let prompts = manager.get_all_prompts_or_return_empty()?;
    manager
        .select_interactive_prompts(prompts)?
        .ok_or_else(|| AppError::System("Prompt selection cancelled".to_string()))
}

fn confirm_delete(
    manager: &PromptOperations,
    prompt: &Prompt,
    force: bool,
) -> Result<(), AppError> {
    if !force && !utils::prompt_yes_no("\nAre you sure you want to delete this prompt?")? {
        return Err(AppError::System("Prompt not deleted".to_string()));
    }

    let id = prompt
        .id
        .as_ref()
        .ok_or_else(|| AppError::System("Cannot delete prompt: missing ID".to_string()))?;

    manager.delete_prompt(id)?;
    println!("✓ Prompt '{}' deleted successfully!", prompt.description);

    Ok(())
}
