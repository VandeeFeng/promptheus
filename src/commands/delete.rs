use crate::cli::DeleteArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::utils;
use crate::utils::{OutputStyle, handle_not_found, print_cancelled, print_system_error, print_empty_result};
use anyhow::Result;

pub fn handle_delete_command(
    config: Config,
    args: &DeleteArgs,
    _interactive: bool,
) -> Result<()> {
    let storage = Manager::new(config.clone());

    // Find prompt by ID or description
    let prompt = if let Some(found) = storage.find_prompt_by_id(&args.identifier)? {
        found
    } else {
        // Search by description
        let prompts = storage.search_prompts(None, None)?;
        let found = prompts.iter()
            .find(|p| p.description.to_lowercase().contains(&args.identifier.to_lowercase()));

        if let Some(prompt) = found {
            prompt.clone()
        } else {
            // If no exact match found and identifier is just "delete", show interactive selection
            if args.identifier.to_lowercase() == "delete" || prompts.len() > 1 {
                // Handle empty prompts list
                if prompts.is_empty() {
                    print_empty_result("prompts");
                    return Ok(());
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
                        prompt.content
                    );
                    display_strings.push(display);
                }

                println!("🗑️  Select a prompt to delete:");
                if let Some(selected_index) = utils::select_from_list(&display_strings) {
                    prompts[selected_index].clone()
                } else {
                    print_cancelled("Prompt selection cancelled");
                    return Ok(());
                }
            } else {
                // Handle not found case as a notification, not an error
                handle_not_found("Prompt", &args.identifier);
                return Ok(());
            }
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
        storage.delete_prompt(id)?;
    } else {
        print_system_error("Cannot delete prompt: missing ID");
        return Err(anyhow::anyhow!("Cannot delete prompt: missing ID"));
    }
    println!("✓ Prompt '{}' deleted successfully!", prompt.description);

    Ok(())
}