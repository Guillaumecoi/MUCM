/// Reinitialize command handlers for adding missing methodology fields.
use anyhow::Result;

use crate::cli::standard::runner::CliRunner;
use crate::presentation::DisplayResultFormatter;

/// Handle the reinitialize command to add missing methodology fields.
///
/// Scans use case TOML files and ensures all fields defined in their enabled
/// methodology views are present. Missing fields are initialized with empty values
/// based on their type. Existing field values are never overwritten.
///
/// # Arguments
/// * `runner` - CLI runner instance
/// * `use_case_id` - Optional specific use case to reinitialize (all if None)
/// * `dry_run` - If true, shows what would be added without making changes
pub fn handle_reinitialize_command(
    runner: &mut CliRunner,
    use_case_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let result = runner.reinitialize_methodology_fields(use_case_id, dry_run)?;
    DisplayResultFormatter::display(&result);
    Ok(())
}
