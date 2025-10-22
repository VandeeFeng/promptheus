//! Core trait definitions for prompt management
//!
//! These traits define the fundamental operations that can be performed
//! on prompts, providing a unified interface for different implementations.

use crate::cli::ListFormat;
use crate::core::data::{Prompt, PromptCollection, PromptStats};
use crate::utils::error::AppResult;

/// Storage operations for prompts
///
/// This trait defines the interface for loading and saving prompt collections
/// from/to persistent storage.
pub trait PromptStorage {
    /// Load prompts from storage
    fn load_prompts(&self) -> AppResult<PromptCollection>;

    /// Save prompts to storage
    fn save_prompts(&self, collection: &PromptCollection) -> AppResult<()>;

    /// Ensure the storage location exists
    fn ensure_storage_exists(&self) -> AppResult<()>;
}

/// Search operations for prompts
///
/// This trait defines the interface for searching and filtering prompts
/// based on various criteria.
pub trait PromptSearch {
    /// Search prompts with query and tag filtering
    fn search_prompts(&self, query: Option<&str>, tag: Option<&str>) -> AppResult<Vec<Prompt>>;

    /// Find a specific prompt by identifier
    fn find_prompt(&self, identifier: &str) -> AppResult<Option<Prompt>>;

    /// Get all unique tags from the collection
    fn get_all_tags(&self) -> AppResult<Vec<String>>;

    /// Get all unique categories from the collection
    fn get_categories(&self) -> AppResult<Vec<String>>;

    /// Get prompt statistics
    fn get_prompt_stats(&self) -> AppResult<PromptStats>;
}

/// Display formatting operations
///
/// This trait defines the interface for formatting and displaying prompts
/// in various formats and styles.
pub trait PromptDisplay {
    /// Format prompts list according to the specified format
    fn format_list(&self, prompts: &[Prompt], format: &ListFormat) -> AppResult<()>;

    /// Format a single prompt for selection interfaces
    fn format_prompt_for_selection(&self, prompt: &Prompt) -> String;

    /// Print prompt statistics
    fn print_stats(&self, stats: &PromptStats) -> AppResult<()>;

    /// Print tags list
    fn print_tags(&self, tags: &[String]) -> AppResult<()>;

    /// Print categories list
    fn print_categories(&self, categories: &[String]) -> AppResult<()>;
}

/// Interaction operations for user input
///
/// This trait defines the interface for interactive user operations
/// like selecting prompts from lists and getting user input.
pub trait PromptInteraction {
    /// Execute prompt with variable substitution
    fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> AppResult<()>;

    /// Select prompts interactively using standard formatting
    fn select_interactive_prompts(&self, prompts: Vec<Prompt>) -> AppResult<Option<Prompt>>;
}

/// CRUD operations for prompts
///
/// This trait combines storage and search operations for complete
/// Create, Read, Update, Delete functionality.
pub trait PromptCrud: PromptStorage + PromptSearch {
    /// Add a new prompt
    fn add_prompt(&self, prompt: Prompt) -> AppResult<()>;

    /// Delete a prompt by ID
    fn delete_prompt(&self, id: &str) -> AppResult<()>;
}
