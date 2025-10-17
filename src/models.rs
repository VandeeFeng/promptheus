use serde::{Deserialize, Serialize, Serializer};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::utils::time_format;
use crate::config::Config;
use crate::manager::Manager;
use anyhow::Result;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "Category", serialize_with = "serialize_category")]
    pub category: Option<String>,
    #[serde(rename = "Tag", serialize_with = "serialize_tag")]
    pub tag: Option<Vec<String>>,
    #[serde(rename = "Output")]
    pub output: Option<String>,
    #[serde(rename = "Created_at")]
    #[serde(with = "time_format")]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptCollection {
    pub prompts: Vec<Prompt>,
}

impl Prompt {
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

    pub fn add_tag(&mut self, tag: String) {
        if self.tag.is_none() {
            self.tag = Some(vec![tag]);
        } else if let Some(ref mut tags) = self.tag
            && !tags.contains(&tag) {
                tags.push(tag);
                self.updated_at = Utc::now();
            }
    }

}


// Custom serialization functions to always include tag and category fields
fn serialize_tag<S>(tag: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match tag {
        Some(tags) => serializer.serialize_some(&tags),
        None => serializer.serialize_some(&Vec::<String>::new()),
    }
}

fn serialize_category<S>(category: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match category {
        Some(cat) => serializer.serialize_some(cat),
        None => serializer.serialize_some(&String::new()),
    }
}

/// Service for handling prompt operations with unified logic
pub struct PromptService {
    manager: Manager,
    config: Config,
}

impl PromptService {
    pub fn new(config: Config) -> Self {
        Self {
            manager: Manager::new(config.clone()),
            config,
        }
    }

    /// Search prompts and return formatted display strings for selection
    pub fn search_and_format_for_selection(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<(Prompt, String)>> {
        let prompts = self.manager.search_prompts(query, tag)?;

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
            let display_string = crate::utils::OutputStyle::format_prompt_for_selection(&prompt, &self.config);
            result.push((prompt, display_string));
        }

        Ok(result)
    }

    /// Execute a prompt with variable substitution
    pub fn execute_prompt(&self, prompt: &Prompt, copy_to_clipboard: bool) -> Result<()> {
        use crate::utils::{parse_command_variables, prompt_for_variables, replace_command_variables, OutputStyle};
        use crate::utils::copy_to_clipboard as copy_to_clipboard_fn;

        let variables = parse_command_variables(&prompt.content);

        let rendered_content = if variables.is_empty() {
            // No variables, just use the content as-is
            prompt.content.clone()
        } else {
            // Prompt user for variable values
            OutputStyle::print_variables_list(&variables);
            let user_values = prompt_for_variables(variables)?;
            replace_command_variables(&prompt.content, &user_values)
        };

        if copy_to_clipboard {
            copy_to_clipboard_fn(&rendered_content)?;
            OutputStyle::print_clipboard_success();
        } else {
            OutputStyle::print_rendered_content(&rendered_content);
        }

        Ok(())
    }

    /// Find prompt by parsing its display line
    pub fn find_prompt_by_display_line(
        &self,
        prompts: &[Prompt],
        selected_line: &str
    ) -> Result<Option<usize>> {
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

    /// Get prompt statistics
    pub fn get_stats(&self) -> Result<crate::manager::PromptStats> {
        self.manager.get_prompt_stats()
    }

    /// Get all tags
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        self.manager.get_all_tags()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Result<Vec<String>> {
        self.manager.get_categories()
    }

    /// Find a prompt by identifier (ID or description)
    pub fn find_prompt(&self, identifier: &str) -> Result<Option<Prompt>> {
        self.manager.find_prompt(identifier)
    }
}
