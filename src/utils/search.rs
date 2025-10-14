use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn fuzzy_search(items: &[String], query: &str) -> Vec<(usize, f64)> {
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let mut results = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        let item_chars: Vec<char> = item.to_lowercase().chars().collect();
        let score = calculate_fuzzy_score(&item_chars, &query_chars);
        results.push((idx, score));
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn calculate_fuzzy_score(item: &[char], query: &[char]) -> f64 {
    if query.is_empty() {
        return 1.0;
    }

    if item.is_empty() {
        return 0.0;
    }

    let mut query_idx = 0;
    let mut matches = 0;
    let mut total_distance = 0;

    for (_item_idx, &item_char) in item.iter().enumerate() {
        if query_idx < query.len() && item_char == query[query_idx] {
            matches += 1;
            query_idx += 1;
        } else if matches > 0 {
            total_distance += 1;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let match_ratio = matches as f64 / query.len() as f64;
    let completion_bonus = (query_idx as f64 / query.len() as f64).powi(2);
    let distance_penalty = if matches > 1 { total_distance as f64 / (matches - 1) as f64 } else { 0.0 };

    match_ratio * completion_bonus * (1.0 - distance_penalty / 10.0).max(0.0)
}

/// Select a prompt file from available files (main + directories)
/// Returns the selected file path
pub fn select_prompt_file(
    main_file: &std::path::Path,
    prompt_dirs: &[std::path::PathBuf],
    select_cmd: &str,
    tag: Option<&str>,
) -> Result<Option<std::path::PathBuf>> {

    let mut available_files = Vec::new();
    let mut display_strings = Vec::new();

    // Always include main file
    if main_file.exists() {
        available_files.push(main_file.to_path_buf());
        display_strings.push(format!("Main: {}", main_file.display()));
    }

    // Add files from prompt directories
    for dir in prompt_dirs {
        if dir.exists() && dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        // Only consider TOML files
                        if let Some(ext) = path.extension() {
                            if ext == "toml" {
                                let display_name = format!("Dir: {}", path.file_name()
                                    .unwrap_or_default().to_string_lossy());
                                available_files.push(path);
                                display_strings.push(display_name);
                            }
                        }
                    }
                }
            }
        }
    }

    if available_files.is_empty() {
        return Ok(None);
    }

    // If only one file, return it directly
    if available_files.len() == 1 {
        return Ok(Some(available_files[0].clone()));
    }

    // Use external tool for selection if available
    if let Some(selected_line) = interactive_search_with_external_tool(
        &display_strings,
        select_cmd,
        tag
    )? {
        // Find the selected file
        for (i, display) in display_strings.iter().enumerate() {
            if *display == selected_line {
                return Ok(Some(available_files[i].clone()));
            }
        }
    }

    // Fallback to internal selection
    if let Some(selected_index) = crate::utils::interactive::select_from_list(&display_strings)? {
        Ok(Some(available_files[selected_index].clone()))
    } else {
        Ok(None)
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
        cmd.args(&[
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
