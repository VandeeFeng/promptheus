use crate::cli::{ListArgs, ListFormat};
use crate::config::Config;
use crate::storage::Storage;
use anyhow::{Context, Result};

use crate::utils::format_datetime;

pub fn handle_list_command(
    config: Config,
    args: &ListArgs,
) -> Result<()> {
    let storage = Storage::new(config.clone());

    if args.stats {
        return show_stats(&storage);
    }

    let prompts = storage.search_prompts(None, args.tag.as_deref())?;

    if prompts.is_empty() {
        println!("No prompts found.");
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
        println!("No prompts found matching the criteria.");
        return Ok(());
    }

    let format = args.format.as_ref().unwrap_or(&ListFormat::Simple);

    match format {
        ListFormat::Simple => print_simple_list(&filtered_prompts, &config),
        ListFormat::Detailed => print_detailed_list(&filtered_prompts),
        ListFormat::Table => print_table_list(&filtered_prompts, &config),
        ListFormat::Json => print_json_list(&filtered_prompts)?,
    }

    Ok(())
}

fn show_stats(storage: &Storage) -> Result<()> {
    let stats = storage.get_prompt_stats()?;

    println!("📊 Prompt Statistics");
    println!("==================");
    println!("Total prompts: {}", stats.total_prompts);
    println!("Total tags: {}", stats.total_tags);
    println!("Categories used: {}", stats.total_categories);

    if !stats.tag_counts.is_empty() {
        println!("\n🏷️  Most used tags:");
        let mut sorted_tags: Vec<_> = stats.tag_counts.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in sorted_tags.iter().take(10) {
            println!("  {}: {}", tag, count);
        }
    }

    if !stats.category_counts.is_empty() {
        println!("\n📁 Categories:");
        let mut sorted_categories: Vec<_> = stats.category_counts.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(a.1));

        for (category, count) in sorted_categories {
            println!("  {}: {}", category, count);
        }
    }

    Ok(())
}

fn print_simple_list(prompts: &[crate::prompt::Prompt], config: &Config) {
    println!("📝 Prompts ({} found)", prompts.len());
    println!("{}", "=".repeat(50));

    for prompt in prompts {
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

        // Show content preview if enabled, otherwise show full content
        let content_display = if config.general.content_preview {
            if prompt.content.len() > 100 {
                format!("{}...", &prompt.content[..100])
            } else {
                prompt.content.clone()
            }
        } else {
            prompt.content.clone()
        };

        println!("{}{}{}: {}",
            prompt.description,
            category,
            tags,
            content_display
        );
    }
}

fn print_detailed_list(prompts: &[crate::prompt::Prompt]) {
    println!("📝 Detailed Prompt List");
    println!("=======================");

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n{}. {}", i + 1, prompt.description);
        if let Some(id) = &prompt.id {
            println!("   ID: {}", id);
        }

        if let Some(category) = &prompt.category {
            println!("   Category: {}", category);
        }

        if let Some(ref tags) = prompt.tag
            && !tags.is_empty() {
                println!("   Tags: {}", tags.join(", "));
            }

        println!("   Created: {}", format_datetime(&prompt.created_at));
        println!("   Updated: {}", format_datetime(&prompt.updated_at));

        // Show preview of content
        let lines: Vec<&str> = prompt.content.lines().take(3).collect();
        if !lines.is_empty() {
            println!("   Preview:");
            for line in lines {
                println!("     {}", line);
            }
            if prompt.content.lines().count() > 3 {
                println!("     ...");
            }
        }

        if i < prompts.len() - 1 {
            println!("{}", "-".repeat(50));
        }
    }
}

fn print_table_list(prompts: &[crate::prompt::Prompt], _config: &Config) {

    println!("📝 Prompts ({} found)", prompts.len());

    // Calculate column widths
    let mut max_title_width = 10; // "Title" length
    let mut max_tag_width = 8;     // "Tags" length

    for prompt in prompts {
        max_title_width = max_title_width.max(prompt.description.len());

        let tag_str = prompt.tag.iter().flatten().cloned().collect::<Vec<_>>().join(", ");
        max_tag_width = max_tag_width.max(tag_str.len());
    }

    // Limit column widths
    max_title_width = max_title_width.min(50);
    max_tag_width = max_tag_width.min(20);

    // Print header
    println!("┌─{}─┬─{}─┬─{}─┐",
        "─".repeat(max_title_width),
        "─".repeat(max_tag_width),
        "─".repeat(19) // Date column
    );
    println!("│ {:<width_title$} │ {:<width_tags$} │ {:^19} │",
        "Description", "Tags", "Updated",
        width_title = max_title_width,
        width_tags = max_tag_width
    );
    println!("├─{}─┼─{}─┼─{}─┤",
        "─".repeat(max_title_width),
        "─".repeat(max_tag_width),
        "─".repeat(19)
    );

    // Print rows
    for prompt in prompts {
        let description = if prompt.description.len() > max_title_width {
            format!("{}...", &prompt.description[..max_title_width-3])
        } else {
            prompt.description.clone()
        };

        let tag_str = if let Some(ref tags) = prompt.tag {
            if tags.is_empty() {
                String::new()
            } else {
                let tag_string = tags.join(", ");
                if tag_string.len() > max_tag_width {
                    format!("{}...", &tag_string[..max_tag_width-3])
                } else {
                    tag_string
                }
            }
        } else {
            String::new()
        };

        println!("│ {:<width_title$} │ {:<width_tags$} │ {} │",
            description,
            tag_str,
            format_datetime(&prompt.updated_at),
            width_title = max_title_width,
            width_tags = max_tag_width
        );
    }

    println!("└─{}─┴─{}─┴─{}─┘",
        "─".repeat(max_title_width),
        "─".repeat(max_tag_width),
        "─".repeat(19)
    );
}

fn print_json_list(prompts: &[crate::prompt::Prompt]) -> Result<()> {
    let json = serde_json::to_string_pretty(prompts)
        .context("Failed to serialize prompts to JSON")?;
    println!("{}", json);
    Ok(())
}
