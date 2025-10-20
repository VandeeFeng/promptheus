//! Core operations implementation
//!
//! This module provides the main implementation of all core traits,
//! serving as the central hub for prompt management operations.

use anyhow::{Context, Result};
use crate::core::{
    data::{Prompt, PromptCollection, PromptStats},
    traits::{PromptStorage, PromptSearch, PromptDisplay, PromptInteraction, PromptCrud},
};
use crate::config::Config;
use crate::cli::ListFormat;
use crate::utils::{
    search::{SearchEngine, interactive_search_with_external_tool},
    stats::StatsCalculator,
    output::DisplayFormatter,
    command::{parse_command_variables, prompt_for_variables, replace_command_variables},
};

/// Main operations hub that implements all core traits
///
/// This struct serves as the central point for all prompt operations,
/// combining storage, search, display, and interaction capabilities.
pub struct PromptOperations {
    config: Config,
}

impl PromptOperations {
    /// Create a new PromptOperations instance with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    
    /// Load prompts with proper error handling and ID generation
    fn load_prompts_with_ids(&self) -> Result<PromptCollection> {
        self.ensure_storage_exists()?;

        let collection = self.load_prompts()?;

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

    /// Save prompts with error handling
    fn save_prompts_internal(&self, collection: &PromptCollection) -> Result<()> {
        let content = toml::to_string_pretty(collection)
            .with_context(|| "Failed to serialize prompt collection")?;

        std::fs::write(&self.config.general.prompt_file, content)
            .with_context(|| format!("Failed to write prompt file: {}", self.config.general.prompt_file.display()))?;

        Ok(())
    }

    /// Format prompts for selection with display strings
    pub fn format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(SearchEngine::format_for_selection(&collection, query, tag, category, &self.config))
    }

    /// Find prompt by parsing its display line
    pub fn find_prompt_by_display_line(&self, prompts: &[Prompt], selected_line: &str) -> Option<usize> {
        SearchEngine::find_by_display_line(prompts, selected_line)
    }

    /// Search and format prompts for selection (convenience method)
    pub fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        self.format_for_selection(query, tag, category)
    }
}

// Implement PromptStorage trait
impl PromptStorage for PromptOperations {
    fn load_prompts(&self) -> Result<PromptCollection> {
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

        Ok(collection)
    }

    fn save_prompts(&self, collection: &PromptCollection) -> Result<()> {
        self.save_prompts_internal(collection)
    }

    fn ensure_storage_exists(&self) -> Result<()> {
        if !self.config.general.prompt_file.exists() {
            if let Some(parent) = self.config.general.prompt_file.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create prompt directory: {}", parent.display()))?;
            }

            let default_collection = PromptCollection::default();
            let content = toml::to_string_pretty(&default_collection)
                .with_context(|| "Failed to create default prompt collection")?;

            std::fs::write(&self.config.general.prompt_file, content)
                .with_context(|| format!("Failed to create prompt file: {}", self.config.general.prompt_file.display()))?;
        }

        Ok(())
    }
}

// Implement PromptSearch trait
impl PromptSearch for PromptOperations {
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> Result<Vec<Prompt>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.search(query, tag, &self.config))
    }

    fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.find(identifier).cloned())
    }

    fn get_all_tags(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_all_tags())
    }

    fn get_categories(&self) -> Result<Vec<String>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_categories())
    }

    fn get_prompt_stats(&self) -> Result<PromptStats> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_stats())
    }
}

// Implement PromptDisplay trait
impl PromptDisplay for PromptOperations {
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> Result<()> {
        DisplayFormatter::format_list(prompts, format, &self.config)
    }

    fn format_prompt_for_selection(&self, prompt: &Prompt) -> String {
        crate::utils::output::OutputStyle::format_prompt_for_selection(prompt, &self.config)
    }

    fn print_stats(&self, stats: &PromptStats) -> Result<()> {
        StatsCalculator::print_stats(stats);
        Ok(())
    }

    fn print_tags(&self, tags: &[String]) -> Result<()> {
        DisplayFormatter::print_tags(tags)
    }

    fn print_categories(&self, categories: &[String]) -> Result<()> {
        DisplayFormatter::print_categories(categories)
    }
}

// Implement PromptInteraction trait
impl PromptInteraction for PromptOperations {
    
    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()> {
        use crate::utils::copy_to_clipboard as copy_fn;

        let variables = parse_command_variables(&prompt.content);

        let rendered_content = if variables.is_empty() {
            prompt.content.clone()
        } else {
            crate::utils::output::OutputStyle::print_variables_list(&variables);
            let user_values = prompt_for_variables(variables)?;
            replace_command_variables(&prompt.content, &user_values)
        };

        if copy_to_clipboard {
            copy_fn(&rendered_content)?;
            crate::utils::output::OutputStyle::print_clipboard_success();
        } else {
            crate::utils::output::OutputStyle::print_rendered_content(&rendered_content);
        }

        Ok(())
    }

    fn select_interactive_prompts(&self, prompts: Vec<Prompt>) -> Result<Option<Prompt>> {
        if prompts.is_empty() {
            return Ok(None);
        }

        // Convert prompts to display strings for selection
        let display_strings: Vec<String> = prompts.iter()
            .map(|prompt| self.format_prompt_for_selection(prompt))
            .collect();

        if let Some(selected_line) = interactive_search_with_external_tool(
            &display_strings,
            &self.config.general.select_cmd,
            None
        )? {
            if let Some(index) = display_strings.iter().position(|d| d == &selected_line) {
                Ok(Some(prompts[index].clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None) // User cancelled
        }
    }
}

// Implement PromptCrud trait
impl PromptCrud for PromptOperations {
    fn add_prompt(&self, prompt: Prompt) -> Result<()> {
        let mut collection = self.load_prompts_with_ids()?;
        collection.add_prompt(prompt);
        self.save_prompts(&collection)
    }

    
    fn delete_prompt(&self, id: &str) -> Result<()> {
        let mut collection = self.load_prompts_with_ids()?;
        collection.delete_prompt(id)
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", id))?;
        self.save_prompts(&collection)
    }
}