//! Core operations implementation
//!
//! This module provides the main implementation of all core traits,
//! serving as the central hub for prompt management operations.

use crate::cli::ListFormat;
use crate::config::Config;
use crate::core::{
    data::{Prompt, PromptCollection, PromptStats},
    traits::{PromptCrud, PromptDisplay, PromptInteraction, PromptSearch, PromptStorage},
};
use crate::utils::error::{AppError, AppResult};
use crate::utils::{
    console::{parse_command_variables, prompt_for_variables, replace_command_variables},
    output::DisplayFormatter,
    search::{SearchEngine, interactive_search_with_external_tool},
    stats::StatsCalculator,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Main operations hub that implements all core traits
///
/// This struct serves as the central point for all prompt operations,
/// combining storage, search, display, and interaction capabilities.
pub struct PromptOperations {
    config: Config,
}

impl PromptOperations {
    /// Create a new PromptOperations instance with the given configuration
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Generate deterministic ID based on description and created_at timestamp
    fn generate_deterministic_id(
        description: &str,
        created_at: &chrono::DateTime<chrono::Utc>,
    ) -> String {
        let input = format!("{}{}", description, created_at.to_rfc3339());
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    fn render_prompt_content(&self, prompt: &Prompt) -> AppResult<String> {
        let variables = parse_command_variables(&prompt.content);

        if variables.is_empty() {
            return Ok(prompt.content.clone());
        }

        crate::utils::output::OutputStyle::print_variables_list(&variables);
        let user_values = prompt_for_variables(variables)?;
        Ok(replace_command_variables(&prompt.content, &user_values))
    }

    /// Load prompts with proper error handling and deterministic ID generation
    fn load_prompts_with_ids(&self) -> AppResult<PromptCollection> {
        self.ensure_storage_exists()?;

        let collection = self.load_prompts()?;

        // Ensure all prompts have deterministic IDs
        let mut prompts = Vec::new();
        for mut prompt in collection.prompts {
            if prompt.id.is_none() {
                prompt.id = Some(Self::generate_deterministic_id(
                    &prompt.description,
                    &prompt.created_at,
                ));
            }
            prompts.push(prompt);
        }

        Ok(PromptCollection { prompts })
    }

    /// Save prompts with error handling
    fn save_prompts_internal(&self, collection: &PromptCollection) -> AppResult<()> {
        let content = toml::to_string_pretty(collection).map_err(|e| {
            AppError::System(format!("Failed to serialize prompt collection: {}", e))
        })?;

        std::fs::write(&self.config.general.prompt_file, content).map_err(|e| {
            AppError::Io(format!(
                "Failed to write prompt file: {}: {}",
                self.config.general.prompt_file.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Format prompts for selection with display strings
    pub fn format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> AppResult<Vec<(Prompt, String)>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(SearchEngine::format_for_selection(
            &collection,
            query,
            tag,
            category,
            &self.config,
        ))
    }

    /// Find prompt by parsing its display line
    pub fn find_prompt_by_display_line(
        &self,
        prompts: &[Prompt],
        selected_line: &str,
    ) -> Option<usize> {
        SearchEngine::find_by_display_line(prompts, selected_line)
    }

    /// Search and format prompts for selection (convenience method)
    pub fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> AppResult<Vec<(Prompt, String)>> {
        self.format_for_selection(query, tag, category)
    }

    pub fn get_all_prompts(&self) -> AppResult<Vec<Prompt>> {
        self.search_prompts(None, None)
    }

    pub fn get_all_prompts_or_return_empty(&self) -> AppResult<Vec<Prompt>> {
        let prompts = self.get_all_prompts()?;
        if prompts.is_empty() {
            crate::utils::error::handle_flow(crate::utils::error::FlowResult::EmptyList {
                item_type: "prompts".to_string(),
            });
        }
        Ok(prompts)
    }
}

// Implement PromptStorage trait
impl PromptStorage for PromptOperations {
    fn load_prompts(&self) -> AppResult<PromptCollection> {
        let content = std::fs::read_to_string(&self.config.general.prompt_file).map_err(|e| {
            AppError::Io(format!(
                "Failed to read prompt file: {}: {}",
                self.config.general.prompt_file.display(),
                e
            ))
        })?;

        // Handle empty or invalid TOML files
        if content.trim().is_empty() {
            let default_collection = PromptCollection::default();
            self.save_prompts(&default_collection)?;
            return Ok(default_collection);
        }

        let collection: PromptCollection = toml::from_str(&content)
            .map_err(|e| AppError::System(format!("Failed to parse prompt file: {}", e)))?;

        Ok(collection)
    }

    fn save_prompts(&self, collection: &PromptCollection) -> AppResult<()> {
        self.save_prompts_internal(collection)
    }

    fn ensure_storage_exists(&self) -> AppResult<()> {
        if !self.config.general.prompt_file.exists() {
            if let Some(parent) = self.config.general.prompt_file.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::Io(format!(
                        "Failed to create prompt directory: {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            let default_collection = PromptCollection::default();
            let content = toml::to_string_pretty(&default_collection).map_err(|e| {
                AppError::System(format!("Failed to create default prompt collection: {}", e))
            })?;

            std::fs::write(&self.config.general.prompt_file, content).map_err(|e| {
                AppError::Io(format!(
                    "Failed to create prompt file: {}: {}",
                    self.config.general.prompt_file.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }
}

// Implement PromptSearch trait
impl PromptSearch for PromptOperations {
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> AppResult<Vec<Prompt>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.search(query, tag, &self.config))
    }

    fn find_prompt(&self, identifier: &str) -> AppResult<Option<Prompt>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.find_prompt(identifier).cloned())
    }

    fn get_all_tags(&self) -> AppResult<Vec<String>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_all_tags())
    }

    fn get_categories(&self) -> AppResult<Vec<String>> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_categories())
    }

    fn get_prompt_stats(&self) -> AppResult<PromptStats> {
        let collection = self.load_prompts_with_ids()?;
        Ok(collection.get_stats())
    }
}

// Implement PromptDisplay trait
impl PromptDisplay for PromptOperations {
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> AppResult<()> {
        DisplayFormatter::format_list(prompts, format, &self.config)
    }

    fn format_prompt_for_selection(&self, prompt: &Prompt) -> String {
        crate::utils::output::OutputStyle::format_prompt_for_selection(prompt, &self.config)
    }

    fn print_stats(&self, stats: &PromptStats) -> AppResult<()> {
        StatsCalculator::print_stats(stats);
        Ok(())
    }

    fn print_tags(&self, tags: &[String]) -> AppResult<()> {
        DisplayFormatter::print_tags(tags)
    }

    fn print_categories(&self, categories: &[String]) -> AppResult<()> {
        DisplayFormatter::print_categories(categories)
    }
}

// Implement PromptInteraction trait
impl PromptInteraction for PromptOperations {
    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> AppResult<()> {
        use crate::utils::copy_to_clipboard as copy_fn;

        let rendered_content = self.render_prompt_content(prompt)?;

        if copy_to_clipboard {
            copy_fn(&rendered_content)?;
            crate::utils::print_success("Prompt copied to clipboard!");
        }

        crate::utils::output::OutputStyle::ask_and_display_content(&rendered_content, "📄 Content")
    }

    fn select_interactive_prompts(&self, prompts: Vec<Prompt>) -> AppResult<Option<Prompt>> {
        if prompts.is_empty() {
            return Ok(None);
        }

        // Convert prompts to display strings for selection
        let display_strings: Vec<String> = prompts
            .iter()
            .map(|prompt| self.format_prompt_for_selection(prompt))
            .collect();

        if let Some(selected_line) = interactive_search_with_external_tool(
            &display_strings,
            &self.config.general.select_cmd,
            None,
        )? {
            if let Some(index) = self.find_prompt_by_display_line(&prompts, &selected_line) {
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
    fn add_prompt(&self, prompt: Prompt) -> AppResult<()> {
        let mut collection = self.load_prompts_with_ids()?;
        collection.add_prompt(prompt);
        self.save_prompts(&collection)
    }

    fn delete_prompt(&self, id: &str) -> AppResult<()> {
        let mut collection = self.load_prompts_with_ids()?;
        collection
            .delete_prompt(id)
            .ok_or_else(|| AppError::System(format!("Prompt with ID '{}' not found", id)))?;
        self.save_prompts(&collection)
    }
}
