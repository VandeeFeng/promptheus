use colored::*;
use crate::core::data::Prompt;
use crate::utils::format::{format_datetime, format_tags_comma, format_tags_hash, format_category_info, truncate_string};
use crate::config::Config;
use crate::cli::ListFormat;
use crate::utils::error::{AppResult, AppError};

/// Display components for a prompt, used for consistent formatting
pub struct PromptDisplay {
    pub description: String,
    pub content_preview: String,
    pub tags_formatted: String,
    pub category_formatted: String,
}

/// Prompt field types for unified display
enum PromptField {
    Id,
    Description,
    Category,
    Tags,
    Created,
}

pub struct OutputStyle;

impl OutputStyle {
    // Primary colors for different field types
    pub fn description(text: &str) -> ColoredString {
        text.bright_green()
    }

    pub fn command(text: &str) -> ColoredString {
        text.bright_yellow()
    }

    pub fn content(text: &str) -> ColoredString {
        text.clear()
    }

    pub fn tags(text: &str) -> ColoredString {
        text.bright_cyan()
    }

    pub fn tag(text: &str) -> ColoredString {
        text.cyan()
    }

    pub fn title(text: &str) -> ColoredString {
        text.bright_blue().bold()
    }

    pub fn header(text: &str) -> ColoredString {
        text.bold()
    }

    pub fn label(text: &str) -> ColoredString {
        text.cyan()
    }

    pub fn success(text: &str) -> ColoredString {
        text.green()
    }

    pub fn error(text: &str) -> ColoredString {
        text.red()
    }

    pub fn warning(text: &str) -> ColoredString {
        text.yellow()
    }

    pub fn info(text: &str) -> ColoredString {
        text.blue()
    }

    pub fn muted(text: &str) -> ColoredString {
        text.dimmed()
    }

    // Formatting helpers
    pub fn separator() -> String {
        "─".repeat(50)
    }

    pub fn header_separator() -> String {
        "═".repeat(50)
    }

    pub fn print_header(title: &str) {
        println!("{}", Self::title(title));
        println!("{}", Self::header_separator());
    }

    pub fn print_field_colored(label: &str, value: &str, color_fn: impl Fn(&str) -> ColoredString) {
        println!("{:>12}: {}", Self::label(label), color_fn(value));
    }

    /// Unified prompt field printing function
    fn print_prompt_field(field: PromptField, prompt: &Prompt) {
        match field {
            PromptField::Id => {
                if let Some(id) = &prompt.id {
                    Self::print_field_colored("ID", id, Self::muted);
                }
            }
            PromptField::Description =>
                Self::print_field_colored("Description", &prompt.description, Self::content),
            PromptField::Category => {
                let (cat_str, is_empty) = format_category_info(&prompt.category);
                if is_empty {
                    Self::print_field_colored("Category", "", Self::content);
                } else {
                    Self::print_field_colored("Category", &cat_str, Self::content);
                }
            }
            PromptField::Tags => {
                let tags_str = format_tags_comma(&prompt.tag);
                if tags_str.is_empty() {
                    Self::print_field_colored("Tags", "", Self::command);
                } else {
                    Self::print_field_colored("Tags", &tags_str, Self::command);
                }
            }
            PromptField::Created =>
                Self::print_field_colored("Created", &format_datetime(&prompt.created_at), Self::muted),
        }
    }

    /// Print all prompt metadata fields (ID, Description, Category, Tags, Created)
    pub fn print_prompt_metadata(prompt: &Prompt) {
        Self::print_prompt_field(PromptField::Id, prompt);
        Self::print_prompt_field(PromptField::Description, prompt);
        Self::print_prompt_field(PromptField::Category, prompt);
        Self::print_prompt_field(PromptField::Tags, prompt);
        Self::print_prompt_field(PromptField::Created, prompt);
    }

    /// Print basic prompt metadata fields (Description, Created)
    pub fn print_prompt_metadata_basic(prompt: &Prompt) {
        println!("  Description: {}", Self::description(&prompt.description));
        println!("  Created: {}", Self::muted(&format_datetime(&prompt.created_at)));
    }

    // Unified prompt display functions
    pub fn print_prompt_basic(prompt: &Prompt) {
        println!("{}", Self::title("📝 Prompt Details"));
        Self::print_prompt_metadata_basic(prompt);

        println!("\n{}:", Self::title("📄 Content"));
        Self::print_content_full(&prompt.content);
    }

