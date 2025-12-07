//! # Scenario Workflow
//!
//! Interactive scenario management within use cases.
//! Provides guided workflows for scenario operations.

use anyhow::Result;
use inquire::{Confirm, Select, Text};

use crate::cli::interactive::{prompts, runner::InteractiveRunner, ui::UI};
use crate::controller::ScenarioController;

/// Scenario workflow handler
pub struct ScenarioWorkflow;

impl ScenarioWorkflow {
    /// Main scenario management entry point for a use case
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to manage scenarios for
    pub fn manage_scenarios(use_case_id: &str) -> Result<()> {
        loop {
            UI::show_section_header(&format!("Manage Scenarios - {}", use_case_id), "🎬")?;

            // Show existing scenarios
            let mut controller = ScenarioController::new()?;
            let scenarios = controller.get_scenarios(use_case_id)?;

            if scenarios.is_empty() {
                println!("\n  No scenarios yet.\n");
            } else {
                println!("\n  Existing scenarios:");
                for scenario in &scenarios {
                    if scenario.is_main {
                        println!(
                            "    • {} - {} [Main] ({} steps)",
                            scenario.id,
                            scenario.title,
                            scenario.steps.len()
                        );
                    } else {
                        let extension_info = if let Some(ref parent) = scenario.extends_scenario_id
                        {
                            format!(
                                " → extends {} at step {}",
                                parent,
                                scenario
                                    .extends_at_step
                                    .as_ref()
                                    .unwrap_or(&"?".to_string())
                            )
                        } else {
                            String::new()
                        };
                        println!(
                            "    • {} - {} [{}] ({} steps){}",
                            scenario.id,
                            scenario.title,
                            scenario.scenario_type,
                            scenario.steps.len(),
                            extension_info
                        );
                    }
                }
                println!();
            }

            // Show action menu
            let actions = vec![
                "Create main scenario",
                "Create alternative scenario",
                "Create exception scenario",
                "Edit scenario",
                "Delete scenario",
                "Validate scenarios",
                "Back to use case menu",
            ];

            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Create main scenario" => {
                    Self::create_scenario(use_case_id)?;
                }
                "Create alternative scenario" => {
                    Self::create_alternative_scenario(use_case_id)?;
                }
                "Create exception scenario" => {
                    Self::create_exception_scenario(use_case_id)?;
                }
                "Edit scenario" => {
                    Self::edit_scenario(use_case_id)?;
                }
                "Delete scenario" => {
                    Self::delete_scenario(use_case_id)?;
                }
                "Validate scenarios" => {
                    Self::validate_scenarios(use_case_id)?;
                }
                "Back to use case menu" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a scenario for a specific use case (called after use case creation)
    pub fn create_scenario_for_use_case(use_case_id: &str) -> Result<()> {
        Self::create_scenario(use_case_id)
    }

    /// Create a new main scenario interactively
    /// Note: Alternatives and exceptions are created via "Create extension scenario" or from step management
    fn create_scenario(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Create Main Scenario", "➕")?;

        println!("  ℹ️  Creating a main scenario (happy path).");
        println!("  To add alternatives/exceptions, use 'Create extension scenario' or add from a step.\n");

        let title = Text::new("Scenario title:")
            .with_help_message("Brief, descriptive title (e.g., 'User successfully logs in')")
            .prompt()?;

        let description = Text::new("Description (optional):")
            .with_help_message("Describe what this scenario covers. Press Enter to skip.")
            .prompt()
            .ok();

        // Prompt for status
        let statuses = vec!["Planned", "InProgress", "Implemented", "Tested", "Deployed"];
        let status = Select::new("Status:", statuses)
            .with_help_message("Select the current status of this scenario")
            .prompt()?;

        // Collect preconditions using prompts
        let preconditions = prompts::collect_conditions(
            "preconditions",
            false, // No references for simpler flow
            vec![],
        )?;
        let preconditions = if preconditions.is_empty() {
            None
        } else {
            Some(preconditions)
        };

        // Collect postconditions using prompts
        let postconditions = prompts::collect_conditions("postconditions", false, vec![])?;
        let postconditions = if postconditions.is_empty() {
            None
        } else {
            Some(postconditions)
        };

        // Select primary actor
        let primary_actor = prompts::select_actor("Select primary actor:")?;

        // Controller handles creating main scenarios - no type selection needed
        let mut controller = ScenarioController::new()?;
        let result = controller.create_main_scenario(
            use_case_id.to_string(),
            title.clone(),
            description,
            preconditions,
            postconditions,
            primary_actor,
        )?;

        // Update status if not default
        if status != "Planned" {
            let scenario_id = result
                .message
                .split(':')
                .nth(1)
                .and_then(|part| part.trim().split(" - ").next())
                .map(|id| id.trim())
                .unwrap_or("");
            if !scenario_id.is_empty() {
                controller.edit_scenario(
                    use_case_id.to_string(),
                    scenario_id.to_string(),
                    None,
                    None,
                    None,
                    Some(status.to_string()),
                )?;
            }
        }

        UI::show_success(&result.message)?;

        // Extract scenario_id from success message (format: "✅ Created scenario: UC-XXX-S## - Title")
        let scenario_id = result
            .message
            .split(':')
            .nth(1)
            .and_then(|part| part.trim().split(" - ").next())
            .map(|id| id.trim())
            .unwrap_or("");

        // Prompt to add steps immediately after creation
        let add_steps = Confirm::new("Add steps to this scenario now?")
            .with_default(true)
            .with_help_message("You can also add steps later via Edit Scenario")
            .prompt()?;

        if add_steps && !scenario_id.is_empty() {
            println!("\n  📝 Adding steps to: {}\n", title);
            loop {
                let actor = prompts::select_actor("Select actor for this step:")?;

                let add_receiver = Confirm::new("Add a receiving actor?")
                    .with_default(false)
                    .with_help_message("Does this action have a target/receiver?")
                    .prompt()?;

                let receiver = if add_receiver {
                    Some(prompts::select_actor("Select receiving actor:")?)
                } else {
                    None
                };

                let description = Text::new("Step description:")
                    .with_help_message(
                        "Describe the action (e.g., 'enters credentials', 'validates input')",
                    )
                    .prompt()?;

                let step_result = controller.add_step(
                    use_case_id.to_string(),
                    scenario_id.to_string(),
                    description,
                    None,
                    Some(actor),
                    receiver,
                )?;

                UI::show_success(&step_result.message)?;

                let add_more = Confirm::new("Add another step?")
                    .with_default(true)
                    .prompt()?;

                if !add_more {
                    break;
                }
            }
        }

        UI::pause_for_input()?;

        Ok(())
    }

    /// Create an alternative scenario from the top-level menu
    fn create_alternative_scenario(use_case_id: &str) -> Result<()> {
        Self::create_extension_scenario_top_level(use_case_id, "alternative")
    }

    /// Create an exception scenario from the top-level menu
    fn create_exception_scenario(use_case_id: &str) -> Result<()> {
        Self::create_extension_scenario_top_level(use_case_id, "exception")
    }

    /// Helper function to create alternative or exception scenarios from top-level menu
    fn create_extension_scenario_top_level(use_case_id: &str, scenario_type: &str) -> Result<()> {
        UI::show_section_header(
            &format!("Create {} Scenario", scenario_type.to_uppercase()),
            "🔀",
        )?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        // For exceptions, allow extending both main and alternative scenarios
        // For alternatives, only allow extending main scenarios (existing behavior)
        let extendable_scenarios: Vec<_> = if scenario_type == "exception" {
            scenarios
                .iter()
                .filter(|s| {
                    s.is_main || s.scenario_type == crate::core::ScenarioType::AlternativeFlow
                })
                .collect()
        } else {
            scenarios.iter().filter(|s| s.is_main).collect()
        };

        if extendable_scenarios.is_empty() {
            println!(
                "\n  ❌ No {} scenarios found.",
                if scenario_type == "exception" {
                    "main or alternative"
                } else {
                    "main"
                }
            );
            println!(
                "  Create a {} scenario first before adding {} scenarios.",
                if scenario_type == "exception" {
                    "main or alternative"
                } else {
                    "main"
                },
                scenario_type
            );
            println!("  Go to: 'Create main scenario' in the menu above.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Filter to scenarios with steps
        let main_with_steps: Vec<_> = extendable_scenarios
            .iter()
            .filter(|s| !s.steps.is_empty())
            .copied()
            .collect();

        if main_with_steps.is_empty() {
            let scenario_label = if scenario_type == "exception" {
                "Scenarios"
            } else {
                "Main scenarios"
            };
            println!("\n  ⚠️  {} exist, but none have steps yet.", scenario_label);
            println!(
                "  Add at least one step to a scenario before creating {} scenarios.",
                scenario_type
            );
            println!("  Go to: Edit scenario → Manage steps → Add step\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select parent scenario
        let scenario_options: Vec<String> = main_with_steps
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let prompt_text = if scenario_type == "exception" {
            format!("Select scenario to extend with {}:", scenario_type)
        } else {
            format!("Select main scenario to extend with {}:", scenario_type)
        };

        let selected = Select::new(&prompt_text, scenario_options).prompt()?;
        let parent_id = selected.split(" - ").next().unwrap();

        // Get the selected scenario
        let parent_scenario = main_with_steps.iter().find(|s| s.id == parent_id).unwrap();

        // Select the divergence step
        let step_choices: Vec<String> = parent_scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let selected_step = Select::new(
            &format!("At which step should the {} diverge?", scenario_type),
            step_choices.clone(),
        )
        .prompt()?;

        let extends_at_step = selected_step
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Get title and description
        let title = Text::new(&format!("{} scenario title:", scenario_type))
            .with_help_message("E.g., 'Invalid password', 'Login with OAuth'")
            .prompt()?;

        let description = Text::new("Description (optional):")
            .with_help_message("Describe what happens in this scenario")
            .prompt()
            .ok();

        // Ask about return to main flow
        let should_return = Confirm::new(&format!(
            "Does this {} return to the main flow?",
            scenario_type
        ))
        .with_default(scenario_type == "alternative")
        .with_help_message("Alternatives typically return, exceptions typically don't")
        .prompt()?;

        let returns_at_step = if should_return {
            let return_step =
                Select::new("At which step does it return?", step_choices).prompt()?;
            Some(
                return_step
                    .split(':')
                    .next()
                    .unwrap()
                    .replace("Step ", "")
                    .trim()
                    .to_string(),
            )
        } else {
            None
        };

        // Select primary actor
        let primary_actor = prompts::select_actor(&format!(
            "Select primary actor for this {} scenario:",
            scenario_type
        ))?;

        // Prompt for status
        let statuses = vec!["Planned", "InProgress", "Implemented", "Tested", "Deployed"];
        let status = Select::new("Status:", statuses)
            .with_help_message("Select the current status of this scenario")
            .prompt()?;

        // Collect preconditions
        let preconditions = prompts::collect_conditions("preconditions", false, vec![])?;

        // Collect postconditions
        let postconditions = prompts::collect_conditions("postconditions", false, vec![])?;

        // Create the extension scenario
        let scenario_type_enum = match scenario_type.to_lowercase().as_str() {
            "alternative" => crate::core::ScenarioType::AlternativeFlow,
            "exception" => crate::core::ScenarioType::ExceptionFlow,
            _ => crate::core::ScenarioType::Extension,
        };

        let params = crate::controller::CreateExtensionParams {
            use_case_id: use_case_id.to_string(),
            parent_scenario_id: parent_id.to_string(),
            extends_at_step,
            returns_at_step,
            title: title.clone(),
            description: description.unwrap_or_default(),
            primary_actor,
            scenario_type: scenario_type_enum,
        };

        let result = controller.create_extension_scenario(params)?;

        UI::show_success(&result.message)?;

        // Extract scenario_id from success message
        let scenario_id = result
            .message
            .split(':')
            .nth(1)
            .and_then(|part| part.trim().split(" - ").next())
            .map(|id| id.trim())
            .unwrap_or("");

        // Update status if not default
        if !scenario_id.is_empty() && status != "Planned" {
            controller.edit_scenario(
                use_case_id.to_string(),
                scenario_id.to_string(),
                None,
                None,
                None,
                Some(status.to_string()),
            )?;
        }

        // Add preconditions
        if !scenario_id.is_empty() {
            for condition in preconditions {
                controller.add_precondition(
                    use_case_id.to_string(),
                    scenario_id.to_string(),
                    condition,
                )?;
            }
        }

        // Add postconditions
        if !scenario_id.is_empty() {
            for condition in postconditions {
                controller.add_postcondition(
                    use_case_id.to_string(),
                    scenario_id.to_string(),
                    condition,
                )?;
            }
        }

        // Prompt to add steps immediately after creation
        let add_steps = Confirm::new("Add steps to this scenario now?")
            .with_default(true)
            .with_help_message("You can also add steps later via Edit Scenario")
            .prompt()?;

        if add_steps && !scenario_id.is_empty() {
            println!("\n  📝 Adding steps to: {}\n", title);
            loop {
                let actor = prompts::select_actor("Select actor for this step:")?;

                let add_receiver = Confirm::new("Add a receiving actor?")
                    .with_default(false)
                    .with_help_message("Does this action have a target/receiver?")
                    .prompt()?;

                let receiver = if add_receiver {
                    Some(prompts::select_actor("Select receiving actor:")?)
                } else {
                    None
                };

                let description = Text::new("Step description:")
                    .with_help_message(
                        "Describe the action (e.g., 'enters credentials', 'validates input')",
                    )
                    .prompt()?;

                let step_result = controller.add_step(
                    use_case_id.to_string(),
                    scenario_id.to_string(),
                    description,
                    None,
                    Some(actor),
                    receiver,
                )?;

                UI::show_success(&step_result.message)?;

                let add_more = Confirm::new("Add another step?")
                    .with_default(true)
                    .prompt()?;

                if !add_more {
                    break;
                }
            }
        }

        UI::pause_for_input()?;

        Ok(())
    }

    /// Helper to select multiple actors interactively
    /// Helper to select a single actor for a step
    /// Edit an existing scenario
    fn edit_scenario(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Edit Scenario", "✏️")?;

        let mut controller = ScenarioController::new()?;
        let runner = InteractiveRunner::new();
        let scenarios = controller.get_scenarios(use_case_id)?;

        if scenarios.is_empty() {
            println!("\n  No scenarios to edit.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario to edit - group by type
        let mut scenario_options: Vec<String> = Vec::new();

        // Add main scenarios first
        let main_scenarios: Vec<_> = scenarios.iter().filter(|s| s.is_main).collect();
        if !main_scenarios.is_empty() {
            scenario_options.push("─── Main Scenarios ───".to_string());
            for s in main_scenarios {
                scenario_options.push(format!("{} - {} [Main]", s.id, s.title));
            }
        }

        // Add extension scenarios
        let extension_scenarios: Vec<_> = scenarios.iter().filter(|s| !s.is_main).collect();
        if !extension_scenarios.is_empty() {
            scenario_options.push("─── Extensions ───".to_string());
            for s in extension_scenarios {
                let ext_type = &s.scenario_type;
                scenario_options.push(format!("{} - {} [{}]", s.id, s.title, ext_type));
            }
        }

        // Add cancel option
        scenario_options.push("[Cancel]".to_string());

        let selected = Select::new("Select scenario to edit:", scenario_options).prompt()?;

        if selected == "[Cancel]" {
            return Ok(());
        }

        let scenario_id = selected.split(" - ").next().unwrap();

        // Get current scenario
        let _scenario = scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .unwrap()
            .clone();

        loop {
            // Refresh scenario data
            let scenario = controller.get_scenario(use_case_id, scenario_id)?;

            println!("\n  Current values:");
            println!("    Title: {}", scenario.title);
            println!(
                "    Kind: {}",
                if scenario.is_main {
                    "Main scenario"
                } else {
                    "Extension scenario"
                }
            );
            println!("    Type: {}", scenario.scenario_type);
            println!("    Description: {}", scenario.description);
            println!("    Status: {}", scenario.status);
            println!("    Primary Actor: {}", scenario.primary_actor);
            println!("    Steps: {}", scenario.steps.len());

            // Show extension information
            if !scenario.is_main {
                if let Some(ref parent) = scenario.extends_scenario_id {
                    println!(
                        "    Extends: {} at step {}",
                        parent,
                        scenario
                            .extends_at_step
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    );
                    if let Some(ref returns) = scenario.returns_at_step {
                        println!("    Returns at: step {}", returns);
                    } else {
                        println!("    Returns: doesn't return to main");
                    }
                }
            }

            println!("    Preconditions: {}", scenario.preconditions.len());
            println!("    Postconditions: {}", scenario.postconditions.len());
            if let Some(ref p) = scenario.persona {
                println!("    Persona: {}", p);
            }
            println!();

            let fields = vec![
                "Edit title",
                "Edit description",
                "Edit status",
                "Manage steps",
                "Manage conditions",
                "Done editing",
            ];

            let choice = Select::new("What would you like to edit?", fields).prompt()?;

            match choice {
                "Edit title" => {
                    let new_title = Text::new("New title:")
                        .with_default(&scenario.title)
                        .prompt()?;

                    controller.edit_scenario(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        Some(new_title),
                        None,
                        None,
                        None,
                    )?;

                    UI::show_success("✓ Title updated")?;
                }
                "Edit description" => {
                    let new_desc = Text::new("New description:")
                        .with_default(&scenario.description)
                        .prompt()?;

                    controller.edit_scenario(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        None,
                        Some(new_desc),
                        None,
                        None,
                    )?;

                    UI::show_success("✓ Description updated")?;
                }
                "Edit status" => {
                    let statuses =
                        vec!["Planned", "InProgress", "Implemented", "Tested", "Deployed"];
                    let new_status = Select::new("New status:", statuses).prompt()?;

                    controller.edit_scenario(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        None,
                        None,
                        None,
                        Some(new_status.to_string()),
                    )?;

                    UI::show_success("✓ Status updated")?;
                }
                "Manage steps" => {
                    Self::manage_steps_inline(use_case_id, scenario_id, &mut controller)?;
                }
                "Manage conditions" => {
                    Self::manage_all_conditions_inline(
                        use_case_id,
                        scenario_id,
                        &mut controller,
                        &runner,
                    )?;
                }
                "Done editing" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Delete a scenario
    fn delete_scenario(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Delete Scenario", "🗑️")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        if scenarios.is_empty() {
            println!("\n  No scenarios to delete.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario to delete
        let mut scenario_options: Vec<String> = scenarios
            .iter()
            .map(|s| format!("{} - {}", s.id, s.title))
            .collect();

        // Add cancel option
        scenario_options.push("[Cancel]".to_string());

        let selected = Select::new("Select scenario to delete:", scenario_options).prompt()?;

        if selected == "[Cancel]" {
            return Ok(());
        }

        let scenario_id = selected.split(" - ").next().unwrap();

        // Confirm deletion
        let confirm = Select::new(
            &format!("Are you sure you want to delete '{}'?", scenario_id),
            vec!["No", "Yes"],
        )
        .prompt()?;

        if confirm == "Yes" {
            let result =
                controller.delete_scenario(use_case_id.to_string(), scenario_id.to_string())?;
            UI::show_success(&result.message)?;
        } else {
            println!("\n✓ Deletion cancelled.");
        }

        UI::pause_for_input()?;
        Ok(())
    }

    /// Inline helper to manage steps within the edit scenario context
    fn manage_steps_inline(
        use_case_id: &str,
        scenario_id: &str,
        controller: &mut ScenarioController,
    ) -> Result<()> {
        loop {
            let scenario = controller.get_scenario(use_case_id, scenario_id)?;

            println!("\n  Current steps:");
            if scenario.steps.is_empty() {
                println!("    (no steps)");
            } else {
                for step in &scenario.steps {
                    let receiver_str = step
                        .receiving_actor()
                        .map(|r| format!(" → {}", r))
                        .unwrap_or_default();
                    println!(
                        "    {}. {}{} - {}",
                        step.order,
                        step.acting_actor(),
                        receiver_str,
                        step.action
                    );
                }
            }
            println!();

            let mut actions = vec![
                "Add step",
                "Insert step",
                "Edit step",
                "Delete step",
                "Reorder steps",
            ];

            // Only allow adding extensions and loops from main scenarios
            if scenario.is_main {
                actions.push("Create extension from step");
                actions.push("Add repeat block (loop)");
                actions.push("Remove repeat block");
            }

            actions.push("Back");

            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Add step" => {
                    let actor = prompts::select_actor("Select actor for this step:")?;

                    let add_receiver = Confirm::new("Add a receiving actor?")
                        .with_default(false)
                        .with_help_message("Does this action have a target/receiver?")
                        .prompt()?;

                    let receiver = if add_receiver {
                        Some(prompts::select_actor("Select actor for this step:")?)
                    } else {
                        None
                    };

                    let description = Text::new("Step description:")
                        .with_help_message(
                            "Describe the action (e.g., 'enters credentials', 'validates input')",
                        )
                        .prompt()?;

                    let result = controller.add_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        description,
                        None,
                        Some(actor),
                        receiver,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Edit step" => {
                    if scenario.steps.is_empty() {
                        println!("\n  No steps to edit.\n");
                        continue;
                    }

                    let step_choices: Vec<String> = scenario
                        .steps
                        .iter()
                        .map(|s| format!("{}. {}", s.order, s.action))
                        .collect();

                    let selected_step =
                        Select::new("Select step to edit:", step_choices).prompt()?;
                    let step_order: u32 =
                        selected_step.split('.').next().unwrap().trim().parse()?;

                    let actor = prompts::select_actor("Select actor for this step:")?;
                    let new_description = Text::new("Step description:").prompt()?;

                    let result = controller.edit_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        step_order,
                        Some(actor),
                        new_description,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Insert step" => {
                    Self::insert_step_inline(use_case_id, scenario_id, controller)?;
                }
                "Delete step" => {
                    Self::delete_step_inline(use_case_id, scenario_id, controller)?;
                }
                "Reorder steps" => {
                    if scenario.steps.len() < 2 {
                        println!("\n  Need at least 2 steps to reorder.\n");
                        continue;
                    }

                    let step_choices: Vec<String> = scenario
                        .steps
                        .iter()
                        .map(|s| format!("{}. {}", s.order, s.action))
                        .collect();

                    let selected_step =
                        Select::new("Select step to move:", step_choices).prompt()?;
                    let step_order: u32 =
                        selected_step.split('.').next().unwrap().trim().parse()?;

                    let move_options = vec!["Move up", "Move down", "Move to specific position"];
                    let move_choice =
                        Select::new("How to move this step?", move_options).prompt()?;

                    let new_order: u32 = match move_choice {
                        "Move up" => {
                            if step_order == 1 {
                                println!("\n  Step is already at the top.\n");
                                continue;
                            }
                            step_order - 1
                        }
                        "Move down" => {
                            if step_order >= scenario.steps.len() as u32 {
                                println!("\n  Step is already at the bottom.\n");
                                continue;
                            }
                            step_order + 1
                        }
                        "Move to specific position" => {
                            let position_input = Text::new("New position (1-based):")
                                .with_help_message(&format!(
                                    "Enter a number between 1 and {}",
                                    scenario.steps.len()
                                ))
                                .prompt()?;

                            let new_pos: u32 = position_input.trim().parse().unwrap_or(step_order);

                            if new_pos < 1 || new_pos > scenario.steps.len() as u32 {
                                println!("\n  Invalid position.\n");
                                continue;
                            }
                            new_pos
                        }
                        _ => step_order,
                    };

                    if new_order == step_order {
                        println!("\n  No change in position.\n");
                        continue;
                    }

                    // Build reordering map (using String keys and values)
                    let mut reorderings = std::collections::HashMap::new();

                    // Simple swap or shift logic
                    if (new_order as i32 - step_order as i32).abs() == 1 {
                        // Simple adjacent swap
                        reorderings.insert(step_order.to_string(), new_order.to_string());
                        reorderings.insert(new_order.to_string(), step_order.to_string());
                    } else {
                        // Complex reordering - move step and shift others
                        if new_order < step_order {
                            // Moving up
                            for i in new_order..step_order {
                                reorderings.insert(i.to_string(), (i + 1).to_string());
                            }
                            reorderings.insert(step_order.to_string(), new_order.to_string());
                        } else {
                            // Moving down
                            for i in (step_order + 1)..=new_order {
                                reorderings.insert(i.to_string(), (i - 1).to_string());
                            }
                            reorderings.insert(step_order.to_string(), new_order.to_string());
                        }
                    }

                    let result = controller.reorder_steps(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        reorderings,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Create extension from step" => {
                    if scenario.steps.is_empty() {
                        println!("\n  ⚠️  No steps available. Add steps first.\n");
                        UI::pause_for_input()?;
                        continue;
                    }

                    // Select the step where the extension diverges
                    let step_choices: Vec<String> = scenario
                        .steps
                        .iter()
                        .map(|s| format!("Step {}: {}", s.order, s.action))
                        .collect();

                    let selected_step = Select::new(
                        "At which step should the extension diverge?",
                        step_choices.clone(),
                    )
                    .prompt()?;

                    let extends_at_step = selected_step
                        .split(':')
                        .next()
                        .unwrap()
                        .replace("Step ", "")
                        .trim()
                        .to_string();

                    // Select extension type
                    let ext_types = vec!["alternative", "exception"];
                    let extension_type = Select::new("Extension type:", ext_types).prompt()?;

                    // Get title and description
                    let title = Text::new(&format!("{} scenario title:", extension_type))
                        .with_help_message("E.g., 'Invalid password', 'Network error'")
                        .prompt()?;

                    let description = Text::new("Description (optional):").prompt().ok();

                    // Ask about return point
                    let should_return =
                        Confirm::new("Does this extension return to the main flow?")
                            .with_default(false)
                            .with_help_message(
                                "Alternatives may return, exceptions typically don't",
                            )
                            .prompt()?;

                    let returns_at_step = if should_return {
                        let return_step =
                            Select::new("At which step does it return?", step_choices).prompt()?;
                        Some(
                            return_step
                                .split(':')
                                .next()
                                .unwrap()
                                .replace("Step ", "")
                                .trim()
                                .to_string(),
                        )
                    } else {
                        None
                    };

                    // Select primary actor
                    let primary_actor = prompts::select_actor("Select actor for this step:")?;

                    // Create the extension scenario
                    let params = crate::controller::CreateExtensionParams {
                        use_case_id: use_case_id.to_string(),
                        parent_scenario_id: scenario_id.to_string(),
                        extends_at_step,
                        returns_at_step,
                        title,
                        description: description.unwrap_or_default(),
                        primary_actor,
                        scenario_type: crate::core::ScenarioType::Extension,
                    };
                    let result = controller.create_extension_scenario(params)?;

                    UI::show_success(&result.message)?;
                    println!(
                        "\n  💡 Tip: Use 'Edit scenario' to add steps to the new extension.\n"
                    );
                    UI::pause_for_input()?;
                }
                "Add repeat block (loop)" => {
                    if scenario.steps.is_empty() {
                        println!("\n  ⚠️  No steps available. Add steps first.\n");
                        UI::pause_for_input()?;
                        continue;
                    }

                    // Select from_step
                    let step_choices: Vec<String> = scenario
                        .steps
                        .iter()
                        .map(|s| format!("Step {}: {}", s.order, s.action))
                        .collect();

                    let from_step_str = Select::new("Loop starts from step:", step_choices.clone())
                        .with_help_message("First step in the repeating block")
                        .prompt()?;
                    let from_step = from_step_str
                        .split(':')
                        .next()
                        .unwrap()
                        .replace("Step ", "")
                        .trim()
                        .to_string();

                    // Select to_step
                    let to_step_str = Select::new("Loop ends at step:", step_choices)
                        .with_help_message("Last step in the repeating block")
                        .prompt()?;
                    let to_step = to_step_str
                        .split(':')
                        .next()
                        .unwrap()
                        .replace("Step ", "")
                        .trim()
                        .to_string();

                    // Get condition
                    let condition = Text::new("Loop condition:")
                        .with_help_message("E.g., 'until payment succeeds', 'while retries < 3'")
                        .prompt()?;

                    let result = controller.add_repeat_block(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        from_step,
                        to_step,
                        condition,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Remove repeat block" => {
                    Self::remove_repeat_block_inline(use_case_id, scenario_id, controller)?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Inline helper to manage conditions (pre/post) within the edit scenario context
    /// Manage both preconditions and postconditions in a unified interface
    fn manage_all_conditions_inline(
        use_case_id: &str,
        scenario_id: &str,
        controller: &mut ScenarioController,
        runner: &InteractiveRunner,
    ) -> Result<()> {
        loop {
            let scenario = controller.get_scenario(use_case_id, scenario_id)?;

            UI::clear_screen()?;
            println!("\n  📋 Conditions for: {}\n", scenario.title);

            // Show preconditions
            println!("  ⬇️  Preconditions:");
            if scenario.preconditions.is_empty() {
                println!("    (none)");
            } else {
                for (i, cond) in scenario.preconditions.iter().enumerate() {
                    println!("    {}. {}", i + 1, cond);
                }
            }
            println!();

            // Show postconditions
            println!("  ⬆️  Postconditions:");
            if scenario.postconditions.is_empty() {
                println!("    (none)");
            } else {
                for (i, cond) in scenario.postconditions.iter().enumerate() {
                    println!("    {}. {}", i + 1, cond);
                }
            }
            println!();

            let actions = vec![
                "Add Precondition",
                "Remove Precondition",
                "Add Postcondition",
                "Remove Postcondition",
                "Back",
            ];
            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Add Precondition" => {
                    let condition_types = vec!["Text description", "Reference to use case"];
                    let cond_type = Select::new("Precondition type:", condition_types).prompt()?;

                    if cond_type == "Text description" {
                        let condition = Text::new("Enter precondition:")
                            .with_help_message(
                                "Describe what must be true before this scenario starts",
                            )
                            .prompt()?;

                        let result = controller.add_precondition(
                            use_case_id.to_string(),
                            scenario_id.to_string(),
                            condition,
                        )?;

                        UI::show_success(&result.message)?;
                    } else {
                        // Reference to use case
                        let runner = InteractiveRunner::new();
                        let all_use_cases = runner.get_available_use_cases()?;

                        if all_use_cases.is_empty() {
                            println!("\n  No use cases available to reference.\n");
                            UI::pause_for_input()?;
                            continue;
                        }

                        let uc_options: Vec<String> = all_use_cases
                            .iter()
                            .map(|uc| format!("{} - {}", uc.id, uc.title))
                            .collect();

                        let selected =
                            Select::new("Select use case to reference:", uc_options).prompt()?;
                        let referenced_uc_id = selected.split(" - ").next().unwrap();

                        let relationships = vec![
                            "must be completed",
                            "must be in progress",
                            "depends on",
                            "requires",
                        ];
                        let relationship = Select::new("Relationship:", relationships).prompt()?;

                        let text = Text::new("Precondition description:")
                            .with_help_message(&format!(
                                "E.g., 'User must complete {}'",
                                referenced_uc_id
                            ))
                            .with_default(&format!("{} {}", referenced_uc_id, relationship))
                            .prompt()?;

                        let result = controller.add_precondition_with_use_case(
                            use_case_id.to_string(),
                            scenario_id.to_string(),
                            text,
                            referenced_uc_id.to_string(),
                            relationship.to_string(),
                        )?;

                        UI::show_success(&result.message)?;
                    }
                }
                "Remove Precondition" => {
                    if scenario.preconditions.is_empty() {
                        println!("\n  No preconditions to remove.\n");
                        UI::pause_for_input()?;
                        continue;
                    }

                    let mut options: Vec<String> = scenario
                        .preconditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect();
                    options.push("[Cancel]".to_string());
                    let selected =
                        Select::new("Select precondition to remove:", options).prompt()?;

                    if selected == "[Cancel]" {
                        continue;
                    }

                    // Find the matching condition by text
                    let condition_text = scenario
                        .preconditions
                        .iter()
                        .find(|c| c.to_string() == selected)
                        .map(|c| c.text.clone())
                        .unwrap_or_else(|| selected.clone());

                    let result = controller.remove_precondition(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        condition_text,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Add Postcondition" => {
                    // Let user choose between text or use case reference
                    let condition_type = Select::new(
                        "How would you like to specify the postcondition?",
                        vec!["Text description", "Reference to use case"],
                    )
                    .prompt()?;

                    let result = match condition_type {
                        "Text description" => {
                            let condition = Text::new("Enter postcondition:")
                                .with_help_message(
                                    "Describe what must be true after this scenario completes",
                                )
                                .prompt()?;

                            controller.add_postcondition(
                                use_case_id.to_string(),
                                scenario_id.to_string(),
                                condition,
                            )?
                        }
                        "Reference to use case" => {
                            // Get list of available use cases
                            let available_use_cases = runner.get_available_use_cases()?;
                            if available_use_cases.is_empty() {
                                println!("\n  No use cases available to reference.\n");
                                UI::pause_for_input()?;
                                continue;
                            }

                            let use_case_options: Vec<String> = available_use_cases
                                .iter()
                                .map(|uc| format!("{} - {}", uc.id, uc.title))
                                .collect();

                            let selected = Select::new(
                                "Select use case to reference:",
                                use_case_options.clone(),
                            )
                            .prompt()?;

                            // Extract the use case ID
                            let target_use_case_id =
                                selected.split(" - ").next().unwrap().to_string();

                            // Ask for relationship
                            let relationship = Text::new("Describe the relationship:")
                                .with_help_message(
                                    "e.g., 'must be completed', 'triggers', 'creates'",
                                )
                                .prompt()?;

                            // Ask for description text
                            let text = Text::new("Enter postcondition description:")
                                .with_help_message("Describe what must be true after this scenario")
                                .prompt()?;

                            controller.add_postcondition_with_use_case(
                                use_case_id.to_string(),
                                scenario_id.to_string(),
                                text,
                                target_use_case_id,
                                relationship,
                            )?
                        }
                        _ => unreachable!(),
                    };

                    UI::show_success(&result.message)?;
                }
                "Remove Postcondition" => {
                    if scenario.postconditions.is_empty() {
                        println!("\n  No postconditions to remove.\n");
                        UI::pause_for_input()?;
                        continue;
                    }

                    let mut options: Vec<String> = scenario
                        .postconditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect();
                    options.push("[Cancel]".to_string());
                    let selected =
                        Select::new("Select postcondition to remove:", options).prompt()?;

                    if selected == "[Cancel]" {
                        continue;
                    }

                    // Find the matching condition by text
                    let condition_text = scenario
                        .postconditions
                        .iter()
                        .find(|c| c.to_string() == selected)
                        .map(|c| c.text.clone())
                        .unwrap_or_else(|| selected.clone());

                    let result = controller.remove_postcondition(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        condition_text,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Inline helper to remove repeat block within manage_steps context
    fn remove_repeat_block_inline(
        use_case_id: &str,
        scenario_id: &str,
        controller: &mut ScenarioController,
    ) -> Result<()> {
        let scenario = controller.get_scenario(use_case_id, scenario_id)?;

        if scenario.repeat_blocks.is_empty() {
            println!("\n  No repeat blocks to remove.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let block_options: Vec<String> = scenario
            .repeat_blocks
            .iter()
            .map(|b| format!("Steps {} to {}: {}", b.from_step, b.to_step, b.condition))
            .collect();

        let selected = Select::new("Select repeat block to remove:", block_options).prompt()?;

        let from_step = selected
            .split("Steps ")
            .nth(1)
            .and_then(|s| s.split(" to ").next())
            .unwrap()
            .to_string();

        let to_step = selected
            .split(" to ")
            .nth(1)
            .and_then(|s| s.split(":").next())
            .unwrap()
            .to_string();

        let result = controller.remove_repeat_block(
            use_case_id.to_string(),
            scenario_id.to_string(),
            from_step,
            to_step,
        )?;

        UI::show_success(&result.message)?;
        Ok(())
    }

    /// Inline helper to insert step with auto-renumbering
    fn insert_step_inline(
        use_case_id: &str,
        scenario_id: &str,
        controller: &mut ScenarioController,
    ) -> Result<()> {
        let scenario = controller.get_scenario(use_case_id, scenario_id)?;

        if scenario.steps.is_empty() {
            println!("\n  No steps yet. Use 'Add step' first.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let after_step = Select::new("Insert after which step?", step_options).prompt()?;
        let after_step_order = after_step
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        let actor = prompts::select_actor("Select actor for this step:")?;

        let add_receiver = Confirm::new("Add a receiving actor?")
            .with_default(false)
            .prompt()?;

        let receiver = if add_receiver {
            Some(prompts::select_actor("Select actor for this step:")?)
        } else {
            None
        };

        let description = Text::new("Step description:").prompt()?;

        let params = crate::controller::InsertStepParams {
            use_case_id: use_case_id.to_string(),
            scenario_id: scenario_id.to_string(),
            after_step: after_step_order,
            actor,
            receiver,
            action: description,
            expected_result: None,
        };
        let result = controller.insert_step(params)?;

        UI::show_success(&result.message)?;
        Ok(())
    }

    /// Inline helper to delete step with auto-renumbering
    fn delete_step_inline(
        use_case_id: &str,
        scenario_id: &str,
        controller: &mut ScenarioController,
    ) -> Result<()> {
        let scenario = controller.get_scenario(use_case_id, scenario_id)?;

        if scenario.steps.is_empty() {
            println!("\n  No steps to delete.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let selected_step = Select::new("Select step to delete:", step_options).prompt()?;
        let step_order = selected_step
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        let result =
            controller.delete_step(use_case_id.to_string(), scenario_id.to_string(), step_order)?;

        UI::show_success(&result.message)?;
        Ok(())
    }

    /// Inline helper to renumber steps within manage_steps context
    fn validate_scenarios(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Validate Scenarios", "✅")?;

        let mut controller = ScenarioController::new()?;
        let result = controller.validate_scenarios(use_case_id.to_string())?;

        if result.is_success() {
            UI::show_success(&result.message)?;
        } else {
            println!("\n{}\n", result.message);
        }

        UI::pause_for_input()?;
        Ok(())
    }
}
