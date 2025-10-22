pub mod gist;

use crate::utils::error::AppResult;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RemoteSnippet {
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait SyncClient {
    async fn get_remote(&self) -> AppResult<RemoteSnippet>;
    async fn upload(&self, content: String) -> AppResult<()>;
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

pub fn get_github_token() -> Option<String> {
    std::env::var("PROMPTHEUS_GITHUB_ACCESS_TOKEN").ok()
}
