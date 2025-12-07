use crate::cli::standard::CliRunner;
use crate::controller::DisplayResult;
use crate::presentation::DisplayResultFormatter;
use anyhow::Result;

/// Handles the 'methodologies' CLI command.
///
/// Retrieves and displays the list of available methodologies
/// that can be used for structuring and generating use case documentation.
/// The output is printed to stdout for user reference.
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for listing methodologies.
///
/// # Returns
/// Returns `Ok(())` on successful display, or an error if retrieval fails.
pub fn handle_list_methodologies_command(runner: &mut CliRunner) -> Result<()> {
    let result = runner.list_methodologies()?;
    println!("{}", result);
    Ok(())
}

/// Handles the 'methodology-info' CLI command.
///
/// Retrieves and displays detailed information about a specific methodology,
/// including its structure, templates, and configuration options.
/// The output is printed to stdout for user reference.
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for retrieving methodology info.
/// * `name` - The name of the methodology to get information about.
///
/// # Returns
/// Returns `Ok(())` on successful display, or an error if the methodology is not found or retrieval fails.
pub fn handle_methodology_info_command(runner: &mut CliRunner, name: String) -> Result<()> {
    let result = runner.get_methodology_info(name)?;
    println!("{}", result);
    Ok(())
}

/// Handles the 'regenerate' CLI command.
///
/// Regenerates use case documentation using specified methodologies.
/// Supports multiple modes of operation based on the provided arguments:
/// - No arguments or --all flag: Regenerates all use cases with their current methodologies.
/// - With use_case_id only: Regenerates a single use case with its current methodology.
/// - With use_case_id and --methodology: Regenerates a single use case with a different methodology.
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for regeneration.
/// * `use_case_id` - Optional ID of the specific use case to regenerate.
/// * `methodology` - Optional name of the methodology to use for regeneration.
/// * `all` - Flag indicating whether to regenerate all use cases.
///
/// # Returns
/// Returns `Ok(())` on successful regeneration, or an error if regeneration fails or invalid arguments are provided.
pub fn handle_regenerate_command(
    runner: &mut CliRunner,
    use_case_id: Option<String>,
    methodology: Option<String>,
    all: bool,
) -> Result<()> {
    match (use_case_id, methodology, all) {
        // No args or --all flag: regenerate all use cases
        (None, None, _) | (None, Some(_), true) => match runner.regenerate_all_use_cases() {
            Ok(_) => {
                println!("✅ Regenerated all use case documentation");
                Ok(())
            }
            Err(e) => {
                DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
                std::process::exit(1);
            }
        },
        // Use case ID + methodology: regenerate with different methodology
        (Some(id), Some(method), _) => {
            match runner.regenerate_use_case_with_methodology(id, method) {
                Ok(result) => {
                    DisplayResultFormatter::display(&result);
                    if result.success {
                        Ok(())
                    } else {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
                    std::process::exit(1);
                }
            }
        }
        // Use case ID only: regenerate with current methodology
        (Some(id), None, _) => match runner.regenerate_use_case(id.clone()) {
            Ok(_) => {
                println!("✅ Regenerated documentation for {}", id);
                Ok(())
            }
            Err(e) => {
                DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
                std::process::exit(1);
            }
        },
        // --all with methodology but no ID: error (doesn't make sense)
        (None, Some(_), false) => {
            DisplayResultFormatter::display(&DisplayResult::error(
                "Cannot specify --methodology without a use case ID. To regenerate all, use: mucm regenerate".to_string()
            ));
            std::process::exit(1);
        }
    }
}

/// Handles the 'regenerate-tests' CLI command.
///
/// Regenerates test files while preserving user code in safe zones (START/END USER IMPLEMENTATION markers).
/// Can regenerate a single use case's test or all use case tests.
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for test regeneration.
/// * `use_case_id` - Optional ID of the specific use case to regenerate tests for.
/// * `all` - Flag indicating whether to regenerate all use case tests.
///
/// # Returns
/// Returns `Ok(())` on successful regeneration, or an error if regeneration fails.
pub fn handle_regenerate_tests_command(
    runner: &mut CliRunner,
    use_case_id: Option<String>,
    all: bool,
) -> Result<()> {
    match (use_case_id, all) {
        // No args or --all flag: regenerate all tests
        (None, _) => match runner.regenerate_all_tests() {
            Ok(result) => {
                DisplayResultFormatter::display(&result);
                if result.success {
                    Ok(())
                } else {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
                std::process::exit(1);
            }
        },
        // Use case ID: regenerate test for specific use case
        (Some(id), _) => match runner.regenerate_test(id.clone()) {
            Ok(result) => {
                DisplayResultFormatter::display(&result);
                if result.success {
                    Ok(())
                } else {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
                std::process::exit(1);
            }
        },
    }
}
