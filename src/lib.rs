//! Promptheus - A Rust-based prompt CLI management tool
//!
//! This library provides the core functionality for managing prompts,
//! including storage, searching, display, and interaction capabilities.

pub mod cli;
pub mod manager;
pub mod config;
pub mod core;
pub mod sync;
pub mod utils;

// Re-export core types and traits for easier use
pub use core::{
    traits::{PromptStorage, PromptSearch, PromptDisplay, PromptInteraction},
    data::{Prompt, PromptCollection, PromptStats},
    operations::PromptOperations,
};


/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main library interface for external usage
pub struct Promptheus {
    operations: PromptOperations,
}

impl Promptheus {
    /// Create a new Promptheus instance with the given configuration
    pub fn new(config: config::Config) -> Self {
        Self {
            operations: PromptOperations::new(&config),
        }
    }

    /// Get the underlying operations for direct access
    pub fn operations(&self) -> &PromptOperations {
        &self.operations
    }
}