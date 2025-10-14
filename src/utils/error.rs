use crate::utils::output::OutputStyle;

/// Unified error handling and user feedback utilities
///
/// Distinguishes between two types of situations:
/// 1. System errors: Technical issues that should return Err
/// 2. Flow results: Normal business logic results that need user notification
///
/// System-level error output (red)
/// Used for file, network, process, and other technical issues
pub fn print_system_error(message: &str) {
    eprintln!("❌ {}", OutputStyle::error(message));
}


/// Not found result notification (yellow)
/// Used for search without results, prompt not found, etc.
pub fn print_not_found(message: &str) {
    println!("⚠️  {}", OutputStyle::warning(message));
}

/// Empty result notification (gray)
/// Used for empty lists, no data, etc.
pub fn print_empty_result(message: &str) {
    println!("{}", OutputStyle::muted(message));
}

/// User cancelled operation notification
pub fn print_cancelled(message: &str) {
    println!("⏹️  {}", OutputStyle::muted(message));
}

/// Auto-sync failure notification (warning level)
pub fn print_sync_warning(message: &str) {
    println!("⚠️  {}", OutputStyle::warning(&format!("Sync: {}", message)));
}

/// Network issue notification
pub fn print_network_error(message: &str) {
    println!("🌐 {}", OutputStyle::error(&format!("Network: {}", message)));
}

/// Handle "not found" situation output uniformly
pub fn handle_not_found(item_type: &str, search_term: &str) {
    print_not_found(&format!("{} '{}' not found", item_type, search_term));
}

/// Handle empty list output uniformly
pub fn handle_empty_list(item_type: &str) {
    print_empty_result(&format!("No {} found", item_type));
}
