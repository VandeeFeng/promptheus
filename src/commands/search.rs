use crate::cli::SearchArgs;
use crate::config::Config;
use crate::utils;
use anyhow::Result;
use crate::utils::{handle_empty_list, print_cancelled};

pub fn handle_search_command(
    config: Config,
    args: &SearchArgs,
) -> Result<()> {
    let manager = crate::manager::Manager::new(config.clone());

    let search_results = manager.search_and_format_for_selection(
        args.query.as_deref(),
        args.tag.as_deref(),
        args.category.as_deref(),
    )?;

    if search_results.is_empty() {
        handle_empty_list("prompts matching your criteria");
        return Ok(());
    }

    let (prompts, display_strings): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    // Use unified interactive selection
    let selected_prompt = if let Some(selected_line) = utils::interactive_search_with_external_tool(
        &display_strings,
        &config.general.select_cmd,
        args.query.as_deref()
    )? {
        if let Some(index) = manager.find_prompt_by_display_line(&prompts, &selected_line)? {
            Some(&prompts[index])
        } else {
            None
        }
    } else {
        print_cancelled("Search cancelled");
        return Ok(());
    };

    if let Some(prompt) = selected_prompt {
        if args.execute {
            manager.execute_prompt(prompt, args.copy)?;
        } else {
            use crate::utils::OutputStyle;
            OutputStyle::print_prompt_detailed(prompt);
        }
    }

    Ok(())
}
