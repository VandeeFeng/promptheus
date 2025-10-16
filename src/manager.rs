use crate::models::{Prompt, PromptCollection};
use crate::config::Config;
use anyhow::{Context, Result};
use std::collections::HashMap;

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
            .with_context(|| format!("Failed to read prompt file: {:?}", self.config.general.prompt_file))?;

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
            .with_context(|| format!("Failed to write prompt file: {:?}", self.config.general.prompt_file))?;

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

    pub fn get_all_prompts(&self) -> Result<Vec<Prompt>> {
        let collection = self.load_prompts()?;
        Ok(collection.prompts)
    }

    pub fn get_prompt_stats(&self) -> Result<PromptStats> {
        let collection = self.load_prompts()?;
        let total_prompts = collection.prompts.len();
        let total_tags = collection.prompts.iter()
            .map(|p| p.tag.iter().map(|tags| tags.len()).sum::<usize>())
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
}

#[derive(Debug)]
pub struct PromptStats {
    pub total_prompts: usize,
    pub total_tags: usize,
    pub total_categories: usize,
    pub tag_counts: HashMap<String, usize>,
    pub category_counts: HashMap<String, usize>,
}