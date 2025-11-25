//! # Scenario Workflow
//!
//! Interactive scenario management within use cases.
//! Provides guided workflows for scenario operations.

use anyhow::Result;
use inquire::{Confirm, Select, Text};

use crate::cli::interactive::{runner::InteractiveRunner, ui::UI};
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
                "Create extension (alternative/exception)",
                "Edit scenario",
                "Delete scenario",
                "Advanced operations",
                "Validate scenarios",
                "Back to use case menu",
            ];

            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Create main scenario" => {
                    Self::create_scenario(use_case_id)?;
                }
                "Create extension (alternative/exception)" => {
                    Self::create_extension_scenario(use_case_id)?;
                }
                "Edit scenario" => {
                    Self::edit_scenario(use_case_id)?;
                }
                "Delete scenario" => {
                    Self::delete_scenario(use_case_id)?;
                }
                "Advanced operations" => {
                    Self::advanced_operations(use_case_id)?;
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

        // Collect preconditions
        let preconditions = Self::collect_conditions("preconditions", use_case_id)?;

        // Collect postconditions
        let postconditions = Self::collect_conditions("postconditions", use_case_id)?;

        // Controller handles creating main scenarios - no type selection needed
        let mut controller = ScenarioController::new()?;
        let result = controller.create_main_scenario(
            use_case_id.to_string(),
            title.clone(),
            description,
            preconditions,
            postconditions,
        )?;

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
                let actor = Self::select_actor_for_step()?;

                let add_receiver = Confirm::new("Add a receiving actor?")
                    .with_default(false)
                    .with_help_message("Does this action have a target/receiver?")
                    .prompt()?;

                let receiver = if add_receiver {
                    Self::select_actor_for_step()?
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
                    actor,
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

    /// Helper to collect preconditions or postconditions interactively
    /// Returns simple text conditions (use case references handled separately in manage_conditions_inline)
    fn collect_conditions(
        condition_type: &str,
        _current_use_case_id: &str,
    ) -> Result<Option<Vec<String>>> {
        let add_conditions = Confirm::new(&format!("Add {}?", condition_type))
            .with_default(false)
            .prompt()?;

        if !add_conditions {
            return Ok(None);
        }

        println!("\n  💡 Tip: You can add use case references later via 'Manage conditions'\n");

        let mut conditions = Vec::new();
        loop {
            let condition = Text::new(&format!(
                "  {} (or press Enter to finish):",
                condition_type.trim_end_matches('s')
            ))
            .with_help_message(&format!(
                "Enter a text description (e.g., 'User must be logged in')",
            ))
            .prompt()?;

            if condition.trim().is_empty() {
                break;
            }

            conditions.push(condition);

            let add_more = Confirm::new(&format!(
                "Add another {}?",
                condition_type.trim_end_matches('s')
            ))
            .with_default(true)
            .prompt()?;

            if !add_more {
                break;
            }
        }

        if conditions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(conditions))
        }
    }

    /// Helper to select multiple actors interactively
    /// Helper to select a single actor for a step
    fn select_actor_for_step() -> Result<Option<String>> {
        let runner = InteractiveRunner::new();
        let mut available_actors = runner.get_available_actors()?;

        if available_actors.is_empty() {
            println!("\n  No actors available. Using default 'Actor'.\n");
            return Ok(None);
        }

        // Add built-in actors
        available_actors.insert(0, "User".to_string());
        available_actors.insert(1, "System".to_string());
        available_actors.insert(2, "Default (Actor)".to_string());

        let choice = Select::new("Select actor for this step:", available_actors).prompt()?;

        if choice == "Default (Actor)" {
            Ok(None)
        } else if choice == "User" || choice == "System" {
            Ok(Some(choice))
        } else {
            // Extract actor ID from display string
            if let Some(id) = choice.split('(').nth(1).and_then(|s| s.strip_suffix(')')) {
                Ok(Some(format!("ref:{}", id)))
            } else {
                Ok(Some(choice))
            }
        }
    }

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

        // Select scenario to edit
        let mut scenario_options: Vec<String> = scenarios
            .iter()
            .map(|s| format!("{} - {}", s.id, s.title))
            .collect();

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
                        .receiver()
                        .map(|r| format!(" → {}", r.name()))
                        .unwrap_or_default();
                    println!(
                        "    {}. {}{} - {}",
                        step.order,
                        step.sender().name(),
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
                "Remove step",
                "Reorder step",
            ];

            // Only allow adding extensions from main scenarios
            if scenario.is_main {
                actions.push("Add extension (alternative/exception) from step");
            }

            actions.push("Back");

            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Add step" => {
                    let actor = Self::select_actor_for_step()?;

                    let add_receiver = Confirm::new("Add a receiving actor?")
                        .with_default(false)
                        .with_help_message("Does this action have a target/receiver?")
                        .prompt()?;

                    let receiver = if add_receiver {
                        Self::select_actor_for_step()?
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
                        actor,
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

                    let new_description = Text::new("New description:").prompt()?;

                    let result = controller.edit_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        step_order,
                        new_description,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Insert step" => {
                    if scenario.steps.is_empty() {
                        println!("\n  No steps yet. Use 'Add step' to create the first step.\n");
                        continue;
                    }

                    // Ask for position to insert
                    let mut position_options: Vec<String> = Vec::new();
                    position_options.push("At beginning (before step 1)".to_string());
                    for step in &scenario.steps {
                        position_options
                            .push(format!("After step {} ({})", step.order, step.action));
                    }

                    let position_choice =
                        Select::new("Where to insert the new step?", position_options).prompt()?;

                    let insert_order: u32 = if position_choice.starts_with("At beginning") {
                        1
                    } else {
                        // Extract step number and add 1
                        let after_step: u32 = position_choice
                            .split("step ")
                            .nth(1)
                            .and_then(|s| s.split(' ').next())
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(1);
                        after_step + 1
                    };

                    let actor = Self::select_actor_for_step()?;

                    let add_receiver = Confirm::new("Add a receiving actor?")
                        .with_default(false)
                        .with_help_message("Does this action have a target/receiver?")
                        .prompt()?;

                    let receiver = if add_receiver {
                        Self::select_actor_for_step()?
                    } else {
                        None
                    };

                    let description = Text::new("Step description:")
                        .with_help_message(
                            "Describe the action (e.g., 'enters credentials', 'validates input')",
                        )
                        .prompt()?;

                    // Add step at specified position
                    let result = controller.add_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        description,
                        Some(insert_order),
                        actor,
                        receiver,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Remove step" => {
                    if scenario.steps.is_empty() {
                        println!("\n  No steps to remove.\n");
                        continue;
                    }

                    let mut step_choices: Vec<String> = scenario
                        .steps
                        .iter()
                        .map(|s| format!("{}. {}", s.order, s.action))
                        .collect();
                    step_choices.push("[Cancel]".to_string());

                    let selected_step =
                        Select::new("Select step to remove:", step_choices).prompt()?;

                    if selected_step == "[Cancel]" {
                        continue;
                    }

                    let step_order: u32 =
                        selected_step.split('.').next().unwrap().trim().parse()?;

                    let result = controller.remove_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        step_order,
                    )?;

                    UI::show_success(&result.message)?;
                }
                "Reorder step" => {
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
                "Add extension (alternative/exception) from step" => {
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
                    let primary_actor =
                        Self::select_actor_for_step()?.unwrap_or_else(|| "User".to_string());

                    // Create the extension scenario
                    let result = controller.create_extension_scenario(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        extends_at_step,
                        returns_at_step,
                        title,
                        description.unwrap_or_default(),
                        primary_actor,
                    )?;

                    UI::show_success(&result.message)?;
                    println!(
                        "\n  💡 Tip: Use 'Edit scenario' to add steps to the new extension.\n"
                    );
                    UI::pause_for_input()?;
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

    /// Create an extension scenario that diverges from a main scenario
    fn create_extension_scenario(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Create Extension Scenario", "🔀")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        // Filter to show only main scenarios
        let main_scenarios: Vec<_> = scenarios.iter().filter(|s| s.is_main).collect();

        if main_scenarios.is_empty() {
            println!("\n  No main scenarios found. Create a main scenario first.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select parent scenario
        let scenario_options: Vec<String> = main_scenarios
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let selected = Select::new("Select main scenario to extend:", scenario_options).prompt()?;
        let parent_id = selected.split(" - ").next().unwrap();

        let parent = scenarios.iter().find(|s| s.id == parent_id).unwrap();

        // Select divergence point
        if parent.steps.is_empty() {
            println!("\n  Parent scenario has no steps. Add steps first.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let step_options: Vec<String> = parent
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let extends_step =
            Select::new("Extension diverges at which step?", step_options).prompt()?;
        let extends_at_step = extends_step
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Optional return point
        let add_return = Confirm::new("Does this extension return to the main scenario?")
            .with_default(false)
            .with_help_message("Some extensions rejoin the main flow, others end independently")
            .prompt()?;

        let returns_at_step = if add_return {
            let return_options: Vec<String> = parent
                .steps
                .iter()
                .filter(|s| s.order > extends_at_step)
                .map(|s| format!("Step {}: {}", s.order, s.action))
                .collect();

            if return_options.is_empty() {
                println!("\n  No steps after divergence point. Extension cannot return.\n");
                None
            } else {
                let return_step =
                    Select::new("Extension returns at which step?", return_options).prompt()?;
                Some(
                    return_step
                        .split(':')
                        .next()
                        .unwrap()
                        .replace("Step ", "")
                        .trim()
                        .to_string(),
                )
            }
        } else {
            None
        };

        // Get extension details
        let title = Text::new("Extension scenario title:")
            .with_help_message("Brief, descriptive title (e.g., 'Invalid credentials error')")
            .prompt()?;

        let description = Text::new("Description:")
            .with_help_message("Describe what this extension handles")
            .prompt()?;

        // Select primary actor
        let actor = Self::select_actor_for_step()?.unwrap_or_else(|| "User".to_string());

        // Create extension
        let result = controller.create_extension_scenario(
            use_case_id.to_string(),
            parent_id.to_string(),
            extends_at_step.clone(),
            returns_at_step.clone(),
            title.clone(),
            description,
            actor,
        )?;

        UI::show_success(&result.message)?;

        // Prompt to add steps
        let add_steps = Confirm::new("Add steps to this extension now?")
            .with_default(true)
            .prompt()?;

        if add_steps {
            // Extract scenario_id from result message
            let scenario_id = result
                .message
                .split(':')
                .nth(1)
                .and_then(|part| part.trim().split(" - ").next())
                .map(|id| id.trim())
                .unwrap_or("");

            if !scenario_id.is_empty() {
                println!("\n  📝 Adding steps to extension: {}\n", title);
                loop {
                    let step_actor = Self::select_actor_for_step()?;

                    let add_receiver = Confirm::new("Add a receiving actor?")
                        .with_default(false)
                        .prompt()?;

                    let receiver = if add_receiver {
                        Self::select_actor_for_step()?
                    } else {
                        None
                    };

                    let step_description = Text::new("Step description:").prompt()?;

                    let step_result = controller.add_step(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        step_description,
                        None,
                        step_actor,
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
        }

        UI::pause_for_input()?;
        Ok(())
    }

    /// Advanced scenario operations menu
    fn advanced_operations(use_case_id: &str) -> Result<()> {
        loop {
            UI::show_section_header("Advanced Scenario Operations", "⚙️")?;

            let actions = vec![
                "Add repeat block",
                "Remove repeat block",
                "Smart insert step (with extension updates)",
                "Smart delete step (with validation)",
                "Renumber steps",
                "Back",
            ];

            let choice = Select::new("Select operation:", actions).prompt()?;

            match choice {
                "Add repeat block" => {
                    Self::add_repeat_block(use_case_id)?;
                }
                "Remove repeat block" => {
                    Self::remove_repeat_block(use_case_id)?;
                }
                "Smart insert step (with extension updates)" => {
                    Self::smart_insert_step(use_case_id)?;
                }
                "Smart delete step (with validation)" => {
                    Self::smart_delete_step(use_case_id)?;
                }
                "Renumber steps" => {
                    Self::renumber_steps(use_case_id)?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Add a repeat block to a scenario
    fn add_repeat_block(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Add Repeat Block", "🔁")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        if scenarios.is_empty() {
            println!("\n  No scenarios available.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario
        let scenario_options: Vec<String> = scenarios
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let selected = Select::new("Select scenario:", scenario_options).prompt()?;
        let scenario_id = selected.split(" - ").next().unwrap();

        let scenario = scenarios.iter().find(|s| s.id == scenario_id).unwrap();

        if scenario.steps.len() < 2 {
            println!("\n  Need at least 2 steps for a repeat block.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select from_step
        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let from_choice = Select::new("Repeat block starts at:", step_options.clone()).prompt()?;
        let from_step = from_choice
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Select to_step (must be after from_step)
        let to_options: Vec<String> = scenario
            .steps
            .iter()
            .filter(|s| s.order > from_step)
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        if to_options.is_empty() {
            println!("\n  No steps after the selected start step.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let to_choice = Select::new("Repeat block ends at:", to_options).prompt()?;
        let to_step = to_choice
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Get condition
        let condition = Text::new("Repeat condition:")
            .with_help_message("Describe when to repeat (e.g., 'User has more items to process')")
            .prompt()?;

        let result = controller.add_repeat_block(
            use_case_id.to_string(),
            scenario_id.to_string(),
            from_step,
            to_step,
            condition,
        )?;

        UI::show_success(&result.message)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Remove a repeat block from a scenario
    fn remove_repeat_block(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Remove Repeat Block", "❌")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        // Filter scenarios with repeat blocks
        let scenarios_with_blocks: Vec<_> = scenarios
            .iter()
            .filter(|s| !s.repeat_blocks.is_empty())
            .collect();

        if scenarios_with_blocks.is_empty() {
            println!("\n  No scenarios with repeat blocks.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario
        let scenario_options: Vec<String> = scenarios_with_blocks
            .iter()
            .map(|s| {
                format!(
                    "{} - {} ({} repeat blocks)",
                    s.id,
                    s.title,
                    s.repeat_blocks.len()
                )
            })
            .collect();

        let selected = Select::new("Select scenario:", scenario_options).prompt()?;
        let scenario_id = selected.split(" - ").next().unwrap();

        let scenario = scenarios.iter().find(|s| s.id == scenario_id).unwrap();

        // Select repeat block
        let block_options: Vec<String> = scenario
            .repeat_blocks
            .iter()
            .map(|b| format!("Steps {} to {} ({})", b.from_step, b.to_step, b.condition))
            .collect();

        let block_choice = Select::new("Select repeat block to remove:", block_options).prompt()?;

        // Extract from_step and to_step
        let parts: Vec<&str> = block_choice.split_whitespace().collect();
        let from_step = parts[1].to_string();
        let to_step = parts[3].to_string();

        let result = controller.remove_repeat_block(
            use_case_id.to_string(),
            scenario_id.to_string(),
            from_step,
            to_step,
        )?;

        UI::show_success(&result.message)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Smart insert step with automatic extension updates
    fn smart_insert_step(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Smart Insert Step", "➕")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        // Filter to main scenarios only
        let main_scenarios: Vec<_> = scenarios.iter().filter(|s| s.is_main).collect();

        if main_scenarios.is_empty() {
            println!("\n  No main scenarios. Smart insert only works on main scenarios.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario
        let scenario_options: Vec<String> = main_scenarios
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let selected = Select::new("Select main scenario:", scenario_options).prompt()?;
        let scenario_id = selected.split(" - ").next().unwrap();

        let scenario = scenarios.iter().find(|s| s.id == scenario_id).unwrap();

        if scenario.steps.is_empty() {
            println!("\n  No steps. Use regular 'Add step' first.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select insertion point
        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("After step {}: {}", s.order, s.action))
            .collect();

        let after_choice = Select::new("Insert after which step?", step_options).prompt()?;
        let after_step = after_choice
            .split(':')
            .next()
            .unwrap()
            .replace("After step ", "")
            .trim()
            .to_string();

        // Get step details
        let actor = Self::select_actor_for_step()?.unwrap_or_else(|| "User".to_string());

        let add_receiver = Confirm::new("Add a receiving actor?")
            .with_default(false)
            .prompt()?;

        let receiver = if add_receiver {
            Self::select_actor_for_step()?
        } else {
            None
        };

        let action = Text::new("Step action:").prompt()?;

        let expected_result = Text::new("Expected result (optional):")
            .prompt()
            .ok()
            .filter(|s| !s.is_empty());

        let result = controller.insert_step_smart(
            use_case_id.to_string(),
            scenario_id.to_string(),
            after_step,
            actor,
            receiver,
            action,
            expected_result,
        )?;

        UI::show_success(&result.message)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Smart delete step with extension validation
    fn smart_delete_step(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Smart Delete Step", "🗑️")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        // Filter to main scenarios only
        let main_scenarios: Vec<_> = scenarios.iter().filter(|s| s.is_main).collect();

        if main_scenarios.is_empty() {
            println!("\n  No main scenarios. Smart delete only works on main scenarios.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario
        let scenario_options: Vec<String> = main_scenarios
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let selected = Select::new("Select main scenario:", scenario_options).prompt()?;
        let scenario_id = selected.split(" - ").next().unwrap();

        let scenario = scenarios.iter().find(|s| s.id == scenario_id).unwrap();

        if scenario.steps.is_empty() {
            println!("\n  No steps to delete.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select step to delete
        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let step_choice = Select::new("Select step to delete:", step_options).prompt()?;
        let step_order = step_choice
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Confirm deletion
        let confirm = Confirm::new(&format!("Delete step {}?", step_order))
            .with_default(false)
            .with_help_message("This will check if any extensions are affected")
            .prompt()?;

        if !confirm {
            println!("\n  Deletion cancelled.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let result = controller.delete_step_smart(
            use_case_id.to_string(),
            scenario_id.to_string(),
            step_order,
        )?;

        UI::show_success(&result.message)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Renumber steps in a scenario
    fn renumber_steps(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Renumber Steps", "🔢")?;

        let mut controller = ScenarioController::new()?;
        let scenarios = controller.get_scenarios(use_case_id)?;

        if scenarios.is_empty() {
            println!("\n  No scenarios available.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select scenario
        let scenario_options: Vec<String> = scenarios
            .iter()
            .map(|s| format!("{} - {} ({} steps)", s.id, s.title, s.steps.len()))
            .collect();

        let selected = Select::new("Select scenario:", scenario_options).prompt()?;
        let scenario_id = selected.split(" - ").next().unwrap();

        let scenario = scenarios.iter().find(|s| s.id == scenario_id).unwrap();

        if scenario.steps.len() < 2 {
            println!("\n  Need at least 2 steps to renumber.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Select starting point
        let step_options: Vec<String> = scenario
            .steps
            .iter()
            .map(|s| format!("Step {}: {}", s.order, s.action))
            .collect();

        let from_choice = Select::new("Renumber starting from:", step_options).prompt()?;
        let from_step = from_choice
            .split(':')
            .next()
            .unwrap()
            .replace("Step ", "")
            .trim()
            .to_string();

        // Get increment
        let increment_str = Text::new("Increment amount:")
            .with_help_message(
                "Positive to shift forward, negative to shift backward (e.g., 1, -1, 2)",
            )
            .with_default("1")
            .prompt()?;

        let increment: i32 = increment_str.parse().unwrap_or(1);

        let confirm = Confirm::new(&format!(
            "Renumber steps from {} with increment {}?",
            from_step, increment
        ))
        .with_default(true)
        .prompt()?;

        if !confirm {
            println!("\n  Renumbering cancelled.\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        let result = controller.renumber_steps(
            use_case_id.to_string(),
            scenario_id.to_string(),
            from_step,
            increment,
        )?;

        UI::show_success(&result.message)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Validate all scenarios in a use case
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
