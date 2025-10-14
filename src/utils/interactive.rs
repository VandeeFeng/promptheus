use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, EnableBracketedPaste, DisableBracketedPaste},
    execute,
    terminal::{self, ClearType},
    cursor, style,
};
use std::io::{self, Write};
use std::process::Command;
use std::env;
use crate::utils::output::OutputStyle;

pub fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

pub fn prompt_input_with_autocomplete(prompt: &str, suggestions: &[String]) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    terminal::enable_raw_mode()?;

    let result = (|| {
        let mut input = String::new();
        let mut current_suggestion = String::new();

        loop {
            match event::read()? {
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    input.push(c);
                    current_suggestion.clear();

                    // Find matching suggestion only if input has at least 2 characters
                    if input.len() >= 2 {
                        for suggestion in suggestions {
                            if suggestion.starts_with(&input) && suggestion != &input {
                                current_suggestion = suggestion[input.len()..].to_string();
                                break;
                            }
                        }
                    }

                    // Redraw current line with suggestion if any
                    execute!(
                        io::stdout(),
                        cursor::MoveToColumn(0),
                        terminal::Clear(ClearType::CurrentLine),
                        style::Print(&prompt),
                        style::Print(&input),
                        style::Print(OutputStyle::muted(&current_suggestion))
                    )?;
                    io::stdout().flush()?;

                    // Move cursor back to end of actual input
                    if !current_suggestion.is_empty() {
                        execute!(io::stdout(), cursor::MoveLeft(current_suggestion.len() as u16))?;
                        io::stdout().flush()?;
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
                    // Accept current suggestion
                    if !current_suggestion.is_empty() {
                        input.push_str(&current_suggestion);
                        current_suggestion.clear();

                        // Redraw line
                        execute!(
                            io::stdout(),
                            cursor::MoveToColumn(0),
                            terminal::Clear(ClearType::CurrentLine),
                            style::Print(&prompt),
                            style::Print(&input)
                        )?;
                        io::stdout().flush()?;
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    if !input.is_empty() {
                        input.pop();
                        current_suggestion.clear();

                        // Find new matching suggestion only if input has at least 2 characters
                        if input.len() >= 2 {
                            for suggestion in suggestions {
                                if suggestion.starts_with(&input) && suggestion != &input {
                                    current_suggestion = suggestion[input.len()..].to_string();
                                    break;
                                }
                            }
                        }

                        // Redraw current line
                        execute!(
                            io::stdout(),
                            cursor::MoveToColumn(0),
                            terminal::Clear(ClearType::CurrentLine),
                            style::Print(&prompt),
                            style::Print(&input),
                            style::Print(OutputStyle::muted(&current_suggestion))
                        )?;
                        io::stdout().flush()?;

                        // Move cursor back to end of actual input
                        if !current_suggestion.is_empty() {
                            execute!(io::stdout(), cursor::MoveLeft(current_suggestion.len() as u16))?;
                            io::stdout().flush()?;
                        }
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    break;
                }
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Err(anyhow::anyhow!("Input cancelled by user"));
                }
                _ => {}
            }
        }
        Ok(input.trim().to_string())
    })();

    terminal::disable_raw_mode()?;
    println!();
    result
}

pub fn prompt_multiline(prompt: &str) -> Result<String> {
    println!("{}", prompt);

    terminal::enable_raw_mode()?;

    let _ = execute!(io::stdout(), EnableBracketedPaste);

    let result = (|| {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        loop {
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
                    lines.push(current_line.clone());
                    current_line.clear();
                    print!("\r\n");
                    io::stdout().flush()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: event::KeyModifiers::NONE,
                    ..
                }) => {
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
                        current_line.pop();
                        execute!(io::stdout(), cursor::MoveLeft(1), terminal::Clear(ClearType::UntilNewLine))?;
                        io::stdout().flush()?;
                    } else if !lines.is_empty() {
                        current_line = lines.pop().unwrap();
                        execute!(io::stdout(), cursor::MoveUp(1), cursor::MoveToColumn(1), terminal::Clear(ClearType::UntilNewLine))?;
                        print!("{}", current_line);
                        for _ in 0..current_line.len() {
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                        }
                        io::stdout().flush()?;
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Err(anyhow::anyhow!("Input cancelled by user"));
                }
                Event::Paste(pasted_text) => {
                    let mut pasted_lines = pasted_text.lines().peekable();
                    while let Some(line) = pasted_lines.next() {
                        if pasted_lines.peek().is_some() {
                            // This is not the last line, so we add it to the lines buffer
                            lines.push(current_line.clone() + line);
                            current_line.clear();
                            print!("{}\r\n", line);
                        } else {
                            // This is the last line, so it becomes the new current_line
                            current_line.push_str(line);
                            print!("{}", line);
                        }
                    }
                    io::stdout().flush()?;
                }
                _ => {}
            }
        }
        Ok(lines.join("\n"))
    })();

    // Cleanup: disable bracketed paste and raw mode.
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    let _ = terminal::disable_raw_mode();

    println!();
    result
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
                selected = selected.saturating_sub(1);
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


