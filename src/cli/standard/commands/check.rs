/// Check/validate command handler
///
/// Validates use case fields for:
/// - Missing required fields
/// - Irrelevant fields (not defined in current methodology configuration)
///
/// Returns warnings only (not errors) to help identify potential issues.
use anyhow::Result;

use crate::cli::standard::CliRunner;

/// Handle the check/validate command
///
/// # Arguments
/// * `runner` - The CLI runner instance
/// * `use_case_id` - Optional specific use case to validate
///
/// # Returns
/// Ok(()) on successful validation, Err on failure
pub fn handle_check_command(runner: &mut CliRunner, use_case_id: Option<String>) -> Result<()> {
    let result = runner.validate_fields(use_case_id)?;
    println!("{}", result);
    Ok(())
}
