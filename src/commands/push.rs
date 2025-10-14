use anyhow::{Context, Result};
use crate::config::Config;
use crate::manager::Manager;
use crate::sync::{gist::GistClient, SyncClient};
use crate::utils::{print_warning, print_network_error};

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

pub async fn handle_push_command(config: Config) -> Result<()> {
    // Check if sync backend is configured
    let gist_config = config.gist.as_ref()
        .ok_or_else(|| anyhow::Error::msg("No sync backend configured. Please configure Gist in your config."))?;

    println!("🚀 Starting push process...");
    println!("📤 Force uploading local prompts to remote...");

    // Create storage instance
    let storage = Manager::new(config.clone());

    // Load local prompts
    let local_prompts = storage.load_prompts()
        .context("Failed to load local prompts")?;

    if local_prompts.prompts.is_empty() {
        print_warning("No prompts found locally. Nothing to push.");
        return Ok(());
    }

    println!("📋 Found {} local prompt(s)", local_prompts.prompts.len());

    // Create sync client
    let sync_client = GistClient::new(gist_config.clone())
        .context("Failed to create Gist client")
        .map_err(|e| handle_potential_network_error(&e).unwrap_err())?;

    // Serialize local prompts to TOML
    let content = toml::to_string_pretty(&local_prompts)
        .context("Failed to serialize local prompts")?;

    // Upload to remote
    sync_client.upload(content).await
        .context("Failed to upload to remote")
        .map_err(|e| handle_potential_network_error(&e).unwrap_err())?;

    println!("✅ Successfully pushed {} prompt(s) to remote", local_prompts.prompts.len());
    println!("🎉 Push completed successfully!");

    Ok(())
}