pub fn open_editor_custom(
    content: Option<&str>,
    line: Option<u32>,
    editor_cmd: Option<&str>
) -> Result<String> {
    let editor = editor_cmd
        .map(|s| s.to_string())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| {
            // Try to detect a good default editor (same logic as in config.rs)
            if cfg!(windows) {
                "notepad".to_string()
            } else if std::path::Path::new("/usr/bin/code").exists() {
                "code".to_string()
            } else if std::path::Path::new("/usr/bin/nvim").exists() {
                "nvim".to_string()
            } else if std::path::Path::new("/usr/bin/vim").exists() {
                "vim".to_string()
            } else if std::path::Path::new("/usr/bin/nano").exists() {
                "nano".to_string()
            } else {
                "vi".to_string()
            }
        });

    let temp_file = std::env::temp_dir().join(format!("promptheus_{}.tmp", std::process::id()));

    if let Some(content) = content {
        std::fs::write(&temp_file, content)?;
    } else {
        std::fs::File::create(&temp_file)?;
    }

    let mut cmd = Command::new(&editor);

    // Add line number argument if supported
    if let Some(line_num) = line {
        match editor.as_str() {
            e if e.contains("vim") || e.contains("vi") || e.contains("nvim") => {
                cmd.arg(format!("+{}", line_num));
            }
            e if e.contains("emacs") || e.contains("nano") => {
                cmd.arg(format!("+{}", line_num));
            }
            e if e.contains("code") => {
                cmd.arg("--goto");
                cmd.arg(format!("{}:{}", temp_file.display(), line_num));
                // For VS Code, we don't need to add the file separately
                let status = cmd.status()
                    .with_context(|| format!("Failed to execute editor: {}", editor))?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Editor exited with non-zero status"));
                }

                let content = std::fs::read_to_string(&temp_file)?;
                std::fs::remove_file(&temp_file)?;
                return Ok(content.trim().to_string());
            }
            _ => {
                // Unknown editor, try with line parameter
                cmd.arg(format!("+{}", line_num));
            }
        }
    }

    cmd.arg(&temp_file);

    let status = cmd.status()
        .with_context(|| format!("Failed to execute editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    let content = std::fs::read_to_string(&temp_file)?;
    std::fs::remove_file(&temp_file)?;

    Ok(content.trim().to_string())
}

/// Edit a file directly (not temporary) with line number support
pub fn edit_file_direct(
    file_path: &std::path::Path,
    line: Option<u32>,
    editor_cmd: Option<&str>
) -> Result<()> {
    let editor = editor_cmd
        .map(|s| s.to_string())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| {
            // Try to detect a good default editor (same logic as in config.rs)
            if cfg!(windows) {
                "notepad".to_string()
            } else if std::path::Path::new("/usr/bin/code").exists() {
                "code".to_string()
            } else if std::path::Path::new("/usr/bin/nvim").exists() {
                "nvim".to_string()
            } else if std::path::Path::new("/usr/bin/vim").exists() {
                "vim".to_string()
            } else if std::path::Path::new("/usr/bin/nano").exists() {
                "nano".to_string()
            } else {
                "vi".to_string()
            }
        });

    let mut cmd = Command::new(&editor);

    // Add line number argument if supported
    if let Some(line_num) = line {
        match editor.as_str() {
            e if e.contains("vim") || e.contains("vi") || e.contains("nvim") => {
                cmd.arg(format!("+{}", line_num));
            }
            e if e.contains("emacs") || e.contains("nano") => {
                cmd.arg(format!("+{}", line_num));
            }
            e if e.contains("code") => {
                cmd.arg("--goto");
                cmd.arg(format!("{}:{}", file_path.display(), line_num));
                // For VS Code, this is sufficient
                let status = cmd.status()
                    .with_context(|| format!("Failed to execute editor: {}", editor))?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Editor exited with non-zero status"));
                }
                return Ok(());
            }
            _ => {
                cmd.arg(format!("+{}", line_num));
            }
        }
    }

    cmd.arg(file_path);

    let status = cmd.status()
        .with_context(|| format!("Failed to execute editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    Ok(())
}


#[derive(Debug, Clone, Copy)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

/// Detect the current display server (Wayland or X11) on Linux systems
fn detect_display_server() -> DisplayServer {
    // Check XDG_SESSION_TYPE first (most reliable)
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => {}
        }
    }

    // Fallback checks
    if env::var("WAYLAND_DISPLAY").is_ok() {
        DisplayServer::Wayland
    } else if env::var("DISPLAY").is_ok() {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

/// Check if a command is available in the system
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .is_ok()
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
        let display_server = detect_display_server();

        let tools: Vec<(&str, Vec<&str>)> = match display_server {
            DisplayServer::Wayland => {
                // On Wayland, prefer wl-clipboard tools
                vec![
                    ("wl-copy", vec![]),
                    ("xclip", vec!["-selection", "clipboard"]),
                    ("xsel", vec!["--clipboard", "--input"]),
                ]
            }
            DisplayServer::X11 => {
                // On X11, prefer X11 tools but keep wl-clipboard as fallback
                vec![
                    ("xclip", vec!["-selection", "clipboard"]),
                    ("xsel", vec!["--clipboard", "--input"]),
                    ("wl-copy", vec![]),
                ]
            }
            DisplayServer::Unknown => {
                // Unknown system, try all available tools in reasonable order
                vec![
                    ("wl-copy", vec![]),
                    ("xclip", vec!["-selection", "clipboard"]),
                    ("xsel", vec!["--clipboard", "--input"]),
                ]
            }
        };

        let mut last_error = None;
        let mut available_tools = Vec::new();

        for (tool, args) in tools {
            if command_exists(tool) {
                available_tools.push(tool);

                if let Ok(mut child) = Command::new(tool)
                    .args(args)
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = child.stdin.as_mut()
                        && let Err(e) = stdin.write_all(text.as_bytes()) {
                            last_error = Some(anyhow::anyhow!("Failed to write to {}: {}", tool, e));
                            continue;
                        }

                    match child.wait() {
                        Ok(status) if status.success() => return Ok(()),
                        Ok(_) => last_error = Some(anyhow::anyhow!("{} failed", tool)),
                        Err(e) => last_error = Some(anyhow::anyhow!("Failed to wait for {}: {}", tool, e)),
                    }
                } else {
                    last_error = Some(anyhow::anyhow!("Failed to spawn {}", tool));
                }
            }
        }

        // Provide helpful error message based on display server
        if available_tools.is_empty() {
            match display_server {
                DisplayServer::Wayland => {
                    return Err(anyhow::anyhow!(
                        "No clipboard tools found. Please install wl-clipboard:\n  sudo pacman -S wl-clipboard  # Arch\n  sudo apt install wl-clipboard  # Ubuntu/Debian"
                    ));
                }
                DisplayServer::X11 => {
                    return Err(anyhow::anyhow!(
                        "No clipboard tools found. Please install one of:\n  sudo pacman -S xclip  # Arch\n  sudo apt install xclip  # Ubuntu/Debian"
                    ));
                }
                DisplayServer::Unknown => {
                    return Err(anyhow::anyhow!(
                        "No clipboard tools found. Please install:\n  sudo pacman -S wl-clipboard xclip  # Arch\n  sudo apt install wl-clipboard xclip  # Ubuntu/Debian"
                    ));
                }
            }
        }

        if let Some(error) = last_error {
            return Err(error);
        }
        return Err(anyhow::anyhow!("All available clipboard tools failed"));
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

    // This line should never be reached due to the platform-specific returns above
    #[allow(unreachable_code)]
    Ok(())
}
