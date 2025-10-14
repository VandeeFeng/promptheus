use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::utils::time_format;


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
    #[serde(with = "time_format")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "Updated_at")]
    #[serde(with = "time_format")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "Category")]
    pub category: Option<String>,
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

}

impl Default for PromptCollection {
    fn default() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }
}