    pub fn print_prompt_list_preview(prompt: &Prompt) {
        Self::print_field_colored("Description", &prompt.description, Self::description);
        Self::print_prompt_field(PromptField::Id, prompt);

        // Category with tag color
        let (cat_str, _) = format_category_info(&prompt.category);
        if !cat_str.is_empty() {
            Self::print_field_colored("Category", &cat_str, Self::tag);
        }

        // Tags with tags color
        let tags_str = format_tags_comma(&prompt.tag);
        if !tags_str.is_empty() {
            Self::print_field_colored("Tags", &tags_str, Self::tags);
        }

        Self::print_prompt_field(PromptField::Created, prompt);

        // Show truncated content for list preview
        if !prompt.content.trim().is_empty() {
            println!("   {}:", Self::label("Preview"));
            Self::print_content_truncated(&prompt.content);
        }
    }

    /// Build display components for a prompt
    pub fn build_prompt_display(prompt: &Prompt, config: &Config) -> PromptDisplay {
        let tags_formatted = format_tags_hash(&prompt.tag);

        let category_formatted = if let Some(cat) = &prompt.category {
            format!(" [{}]", cat)
        } else {
            String::new()
        };

        let content_preview = if config.general.content_preview {
            truncate_string(&prompt.content, 100)
        } else {
            String::new()
        };

        PromptDisplay {
            description: prompt.description.clone(),
            content_preview,
            tags_formatted,
            category_formatted,
        }
    }

    /// Format a prompt for display in selection interfaces (fzf, search results)
    pub fn format_prompt_for_selection(prompt: &Prompt, config: &Config) -> String {
        let display = Self::build_prompt_display(prompt, config);

        let first_line = format!("[{}]:{}{}",
                                display.description,
                                display.category_formatted,
                                display.tags_formatted
        );

        if config.general.content_preview && !display.content_preview.is_empty() {
            format!("{}\n{}", first_line, display.content_preview)
        } else {
            first_line
        }
    }

    /// Format a prompt for simple list display
    pub fn format_prompt_line(prompt: &Prompt, config: &Config) -> String {
        let display = Self::build_prompt_display(prompt, config);

        let content_display = if config.general.content_preview {
            display.content_preview
        } else {
            String::new()
        };

        let category_display = if let Some(category) = &prompt.category {
            if category.trim().is_empty() {
                Self::muted("[] ")
            } else {
                Self::tag(&format!("[{}] ", category))
            }
        } else {
            Self::muted("[] ")
        };

        format!("{}: {}{}{}",
                Self::description(&display.description),
                category_display,
                display.tags_formatted,
                Self::content(&content_display)
        )
    }

    /// Print variables found in prompt content
    pub fn print_variables_list(variables: &[(String, Option<String>)]) {
        println!("\n🔧 {}:", Self::header("This prompt contains variables"));
        for (name, default) in variables {
            if let Some(default_val) = default {
                println!("  <{}={}>", Self::command(&format!("<{}>", name)), Self::muted(default_val));
            } else {
                println!("  {}", Self::command(&format!("<{}>", name)));
            }
        }
    }

    /// Print full prompt content
    pub fn print_content_full(content: &str) {
        println!("{}", Self::content(content));
    }

    /// Print truncated prompt content (beginning + [...] + end)
    pub fn print_content_truncated(content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() <= 10 {
            // If content is short, show full content
            println!("{}", Self::content(content));
        } else {
            // Show first 5 lines
            for line in lines.iter().take(5) {
                println!("{}", Self::content(line));
            }
            // Show truncation indicator
            println!("{}", Self::muted("[...]"));
            // Show last 5 lines
            for line in lines.iter().skip(lines.len() - 5) {
                println!("{}", Self::content(line));
            }
        }
    }

    /// Ask user about pagination and display content accordingly
    pub fn ask_and_display_content(content: &str, title: &str) -> AppResult<()> {
        use std::io::{self, Write};
        use crate::utils::{get_terminal_size, should_paginate, paginate_static_content};

        // Check if content should be paginated
        let (_, terminal_height) = get_terminal_size().unwrap_or((24, 80));

        if should_paginate(content, terminal_height) {
            // First, show title and truncated content
            println!("\n{}:", Self::title(title));
            Self::print_content_truncated(content);

            // Then ask user if they want to view the full content in pager
            print!("\n{} View {} in pager? (y/N): ", Self::info("?"), title);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                let content_display = format!("\n{}:\n{}",
                                              Self::title(title),
                                              content
                );
                paginate_static_content(&content_display)?;
            }
        } else {
            // Show full content for short content
            println!("\n{}:", Self::title(title));
            Self::print_content_full(content);
        }

