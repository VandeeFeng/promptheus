use crate::models::{Prompt, PromptCollection, PromptStats};
use crate::config::Config;
use crate::utils::{OutputStyle, search::{SearchEngine, interactive_search_with_external_tool}, stats::StatsCalculator, output::DisplayFormatter};
use crate::cli::ListFormat;
use anyhow::{Context, Result};

pub struct Manager {
    config: Config,
}

impl Manager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    // ========== Data Persistence Methods ==========

    /// Load prompts from file with proper error handling
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

    /// Save prompts to file
    pub fn save_prompts(&self, collection: &PromptCollection) -> Result<()> {
        let content = toml::to_string_pretty(collection)
            .with_context(|| "Failed to serialize prompt collection")?;

        std::fs::write(&self.config.general.prompt_file, content)
            .with_context(|| format!("Failed to write prompt file: {}", self.config.general.prompt_file.display()))?;

        Ok(())
    }

    // ========== CRUD Operations (delegated to models) ==========

    /// Add a new prompt
    pub fn add_prompt(&self, prompt: Prompt) -> Result<()> {
        let mut collection = self.load_prompts()?;
        collection.add_prompt(prompt);
        self.save_prompts(&collection)
    }

    /// Delete a prompt by ID
    pub fn delete_prompt(&self, id: &str) -> Result<()> {
        let mut collection = self.load_prompts()?;
        collection.delete_prompt(id)
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", id))?;
        self.save_prompts(&collection)
    }

  
    /// Find a prompt by ID or description
    pub fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>> {
        let collection = self.load_prompts()?;
        Ok(collection.find(identifier).cloned())
    }

    // ========== Search Operations (delegated to SearchEngine) ==========

    /// Search prompts with query and tag filtering
    pub fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> Result<Vec<Prompt>> {
        let collection = self.load_prompts()?;
        Ok(SearchEngine::search(&collection, query, tag, &self.config))
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts()?;
        Ok(collection.get_all_tags())
    }

    /// Get all unique categories
    pub fn get_categories(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts()?;
        Ok(collection.get_categories())
    }

    /// Get prompt statistics
    pub fn get_prompt_stats(&self) -> Result<PromptStats> {
        let collection = self.load_prompts()?;
        Ok(collection.get_stats())
    }

    // ========== Selection and Formatting Methods ==========

    /// Format prompts for selection with display strings
    pub fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        let collection = self.load_prompts()?;
        Ok(SearchEngine::format_for_selection(&collection, query, tag, category, &self.config))
    }

    /// Find prompt by parsing its display line
    pub fn find_prompt_by_display_line(&self, prompts: &[Prompt], selected_line: &str) -> Result<Option<usize>> {
        Ok(SearchEngine::find_by_display_line(prompts, selected_line))
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

    // ========== Output Formatting Methods (delegated to DisplayFormatter) ==========

    /// Format prompts list according to the specified format
    pub fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> Result<()> {
        DisplayFormatter::format_list(prompts, format, &self.config)
    }

    /// Print prompt statistics
    pub fn print_stats(&self, stats: &PromptStats) -> Result<()> {
        StatsCalculator::print_stats(stats);
        Ok(())
    }

    /// Print tags list
    pub fn print_tags(&self, tags: &[String]) -> Result<()> {
        DisplayFormatter::print_tags(tags)
    }

    /// Print categories list
    pub fn print_categories(&self, categories: &[String]) -> Result<()> {
        DisplayFormatter::print_categories(categories)
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

        if let Some(selected_line) = interactive_search_with_external_tool(
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
}