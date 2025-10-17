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

/// Interactive input errors that distinguish between system failures and user actions
#[derive(Debug)]
pub enum InteractiveError {
    /// System-level errors (terminal, IO, etc.)
    SystemError(anyhow::Error),
    /// User cancelled the operation
    Cancelled,
}

impl std::fmt::Display for InteractiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractiveError::SystemError(e) => write!(f, "System error: {}", e),
            InteractiveError::Cancelled => write!(f, "Operation cancelled by user"),
        }
    }
}

impl std::error::Error for InteractiveError {}

/// Convert system errors to InteractiveError
impl From<io::Error> for InteractiveError {
    fn from(err: io::Error) -> Self {
        InteractiveError::SystemError(anyhow::anyhow!("IO error: {}", err))
    }
}

/// Convert anyhow errors to InteractiveError
impl From<anyhow::Error> for InteractiveError {
    fn from(err: anyhow::Error) -> Self {
        InteractiveError::SystemError(err)
    }
}

struct RawModeGuard {
    bracketed_paste: bool,
}

impl RawModeGuard {
    fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(RawModeGuard { bracketed_paste: false })
    }

    fn with_bracketed_paste() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let _ = execute!(io::stdout(), EnableBracketedPaste);
        Ok(RawModeGuard { bracketed_paste: true })
    }

    /// Ensure terminal is in a clean state before dropping
    fn ensure_clean_state(&self) {
        let _ = execute!(io::stdout(), style::Print("\r\n"));
        let _ = io::stdout().flush();
    }

    /// Clear current line and move cursor to beginning
    fn clear_line(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine)
        )?;
        io::stdout().flush()?;
        Ok(())
    }

    /// Print text and flush output
    fn print_and_flush(&self, text: &str) -> Result<()> {
        execute!(io::stdout(), style::Print(text))?;
        io::stdout().flush()?;
        Ok(())
    }

    /// Print formatted line with prompt and content
    fn print_line(&self, prompt: &str, content: &str, suggestion: Option<&str>) -> Result<()> {
        self.clear_line()?;
        execute!(
            io::stdout(),
            style::Print(prompt),
            style::Print(content)
        )?;

        if let Some(suggestion) = suggestion {
            execute!(io::stdout(), style::Print(OutputStyle::muted(suggestion)))?;
        }

        io::stdout().flush()?;
        Ok(())
    }

    /// Move cursor left by specified positions
    fn move_cursor_left(&self, positions: u16) -> Result<()> {
        if positions > 0 {
            execute!(io::stdout(), cursor::MoveLeft(positions))?;
            io::stdout().flush()?;
        }
        Ok(())
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Ensure terminal state is clean before disabling raw mode
        self.ensure_clean_state();

        if self.bracketed_paste {
            let _ = execute!(io::stdout(), DisableBracketedPaste);
        }
        let _ = terminal::disable_raw_mode();
    }
}

fn detect_editor(editor_cmd: Option<&str>) -> String {
    editor_cmd
        .map(|s| s.to_string())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| {
            // Try to detect a good default editor
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
        })
}

pub fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Ensure proper newline after input
    println!();

    Ok(input.trim().to_string())
}

/// Generic error handling wrapper that converts InteractiveError to Option
fn handle_interactive_result<T>(result: Result<T, InteractiveError>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(InteractiveError::Cancelled) => {
            crate::utils::error::print_cancelled("Operation cancelled by user");
            None
        }
        Err(InteractiveError::SystemError(e)) => {
            eprintln!("❌ Terminal error: {}", e);
            None
        }
    }
}

/// Find matching autocomplete suggestion
fn find_autocomplete_suggestion(input: &str, suggestions: &[String]) -> String {
    if input.len() >= 2 {
        for suggestion in suggestions {
            if suggestion.starts_with(input) && suggestion != input {
                return suggestion[input.len()..].to_string();
            }
        }
    }
    String::new()
}

