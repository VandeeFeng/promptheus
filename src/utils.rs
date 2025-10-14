use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
    cursor, style,
};
use std::io::{self, Write};
use std::process::Command;

pub fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

pub fn prompt_multiline(prompt: &str) -> Result<String> {
    println!("{}", prompt);
    println!("Enter your content (Enter to save, Ctrl+J for new line):");
    println!();

    let mut lines = Vec::new();
    let mut current_line = String::new();

    loop {
        // Enable raw mode for single character input
        terminal::enable_raw_mode()?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            }) | Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            }) => {
                // Ctrl+J or Shift+Enter: Add current line to lines and start a new line
                lines.push(current_line.clone());
                current_line.clear();

                // Show newline visually and ensure cursor is at absolute start of new line
                print!("\r\n");
                io::stdout().flush()?;

                terminal::disable_raw_mode()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: event::KeyModifiers::NONE,
                ..
            }) => {
                // Enter: Save and finish
                lines.push(current_line.clone());
                break;
            }
            Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                current_line.push(c);
                print!("{}", c);
                io::stdout().flush()?;
            }
            Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                if !current_line.is_empty() {
                    // Delete character from current line
                    current_line.pop();
                    // Move cursor back and clear character
                    execute!(io::stdout(), cursor::MoveLeft(1), terminal::Clear(ClearType::UntilNewLine))?;
                    io::stdout().flush()?;
                } else if !lines.is_empty() {
                    // Current line is empty, go back to previous line
                    current_line = lines.pop().unwrap();

                    // Move cursor up and clear the line, then restore previous line content
                    execute!(io::stdout(), cursor::MoveUp(1), cursor::MoveToColumn(1), terminal::Clear(ClearType::UntilNewLine))?;
                    print!("{}", current_line);

                    // Move cursor to end of the restored line
                    for _ in 0..current_line.len() {
                        execute!(io::stdout(), cursor::MoveLeft(1))?;
                    }
                    io::stdout().flush()?;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                // Allow escape to cancel
                terminal::disable_raw_mode()?;
                return Err(anyhow::anyhow!("Input cancelled by user"));
            }
            _ => {}
        }

        terminal::disable_raw_mode()?;
    }

    // Ensure we're out of raw mode
    let _ = terminal::disable_raw_mode();

    println!(); // Add a newline after input

    // Join all lines with newlines
    Ok(lines.join("\n"))
}

pub fn prompt_yes_no(prompt: &str) -> Result<bool> {
    loop {
        let input = prompt_input(&format!("{} [y/N]: ", prompt))?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}

pub fn select_from_list(items: &[String]) -> Result<Option<usize>> {
    if items.is_empty() {
        return Ok(None);
    }

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let mut selected = 0;
    let result = loop {
        // Clear screen and redraw
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        println!("Use arrow keys to navigate, Enter to select, q to quit:");
        println!();

        for (i, item) in items.iter().enumerate() {
            if i == selected {
                execute!(stdout, style::Print("> "), style::SetForegroundColor(style::Color::Blue))?;
            } else {
                execute!(stdout, style::Print("  "))?;
            }
            println!("{}", item);
        }

        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                if selected < items.len() - 1 {
                    selected += 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                break Ok(Some(selected));
            }
            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                break Ok(None);
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                break Ok(None);
            }
            _ => {}
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    result
}

pub fn multi_select_from_list(items: &[String]) -> Result<Vec<Option<usize>>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let mut selected = 0;
    let mut selected_items = vec![false; items.len()];
    let result = loop {
        // Clear screen and redraw
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        println!("Use arrow keys to navigate, Space to select/deselect, Enter to finish, q to quit:");
        println!();

        for (i, item) in items.iter().enumerate() {
            let marker = if selected_items[i] { "✓" } else { " " };
            if i == selected {
                execute!(stdout, style::Print("> "), style::SetForegroundColor(style::Color::Blue))?;
            } else {
                execute!(stdout, style::Print("  "))?;
            }
            println!("[{}] {}", marker, item);
        }

        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                if selected < items.len() - 1 {
                    selected += 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => {
                selected_items[selected] = !selected_items[selected];
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                let selected_indices: Vec<Option<usize>> = selected_items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &selected)| if selected { Some(Some(i)) } else { None })
                    .collect();
                break Ok(selected_indices);
            }
            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                break Ok(Vec::new());
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                break Ok(Vec::new());
            }
            _ => {}
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    result
}

