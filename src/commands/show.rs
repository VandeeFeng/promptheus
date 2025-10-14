use crate::cli::ShowArgs;
use crate::config::Config;
use crate::manager::Manager;
use anyhow::Result;

use crate::utils::{OutputStyle, handle_not_found};

pub fn handle_show_command(
    config: Config,
    args: &ShowArgs,
) -> Result<()> {
    let storage = Manager::new(config);

    if let Some(prompt) = storage.find_prompt_by_id(&args.identifier)? {
        show_prompt_details(&prompt);
    } else {
        // Try to find by description if ID not found
        if let Some(prompt) = storage.find_prompt_by_description(&args.identifier)? {
            show_prompt_details(&prompt);
        } else {
            handle_not_found("Prompt", &args.identifier);
        }
    }

    Ok(())
}

fn show_prompt_details(prompt: &crate::models::Prompt) {
    OutputStyle::print_prompt_detailed(prompt);
}