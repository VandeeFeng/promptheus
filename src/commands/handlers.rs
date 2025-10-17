use anyhow::Result;
use crate::models::Prompt;
use crate::config::Config;
use crate::utils::{handle_empty_list, interactive_search_with_external_tool, OutputStyle, print_prompt_count, format_datetime};
use crate::cli::ListFormat;
use std::collections::HashMap;

// Import Manager for trait implementations
use crate::manager::Manager;

/// Statistics about prompts
#[derive(Debug)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub total_tags: usize,
    pub total_categories: usize,
    pub tag_counts: HashMap<String, usize>,
    pub category_counts: HashMap<String, usize>,
}

/// Unified interface for prompt operations
pub trait PromptOperations {
    /// Search prompts with filters
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>, category: Option<&str>) -> Result<Vec<Prompt>>;

    /// Find prompt by identifier (ID or description)
    fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>>;

    /// Add a new prompt
    fn add_prompt(&self, prompt: Prompt) -> Result<()>;

    /// Delete a prompt by ID
    fn delete_prompt(&self, id: &str) -> Result<()>;

    /// Get all tags
    fn get_all_tags(&self) -> Result<Vec<String>>;

    /// Get all categories
    fn get_categories(&self) -> Result<Vec<String>>;

    /// Get prompt statistics
    fn get_stats(&self) -> Result<PromptStats>;

    /// Execute prompt with variable substitution
    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()>;

    /// Search and format prompts for selection
    fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>>;
}

/// Trait for interactive selection operations
pub trait InteractiveSelector {
    /// Select an item from a list using interactive selection
    fn select_interactive<T>(
        &self,
        items: Vec<T>,
        formatter: impl Fn(&T) -> String + Copy,
        config: &Config,
    ) -> Result<Option<T>>
    where
        T: Clone;

    /// Select prompts interactively using standard formatting
    fn select_interactive_prompts(
        &self,
        prompts: Vec<Prompt>,
        config: &Config,
    ) -> Result<Option<Prompt>> {
        if prompts.is_empty() {
            return Ok(None);
        }

        self.select_interactive(
            prompts,
            crate::utils::OutputStyle::format_prompt_for_interactive_selection,
            config,
        )
    }

    /// Find prompt by parsing its display line
    fn find_prompt_by_display_line(&self, prompts: &[Prompt], selected_line: &str) -> Result<Option<usize>>;
}

/// Trait for output formatting operations
pub trait OutputFormatter {
    /// Format prompts list according to the specified format
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat, config: &Config) -> Result<()>;

    /// Print prompt statistics
    fn print_stats(&self, stats: &PromptStats) -> Result<()>;

    /// Print tags list
    fn print_tags(&self, tags: &[String]) -> Result<()>;

    /// Print categories list
    fn print_categories(&self, categories: &[String]) -> Result<()>;
}


/// Default implementation of OutputFormatter
pub struct DefaultOutputFormatter;

impl OutputFormatter for DefaultOutputFormatter {
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat, config: &Config) -> Result<()> {
        if prompts.is_empty() {
            handle_empty_list("prompts matching your criteria");
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

    fn print_stats(&self, stats: &PromptStats) -> Result<()> {
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

    fn print_tags(&self, tags: &[String]) -> Result<()> {
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

    fn print_categories(&self, categories: &[String]) -> Result<()> {
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
}

// Helper functions for DefaultOutputFormatter
impl DefaultOutputFormatter {
    fn print_simple_list(prompts: &[Prompt], config: &Config) {
        print_prompt_count(prompts.len());
        println!("{}", OutputStyle::separator());

        for prompt in prompts {
            let formatted_line = OutputStyle::format_prompt_line(prompt, config);
            println!("{}", formatted_line);
        }
    }

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

    fn print_table_list(prompts: &[Prompt], _config: &Config) {
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

    fn print_json_list(prompts: &[Prompt]) -> Result<()> {
        let json = serde_json::to_string_pretty(prompts)
            .map_err(|e| anyhow::anyhow!("Failed to serialize prompts to JSON: {}", e))?;
        println!("{}", json);
        Ok(())
    }
}

// Implement traits for Manager
impl PromptOperations for Manager {
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>, category: Option<&str>) -> Result<Vec<Prompt>> {
        let prompts = Manager::search_prompts(self, query, tag)?;

        // Filter by category if specified
        let filtered_prompts: Vec<_> = if let Some(category) = &category {
            prompts.into_iter()
                .filter(|p| p.category.as_deref() == Some(*category))
                .collect()
        } else {
            prompts
        };

        Ok(filtered_prompts)
    }

    fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>> {
        Manager::find_prompt(self, identifier)
    }

    fn add_prompt(&self, prompt: Prompt) -> Result<()> {
        Manager::add_prompt(self, prompt)
    }

    fn delete_prompt(&self, id: &str) -> Result<()> {
        Manager::delete_prompt(self, id)
    }

    fn get_all_tags(&self) -> Result<Vec<String>> {
        Manager::get_all_tags(self)
    }

    fn get_categories(&self) -> Result<Vec<String>> {
        Manager::get_categories(self)
    }

    fn get_stats(&self) -> Result<PromptStats> {
        Manager::get_prompt_stats(self)
    }

    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()> {
        Manager::execute_prompt(self, prompt, copy_to_clipboard)
    }

    fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        Manager::search_and_format_for_selection(self, query, tag, category)
    }
}

impl InteractiveSelector for Manager {
    fn select_interactive<T>(
        &self,
        items: Vec<T>,
        formatter: impl Fn(&T) -> String + Copy,
        config: &Config,
    ) -> Result<Option<T>>
    where
        T: Clone,
    {
        if items.is_empty() {
            return Ok(None);
        }

        let display_strings: Vec<String> = items.iter()
            .map(formatter)
            .collect();

        if let Some(selected_line) = interactive_search_with_external_tool(
            &display_strings,
            &config.general.select_cmd,
            None
        )? {
            if let Some(index) = display_strings.iter().position(|d| d == &selected_line) {
                Ok(Some(items[index].clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None) // User cancelled
        }
    }

    fn find_prompt_by_display_line(&self, prompts: &[Prompt], selected_line: &str) -> Result<Option<usize>> {
        Manager::find_prompt_by_display_line(self, prompts, selected_line)
    }
}


