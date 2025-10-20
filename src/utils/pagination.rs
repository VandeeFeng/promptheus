use anyhow::Result;
use crossterm::terminal::size;

/// Get terminal size (rows, columns)
pub fn get_terminal_size() -> Result<(u16, u16)> {
    match size() {
        Ok((width, height)) => Ok((height, width)),
        Err(_) => {
            // Fallback to default terminal size
            Ok((24, 80))
        }
    }
}

/// Check if content should be paginated based on terminal height
pub fn should_paginate(content: &str, terminal_height: u16) -> bool {
    let line_count = content.lines().count() as u16;
    // Use pagination if content exceeds 2/3 of terminal height
    line_count > (terminal_height * 2 / 3)
}

/// Display content using minus pager for static content
pub fn paginate_static_content(content: &str) -> Result<()> {
    let pager = minus::Pager::new();
    pager.push_str(content)?;

    match minus::page_all(pager) {
        Ok(_) => {
            // Pager finished normally, continue execution
            Ok(())
        },
        Err(_) => {
            Ok(())
        }
    }
}

