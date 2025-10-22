use crate::utils::output::OutputStyle;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("System error: {0}")]
    System(String),

    #[error("IO error: {0}")]
    Io(String),
}

/// Result type alias for consistent error handling across the application
pub type AppResult<T> = Result<T, AppError>;

pub enum FlowResult {
    NotFound {
        item_type: String,
        search_term: String,
    },
    EmptyList {
        item_type: String,
    },
    Cancelled(String),
    Success(String),
}

pub fn report_error(err: &AppError) {
    match err {
        AppError::Network(msg) => {
            println!("🌐 {}", OutputStyle::error(&format!("Network: {}", msg)));
        }
        AppError::Sync(msg) => {
            println!("⚠️  {}", OutputStyle::warning(&format!("Sync: {}", msg)));
        }
        AppError::Io(e) => {
            eprintln!("❌ {}", OutputStyle::error(e));
        }
        AppError::System(msg) => {
            eprintln!("❌ {}", OutputStyle::error(msg));
        }
    }
}

pub fn handle_flow(flow: FlowResult) {
    match flow {
        FlowResult::NotFound {
            item_type,
            search_term,
        } => {
            let msg = format!("{} '{}' not found", item_type, search_term);
            println!("⚠️  {}", OutputStyle::warning(&msg));
        }
        FlowResult::EmptyList { item_type } => {
            let msg = format!("No {} found", item_type);
            println!("{}", OutputStyle::muted(&msg));
        }
        FlowResult::Cancelled(msg) => {
            println!("⏹️  {}", OutputStyle::muted(&msg));
        }
        FlowResult::Success(msg) => {
            println!("✅ {}", OutputStyle::success(&msg));
        }
    }
}
