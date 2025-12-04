//! # Standard CLI Module
//!
//! Traditional command-line interface for the Markdown Use Case Manager.
//! Provides subcommand-based interaction for users who prefer standard CLI workflows.
//!
//! ## Architecture
//!
//! This module contains the standard (non-interactive) CLI implementation:
//! - `commands/`: Individual command handlers for different operations
//! - `runner.rs`: Business logic coordinator for CLI operations
//!
//! ## Commands
//!
//! - `init`: Initialize a new project
//! - `create`: Create use cases
//! - `list`: List use cases
//! - `methodologies`: Show available methodologies
//! - `languages`: Show available languages
//! - `status`: Show project status

mod commands;
mod runner;

// Re-export the CLI runner as the main interface
pub use runner::CliRunner;

// Re-export command functions for the main CLI dispatcher
pub use commands::{
    handle_actor_command, handle_cleanup_command, handle_create_command, handle_init_command,
    handle_languages_command, handle_list_command, handle_list_methodologies_command,
    handle_methodology_info_command, handle_migrate_command, handle_postcondition_add_command,
    handle_postcondition_list_command, handle_postcondition_remove_command,
    handle_precondition_add_command, handle_precondition_list_command,
    handle_precondition_remove_command, handle_reference_add_command,
    handle_reference_list_command, handle_reference_remove_command, handle_regenerate_command,
    handle_reinitialize_command, handle_status_command, handle_usecase_scenario_command,
};
