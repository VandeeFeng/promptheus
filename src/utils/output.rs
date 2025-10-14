use colored::*;
use crate::models::Prompt;
use crate::utils::format_datetime;

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
        Self::print_header("📝 Prompt Details");

        Self::print_field_colored("Title", &prompt.description, Self::description);
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

        println!("\n{}:", Self::header("📄 Content"));
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

pub fn print_no_prompts_found() {
    println!("{}", OutputStyle::muted("No prompts found matching the criteria."));
}

pub fn print_error(message: &str) {
    eprintln!("❌ {}", OutputStyle::error(message));
}

pub fn print_warning(message: &str) {
    println!("⚠️  {}", OutputStyle::warning(message));
}

pub fn print_success(message: &str) {
    println!("✅ {}", OutputStyle::success(message));
}
