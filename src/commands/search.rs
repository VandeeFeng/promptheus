use crate::cli::SearchArgs;
use crate::config::Config;
use crate::models::PromptService;
use crate::utils;
use anyhow::Result;
use crate::utils::{handle_empty_list, print_cancelled};

pub fn handle_search_command(
    config: Config,
    args: &SearchArgs,
) -> Result<()> {
    let prompt_service = PromptService::new(config.clone());

    let search_results = prompt_service.search_and_format_for_selection(
        args.query.as_deref(),
        args.tag.as_deref(),
        args.category.as_deref(),
    )?;

    if search_results.is_empty() {
        handle_empty_list("prompts matching your criteria");
        return Ok(());
    }

    let (prompts, display_strings): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    let selected_index = if let Some(query) = &args.query {
        // Try external tool first (like fzf), fall back to fuzzy search
        if let Some(selected_line) = utils::interactive_search_with_external_tool(
            &display_strings,
            &config.general.select_cmd,
            Some(query)
        )? {
            // Find the matching prompt by parsing the selected line
            prompt_service.find_prompt_by_display_line(&prompts, &selected_line)?
        } else {
            // External tool was cancelled, exit gracefully
            print_cancelled("Search cancelled");
            return Ok(());
        }
    } else {
        // Try external tool for general interactive selection
        if let Some(selected_line) = utils::interactive_search_with_external_tool(
            &display_strings,
            &config.general.select_cmd,
            None
        )? {
            prompt_service.find_prompt_by_display_line(&prompts, &selected_line)?
        } else {
            // External tool was cancelled, exit gracefully
            print_cancelled("Search cancelled");
            return Ok(());
        }
    };

    if let Some(index) = selected_index {
        let prompt = &prompts[index];

        if args.execute {
            prompt_service.execute_prompt(prompt, args.copy)?;
        } else {
            use crate::utils::OutputStyle;
            OutputStyle::print_prompt_detailed(prompt);
        }
    }

    Ok(())
}
