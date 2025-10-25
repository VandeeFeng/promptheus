// Business logic management modules
pub mod config;
pub mod crud; // CRUD operations management
pub mod query; // Query and execution management
pub mod sync; // Synchronization operations management // Configuration management

// Re-export functions for backward compatibility
pub use config::handle_config_command;
pub use crud::{
    handle_delete_command, handle_edit_command, handle_new_command, handle_show_command,
};
pub use query::{handle_exec_command, handle_list_command, handle_search_command};
pub use sync::{handle_export_command, handle_push_command, handle_sync_command};
