use colored::*;
use crate::models::Prompt;
use crate::utils::format_datetime;
use crate::config::Config;

/// Display components for a prompt, used for consistent formatting
pub struct PromptDisplay {
    pub description: String,
    pub content_preview: String,
    pub tags_formatted: String,
    pub category_formatted: String,
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

    // Unified prompt display functions
    pub fn print_prompt_basic(prompt: &Prompt) {
        println!("  Description: {}", Self::description(&prompt.description));
        println!("  Content: {}", Self::content(&prompt.content));
        println!("  Created: {}", Self::muted(&format_datetime(&prompt.created_at)));
    }

    pub fn print_prompt_detailed(prompt: &Prompt) {
        println!("{}", Self::title("📝 Prompt Details"));

        if let Some(id) = &prompt.id {
            Self::print_field_colored("ID", id, Self::muted);
        }
        Self::print_field_colored("Description", &prompt.description, Self::content);

        match &prompt.category {
            Some(category) if !category.trim().is_empty() => {
                Self::print_field_colored("Category", category, Self::content);
            }
            _ => {
                Self::print_field_colored("Category", "", Self::content);
            }
        }

        if let Some(ref tags) = prompt.tag {
            if tags.is_empty() {
                Self::print_field_colored("Tags", "", Self::command);
            } else {
                Self::print_field_colored("Tags", &tags.join(", "), Self::command);
            }
        } else {
            Self::print_field_colored("Tags", "", Self::command);
        }

        Self::print_field_colored("Created", &format_datetime(&prompt.created_at), Self::muted);

        println!("\n{}:", Self::title("📄 Content"));
        println!("{}", Self::content(&prompt.content));
    }

    pub fn print_prompt_list_preview(prompt: &Prompt) {
        Self::print_field_colored("Description", &prompt.description, Self::description);
        if let Some(id) = &prompt.id {
            Self::print_field_colored("ID", id, Self::muted);
        }

        if let Some(category) = &prompt.category {
            Self::print_field_colored("Category", category, Self::tag);
        }

        if let Some(ref tags) = prompt.tag && !tags.is_empty() {
            Self::print_field_colored("Tags", &tags.join(", "), Self::tags);
        }

        Self::print_field_colored("Created", &format_datetime(&prompt.created_at), Self::muted);
        Self::print_field_colored("Updated", &format_datetime(&prompt.updated_at), Self::muted);

        // Show preview of content
        let lines: Vec<&str> = prompt.content.lines().take(3).collect();
        if !lines.is_empty() {
            println!("   {}:", Self::label("Preview"));
            for line in lines {
                println!("     {}", Self::content(line));
            }
            if prompt.content.lines().count() > 3 {
                println!("     {}", Self::muted("..."));
            }
        }
    }

    /// Build display components for a prompt
    pub fn build_prompt_display(prompt: &Prompt, config: &Config) -> PromptDisplay {
        let tags_formatted = if let Some(ref tags) = prompt.tag {
            if tags.is_empty() {
                String::new()
            } else {
                format!(" #{}", tags.join(" #"))
            }
        } else {
            String::new()
        };

        let category_formatted = if let Some(cat) = &prompt.category {
            format!(" [{}]", cat)
        } else {
            String::new()
        };

        let content_preview = if config.general.content_preview {
            if prompt.content.len() > 100 {
                format!("{}...", &prompt.content[..100])
            } else {
                prompt.content.clone()
            }
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

        if config.general.content_preview {
            format!("[{}]: {}{}",
                    display.description,
                    display.content_preview,
                    display.tags_formatted + &display.category_formatted
            )
        } else {
            format!("[{}]{}{}",
                    display.description,
                    display.tags_formatted,
                    display.category_formatted
            )
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

        format!("{}{}: {}{}",
                Self::description(&display.description),
                Self::tag(display.category_formatted.trim_start_matches(" [").trim_end_matches("]")),
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

    /// Print rendered prompt content with formatting
    pub fn print_rendered_content(content: &str) {
        println!("\n{}:", Self::header("📤 Rendered Prompt"));
        println!("{}", Self::header_separator());
        println!("{}", Self::content(content));
        println!("{}", Self::header_separator());
    }

    /// Print success message for clipboard operation
    pub fn print_clipboard_success() {
        println!("✓ {}", Self::success("Prompt copied to clipboard!"));
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
