use crate::cli::SyncArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::sync::{gist::GistClient, SyncClient, should_sync, SyncDirection};
use crate::utils::{print_warning, print_network_error};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use std::io::{self, Write};

/// Check if an error is likely network-related and provide appropriate user feedback
fn handle_potential_network_error(error: &anyhow::Error) -> Result<()> {
    let error_msg = error.to_string().to_lowercase();

    // Check for common network-related error indicators
    if error_msg.contains("network") ||
       error_msg.contains("connection") ||
       error_msg.contains("timeout") ||
       error_msg.contains("dns") ||
       error_msg.contains("unreachable") ||
       error_msg.contains("refused") ||
       error_msg.contains("host") ||
       error_msg.contains("ssl") ||
       error_msg.contains("certificate") ||
       error_msg.contains("tcp") ||
       error_msg.contains("http") {
        print_network_error(&format!("Request failed: {}. Please check your internet connection and try again.", error));
    }

    // Still return the original error so the calling code can handle it
    Err(anyhow::Error::msg(error.to_string()))
}

pub async fn handle_sync_command(config: Config, args: &SyncArgs) -> Result<()> {
    // Check if any sync backend is configured
    let _gist_config = config.gist.as_ref()
        .ok_or_else(|| anyhow!("No sync backend configured. Please configure Gist or GitLab in your config."))?;

    println!("🔄 Starting sync process...");

    // Create storage instance
    let storage = Manager::new(config.clone());

    // Load local prompts
    let local_prompts = storage.load_prompts()
        .context("Failed to load local prompts")?;

    // Get the most recent local update time
    let local_updated = local_prompts.prompts
        .iter()
        .map(|p| p.updated_at)
        .max()
        .unwrap_or_else(Utc::now);

    // Create sync client
    let sync_client: Box<dyn SyncClient> = if let Some(gist_config) = &config.gist {
        Box::new(GistClient::new(gist_config.clone())?)
    } else {
        return Err(anyhow!("No supported sync backend configured"));
    };

    // Get remote snippet
    println!("📥 Fetching remote content...");
    let remote_snippet = sync_client.get_remote().await
        .context("Failed to fetch remote content")
        .map_err(|e| handle_potential_network_error(&e).unwrap_err())?;

    // Determine sync direction
    let sync_direction = should_sync(local_updated, remote_snippet.updated_at, args.force);

    match sync_direction {
        SyncDirection::Upload => {
            if !args.download {
                upload_to_remote(&storage, &*sync_client, &local_prompts).await?;
            } else {
                print_warning("Both upload and download specified. Downloading takes precedence.");
                download_from_remote(&storage, &remote_snippet).await?;
            }
        }
        SyncDirection::Download => {
            if !args.upload {
                download_from_remote(&storage, &remote_snippet).await?;
            } else {
                print_warning("Both upload and download specified. Uploading takes precedence.");
                upload_to_remote(&storage, &*sync_client, &local_prompts).await?;
            }
        }
        SyncDirection::None => {
            println!("✅ Local and remote are already in sync.");
            if args.force {
                println!("🔧 Force flag specified. No action needed.");
            }
        }
    }

    Ok(())
}

async fn upload_to_remote(
    _storage: &Manager,
    sync_client: &dyn SyncClient,
    local_prompts: &crate::models::PromptCollection,
) -> Result<()> {
    print!("📤 Uploading local changes to remote... ");
    io::stdout().flush()?;

    // Serialize local prompts to TOML
    let content = toml::to_string_pretty(local_prompts)
        .context("Failed to serialize local prompts")?;

    // Upload to remote
    sync_client.upload(content).await
        .context("Failed to upload to remote")
        .map_err(|e| handle_potential_network_error(&e).unwrap_err())?;

    println!("✅ Done");
    Ok(())
}

async fn download_from_remote(
    storage: &Manager,
    remote_snippet: &crate::sync::RemoteSnippet,
) -> Result<()> {
    print!("📥 Downloading remote changes... ");
    io::stdout().flush()?;

    // Parse remote content
    let remote_prompts: crate::models::PromptCollection = toml::from_str(&remote_snippet.content)
        .context("Failed to parse remote content")?;

    // Save remote prompts locally
    storage.save_prompts(&remote_prompts)
        .context("Failed to save remote prompts locally")?;

    println!("✅ Done");
    Ok(())
}

