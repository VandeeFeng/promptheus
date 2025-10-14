use crate::cli::SearchArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::utils;
use anyhow::Result;
use crate::utils::{format_datetime, OutputStyle, print_no_prompts_found};

pub fn handle_search_command(
    config: Config,
    args: &SearchArgs,
) -> Result<()> {
    let storage = Manager::new(config.clone());

    let prompts = storage.search_prompts(args.query.as_deref(), args.tag.as_deref())?;

    if prompts.is_empty() {
        print_no_prompts_found();
        return Ok(());
    }

    // Filter by category if specified
    let filtered_prompts: Vec<_> = if let Some(category) = &args.category {
        prompts.into_iter()
            .filter(|p| p.category.as_ref() == Some(category))
            .collect()
    } else {
        prompts
    };

    if filtered_prompts.is_empty() {
        print_no_prompts_found();
        return Ok(());
    }

    // Create display strings for selection - use pet-like format
    let mut display_strings = Vec::new();
    for prompt in filtered_prompts.iter() {
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

    let selected_index = if let Some(query) = &args.query {
        // Try external tool first (like fzf), fall back to fuzzy search
        if let Some(selected_line) = utils::interactive_search_with_external_tool(
            &display_strings,
            &config.general.select_cmd,
            Some(query)
        )? {
            // Find the matching prompt by parsing the selected line
            find_prompt_by_display_line(&filtered_prompts, &selected_line)?
        } else {
            // External tool was cancelled, exit gracefully
            return Ok(());
        }
    } else {
        // Try external tool for general interactive selection
        if let Some(selected_line) = utils::interactive_search_with_external_tool(
            &display_strings,
            &config.general.select_cmd,
            None
        )? {
            find_prompt_by_display_line(&filtered_prompts, &selected_line)?
        } else {
            // External tool was cancelled, exit gracefully
            return Ok(());
        }
    };

    if let Some(index) = selected_index {
        let prompt = &filtered_prompts[index];

        if args.execute {
            handle_prompt_execution(prompt, args.copy)?;
        } else {
            show_prompt_details(prompt);
        }
    }

    Ok(())
}

fn show_prompt_details(prompt: &crate::models::Prompt) {
    OutputStyle::print_header("📝 Prompt Details");

    OutputStyle::print_field_colored("Description", &prompt.description, OutputStyle::description);
    if let Some(id) = &prompt.id {
        OutputStyle::print_field_colored("ID", id, OutputStyle::muted);
    }

    // Display Tag field (first tag only, or empty line if no tags)
    if let Some(ref tags) = prompt.tag && !tags.is_empty() {
        OutputStyle::print_field_colored("Tag", &tags[0], OutputStyle::tag);
    } else {
        OutputStyle::print_field("Tag", "");
    }

    if let Some(category) = &prompt.category {
        OutputStyle::print_field_colored("Category", category, OutputStyle::tag);
    }

    if let Some(ref tags) = prompt.tag
        && !tags.is_empty() {
            OutputStyle::print_field_colored("Tags", &tags.join(", "), OutputStyle::tags);
        }

    OutputStyle::print_field_colored("Created", &format_datetime(&prompt.created_at), OutputStyle::muted);

    println!("\n{}:", OutputStyle::header("📄 Content"));
    println!("{}", OutputStyle::separator());
    println!("{}", OutputStyle::content(&prompt.content));
    println!("{}", OutputStyle::separator());
}

fn handle_prompt_execution(prompt: &crate::models::Prompt, copy_to_clipboard: bool) -> Result<()> {
    // Parse variables in the prompt content
    let variables = utils::parse_command_variables(&prompt.content);

    let rendered_content = if variables.is_empty() {
        // No variables, just use the content as-is
        prompt.content.clone()
    } else {
        // Prompt user for variable values
        println!("\n🔧 {}:", OutputStyle::header("This prompt contains variables"));
        for (name, default) in &variables {
            if let Some(default_val) = default {
                println!("  <{}={}>", OutputStyle::command(&format!("<{}>", name)), OutputStyle::muted(default_val));
            } else {
                println!("  {}", OutputStyle::command(&format!("<{}>", name)));
            }
        }

        let user_values = utils::prompt_for_variables(variables)?;
        utils::replace_command_variables(&prompt.content, &user_values)
    };

    if copy_to_clipboard {
        utils::copy_to_clipboard(&rendered_content)?;
        println!("✓ {}", OutputStyle::success("Prompt copied to clipboard!"));
    } else {
        println!("\n{}:", OutputStyle::header("📤 Rendered Prompt"));
        println!("{}", OutputStyle::header_separator());
        println!("{}", OutputStyle::content(&rendered_content));
        println!("{}", OutputStyle::header_separator());
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
