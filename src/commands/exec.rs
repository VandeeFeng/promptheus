use crate::cli::ExecArgs;
use crate::config::Config;
use crate::storage::Storage;
use anyhow::Result;

pub fn handle_exec_command(
    config: Config,
    args: &ExecArgs,
) -> Result<()> {
    let storage = Storage::new(config);

    let prompt = storage.find_prompt_by_id(&args.identifier)?
            .ok_or_else(|| anyhow::anyhow!("Prompt with ID '{}' not found", args.identifier))?;

    // Execute the prompt (copy to clipboard or display)
    let rendered_content = prompt.content.clone();

    if args.copy {
        crate::utils::copy_to_clipboard(&rendered_content)?;
        println!("✓ Prompt copied to clipboard!");
    } else {
        println!("\n📤 Rendered Prompt:");
        println!("{}", "=".repeat(50));
        println!("{}", rendered_content);
        println!("{}", "=".repeat(50));
    }

    Ok(())
}