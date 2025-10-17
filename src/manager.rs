use crate::models::{Prompt, PromptCollection};
use crate::config::Config;
use crate::utils::OutputStyle;
use crate::cli::ListFormat;
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Statistics about prompts
#[derive(Debug)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub total_tags: usize,
    pub total_categories: usize,
    pub tag_counts: HashMap<String, usize>,
    pub category_counts: HashMap<String, usize>,
}

pub struct Manager {
    config: Config,
}

impl Manager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn load_prompts(&self) -> Result<PromptCollection> {
        self.config.ensure_prompt_file_exists()?;

        let content = std::fs::read_to_string(&self.config.general.prompt_file)
            .with_context(|| format!("Failed to read prompt file: {}", self.config.general.prompt_file.display()))?;

        // Handle empty or invalid TOML files
        if content.trim().is_empty() {
            let default_collection = PromptCollection::default();
            self.save_prompts(&default_collection)?;
            return Ok(default_collection);
        }

        let collection: PromptCollection = toml::from_str(&content)
            .with_context(|| "Failed to parse prompt file")?;

        // Ensure all prompts have IDs
        let mut prompts = Vec::new();
        for mut prompt in collection.prompts {
            if prompt.id.is_none() {
                prompt.id = Some(uuid::Uuid::new_v4().to_string());
            }
            prompts.push(prompt);
        }

