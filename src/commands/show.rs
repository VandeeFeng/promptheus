use crate::cli::ShowArgs;
use crate::config::Config;
use crate::manager::Manager;
use anyhow::Result;

use crate::utils::{format_datetime, OutputStyle};

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
    OutputStyle::print_header("📝 Prompt Details");

    OutputStyle::print_field_colored("Title", &prompt.description, OutputStyle::description);
    if let Some(id) = &prompt.id {
        OutputStyle::print_field_colored("ID", id, OutputStyle::muted);
    }

    if let Some(category) = &prompt.category {
        OutputStyle::print_field_colored("Category", category, OutputStyle::tag);
    }

    if let Some(ref tags) = prompt.tag
        && !tags.is_empty() {
            OutputStyle::print_field_colored("Tags", &tags.join(", "), OutputStyle::tags);
        }

    OutputStyle::print_field_colored("Created", &format_datetime(&prompt.created_at), OutputStyle::muted);
    OutputStyle::print_field_colored("Updated", &format_datetime(&prompt.updated_at), OutputStyle::muted);

    println!("\n{}:", OutputStyle::header("📄 Content"));
    println!("{}", OutputStyle::separator());
    println!("{}", OutputStyle::content(&prompt.content));
    println!("{}", OutputStyle::separator());
}