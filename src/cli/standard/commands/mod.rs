/// CLI command handlers.
///
/// These modules contain thin wrapper functions for CLI commands. They handle
/// command-line specific concerns like output formatting, but delegate all
/// business logic to the CliRunner. This separation keeps the CLI layer
/// focused on user interaction while the runner manages domain operations.
// Private modules
mod actor;
mod check;
mod cleanup;
mod fields;
mod language;
mod methodology;
mod migrate;
mod project;
mod reinitialize;
mod usecase;

// Explicit public exports
pub use actor::handle_actor_command;
pub use check::handle_check_command;
pub use cleanup::handle_cleanup_command;
pub use fields::{
    handle_postcondition_add_command, handle_postcondition_list_command,
    handle_postcondition_remove_command, handle_precondition_add_command,
    handle_precondition_list_command, handle_precondition_remove_command,
    handle_reference_add_command, handle_reference_list_command, handle_reference_remove_command,
};
pub use language::handle_languages_command;
pub use methodology::{
    handle_list_methodologies_command, handle_methodology_info_command, handle_regenerate_command,
};
pub use migrate::handle_migrate_command;
pub use project::{handle_init_command, handle_status_command};
pub use reinitialize::handle_reinitialize_command;
pub use usecase::{handle_create_command, handle_list_command, handle_usecase_scenario_command};