        Ok(PromptCollection { prompts })
    }

    pub fn save_prompts(&self, collection: &PromptCollection) -> Result<()> {
        let content = toml::to_string_pretty(collection)
            .with_context(|| "Failed to serialize prompt collection")?;

        std::fs::write(&self.config.general.prompt_file, content)
            .with_context(|| format!("Failed to write prompt file: {}", self.config.general.prompt_file.display()))?;

        Ok(())
    }

    pub fn add_prompt(&self, prompt: Prompt) -> Result<()> {
        let mut collection = self.load_prompts()?;
        collection.prompts.push(prompt);
        self.save_prompts(&collection)?;
        Ok(())
    }


    pub fn delete_prompt(&self, id: &str) -> Result<()> {
        let mut collection = self.load_prompts()?;

        collection.prompts.retain(|p| p.id.as_ref() != Some(&id.to_string()));
        self.save_prompts(&collection)?;
        Ok(())
    }

    pub fn find_prompt_by_id(&self, id: &str) -> Result<Option<Prompt>> {
        let collection = self.load_prompts()?;
        Ok(collection.prompts.into_iter().find(|p| p.id.as_ref() == Some(&id.to_string())))
    }

    pub fn find_prompt_by_description(&self, description: &str) -> Result<Option<Prompt>> {
        let collection = self.load_prompts()?;
        Ok(collection.prompts.into_iter().find(|p| p.description == description))
    }

    pub fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>> {
        // First try to find by ID
        if let Some(prompt) = self.find_prompt_by_id(identifier)? {
            return Ok(Some(prompt));
        }

        // If not found by ID, try to find by description
        self.find_prompt_by_description(identifier)
    }

    pub fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> Result<Vec<Prompt>> {
        let collection = self.load_prompts()?;
        let mut prompts = collection.prompts;

        // Filter by query
        if let Some(q) = query {
            let search_query = if self.config.general.search_case_sensitive {
                q.to_string()
            } else {
                q.to_lowercase()
            };

            prompts.retain(|p| {
                    let description = if self.config.general.search_case_sensitive {
                        p.description.clone()
                    } else {
                        p.description.to_lowercase()
                    };

                    let content = if self.config.general.search_case_sensitive {
                        p.content.clone()
                    } else {
                        p.content.to_lowercase()
                    };

                    let tags_match = p.tag.iter().flatten().any(|t| {
                        let tag_str = if self.config.general.search_case_sensitive {
                            t.clone()
                        } else {
                            t.to_lowercase()
                        };
                        tag_str.contains(&search_query)
                    });

                    description.contains(&search_query) ||
                    content.contains(&search_query) ||
                    tags_match
                });
        }

        // Filter by tag
        if let Some(t) = tag {
            prompts.retain(|p| p.tag.iter().flatten().any(|tag| tag == &t.to_string()));
        }

        // Sort prompts
        match self.config.general.sort_by {
            crate::config::SortBy::Recency => {
                prompts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            crate::config::SortBy::Title => {
                prompts.sort_by(|a, b| a.description.cmp(&b.description));
            }
            crate::config::SortBy::Description => {
                prompts.sort_by(|a, b| a.description.cmp(&b.description));
            }
            crate::config::SortBy::Updated => {
                prompts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
        }

        Ok(prompts)
    }

    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts()?;
        let mut tags: Vec<String> = collection.prompts
            .iter()
            .flat_map(|p| p.tag.iter().flatten().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        Ok(tags)
    }

    pub fn get_categories(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts()?;
        let mut categories: Vec<String> = collection.prompts
            .into_iter()
            .filter_map(|p| p.category)
            .filter(|cat| !cat.is_empty())
            .collect();
        categories.sort();
        categories.dedup();
        Ok(categories)
    }

    pub fn get_prompt_stats(&self) -> Result<PromptStats> {
        let collection = self.load_prompts()?;
        let total_prompts = collection.prompts.len();
        let total_tags = collection.prompts.iter()
            .map(|p| p.tag.iter().len())
            .sum();
        let total_categories = collection.prompts.iter()
            .filter(|p| p.category.is_some())
            .count();

        let mut tag_counts = HashMap::new();
        let mut category_counts = HashMap::new();

        for prompt in &collection.prompts {
            if let Some(ref tags) = prompt.tag {
                for tag in tags {
                    *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                }
            }

            if let Some(ref category) = prompt.category {
                *category_counts.entry(category.clone()).or_insert(0) += 1;
            }
        }

        Ok(PromptStats {
            total_prompts,
            total_tags,
            total_categories,
            tag_counts,
            category_counts,
        })
    }

    /// Format prompts for selection with display strings
    pub fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        let prompts = self.search_prompts(query, tag)?;

        // Filter by category if specified
        let filtered_prompts: Vec<_> = if let Some(category) = &category {
            prompts.into_iter()
                .filter(|p| p.category.as_deref() == Some(*category))
                .collect()
        } else {
            prompts
        };

        let mut result = Vec::new();

        for prompt in filtered_prompts {
            let display_string = OutputStyle::format_prompt_for_selection(&prompt, &self.config);
            result.push((prompt, display_string));
        }

        Ok(result)
    }

    /// Find prompt by parsing its display line
    pub fn find_prompt_by_display_line(&self, prompts: &[Prompt], selected_line: &str) -> Result<Option<usize>> {
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

    /// Execute prompt with variable substitution
    pub fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()> {
        use crate::utils::{parse_command_variables, prompt_for_variables, replace_command_variables, copy_to_clipboard as copy_fn};

        let variables = parse_command_variables(&prompt.content);

        let rendered_content = if variables.is_empty() {
            prompt.content.clone()
        } else {
            OutputStyle::print_variables_list(&variables);
            let user_values = prompt_for_variables(variables)?;
            replace_command_variables(&prompt.content, &user_values)
        };

        if copy_to_clipboard {
            copy_fn(&rendered_content)?;
            OutputStyle::print_clipboard_success();
        } else {
            OutputStyle::print_rendered_content(&rendered_content);
        }

        Ok(())
    }

    // ========== Output Formatting Methods ==========

    /// Format prompts list according to the specified format
    pub fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> Result<()> {
        if prompts.is_empty() {
            crate::utils::handle_empty_list("prompts matching your criteria");
            return Ok(());
        }

        match format {
            ListFormat::Simple => self.print_simple_list(prompts),
            ListFormat::Detailed => self.print_detailed_list(prompts),
            ListFormat::Table => self.print_table_list(prompts),
            ListFormat::Json => self.print_json_list(prompts)?,
        }

        Ok(())
    }

    /// Print prompt statistics
    pub fn print_stats(&self, stats: &PromptStats) -> Result<()> {
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

    /// Print tags list
    pub fn print_tags(&self, tags: &[String]) -> Result<()> {
        if tags.is_empty() {
            crate::utils::handle_empty_list("tags");
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
    pub fn print_categories(&self, categories: &[String]) -> Result<()> {
        if categories.is_empty() {
            crate::utils::handle_empty_list("categories");
            return Ok(());
        }

        println!("📁 Available Categories ({})", categories.len());
        println!("=======================");
        for category in categories {
            println!("  {}", category);
        }

        Ok(())
    }

    // ========== Interactive Selection Methods ==========

    /// Select an item from a list using interactive selection
    pub fn select_interactive<T>(
        &self,
        items: Vec<T>,
        formatter: impl Fn(&T) -> String + Copy,
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

        if let Some(selected_line) = crate::utils::interactive_search_with_external_tool(
            &display_strings,
            &self.config.general.select_cmd,
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

    /// Select prompts interactively using standard formatting
    pub fn select_interactive_prompts(&self, prompts: Vec<Prompt>) -> Result<Option<Prompt>> {
        if prompts.is_empty() {
            return Ok(None);
        }

        self.select_interactive(
            prompts,
            OutputStyle::format_prompt_for_interactive_selection,
        )
    }

    // ========== Private Helper Methods for Formatting ==========

    fn print_simple_list(&self, prompts: &[Prompt]) {
        crate::utils::print_prompt_count(prompts.len());
        println!("{}", OutputStyle::separator());

        for prompt in prompts {
            let formatted_line = OutputStyle::format_prompt_line(prompt, &self.config);
            println!("{}", formatted_line);
        }
    }

    fn print_detailed_list(&self, prompts: &[Prompt]) {
        OutputStyle::print_header("📝 Detailed Prompt List");

        for (i, prompt) in prompts.iter().enumerate() {
            println!("\n{}. {}", i + 1, OutputStyle::description(&prompt.description));
            OutputStyle::print_prompt_list_preview(prompt);

            if i < prompts.len() - 1 {
                println!("{}", OutputStyle::separator());
            }
        }
    }

    fn print_table_list(&self, prompts: &[Prompt]) {
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

    fn print_json_list(&self, prompts: &[Prompt]) -> Result<()> {
        let json = serde_json::to_string_pretty(prompts)
            .map_err(|e| anyhow::anyhow!("Failed to serialize prompts to JSON: {}", e))?;
        println!("{}", json);
        Ok(())
    }
}