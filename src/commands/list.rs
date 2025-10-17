use crate::cli::{ListArgs, ListFormat};
use crate::config::Config;
use crate::models::PromptService;
use anyhow::{Context, Result};

use crate::utils::{OutputStyle, print_prompt_count, handle_empty_list};

pub fn handle_list_command(
    config: Config,
    args: &ListArgs,
) -> Result<()> {
    let prompt_service = PromptService::new(config.clone());

    // Handle tags listing
    if args.tags {
        return handle_tags_command(config);
    }

    // Handle categories listing
    if args.categories {
        return handle_categories_command(config);
    }

    if args.stats {
        return show_stats(&prompt_service);
    }

    let search_results = prompt_service.search_and_format_for_selection(None, args.tag.as_deref(), args.category.as_deref())?;

    if search_results.is_empty() {
        handle_empty_list("prompts matching your criteria");
        return Ok(());
    }

    let (prompts, _): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    let format = args.format.as_ref().unwrap_or(&ListFormat::Simple);

    match format {
        ListFormat::Simple => print_simple_list(&prompts, &config),
        ListFormat::Detailed => print_detailed_list(&prompts),
        ListFormat::Table => print_table_list(&prompts, &config),
        ListFormat::Json => print_json_list(&prompts)?,
    }

    Ok(())
}

fn show_stats(prompt_service: &PromptService) -> Result<()> {
    let stats = prompt_service.get_stats()?;

    OutputStyle::print_header("📊 Prompt Statistics");

    OutputStyle::print_field_colored("Total prompts", &stats.total_prompts.to_string(), OutputStyle::info);
    OutputStyle::print_field_colored("Total tags", &stats.total_tags.to_string(), OutputStyle::info);
    OutputStyle::print_field_colored("Categories used", &stats.total_categories.to_string(), OutputStyle::info);

    if !stats.tag_counts.is_empty() {
        println!("\n🏷️  {}:", OutputStyle::header("Most used tags"));
        let mut sorted_tags: Vec<_> = stats.tag_counts.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in sorted_tags.iter().take(10) {
            println!("  {}: {}", OutputStyle::tags(tag), OutputStyle::info(&count.to_string()));
        }
    }

    if !stats.category_counts.is_empty() {
        println!("\n📁 {}:", OutputStyle::header("Categories"));
        let mut sorted_categories: Vec<_> = stats.category_counts.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(a.1));

        for (category, count) in sorted_categories {
            println!("  {}: {}", OutputStyle::tag(category), OutputStyle::info(&count.to_string()));
        }
    }

    Ok(())
}

fn print_simple_list(prompts: &[crate::models::Prompt], config: &Config) {
    print_prompt_count(prompts.len());
    println!("{}", OutputStyle::separator());

    for prompt in prompts {
        let formatted_line = OutputStyle::format_prompt_line(prompt, config);
        println!("{}", formatted_line);
    }
}

fn print_detailed_list(prompts: &[crate::models::Prompt]) {
    OutputStyle::print_header("📝 Detailed Prompt List");

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n{}. {}", i + 1, OutputStyle::description(&prompt.description));
        OutputStyle::print_prompt_list_preview(prompt);

        if i < prompts.len() - 1 {
            println!("{}", OutputStyle::separator());
        }
    }
}

fn print_table_list(prompts: &[crate::models::Prompt], _config: &Config) {
    print_prompt_count(prompts.len());

    // Calculate column widths
    let mut max_title_width = 15; // Minimum width for "Description"
    let mut max_tag_width = 10;    // Minimum width for "Tags"

    for prompt in prompts {
        max_title_width = max_title_width.max(prompt.description.len());
        let tag_str = prompt.tag.iter().flatten().cloned().collect::<Vec<_>>().join(", ");
        max_tag_width = max_tag_width.max(tag_str.len());
    }

    // Limit column widths to reasonable size
    max_title_width = max_title_width.min(60);
    max_tag_width = max_tag_width.min(25);

    // Print header with colors
    println!("┌─{}─┬─{}─┬─{}─┐",
        "─".repeat(max_title_width),
        "─".repeat(max_tag_width),
        "─".repeat(19) // Date column
    );
    println!("│ {:<width_title$} │ {:<width_tags$} │ {:^19} │",
        OutputStyle::header("Description"),
        OutputStyle::header("Tags"),
        OutputStyle::header("Updated"),
        width_title = max_title_width,
        width_tags = max_tag_width
    );
    println!("├─{}─┼─{}─┼─{}─┤",
        "─".repeat(max_title_width),
        "─".repeat(max_tag_width),
        "─".repeat(19)
    );

    // Print rows with colors
    for prompt in prompts {
        let description = if prompt.description.len() > max_title_width {
            format!("{}...", &prompt.description[..max_title_width.saturating_sub(3)])
        } else {
            prompt.description.clone()
        };

        let tag_str = if let Some(ref tags) = prompt.tag {
            if tags.is_empty() {
                String::new()
            } else {
                let tag_string = tags.join(", ");
                if tag_string.len() > max_tag_width {
                    format!("{}...", &tag_string[..max_tag_width.saturating_sub(3)])
                } else {
                    tag_string
                }
            }
        } else {
            String::new()
        };

        println!("│ {:<width_title$} │ {:<width_tags$} │ {} │",
            OutputStyle::description(&description),
            OutputStyle::tags(&tag_str),
            OutputStyle::muted(&crate::utils::format_datetime(&prompt.updated_at)),
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

fn print_json_list(prompts: &[crate::models::Prompt]) -> Result<()> {
    let json = serde_json::to_string_pretty(prompts)
        .context("Failed to serialize prompts to JSON")?;
    println!("{}", json);
    Ok(())
}

pub fn handle_tags_command(config: Config) -> Result<()> {
    let prompt_service = PromptService::new(config);
    let tags = prompt_service.get_all_tags()?;

    if tags.is_empty() {
        handle_empty_list("tags");
        return Ok(());
    }

    println!("🏷️  Available Tags ({})", tags.len());
    println!("====================");
    for tag in tags {
        println!("  {}", tag);
    }

    Ok(())
}

pub fn handle_categories_command(config: Config) -> Result<()> {
    let prompt_service = PromptService::new(config);
    let categories = prompt_service.get_categories()?;

    if categories.is_empty() {
        handle_empty_list("categories");
        return Ok(());
    }

    println!("📁 Available Categories ({})", categories.len());
    println!("=======================");
    for category in categories {
        println!("  {}", category);
    }

    Ok(())
}
