use crate::cli::ExecArgs;
use crate::config::Config;
use crate::models::PromptService;
use crate::utils;
use crate::utils::{handle_not_found, handle_empty_list, print_cancelled, copy_to_clipboard, print_success};
use anyhow::Result;

pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    let prompt_service = PromptService::new(config.clone());

    match &args.identifier {
        Some(identifier) => {
            // Direct execution with ID or description
            if let Some(prompt) = prompt_service.find_prompt(identifier)? {
                prompt_service.execute_prompt(&prompt, args.copy)?;
            } else {
                // Handle not found as notification, not error
                handle_not_found("Prompt with ID or description", identifier);
                return Ok(());
            }
        }
        None => {
            // Interactive mode - use fzf to select prompt
            handle_interactive_exec(config, args)?;
        }
    }

    Ok(())
}

fn handle_interactive_exec(config: Config, _args: &ExecArgs) -> Result<()> {
    let prompt_service = PromptService::new(config.clone());

    // Get all prompts for selection with formatted display strings
    let search_results = prompt_service.search_and_format_for_selection(None, None, None)?;

    if search_results.is_empty() {
        handle_empty_list("prompts");
        return Ok(());
    }

    let (prompts, display_strings): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    // Use fzf for interactive selection
    if let Some(selected_line) = utils::interactive_search_with_external_tool(
        &display_strings,
        &config.general.select_cmd,
        None
    )? {
        // Find the matching prompt by parsing the selected line
        if let Some(index) = prompt_service.find_prompt_by_display_line(&prompts, &selected_line)? {
            let prompt = &prompts[index];

            // For interactive mode: show only content and copy to clipboard
            let rendered_content = prompt.content.clone();

            println!("{}", rendered_content);

            // Always copy to clipboard in interactive mode
            copy_to_clipboard(&rendered_content)?;
            print_success("Prompt copied to clipboard!");
        }
    } else {
        // External tool was cancelled, exit gracefully
        print_cancelled("Prompt selection cancelled");
        return Ok(());
    }

    Ok(())
}