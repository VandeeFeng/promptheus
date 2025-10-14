use crate::cli::ExecArgs;
use crate::config::Config;
use crate::manager::Manager;
use crate::utils::{OutputStyle, print_success};
use anyhow::Result;

pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    let storage = Manager::new(config);

    let prompt = storage.find_prompt_by_id(&args.identifier)?
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", args.identifier))?;

    // Execute the prompt (copy to clipboard or display)
    let rendered_content = prompt.content.clone();

    if args.copy {
        crate::utils::copy_to_clipboard(&rendered_content)?;
        print_success("Prompt copied to clipboard!");
    } else {
        println!("\n{}:", OutputStyle::header("📤 Rendered Prompt"));
        println!("{}", OutputStyle::header_separator());
        println!("{}", OutputStyle::content(&rendered_content));
        println!("{}", OutputStyle::header_separator());
    }

    Ok(())
}