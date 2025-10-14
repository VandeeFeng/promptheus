use crate::cli::ShowArgs;
use crate::config::Config;
use crate::storage::Storage;
use anyhow::Result;

pub fn handle_show_command(
    config: Config,
    args: &ShowArgs,
) -> Result<()> {
    let storage = Storage::new(config);

    let prompt = storage.find_prompt_by_id(&args.identifier)?
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", args.identifier))?;

    show_prompt_details(&prompt);

    Ok(())
}

fn show_prompt_details(prompt: &crate::prompt::Prompt) {
    println!("\n📝 Prompt Details");
    println!("=================");
    println!("Title: {}", prompt.description);
    if let Some(id) = &prompt.id {
        println!("ID: {}", id);
    }

    if let Some(category) = &prompt.category {
        println!("Category: {}", category);
    }

    if let Some(ref tags) = prompt.tag {
        if !tags.is_empty() {
            println!("Tags: {}", tags.join(", "));
        }
    }

    println!("Created: {}", prompt.created_at.format("%Y-%m-%d-%H:%M:%S"));
    println!("Updated: {}", prompt.updated_at.format("%Y-%m-%d-%H:%M:%S"));

    println!("\n📄 Content:");
    println!("{}", "-".repeat(50));
    println!("{}", prompt.content);
    println!("{}", "-".repeat(50));
}