        Ok(())
    }

    /// Display complete prompt with metadata and content (handles all logic internally)
    pub fn display_prompt_complete(prompt: &Prompt) -> AppResult<()> {
        // Show prompt details header
        println!("{}", Self::title("📝 Prompt Details"));

        // Show metadata
        Self::print_prompt_metadata(prompt);

        // Show content with pagination if needed
        Self::ask_and_display_content(&prompt.content, "📄 Content")?;

        Ok(())
    }

}

// Utility functions for common patterns
pub fn print_prompt_count(count: usize) {
    if count == 0 {
        println!("{}", OutputStyle::muted("No prompts found."));
    } else {
        println!("📝 {} ({} found)",
                 OutputStyle::header("Prompts"),
                 OutputStyle::info(&count.to_string())
        );
    }
}


pub fn print_warning(message: &str) {
    println!("⚠️  {}", OutputStyle::warning(message));
}

pub fn print_success(message: &str) {
    println!("✅ {}", OutputStyle::success(message));
}

/// Display formatter for different output formats
pub struct DisplayFormatter;

impl DisplayFormatter {
    /// Format prompts list according to the specified format
    pub fn format_list(prompts: &[Prompt], format: &ListFormat, config: &Config) -> AppResult<()> {
        if prompts.is_empty() {
            crate::utils::error::handle_flow(crate::utils::error::FlowResult::EmptyList {
                item_type: "prompts matching your criteria".to_string(),
            });
            return Ok(());
        }

        match format {
            ListFormat::Simple => Self::print_simple_list(prompts, config),
            ListFormat::Detailed => Self::print_detailed_list(prompts),
            ListFormat::Table => Self::print_table_list(prompts, config),
            ListFormat::Json => Self::print_json_list(prompts)?,
        }

        Ok(())
    }

    /// Print tags list
    pub fn print_tags(tags: &[String]) -> AppResult<()> {
        if tags.is_empty() {
            crate::utils::error::handle_flow(crate::utils::error::FlowResult::EmptyList {
                item_type: "tags".to_string(),
            });
            return Ok(());
        }

        println!("🏷️  Available Tags ({})", tags.len());
        println!("====================");
        for tag in tags {
            println!("  {}", tag);
        }

        Ok(())
    }

    /// Print categories list
    pub fn print_categories(categories: &[String]) -> AppResult<()> {
        if categories.is_empty() {
            crate::utils::error::handle_flow(crate::utils::error::FlowResult::EmptyList {
                item_type: "categories".to_string(),
            });
            return Ok(());
        }

        println!("📁 Available Categories ({})", categories.len());
        println!("=========================");
        for category in categories {
            println!("  {}", category);
        }

        Ok(())
    }

    /// Print simple list format
    fn print_simple_list(prompts: &[Prompt], config: &Config) {
        crate::utils::print_prompt_count(prompts.len());
        println!("{}", OutputStyle::separator());

        for prompt in prompts {
            // Show basic info line
            let formatted_line = OutputStyle::format_prompt_line(prompt, config);
            println!("{}", formatted_line);

            // Show truncated content if content exists and is configured for preview
            if config.general.content_preview && !prompt.content.trim().is_empty() {
                OutputStyle::print_content_truncated(&prompt.content);
                println!("{}", OutputStyle::separator());
            }
        }
    }

    /// Print detailed list format
    fn print_detailed_list(prompts: &[Prompt]) {
        OutputStyle::print_header("📝 Detailed Prompt List");

        for (i, prompt) in prompts.iter().enumerate() {
            println!("\n{}. {}", i + 1, OutputStyle::description(&prompt.description));
            OutputStyle::print_prompt_list_preview(prompt);

            if i < prompts.len() - 1 {
                println!("{}", OutputStyle::separator());
            }
        }
    }

    /// Print table format
    fn print_table_list(prompts: &[Prompt], _config: &Config) {
        crate::utils::print_prompt_count(prompts.len());

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
            let description = truncate_string(&prompt.description, max_title_width);
            let tag_str = format_tags_comma(&prompt.tag);
            let tag_str = truncate_string(&tag_str, max_tag_width);

            println!("│ {:<width_title$} │ {:<width_tags$} │ {} │",
                     OutputStyle::description(&description),
                     OutputStyle::tags(&tag_str),
                     OutputStyle::muted(&format_datetime(&prompt.updated_at)),
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

    /// Print JSON format
    fn print_json_list(prompts: &[Prompt]) -> AppResult<()> {
        let json = serde_json::to_string_pretty(prompts)
            .map_err(|e| AppError::System(format!("Failed to serialize prompts to JSON: {}", e)))?;
        println!("{}", json);
        Ok(())
    }
}