/// Internal version that properly propagates system errors
fn prompt_input_with_autocomplete_internal(prompt: &str, suggestions: &[String]) -> Result<String, InteractiveError> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let guard = RawModeGuard::new()?;
    let mut input = String::new();
    let mut current_suggestion = String::new();

    loop {
        let event = event::read()?; // Propagate terminal errors properly
        match event {
            Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                input.push(c);
                current_suggestion = find_autocomplete_suggestion(&input, suggestions);

                // Redraw current line with suggestion if any
                guard.print_line(prompt, &input,
                    if current_suggestion.is_empty() { None } else { Some(&current_suggestion) }
                )?;

                // Move cursor back to end of actual input
                guard.move_cursor_left(current_suggestion.len() as u16)?;
            }
            Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
                // Accept current suggestion
                if !current_suggestion.is_empty() {
                    input.push_str(&current_suggestion);
                    current_suggestion.clear();

                    // Redraw line
                    guard.print_line(prompt, &input, None)?;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                if !input.is_empty() {
                    input.pop();
                    current_suggestion = find_autocomplete_suggestion(&input, suggestions);

                    // Redraw current line
                    guard.print_line(prompt, &input,
                        if current_suggestion.is_empty() { None } else { Some(&current_suggestion) }
                    )?;

                    // Move cursor back to end of actual input
                    guard.move_cursor_left(current_suggestion.len() as u16)?;
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                // Just move to next line without clearing current content
                guard.print_and_flush("\r\n")?;
                break;
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                return Err(InteractiveError::Cancelled);
            }
            _ => {}
        }
    }

    Ok(input.trim().to_string())
}

/// Public interface that maintains Option return for backward compatibility
/// System errors are logged but converted to None for a smoother user experience
pub fn prompt_input_with_autocomplete(prompt: &str, suggestions: &[String]) -> Option<String> {
    handle_interactive_result(prompt_input_with_autocomplete_internal(prompt, suggestions))
}

/// Internal version that properly propagates system errors
fn prompt_multiline_internal(prompt: &str) -> Result<String, InteractiveError> {
    println!("{}", prompt);

    let guard = RawModeGuard::with_bracketed_paste()?;
    let mut lines = Vec::new();
    let mut current_line = String::new();

    loop {
        let event = event::read()?; // Propagate terminal errors properly
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'), modifiers: event::KeyModifiers::CONTROL, ..
            }) | Event::Key(KeyEvent {
                code: KeyCode::Enter, modifiers: event::KeyModifiers::SHIFT, ..
            }) => {
                lines.push(current_line.clone());
                current_line.clear();
                guard.print_and_flush("\r\n")?;
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
                guard.print_and_flush(&c.to_string())?;
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
                return Err(InteractiveError::Cancelled);
            }
            Event::Paste(pasted_text) => {
                let mut pasted_lines = pasted_text.lines().peekable();
                while let Some(line) = pasted_lines.next() {
                    if pasted_lines.peek().is_some() {
                        // This is not the last line, so we add it to the lines buffer
                        lines.push(current_line.clone() + line);
                        current_line.clear();
                        guard.print_and_flush(&format!("{}\r\n", line))?;
                    } else {
                        // This is the last line, so it becomes the new current_line
                        current_line.push_str(line);
                        guard.print_and_flush(line)?;
                    }
                }
            }
            _ => {}
        }
    }

    // Handle normal exit with proper terminal cleanup
    guard.ensure_clean_state();
    Ok(lines.join("\n"))
}

/// Public interface that maintains Option return for backward compatibility
/// System errors are logged but converted to None for a smoother user experience
pub fn prompt_multiline(prompt: &str) -> Option<String> {
    handle_interactive_result(prompt_multiline_internal(prompt))
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



pub fn open_editor_custom(
    content: Option<&str>,
    line: Option<u32>,
    editor_cmd: Option<&str>
) -> Result<String> {
    let editor = detect_editor(editor_cmd);

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
    let editor = detect_editor(editor_cmd);

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
