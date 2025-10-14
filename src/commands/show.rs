use crate::cli::ShowArgs;
use crate::config::Config;
use crate::manager::Manager;
use anyhow::Result;

use crate::utils::OutputStyle;

pub fn handle_show_command(
    config: Config,
    args: &ShowArgs,
) -> Result<()> {
    let storage = Manager::new(config);

    let prompt = storage.find_prompt_by_id(&args.identifier)?
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", args.identifier))?;

    show_prompt_details(&prompt);

    Ok(())
}

fn show_prompt_details(prompt: &crate::models::Prompt) {
    OutputStyle::print_prompt_detailed(prompt);
}