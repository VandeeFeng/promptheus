use serde::{Deserialize, Serialize, Serializer};
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
