//! Core data structures for prompt management
//!
//! This module contains the fundamental data structures used throughout
//! the Promptheus application.

use crate::config::{Config, SortBy};
use crate::utils::format;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "Category", serialize_with = "format::serialize_category")]
    pub category: Option<String>,
    #[serde(rename = "Tag", serialize_with = "format::serialize_tag")]
    pub tag: Option<Vec<String>>,
    #[serde(rename = "Output")]
    pub output: Option<String>,
    #[serde(rename = "Created_at")]
    #[serde(with = "format")]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

/// Collection of prompts with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCollection {
    pub prompts: Vec<Prompt>,
}

/// Statistics about the prompt collection
#[derive(Debug)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub total_tags: usize,
    pub total_categories: usize,
    pub tag_counts: HashMap<String, usize>,
    pub category_counts: HashMap<String, usize>,
}

impl Prompt {
    /// Create a new prompt with the given description and content
    pub fn new(description: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Some(Uuid::new_v4().to_string()),
            description,
            content,
            tag: None,
            output: None,
            created_at: now,
            updated_at: now,
            category: None,
        }
    }

    /// Add a tag to the prompt if it doesn't already exist
    pub fn add_tag(&mut self, tag: String) {
        if self.tag.is_none() {
            self.tag = Some(vec![tag]);
        } else if let Some(ref mut tags) = self.tag
            && !tags.contains(&tag)
        {
            tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
}

impl PromptCollection {
    /// Create a new empty prompt collection
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    /// Add a new prompt to the collection
    pub fn add_prompt(&mut self, prompt: Prompt) {
        self.prompts.push(prompt);
    }

    /// Delete a prompt by ID or description (smart delete)
    pub fn delete_prompt(&mut self, identifier: &str) -> Option<Prompt> {
        if let Some(prompt) = self.find_prompt(identifier) {
            // Find the index of the found prompt to remove it
            if let Some(index) = self
                .prompts
                .iter()
                .position(|p| p.id == prompt.id && p.description == prompt.description)
            {
                return Some(self.prompts.remove(index));
            }
        }
        None
    }

    /// Find a prompt by ID
    pub fn find_by_id(&self, id: &str) -> Option<&Prompt> {
        self.prompts
            .iter()
            .find(|p| p.id.as_ref() == Some(&id.to_string()))
    }

    /// Find a prompt by description
    pub fn find_by_description(&self, description: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.description == description)
    }

    /// Find a prompt by ID or description
    pub fn find_prompt(&self, identifier: &str) -> Option<&Prompt> {
        if let Some(prompt) = self.find_by_id(identifier) {
            return Some(prompt);
        }
        self.find_by_description(identifier)
    }

    /// Search prompts with query and tag filtering
    pub fn search(&self, query: Option<&str>, tag: Option<&str>, config: &Config) -> Vec<Prompt> {
        let mut prompts = self.prompts.clone();

        // Filter by query
        if let Some(q) = query {
            let search_query = if config.general.search_case_sensitive {
                q.to_string()
            } else {
                q.to_lowercase()
            };

            prompts.retain(|p| {
                let description = if config.general.search_case_sensitive {
                    p.description.clone()
                } else {
                    p.description.to_lowercase()
                };

                let content = if config.general.search_case_sensitive {
                    p.content.clone()
                } else {
                    p.content.to_lowercase()
                };

                let tags_match = p.tag.iter().flatten().any(|t| {
                    let tag_str = if config.general.search_case_sensitive {
                        t.clone()
                    } else {
                        t.to_lowercase()
                    };
                    tag_str.contains(&search_query)
                });

                description.contains(&search_query) || content.contains(&search_query) || tags_match
            });
        }

        // Filter by tag
        if let Some(t) = tag {
            prompts.retain(|p| p.tag.iter().flatten().any(|tag| tag == &t.to_string()));
        }

        // Sort prompts
        match config.general.sort_by {
            SortBy::Recency => {
                prompts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortBy::Title => {
                prompts.sort_by(|a, b| a.description.cmp(&b.description));
            }
            SortBy::Description => {
                prompts.sort_by(|a, b| a.description.cmp(&b.description));
            }
            SortBy::Updated => {
                prompts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
        }

        prompts
    }

    /// Get all unique tags from the collection
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .prompts
            .iter()
            .flat_map(|p| p.tag.iter().flatten().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Get all unique categories from the collection
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .prompts
            .clone()
            .into_iter()
            .filter_map(|p| p.category)
            .filter(|cat| !cat.is_empty())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Calculate statistics for the collection
    pub fn get_stats(&self) -> PromptStats {
        let total_prompts = self.prompts.len();
        let total_tags = self.prompts.iter().map(|p| p.tag.iter().len()).sum();
        let total_categories = self.prompts.iter().filter(|p| p.category.is_some()).count();

        let mut tag_counts = HashMap::new();
        let mut category_counts = HashMap::new();

        for prompt in &self.prompts {
            if let Some(ref tags) = prompt.tag {
                for tag in tags {
                    *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                }
            }

            if let Some(ref category) = prompt.category {
                *category_counts.entry(category.clone()).or_insert(0) += 1;
            }
        }

        PromptStats {
            total_prompts,
            total_tags,
            total_categories,
            tag_counts,
            category_counts,
        }
    }
}

impl Default for PromptCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(category) = &self.category {
            write!(f, "{} [{}]", self.description, category)
        } else {
            write!(f, "{}", self.description)
        }
    }
}
