use crate::cli::ExecArgs;
use crate::config::Config;
use crate::utils::{handle_not_found, handle_empty_list, print_cancelled, copy_to_clipboard, print_success};
use anyhow::Result;

pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    let manager = crate::manager::Manager::new(config.clone());

    match &args.identifier {
        Some(identifier) => {
            // Direct execution with ID or description
            if let Some(prompt) = manager.find_prompt(identifier)? {
                manager.execute_prompt(&prompt, args.copy)?;
            } else {
                // Handle not found as notification, not error
                handle_not_found("Prompt with ID or description", identifier);
                return Ok(());
            }
        }
        None => {
            // Interactive mode
            handle_interactive_exec(config, args)?;
        }
    }

    Ok(())
}

fn handle_interactive_exec(config: Config, _args: &ExecArgs) -> Result<()> {
    let manager = crate::manager::Manager::new(config.clone());

    // Get all prompts for selection
    let prompts = manager.search_prompts(None, None)?;

    if prompts.is_empty() {
        handle_empty_list("prompts");
        return Ok(());
    }

    // Use unified interactive selection
    if let Some(prompt) = manager.select_interactive_prompts(prompts)? {
        // For interactive mode: show only content and copy to clipboard
        let rendered_content = prompt.content.clone();

        println!("{}", rendered_content);

        // Always copy to clipboard in interactive mode
        copy_to_clipboard(&rendered_content)?;
        print_success("Prompt copied to clipboard!");
    } else {
        print_cancelled("Prompt selection cancelled");
        return Ok(());
    }

    Ok(())
}