use super::{RemoteSnippet, SyncClient, get_github_token};
use crate::config::GistConfig;
use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Serialize, Deserialize)]
struct Gist {
    id: String,
    description: Option<String>,
    public: bool,
    created_at: String,
    updated_at: String,
    files: HashMap<String, GistFile>,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GistFile {
    filename: Option<String>,
    content: Option<String>,
    size: i64,
    raw_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateGistRequest {
    description: String,
    public: bool,
    files: HashMap<String, GistFileContent>,
}

#[derive(Debug, Serialize)]
struct GistFileContent {
    content: String,
}

#[derive(Debug, Serialize)]
struct UpdateGistRequest {
    description: Option<String>,
    files: HashMap<String, GistFileContent>,
}

pub struct GistClient {
    client: Client,
    config: GistConfig,
    access_token: String,
}

impl GistClient {
    pub fn new(config: GistConfig) -> AppResult<Self> {
        // Try to get access token from config first, then environment
        let access_token = config.access_token
            .clone()
            .or_else(get_github_token)
            .ok_or_else(|| {
                AppError::System("GitHub access token not found. Set it in config or use PROMPTHEUS_GITHUB_ACCESS_TOKEN environment variable".to_string())
            })?;

        Ok(Self {
            client: Client::builder()
                .user_agent("promptheus/0.1.0")
                .build()
                .map_err(|e| AppError::Network(format!("Failed to create HTTP client: {}", e)))?,
            config,
            access_token,
        })
    }

    fn parse_gist_timestamp(&self, timestamp_str: &str) -> AppResult<DateTime<Utc>> {
        let parsed = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|e| AppError::System(format!("Failed to parse gist timestamp: {}", e)))?;
        Ok(parsed.with_timezone(&Utc))
    }

    async fn get_gist(&self) -> AppResult<Gist> {
        let gist_id = self
            .config
            .gist_id
            .as_ref()
            .ok_or_else(|| AppError::Sync("No Gist ID configured".to_string()))?;

        let url = format!("{}/gists/{}", GITHUB_API_BASE, gist_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to fetch gist from GitHub: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "Failed to get gist: {} - {}",
                status, error_text
            )));
        }

        let gist: Gist = response
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Failed to parse gist response: {}", e)))?;

        Ok(gist)
    }

    async fn create_gist(&self, content: String) -> AppResult<String> {
        let url = format!("{}/gists", GITHUB_API_BASE);

        let mut files = HashMap::new();
        files.insert(self.config.file_name.clone(), GistFileContent { content });

        let request = CreateGistRequest {
            description: "Promptheus snippets".to_string(),
            public: self.config.public,
            files,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to create gist: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "Failed to create gist: {} - {}",
                status, error_text
            )));
        }

        let gist: Gist = response.json().await.map_err(|e| {
            AppError::Network(format!("Failed to parse create gist response: {}", e))
        })?;

        Ok(gist.id)
    }

    async fn update_gist(&self, content: String) -> AppResult<()> {
        let gist_id = self
            .config
            .gist_id
            .as_ref()
            .ok_or_else(|| AppError::Sync("No Gist ID configured".to_string()))?;

        let url = format!("{}/gists/{}", GITHUB_API_BASE, gist_id);

        let mut files = HashMap::new();
        files.insert(self.config.file_name.clone(), GistFileContent { content });

        let request = UpdateGistRequest {
            description: Some("Promptheus snippets".to_string()),
            files,
        };

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.access_token)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to update gist: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "Failed to update gist: {} - {}",
                status, error_text
            )));
        }

        Ok(())
    }

    async fn get_gist_content(&self) -> AppResult<(String, DateTime<Utc>)> {
        let gist = self.get_gist().await?;

        // Find the target file
        let gist_file = gist.files.get(&self.config.file_name).ok_or_else(|| {
            AppError::Sync(format!(
                "File '{}' not found in gist",
                self.config.file_name
            ))
        })?;

        let content = gist_file
            .content
            .as_ref()
            .ok_or_else(|| AppError::Sync("File content is empty".to_string()))?
            .clone();

        let updated_at = self.parse_gist_timestamp(&gist.updated_at)?;

        Ok((content, updated_at))
    }
}

#[async_trait]
impl SyncClient for GistClient {
    async fn get_remote(&self) -> AppResult<RemoteSnippet> {
        let (content, updated_at) = self.get_gist_content().await?;
        Ok(RemoteSnippet {
            content,
            updated_at,
        })
    }

    async fn upload(&self, content: String) -> AppResult<()> {
        if self.config.gist_id.is_none() {
            // Create new gist
            let gist_id = self.create_gist(content).await?;
            println!("✅ Created new gist: {}", gist_id);
            println!(
                "💡 Add this gist ID to your config file: gist_id = \"{}\"",
                gist_id
            );
        } else {
            // Update existing gist
            self.update_gist(content).await?;
            println!("✅ Updated existing gist");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gist_timestamp() {
        let client = GistClient {
            client: Client::new(),
            config: GistConfig {
                file_name: "test.toml".to_string(),
                access_token: Some("test".to_string()),
                gist_id: Some("test".to_string()),
                public: false,
                auto_sync: false,
            },
            access_token: "test".to_string(),
        };

        // This test would require a real timestamp string
        // In a real scenario, you'd test with actual RFC3339 timestamps
        let timestamp = "2023-01-01T00:00:00Z";
        let result = client.parse_gist_timestamp(timestamp);
        assert!(result.is_ok());
    }
}
