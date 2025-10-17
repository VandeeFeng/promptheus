use crate::cli::{ListArgs, ListFormat};
use crate::config::Config;
use crate::commands::handlers::{PromptOperations, DefaultOutputFormatter, OutputFormatter};
use anyhow::Result;

pub fn handle_list_command(
    config: Config,
    args: &ListArgs,
) -> Result<()> {
    let manager = crate::manager::Manager::new(config.clone());
    let formatter = DefaultOutputFormatter;

    // Handle tags listing
    if args.tags {
        return formatter.print_tags(&manager.get_all_tags()?);
    }

    // Handle categories listing
    if args.categories {
        return formatter.print_categories(&manager.get_categories()?);
    }

    if args.stats {
        return formatter.print_stats(&manager.get_stats()?);
    }

    let search_results = manager.search_and_format_for_selection(None, args.tag.as_deref(), args.category.as_deref())?;

    if search_results.is_empty() {
        crate::utils::handle_empty_list("prompts matching your criteria");
        return Ok(());
    }

    let (prompts, _): (Vec<_>, Vec<_>) = search_results.into_iter().unzip();

    let format = args.format.as_ref().unwrap_or(&ListFormat::Simple);
    formatter.format_list(&prompts, format, &config)?;

    Ok(())
}
