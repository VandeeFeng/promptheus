use crate::cli::DeleteArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::commands::handlers::InteractiveSelector;
use crate::utils;
use crate::utils::{OutputStyle, print_cancelled, print_system_error, print_empty_result};
use anyhow::Result;

pub fn handle_delete_command(
    config: Config,
    args: &DeleteArgs,
) -> Result<()> {
    let manager = Manager::new(config.clone());

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

        // Use the trait-based interactive selection
        if let Some(selected_prompt) = manager.select_interactive(
            prompts,
            |p| {
                let tags = if let Some(ref tags) = p.tag {
                    if tags.is_empty() {
                        String::new()
                    } else {
                        format!(" #{}", tags.join(" #"))
                    }
                } else {
                    String::new()
                };

                let category = if let Some(cat) = &p.category {
                    format!(" [{}]", cat)
                } else {
                    String::new()
                };

                format!("{}{}{}: {}", p.description, category, tags, p.content)
            },
            &config
        )? {
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