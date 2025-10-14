use anyhow::{Context, Result};
use crate::config::Config;
use crate::storage::Storage;
use crate::sync::{gist::GistClient, SyncClient};

pub async fn handle_push_command(config: Config) -> Result<()> {
    // Check if sync backend is configured
    let gist_config = config.gist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No sync backend configured. Please configure Gist in your config."))?;

    println!("🚀 Starting push process...");
    println!("📤 Force uploading local prompts to remote...");

    // Create storage instance
    let storage = Storage::new(config.clone());

    // Load local prompts
    let local_prompts = storage.load_prompts()
        .context("Failed to load local prompts")?;

    if local_prompts.prompts.is_empty() {
        println!("⚠️  No prompts found locally. Nothing to push.");
        return Ok(());
    }

    println!("📋 Found {} local prompt(s)", local_prompts.prompts.len());

    // Create sync client
    let sync_client = GistClient::new(gist_config.clone())
        .context("Failed to create Gist client")?;

    // Serialize local prompts to TOML
    let content = toml::to_string_pretty(&local_prompts)
        .context("Failed to serialize local prompts")?;

    // Upload to remote
    sync_client.upload(content).await
        .context("Failed to upload to remote")?;

    println!("✅ Successfully pushed {} prompt(s) to remote", local_prompts.prompts.len());
    println!("🎉 Push completed successfully!");

    Ok(())
}