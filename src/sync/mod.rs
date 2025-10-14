pub mod gist;

use anyhow::Result;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RemoteSnippet {
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait SyncClient {
    /// Get remote snippet content and metadata
    async fn get_remote(&self) -> Result<RemoteSnippet>;

    /// Upload local content to remote service
    async fn upload(&self, content: String) -> Result<()>;
}

/// Determine if sync should happen based on timestamps and force flag
pub fn should_sync(local_updated: DateTime<Utc>, remote_updated: DateTime<Utc>, force: bool) -> SyncDirection {
    if force {
        return SyncDirection::Upload;
    }

    if local_updated > remote_updated {
        SyncDirection::Upload
    } else if remote_updated > local_updated {
        SyncDirection::Download
    } else {
        SyncDirection::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncDirection {
    Upload,
    Download,
    None,
}

/// Get GitHub access token from environment variable
pub fn get_github_token() -> Option<String> {
    std::env::var("PROMPTHEUS_GITHUB_ACCESS_TOKEN")
        .or_else(|_| std::env::var("PET_GITHUB_ACCESS_TOKEN"))
        .ok()
}