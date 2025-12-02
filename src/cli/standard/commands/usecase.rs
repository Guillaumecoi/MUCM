use crate::cli::standard::CliRunner;
use crate::controller::DisplayResult;
use crate::presentation::DisplayResultFormatter;
use anyhow::Result;

/// Handles the 'create' CLI command.
///
/// Creates a new use case with the specified title, category, and optional details.
/// Supports both single-view (legacy) and multi-view creation:
/// - If `views` is provided, creates a multi-view use case
/// - If `methodology` is provided, creates a single-view use case (legacy)
/// - Otherwise, uses the project's default methodology
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for use case creation.
/// * `title` - The title of the use case to create.
/// * `category` - The category under which the use case should be organized.
/// * `description` - Optional detailed description of the use case.
/// * `methodology` - Optional methodology to use for documentation generation (legacy).
/// * `views` - Optional comma-separated list of methodology:level pairs (e.g., "feature:simple,business:normal").
///
/// # Returns
/// Returns `Ok(())` on successful creation, or an error if creation fails.
pub fn handle_create_command(
    runner: &mut CliRunner,
    title: String,
    category: String,
    description: Option<String>,
    methodology: Option<String>,
    views: Option<String>,
) -> Result<()> {
    let result = if let Some(views_str) = views {
        // Multi-view creation
        match runner.create_use_case_with_views(title, category, description, views_str) {
            Ok(display_result) => display_result,
            Err(e) => DisplayResult::error(e.to_string()),
        }
    } else if let Some(methodology) = methodology {
        // Single-view with specific methodology (legacy)
        match runner.create_use_case_with_methodology(title, category, description, methodology) {
            Ok(display_result) => display_result,
            Err(e) => DisplayResult::error(e.to_string()),
        }
    } else {
        // Single-view with default methodology
        match runner.create_use_case(title, category, description) {
            Ok(display_result) => display_result,
            Err(e) => DisplayResult::error(e.to_string()),
        }
    };

    DisplayResultFormatter::display(&result);

    if result.success {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Handles the 'list' CLI command.
///
/// Retrieves and displays a list of all existing use cases in the project,
/// including their titles, categories, and current status.
/// The formatted list is printed to stdout for user reference.
///
/// # Arguments
/// * `runner` - A mutable reference to the CLI runner responsible for listing use cases.
///
/// # Returns
/// Returns `Ok(())` on successful display, or an error if retrieval fails.
pub fn handle_list_command(runner: &mut CliRunner) -> Result<()> {
    runner.list_use_cases()
}

/// Handle use case scenario commands
///
/// Dispatches to the appropriate scenario management function based on the command.
/// This is the new nested command structure: `mucm usecase scenario <subcommand>`
pub fn handle_usecase_scenario_command(
    _runner: &mut CliRunner,
    command: crate::cli::args::UseCaseScenarioCommands,
) -> Result<()> {
    use crate::cli::args::UseCaseScenarioCommands;
    use crate::controller::ScenarioController;

    let mut controller = ScenarioController::new()?;

    match command {
        UseCaseScenarioCommands::Add {
            use_case_id,
            title,
            scenario_type,
            description,
            persona,
        } => {
            let params = crate::controller::CreateScenarioParams {
                use_case_id,
                title,
                scenario_type,
                description,
                persona_id: persona,
                preconditions: None,
                postconditions: None,
            };
            let result = controller.create_scenario(params)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::Edit {
            use_case_id,
            scenario_id,
            title,
            description,
            scenario_type,
            status,
        } => {
            let result = controller.edit_scenario(
                use_case_id,
                scenario_id,
                title,
                description,
                scenario_type,
                status,
            )?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::Delete {
            use_case_id,
            scenario_id,
        } => {
            let result = controller.delete_scenario(use_case_id, scenario_id)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::List { use_case_id } => {
            let result = controller.list_scenarios(use_case_id)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::Step { command } => {
            handle_scenario_step_command(&mut controller, command)?;
        }
        UseCaseScenarioCommands::AssignPersona {
            use_case_id,
            scenario_id,
            persona_id,
        } => {
            let result = controller.assign_persona(use_case_id, scenario_id, persona_id)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::UnassignPersona {
            use_case_id,
            scenario_id,
        } => {
            let result = controller.unassign_persona(use_case_id, scenario_id)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioCommands::Reference { command } => {
            handle_scenario_reference_command(&mut controller, command)?;
        }
    }

    Ok(())
}

/// Handle scenario step commands
fn handle_scenario_step_command(
    controller: &mut crate::controller::ScenarioController,
    command: crate::cli::args::ScenarioStepCommands,
) -> Result<()> {
    use crate::cli::args::ScenarioStepCommands;

    match command {
        ScenarioStepCommands::Add {
            use_case_id,
            scenario_id,
            description,
            order,
        } => {
            let result =
                controller.add_step(use_case_id, scenario_id, description, order, None, None)?;
            DisplayResultFormatter::display(&result);
        }
        ScenarioStepCommands::Edit {
            use_case_id,
            scenario_id,
            order,
            description,
        } => {
            let result =
                controller.edit_step(use_case_id, scenario_id, order, None, description)?;
            DisplayResultFormatter::display(&result);
        }
        ScenarioStepCommands::Remove {
            use_case_id,
            scenario_id,
            order,
        } => {
            let result = controller.remove_step(use_case_id, scenario_id, order)?;
            DisplayResultFormatter::display(&result);
        }
    }

    Ok(())
}

/// Handle scenario reference commands
fn handle_scenario_reference_command(
    controller: &mut crate::controller::ScenarioController,
    command: crate::cli::args::UseCaseScenarioReferenceCommands,
) -> Result<()> {
    use crate::cli::args::UseCaseScenarioReferenceCommands;
    use crate::core::{ReferenceType, ScenarioReference};

    match command {
        UseCaseScenarioReferenceCommands::Add {
            use_case_id,
            scenario_id,
            target_id,
            ref_type,
            relationship,
            description,
        } => {
            // Parse reference type
            let parsed_type = match ref_type.to_lowercase().as_str() {
                "scenario" => ReferenceType::Scenario,
                "usecase" => ReferenceType::UseCase,
                _ => {
                    let result = DisplayResult::error(format!(
                        "Invalid reference type: '{}'. Must be 'scenario' or 'usecase'",
                        ref_type
                    ));
                    DisplayResultFormatter::display(&result);
                    return Ok(());
                }
            };

            let reference = ScenarioReference {
                ref_type: parsed_type,
                target_id,
                relationship,
                description,
            };

            // Add reference via controller
            let result = controller.add_reference(use_case_id, scenario_id, reference)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioReferenceCommands::Remove {
            use_case_id,
            scenario_id,
            target_id,
            relationship,
        } => {
            let result =
                controller.remove_reference(use_case_id, scenario_id, target_id, relationship)?;
            DisplayResultFormatter::display(&result);
        }
        UseCaseScenarioReferenceCommands::List {
            use_case_id,
            scenario_id,
        } => {
            let references = controller.list_references(use_case_id, scenario_id.clone())?;

            if references.is_empty() {
                println!("\nNo references found for scenario {}\n", scenario_id);
            } else {
                println!("\nReferences for scenario {}:", scenario_id);
                for reference in references {
                    println!(
                        "  • {} → {} ({})",
                        reference.relationship, reference.target_id, reference.ref_type
                    );
                    if let Some(desc) = reference.description {
                        println!("    {}", desc);
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
