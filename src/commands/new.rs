use crate::cli::NewArgs;
use crate::config::Config;
use crate::models::Prompt;
use crate::manager::Manager;
use crate::utils::{self, print_sync_warning};
use crate::utils::output::{OutputStyle, print_success};
use anyhow::Result;

pub async fn handle_new_command(
    config: Config,
    args: &NewArgs,
    _interactive: bool,
) -> Result<()> {
    let storage = Manager::new(config.clone());

    let description = match &args.description {
        Some(d) => d.clone(),
        None => match utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Description")), &[]) {
            Some(desc) => desc,
            None => return Ok(()),
        },
    };

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if args.editor {
        utils::open_editor_custom(None, None, Some(&config.general.editor))?
    } else {
        match utils::prompt_multiline(&format!("{}:", OutputStyle::label("Prompt content"))) {
            Some(content) => content,
            None => return Ok(()),
        }
    };

    let mut prompt = Prompt::new(description.clone(), content);

    // Handle tags interactively if not specified
    if let Some(tag_str) = &args.tag {
        let tags: Vec<String> = tag_str.split_whitespace().map(|t| t.to_string()).collect();
        for tag in tags {
            prompt.add_tag(tag);
        }
    } else {
        // Allow adding custom tags with autocomplete
        let existing_tags = storage.get_all_tags()?;
        loop {
            let custom_tag = if existing_tags.is_empty() {
                utils::prompt_input(&format!("{}: ", OutputStyle::label("Add custom tag (leave empty to continue)")))?
            } else {
                utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Add custom tag (leave empty to continue)")), &existing_tags)
                .unwrap_or_default()
            };
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
        // Interactive category input with autocomplete
        let existing_categories = storage.get_categories()?;

        let custom_category = utils::prompt_input_with_autocomplete(&format!("{}: ", OutputStyle::label("Enter category (leave empty for none)")), &existing_categories)
                .unwrap_or_default();
        if !custom_category.is_empty() {
            prompt.category = Some(custom_category);
        }
    }

    storage.add_prompt(prompt)?;
    print_success(&format!("Prompt '{}' saved successfully!", description));

    // Auto-sync if enabled
    if let Err(e) = crate::commands::sync::auto_sync_if_enabled(&config).await {
        print_sync_warning(&e.to_string());
    }

    Ok(())
}
