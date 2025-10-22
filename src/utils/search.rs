use crate::utils::error::{AppResult, AppError};
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
        // Extract description from format: [description]: [category] #tags content
        // Handle multi-line format by taking only the first line
        let first_line = selected_line.lines().next().unwrap_or(selected_line);

        if let Some(desc_end) = first_line.find("]:") {
            let description = &first_line[1..desc_end]; // Remove [ and ]

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
) -> AppResult<Option<String>> {
    if items.is_empty() {
        return Ok(None);
    }

    // Check if the select command is available
    let cmd_parts: Vec<&str> = select_cmd.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Err(AppError::System(format!("Invalid select command: {}", select_cmd)));
    }

    // Check if command exists
    match std::process::Command::new(cmd_parts[0]).arg("--version").output() {
        Ok(_) => {},
        Err(_) => {
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
            "--read0",
            "--ansi",
            "--expect=ctrl-c,esc",
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
        .map_err(|e| AppError::System(format!("Failed to spawn command: {}: {}", select_cmd, e)))?;

    // Write items to stdin
    if let Some(stdin) = child.stdin.as_mut() {
        for item in items {
            // Write each item followed by NULL character for fzf --read0
            stdin.write_all(item.as_bytes()).map_err(|e| AppError::Io(format!("Failed to write to stdin: {}", e)))?;
            stdin.write_all(b"\0").map_err(|e| AppError::Io(format!("Failed to write NULL separator to stdin: {}", e)))?;
        }
    }

    // Read the result
    let output = child.wait_with_output()
        .map_err(|e| AppError::System(format!("Failed to read output from: {}: {}", select_cmd, e)))?;

    // Check if the command was successful
    // Some tools like fzf return exit code 130 when user presses Ctrl+C or Esc
    if !output.status.success() {
        return Ok(None);
    }

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // With --expect, fzf returns key press on first line, selection on second line
    if lines.len() < 2 {
        return Ok(None);
    }

    let selected = lines[1].trim();

    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected.to_string()))
    }
}
