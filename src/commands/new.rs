use crate::cli::NewArgs;
use crate::config::Config;
use crate::models::Prompt;
use crate::manager::Manager;
use crate::utils::{self, print_error};
use anyhow::Result;

pub async fn handle_new_command(
    config: Config,
    args: &NewArgs,
    _interactive: bool,
) -> Result<()> {
    let storage = Manager::new(config.clone());

    let description = match &args.description {
        Some(d) => d.clone(),
        None => utils::prompt_input("Description: ")?,
    };

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if args.editor {
        utils::open_editor_custom(None, None, Some(&config.general.editor))?
    } else {
        utils::prompt_multiline("Prompt content:")?
    };

    let mut prompt = Prompt::new(description.clone(), content);

    // Handle tags interactively if not specified
    if let Some(tag_str) = &args.tag {
        let tags: Vec<String> = tag_str.split_whitespace().map(|t| t.to_string()).collect();
        for tag in tags {
            prompt.add_tag(tag);
        }
    } else {
        // Show existing tags for reference
        let existing_tags = storage.get_all_tags()?;
        if !existing_tags.is_empty() {
            println!("\n🏷️  Existing tags: {}", existing_tags.join(", "));
            println!("Tip: Use --tag <tag1> <tag2> to add tags when creating prompts");
        }

        // Allow adding custom tags
        loop {
            let custom_tag = utils::prompt_input("Add custom tag (leave empty to continue): ")?;
            if custom_tag.is_empty() {
                break;
            }
            prompt.add_tag(custom_tag);
        }
    }

    // Add default tags from config
    for tag in &config.general.default_tags {
        prompt.add_tag(tag.clone());
    }

    // Handle category interactively if not specified
    if let Some(category) = &args.category {
        prompt.category = Some(category.clone());
    } else {
        // Interactive category selection
        let existing_categories = storage.get_categories()?;
        if !existing_categories.is_empty() {
            println!("\n📁 Select a category (use arrow keys, Enter to select, or type new one):");
            let selected_index = utils::select_from_list_with_custom(&existing_categories, "Enter new category name: ")?;

            if let Some(index) = selected_index {
                prompt.category = Some(existing_categories[index].clone());
            } else {
                // User selected custom input
                let custom_category = utils::prompt_input("Enter new category name: ")?;
                if !custom_category.is_empty() {
                    prompt.category = Some(custom_category);
                }
            }
        } else {
            let custom_category = utils::prompt_input("Enter category (leave empty for none): ")?;
            if !custom_category.is_empty() {
                prompt.category = Some(custom_category);
            }
        }
    }

    storage.add_prompt(prompt)?;
    println!("✓ Prompt '{}' saved successfully!", description);

    // Auto-sync if enabled
    if let Err(e) = crate::commands::sync::auto_sync_if_enabled(&config).await {
        print_error(&format!("Auto-sync failed: {}", e));
    }

    Ok(())
}