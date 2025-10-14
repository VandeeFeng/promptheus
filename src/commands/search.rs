use crate::cli::SearchArgs;
use crate::config::Config;
use crate::storage::Storage;
use crate::utils;
use anyhow::Result;

pub fn handle_search_command(
    config: Config,
    args: &SearchArgs,
) -> Result<()> {
    let storage = Storage::new(config.clone());

    let prompts = storage.search_prompts(args.query.as_deref(), args.tag.as_deref())?;

    if prompts.is_empty() {
        println!("No prompts found matching your criteria.");
        return Ok(());
    }

    // Filter by category if specified
    let filtered_prompts: Vec<_> = if let Some(category) = &args.category {
        prompts.into_iter()
            .filter(|p| p.category.as_ref().map_or(false, |c| c == category))
            .collect()
    } else {
        prompts
    };

    if filtered_prompts.is_empty() {
        println!("No prompts found matching the criteria.");
        return Ok(());
    }

    // Create display strings for selection
    let mut display_strings = Vec::new();
    for prompt in &filtered_prompts {
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

        let display = format!("{}{}{}: {}{}",
            prompt.description,
            category,
            tags,
            prompt.description,
            prompt.content
        );
        display_strings.push(display);
    }

    let selected_index = if let Some(query) = &args.query {
        // Perform fuzzy search and show top results
        let results = utils::fuzzy_search(&display_strings, query);
        if results.is_empty() {
            println!("No prompts found matching: {}", query);
            return Ok(());
        }

        // Show results and let user select
        println!("🔍 Found {} prompts matching '{}':", results.len(), query);
        for (i, result) in results.iter().enumerate() {
            println!("{}. {}", i + 1, display_strings[result.0]);
        }

        if results.len() == 1 {
            Some(results[0].0)
        } else {
            println!("\nSelect a prompt (1-{}):", results.len());
            let selection = utils::prompt_input("Enter number: ")?;
            match selection.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= results.len() => Some(results[n - 1].0),
                _ => {
                    println!("Invalid selection.");
                    return Ok(());
                }
            }
        }
    } else {
        // Show interactive selection
        utils::select_from_list(&display_strings)?
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

fn show_prompt_details(prompt: &crate::prompt::Prompt) {
    println!("\n📝 Prompt Details");
    println!("=================");
    println!("Title: {}", prompt.description);
    if let Some(id) = &prompt.id {
        println!("ID: {}", id);
    }

    if let Some(category) = &prompt.category {
        println!("Category: {}", category);
    }

    if let Some(ref tags) = prompt.tag {
        if !tags.is_empty() {
            println!("Tags: {}", tags.join(", "));
        }
    }

    println!("Created: {}", prompt.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", prompt.updated_at.format("%Y-%m-%d %H:%M:%S"));

    println!("\n📄 Content:");
    println!("{}", "-".repeat(50));
    println!("{}", prompt.content);
    println!("{}", "-".repeat(50));
}

fn handle_prompt_execution(prompt: &crate::prompt::Prompt, copy_to_clipboard: bool) -> Result<()> {
    // Render the prompt content (no variable substitution in our simplified version)
    let rendered_content = prompt.content.clone();

    if copy_to_clipboard {
        utils::copy_to_clipboard(&rendered_content)?;
        println!("✓ Prompt copied to clipboard!");
    } else {
        println!("\n📤 Rendered Prompt:");
        println!("{}", "=".repeat(50));
        println!("{}", rendered_content);
        println!("{}", "=".repeat(50));
    }

    Ok(())
}