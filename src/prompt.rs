use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "Tag")]
    pub tag: Option<Vec<String>>,
    #[serde(rename = "Output")]
    pub output: Option<String>,
    #[serde(rename = "Created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "Updated_at")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "Category")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        } else if let Some(ref mut tags) = self.tag {
            if !tags.contains(&tag) {
                tags.push(tag);
                self.updated_at = Utc::now();
            }
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(ref mut tags) = self.tag {
            if let Some(pos) = tags.iter().position(|t| t == tag) {
                tags.remove(pos);
                self.updated_at = Utc::now();
            }
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

impl Default for PromptCollection {
    fn default() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }
}