use crate::utils::error::{AppError, AppResult};
use crossterm::terminal::size;

/// Get terminal size (rows, columns)
pub fn get_terminal_size() -> AppResult<(u16, u16)> {
    size()
        .map(|(width, height)| (height, width))
        .map_err(|e| AppError::System(format!("Failed to get terminal size: {}", e)))
}

/// Check if content should be paginated based on terminal height
pub fn should_paginate(content: &str, terminal_height: u16) -> bool {
    let line_count = content.lines().count() as u16;
    // Use pagination if content exceeds 2/3 of terminal height
    line_count > (terminal_height * 2 / 3)
}

/// Display content using minus pager for static content
pub fn paginate_static_content(content: &str) -> AppResult<()> {
    let pager = minus::Pager::new();
    pager
        .push_str(content)
        .map_err(|e| AppError::System(format!("Failed to push content to pager: {}", e)))?;

    if let Err(e) = minus::page_all(pager) {
        // Don't propagate error if user quits pager (e.g., Ctrl+C)
        if e.to_string().to_lowercase().contains("abort") {
            return Ok(());
        }
        return Err(AppError::System(format!("Failed to run pager: {}", e)));
    }

    Ok(())
}
