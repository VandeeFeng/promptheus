// Business logic management modules
pub mod crud;      // CRUD operations management
pub mod query;     // Query and execution management
pub mod sync;      // Synchronization operations management
pub mod config;    // Configuration management

// Re-export functions for backward compatibility
pub use crud::{handle_new_command, handle_show_command, handle_edit_command, handle_delete_command};
pub use query::{handle_list_command, handle_search_command, handle_exec_command};
pub use sync::{handle_sync_command, handle_push_command, handle_export_command};
pub use config::handle_config_command;