pub fn select_from_list_with_custom(items: &[String], custom_prompt: &str) -> Result<Option<usize>> {
    if items.is_empty() {
        let custom = prompt_input(custom_prompt)?;
        return if custom.is_empty() {
            Ok(None)
        } else {
            Err(anyhow::anyhow!("Custom input not implemented in this function"))
        };
    }

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let mut selected = 0;
    let mut show_custom_input = false;
    let mut custom_input = String::new();

    let result = loop {
        // Clear screen and redraw
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        println!("Use arrow keys to navigate, Enter to select, 'c' for custom, q to quit:");
        println!();

        for (i, item) in items.iter().enumerate() {
            if i == selected && !show_custom_input {
                execute!(stdout, style::Print("> "), style::SetForegroundColor(style::Color::Blue))?;
            } else {
                execute!(stdout, style::Print("  "))?;
            }
            println!("{}", item);
        }

        if show_custom_input {
            execute!(stdout, style::Print("> "), style::SetForegroundColor(style::Color::Blue))?;
            println!("{}: {}", custom_prompt, custom_input);
        } else {
            println!();
            execute!(stdout, style::Print("  "))?;
            println!("+ Add new category");
        }

        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                if selected > 0 && !show_custom_input {
                    selected -= 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                if selected < items.len() && !show_custom_input {
                    selected += 1;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                if show_custom_input {
                    terminal::disable_raw_mode()?;
                    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                    if !custom_input.is_empty() {
                        // Return a special value indicating custom input
                        return Ok(None);
                    }
                    show_custom_input = false;
                    terminal::enable_raw_mode()?;
                } else if selected == items.len() {
                    show_custom_input = true;
                    custom_input.clear();
                } else {
                    break Ok(Some(selected));
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Char('c'), .. }) => {
                show_custom_input = true;
                custom_input.clear();
            }
            Event::Key(KeyEvent { code: KeyCode::Char(ch), .. }) if show_custom_input => {
                if ch.is_ascii() && !ch.is_control() {
                    custom_input.push(ch);
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) if show_custom_input => {
                custom_input.pop();
            }
            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                break Ok(None);
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                break Ok(None);
            }
            _ => {}
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    result
}

pub fn open_editor(content: Option<&str>) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let temp_file = std::env::temp_dir().join(format!("promptheus_{}.tmp", std::process::id()));

    if let Some(content) = content {
        std::fs::write(&temp_file, content)?;
    } else {
        std::fs::File::create(&temp_file)?;
    }

    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .with_context(|| format!("Failed to execute editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    let content = std::fs::read_to_string(&temp_file)?;
    std::fs::remove_file(&temp_file)?;

    Ok(content.trim().to_string())
}

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;

    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn pbcopy")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())
                .context("Failed to write to pbcopy")?;
        }

        let status = child.wait()
            .context("Failed to wait for pbcopy")?;

        if !status.success() {
            return Err(anyhow::anyhow!("pbcopy failed"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel
        if let Ok(mut child) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(text.as_bytes())
                    .context("Failed to write to xclip")?;
            }

            let status = child.wait()
                .context("Failed to wait for xclip")?;

            if status.success() {
                return Ok(());
            }
        }

        let mut child = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn xsel")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())
                .context("Failed to write to xsel")?;
        }

        let status = child.wait()
            .context("Failed to wait for xsel")?;

        if !status.success() {
            return Err(anyhow::anyhow!("xsel failed"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn clip")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())
                .context("Failed to write to clip")?;
        }

        let status = child.wait()
            .context("Failed to wait for clip")?;

        if !status.success() {
            return Err(anyhow::anyhow!("clip failed"));
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err(anyhow::anyhow!("Clipboard not supported on this platform"));
    }

    Ok(())
}

pub fn parse_variables(var_args: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let mut vars = std::collections::HashMap::new();

    for var in var_args {
        if let Some(pos) = var.find('=') {
            let key = var[..pos].trim().to_string();
            let value = var[pos + 1..].trim().to_string();
            vars.insert(key, value);
        } else {
            return Err(anyhow::anyhow!("Invalid variable format: {}. Expected key=value", var));
        }
    }

    Ok(vars)
}

pub fn fuzzy_search(items: &[String], query: &str) -> Vec<(usize, f64)> {
    use std::collections::HashMap;

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

    for (item_idx, &item_char) in item.iter().enumerate() {
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