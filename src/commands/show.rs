use crate::cli::ShowArgs;
use crate::config::Config;
use anyhow::Result;

use crate::utils::{OutputStyle, handle_not_found};

pub fn handle_show_command(
    config: Config,
    args: &ShowArgs,
) -> Result<()> {
    let manager = crate::manager::Manager::new(config);

    if let Some(prompt) = manager.find_prompt(&args.identifier)? {
        OutputStyle::print_prompt_detailed(&prompt);
    } else {
        handle_not_found("Prompt", &args.identifier);
    }

    Ok(())
}