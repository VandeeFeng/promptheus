// Query operations - List, Search, Execute
// Consolidated from list.rs, search.rs, exec.rs

use crate::cli::{ListArgs, ListFormat, SearchArgs, ExecArgs};
use crate::config::Config;
use crate::core::traits::{PromptSearch, PromptInteraction, PromptDisplay};
use crate::core::operations::PromptOperations;
use crate::utils;
use crate::utils::{handle_empty_list, print_cancelled, handle_not_found, copy_to_clipboard, print_success, OutputStyle};
use anyhow::Result;

// List operations
pub fn handle_list_command(
    config: Config,
    args: &ListArgs,
) -> Result<()> {
    let manager = PromptOperations::new(&config);

    // Handle tags listing
    if args.tags {
        return manager.print_tags(&manager.get_all_tags()?);
    }

    // Handle categories listing
    if args.categories {
        return manager.print_categories(&manager.get_categories()?);
    }

    if args.stats {
        return manager.print_stats(&manager.get_prompt_stats()?);
    }

    let search_results = manager.search_and_format_for_selection(None, args.tag.as_deref(), args.category.as_deref())?;

    if search_results.is_empty() {
        handle_empty_list("prompts matching your criteria");
        return Ok(());
    }

    let (prompts, _): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    let format = args.format.as_ref().unwrap_or(&ListFormat::Simple);
    manager.format_list(&prompts, format)?;

    Ok(())
}

// Search operations
pub fn handle_search_command(
    config: Config,
    args: &SearchArgs,
) -> Result<()> {
    let manager = PromptOperations::new(&config);

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
        &manager.config().general.select_cmd,
        args.query.as_deref()
    )? {
        manager.find_prompt_by_display_line(&prompts, &selected_line).map(|index| &prompts[index])
    } else {
        print_cancelled("Search cancelled");
        return Ok(());
    };

    if let Some(prompt) = selected_prompt {
        if args.execute {
            manager.execute_prompt(prompt, args.copy)?;
        } else {
            use crate::utils::OutputStyle;

            // Display complete prompt with all logic handled internally
            OutputStyle::display_prompt_complete(prompt)?;
        }
    }

    Ok(())
}

// Execute operations
pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    match &args.identifier {
        Some(identifier) => {
            // Direct execution with ID or description
            let manager = PromptOperations::new(&config);
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
    let manager = PromptOperations::new(&config);

    // Get all prompts for selection using same method as search
    let search_results = manager.search_and_format_for_selection(None, None, None)?;

    if search_results.is_empty() {
        handle_empty_list("prompts");
        return Ok(());
    }

    let (prompts, display_strings): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    // Use same interactive selection logic as search
    let selected_prompt = if let Some(selected_line) = utils::interactive_search_with_external_tool(
        &display_strings,
        &manager.config().general.select_cmd,
        None
    )? {
        manager.find_prompt_by_display_line(&prompts, &selected_line).map(|index| &prompts[index])
    } else {
        print_cancelled("Prompt selection cancelled");
        return Ok(());
    };

    if let Some(prompt) = selected_prompt {
        let rendered_content = prompt.content.clone();

        // Show content with pagination if needed
        OutputStyle::ask_and_display_content(&rendered_content, "📄 Content")?;

        // Always copy to clipboard in interactive mode
        copy_to_clipboard(&rendered_content)?;
        print_success("Prompt copied to clipboard!");
    }

    Ok(())
}

