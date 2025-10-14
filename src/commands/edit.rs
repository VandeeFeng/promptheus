use crate::cli::EditArgs;
use crate::config::Config;
use crate::storage::Storage;
use crate::utils;
use anyhow::Result;

pub fn handle_edit_command(
    config: Config,
    args: &EditArgs,
    interactive: bool,
) -> Result<()> {
    let storage = Storage::new(config.clone());

    // Find prompt to edit
    let prompt = if let Some(identifier) = args.identifier.as_ref().or(args.id.as_ref()) {
        // Try to find by ID first, then by title
        if let Some(found) = storage.find_prompt_by_id(identifier)? {
            found
        } else {
            // Search by title
            let prompts = storage.search_prompts(None, None)?;
            let found = prompts.into_iter()
                .find(|p| p.description.to_lowercase().contains(&identifier.to_lowercase()));

            if let Some(prompt) = found {
                prompt
            } else {
                return Err(anyhow::anyhow!("Prompt not found: {}", identifier));
            }
        }
    } else {
        // Interactive prompt selection
        let prompts = storage.search_prompts(None, None)?;
        if prompts.is_empty() {
            return Err(anyhow::anyhow!("No prompts found to edit"));
        }

        // Create display strings for selection
        let mut display_strings = Vec::new();
        for prompt in &prompts {
            let tags = if let Some(ref tags) = prompt.tag {
                if tags.is_empty() {
                    String::new()
                } else {
                    format!(" #{}", tags.join(" #"))
                }
            } else {
                String::new()
            };

            let category = if let Some(cat) = &prompt.category {
                format!(" [{}]", cat)
            } else {
                String::new()
            };

            let display = format!("{}{}{}: {}",
                prompt.description,
                category,
                tags,
                prompt.description
            );
            display_strings.push(display);
        }

        println!("🔍 Select a prompt to edit:");
        if let Some(selected_index) = utils::select_from_list(&display_strings)? {
            prompts[selected_index].clone()
        } else {
            return Ok(()); // User cancelled
        }
    };

    let mut prompt = prompt;

    println!("Editing prompt: {}", prompt.description);
    println!("Current content:");
    println!("{}", "─".repeat(50));
    println!("{}", prompt.content);
    println!("{}", "─".repeat(50));

    if utils::prompt_yes_no("Edit content?")? {
        let new_content = utils::open_editor(Some(&prompt.content))?;
        if !new_content.is_empty() {
            prompt.update_content(new_content);
        }
    }

    if utils::prompt_yes_no("Edit title?")? {
        let new_title = utils::prompt_input(&format!("New title [{}]: ", prompt.description))?;
        if !new_title.is_empty() {
            prompt.description = new_title;
        }
    }

    if utils::prompt_yes_no("Edit description?")? {
        let new_desc = utils::prompt_input(&format!("New description [{}]: ", prompt.description))?;
        if !new_desc.is_empty() {
            prompt.description = new_desc;
        }
    }

    storage.update_prompt(&prompt)?;
    println!("✓ Prompt updated successfully!");

    Ok(())
}