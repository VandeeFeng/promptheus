use crate::cli::ExecArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::utils;
use crate::utils::{handle_not_found, handle_empty_list, print_cancelled};
use anyhow::Result;

pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    let storage = Manager::new(config.clone());

    match &args.identifier {
        Some(identifier) => {
            // Direct execution with ID or description
            if let Some(prompt) = storage.find_prompt(identifier)? {
                handle_prompt_execution(&prompt, args.copy)?;
            } else {
                // Handle not found as notification, not error
                handle_not_found("Prompt with ID or description", identifier);
                return Ok(());
            }
        }
        None => {
            // Interactive mode - use fzf to select prompt
            handle_interactive_exec(config, args)?;
        }
    }

    Ok(())
}

fn handle_interactive_exec(config: Config, _args: &ExecArgs) -> Result<()> {
    let storage = Manager::new(config.clone());

    // Get all prompts for selection
    let prompts = storage.search_prompts(None, None)?;

    if prompts.is_empty() {
        handle_empty_list("prompts");
        return Ok(());
    }

    // Create display strings for selection - same format as search.rs
    let mut display_strings = Vec::new();
    for prompt in prompts.iter() {
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

        // Add content preview if enabled in config
        let content_part = if config.general.content_preview {
            // Truncate content for display (first 100 chars)
            let content_preview = if prompt.content.len() > 100 {
                format!("{}...", &prompt.content[..100])
            } else {
                prompt.content.clone()
            };
            format!(": {}{}", content_preview, tags)
        } else {
            tags.to_string()
        };

        // Format: [description]: content #tag1 #tag2 [category] (if preview enabled)
        // or: [description] #tag1 #tag2 [category] (if preview disabled)
        let display = if config.general.content_preview {
            format!("[{}]: {}{}",
                    prompt.description,
                    content_part,
                    category
            )
        } else {
            format!("[{}]{}{}",
                    prompt.description,
                    content_part,
                    category
            )
        };

        display_strings.push(display);
    }

    // Use fzf for interactive selection
    if let Some(selected_line) = utils::interactive_search_with_external_tool(
        &display_strings,
        &config.general.select_cmd,
        None
    )? {
        // Find the matching prompt by parsing the selected line
        if let Some(index) = find_prompt_by_display_line(&prompts, &selected_line)? {
            let prompt = &prompts[index];

            // For interactive mode: show only content and copy to clipboard
            let rendered_content = prompt.content.clone();

            println!("{}", rendered_content);

            // Always copy to clipboard in interactive mode
            utils::copy_to_clipboard(&rendered_content)?;
            utils::print_success("Prompt copied to clipboard!");
        }
    } else {
        // External tool was cancelled, exit gracefully
        print_cancelled("Prompt selection cancelled");
        return Ok(());
    }

    Ok(())
}

fn handle_prompt_execution(prompt: &crate::models::Prompt, copy_to_clipboard: bool) -> Result<()> {
    // Parse variables in the prompt content
    let variables = utils::parse_command_variables(&prompt.content);

    let rendered_content = if variables.is_empty() {
        // No variables, just use the content as-is
        prompt.content.clone()
    } else {
        // Prompt user for variable values
        println!("\n🔧 {}:", utils::OutputStyle::header("This prompt contains variables"));
        for (name, default) in &variables {
            if let Some(default_val) = default {
                println!("  <{}={}>", utils::OutputStyle::command(&format!("<{}>", name)), utils::OutputStyle::muted(default_val));
            } else {
                println!("  {}", utils::OutputStyle::command(&format!("<{}>", name)));
            }
        }

        let user_values = utils::prompt_for_variables(variables)?;
        utils::replace_command_variables(&prompt.content, &user_values)
    };

    if copy_to_clipboard {
        utils::copy_to_clipboard(&rendered_content)?;
        utils::print_success("Prompt copied to clipboard!");
    } else {
        println!("\n{}:", utils::OutputStyle::header("📤 Rendered Prompt"));
        println!("{}", utils::OutputStyle::header_separator());
        println!("{}", utils::OutputStyle::content(&rendered_content));
        println!("{}", utils::OutputStyle::header_separator());
    }

    Ok(())
}

/// Find the index of a prompt by parsing its display line
fn find_prompt_by_display_line(
    prompts: &[crate::models::Prompt],
    selected_line: &str
) -> Result<Option<usize>> {
    // Extract description from format: [description]: content #tags [category]
    if let Some(desc_end) = selected_line.find("]:") {
        let description = &selected_line[1..desc_end]; // Remove [ and ]

        for (i, prompt) in prompts.iter().enumerate() {
            if prompt.description == description {
                return Ok(Some(i));
            }
        }
    }

    Ok(None)
}