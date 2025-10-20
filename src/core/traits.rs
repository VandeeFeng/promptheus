//! Core trait definitions for prompt management
//!
//! These traits define the fundamental operations that can be performed
//! on prompts, providing a unified interface for different implementations.

use anyhow::Result;
use crate::cli::ListFormat;
use crate::core::data::{Prompt, PromptCollection, PromptStats};

/// Storage operations for prompts
///
/// This trait defines the interface for loading and saving prompt collections
/// from/to persistent storage.
pub trait PromptStorage {
    /// Load prompts from storage
    fn load_prompts(&self) -> Result<PromptCollection>;

    /// Save prompts to storage
    fn save_prompts(&self, collection: &PromptCollection) -> Result<()>;

    /// Ensure the storage location exists
    fn ensure_storage_exists(&self) -> Result<()>;
}

/// Search operations for prompts
///
/// This trait defines the interface for searching and filtering prompts
/// based on various criteria.
pub trait PromptSearch {
    /// Search prompts with query and tag filtering
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> Result<Vec<Prompt>>;

    /// Find a specific prompt by identifier
    fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>>;

    /// Get all unique tags from the collection
    fn get_all_tags(&self) -> Result<Vec<String>>;

    /// Get all unique categories from the collection
    fn get_categories(&self) -> Result<Vec<String>>;

    /// Get prompt statistics
    fn get_prompt_stats(&self) -> Result<PromptStats>;
}

/// Display formatting operations
///
/// This trait defines the interface for formatting and displaying prompts
/// in various formats and styles.
pub trait PromptDisplay {
    /// Format prompts list according to the specified format
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> Result<()>;

    /// Format a single prompt for selection interfaces
    fn format_prompt_for_selection(&self, prompt: &Prompt) -> String;

    /// Print prompt statistics
    fn print_stats(&self, stats: &PromptStats) -> Result<()>;

    /// Print tags list
    fn print_tags(&self, tags: &[String]) -> Result<()>;

    /// Print categories list
    fn print_categories(&self, categories: &[String]) -> Result<()>;
}

/// Interaction operations for user input
///
/// This trait defines the interface for interactive user operations
/// like selecting prompts from lists and getting user input.
pub trait PromptInteraction {
    /// Execute prompt with variable substitution
    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()>;

    /// Select prompts interactively using standard formatting
    fn select_interactive_prompts(&self, prompts: Vec<Prompt>) -> Result<Option<Prompt>>;
}

/// CRUD operations for prompts
///
/// This trait combines storage and search operations for complete
/// Create, Read, Update, Delete functionality.
pub trait PromptCrud: PromptStorage + PromptSearch {
    /// Add a new prompt
    fn add_prompt(&self, prompt: Prompt) -> Result<()>;

    /// Delete a prompt by ID
    fn delete_prompt(&self, id: &str) -> Result<()>;
}