pub async fn auto_sync_if_enabled(config: &Config) -> Result<()> {
    // Check if auto-sync is enabled
    let gist_config = if let Some(gist) = &config.gist {
        gist
    } else {
        return Ok(()); // No sync configured, nothing to do
    };

    if !gist_config.auto_sync {
        return Ok(()); // Auto-sync disabled
    }

    // Check if local file exists and has content
    let prompt_file_path = &config.general.prompt_file;

    // Check if file exists and is not empty
    if !prompt_file_path.exists() || tokio::fs::metadata(prompt_file_path).await?.len() == 0 {
        println!("🔄 Local file is empty or missing, downloading from remote...");

        // Download from remote
        let sync_args = SyncArgs {
            upload: false,
            download: true,
            force: false,
        };

        return handle_sync_command(config.clone(), &sync_args).await
            .context("Auto-sync download failed");
    }

    // Get local file modification time
    let local_metadata = tokio::fs::metadata(prompt_file_path).await
        .context("Failed to get local file metadata")?;
    let local_modified = local_metadata.modified()
        .context("Failed to get local file modification time")?
        .into();

    // Create sync client to get remote info
    let sync_client: Box<dyn SyncClient> = Box::new(GistClient::new(gist_config.clone())?);

    // Get remote snippet info
    let remote_snippet = sync_client.get_remote().await
        .context("Failed to fetch remote content")?;

    // Compare timestamps to determine if sync is needed
    let should_sync = match should_sync(local_modified, remote_snippet.updated_at, false) {
        SyncDirection::Upload => {
            println!("🔄 Local changes detected, uploading to remote...");
            true
        }
        SyncDirection::Download => {
            println!("🔄 Remote changes detected, downloading...");
            true
        }
        SyncDirection::None => {
            // No sync needed, but let's verify content is the same
            let local_content = tokio::fs::read_to_string(prompt_file_path).await
                .context("Failed to read local file")?;

            // Try to parse remote content and compare
            match toml::from_str::<crate::models::PromptCollection>(&remote_snippet.content) {
                Ok(remote_prompts) => {
                    match toml::to_string_pretty(&remote_prompts) {
                        Ok(remote_formatted) => {
                            // Normalize both contents for comparison
                            let local_normalized = normalize_toml_content(&local_content);
                            let remote_normalized = normalize_toml_content(&remote_formatted);

                            if local_normalized != remote_normalized {
                                println!("🔄 Content differences detected, syncing...");
                                true
                            } else {
                                println!("✅ Already in sync");
                                false
                            }
                        }
                        Err(_) => {
                            // If we can't format remote content, assume sync needed
                            println!("🔄 Unable to format remote content, syncing...");
                            true
                        }
                    }
                }
                Err(_) => {
                    // If we can't parse remote content, assume sync needed
                    println!("🔄 Unable to parse remote content, syncing...");
                    true
                }
            }
        }
    };

    if should_sync {
        // Perform sync directly without going through handle_sync_command
        // to avoid re-comparing timestamps with prompts' updated_at
        let storage = Manager::new(config.clone());

        if local_modified > remote_snippet.updated_at {
            // Upload local changes
            println!("📤 Uploading local changes to remote...");
            let local_prompts = storage.load_prompts()
                .context("Failed to load local prompts")?;

            upload_to_remote(&storage, &*sync_client, &local_prompts).await
                .context("Failed to upload to remote")?;
        } else if remote_snippet.updated_at > local_modified {
            // Download remote changes
            println!("📥 Downloading remote changes...");
            download_from_remote(&storage, &remote_snippet).await
                .context("Failed to download remote changes")?;
        }
    }

    Ok(())
}

/// Normalize TOML content for comparison by removing insignificant whitespace differences
fn normalize_toml_content(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GistConfig;
    use chrono::Utc;

    #[test]
    fn test_sync_direction_logic() {
        let now = Utc::now();
        let earlier = now - chrono::Duration::hours(1);
        let later = now + chrono::Duration::hours(1);

        // Test normal cases
        assert_eq!(should_sync(later, earlier, false), SyncDirection::Upload);
        assert_eq!(should_sync(earlier, later, false), SyncDirection::Download);
        assert_eq!(should_sync(now, now, false), SyncDirection::None);

        // Test force flag
        assert_eq!(should_sync(earlier, later, true), SyncDirection::Upload);
        assert_eq!(should_sync(later, earlier, true), SyncDirection::Upload);
    }
}