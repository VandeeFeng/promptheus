use colored::*;

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

    pub fn print_field(label: &str, value: &str) {
        println!("{:>12}: {}", Self::label(label), value);
    }

    pub fn print_field_colored(label: &str, value: &str, color_fn: impl Fn(&str) -> ColoredString) {
        println!("{:>12}: {}", Self::label(label), color_fn(value));
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
