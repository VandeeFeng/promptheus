use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use crate::core::data::{Prompt, PromptCollection};
use crate::config::Config;
use crate::utils::output::OutputStyle;


/// Search engine for prompt filtering and operations
pub struct SearchEngine;

impl SearchEngine {
    /// Search prompts with query and tag filtering
    pub fn search(
        collection: &PromptCollection,
        query: Option<&str>,
        tag: Option<&str>,
        config: &Config,
    ) -> Vec<Prompt> {
        collection.search(query, tag, config)
    }

    /// Format prompts for selection with display strings
    pub fn format_for_selection(
        collection: &PromptCollection,
        query: Option<&str>,
        tag: Option<&str>,
        category: Option<&str>,
        config: &Config,
    ) -> Vec<(Prompt, String)> {
        let prompts = Self::search(collection, query, tag, config);

        // Filter by category if specified
        let filtered_prompts: Vec<_> = if let Some(category) = &category {
            prompts.into_iter()
                .filter(|p| p.category.as_deref() == Some(*category))
                .collect()
        } else {
            prompts
        };

        let mut result = Vec::new();

        for prompt in filtered_prompts {
            let display_string = OutputStyle::format_prompt_for_selection(&prompt, config);
            result.push((prompt, display_string));
        }

        result
    }

    /// Find prompt by parsing its display line
    pub fn find_by_display_line(prompts: &[Prompt], selected_line: &str) -> Option<usize> {
        // Extract description from format: [description]: content #tags [category]
        if let Some(desc_end) = selected_line.find("]:") {
            let description = &selected_line[1..desc_end]; // Remove [ and ]

            for (i, prompt) in prompts.iter().enumerate() {
                if prompt.description == description {
                    return Some(i);
                }
            }
        }

        None
    }
}

/// Interactively search using external tools like fzf or peco
/// Returns the selected line content
pub fn interactive_search_with_external_tool(
    items: &[String],
    select_cmd: &str,
    query: Option<&str>
) -> Result<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    // Check if the select command is available
    let cmd_parts: Vec<&str> = select_cmd.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Err(anyhow::anyhow!("Invalid select command: {}", select_cmd));
    }

    // Check if command exists
    match std::process::Command::new(cmd_parts[0]).arg("--version").output() {
        Ok(_) => {}, // Command exists
        Err(_) => {
            // Command doesn't exist, return None to trigger fallback
            return Ok(None);
        }
    }

    let mut cmd = Command::new(cmd_parts[0]);

    // Add remaining arguments
    for arg in &cmd_parts[1..] {
        cmd.arg(arg);
    }

    // Add common fzf options for better experience
    if cmd_parts[0] == "fzf" {
        cmd.args([
            "--height=40%",
            "--layout=reverse",
            "--border",
            "--inline-info",
            "--prompt=o ",
        ]);

        if let Some(q) = query {
            cmd.arg(format!("--query={}", q));
        }
    } else if cmd_parts[0] == "peco" {
        // Peco doesn't need as many options
        if let Some(q) = query {
            cmd.arg("--query");
            cmd.arg(q);
        }
    }

    // Set up stdin/stdout
    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped()); // Capture stderr instead of inheriting

    let mut child = cmd.spawn()
        .with_context(|| format!("Failed to spawn command: {}", select_cmd))?;

    // Write items to stdin
    if let Some(stdin) = child.stdin.as_mut() {
        for item in items {
            writeln!(stdin, "{}", item)?;
        }
    }

    // Read the result
    let output = child.wait_with_output()
        .with_context(|| format!("Failed to read output from: {}", select_cmd))?;

    // Check if the command was successful
    // Some tools like fzf return exit code 130 when user presses Ctrl+C or Esc
    if !output.status.success() {
        return Ok(None);
    }

    let result = String::from_utf8_lossy(&output.stdout);
    let selected = result.trim();

    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected.to_string()))
    }
}
