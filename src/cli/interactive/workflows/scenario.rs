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
                    println!(
                        "    • {} - {} [{}] ({} steps)",
                        scenario.id,
                        scenario.title,
                        scenario.scenario_type,
                        scenario.steps.len()
                    );
                }
                println!();
            }

            // Show action menu
            let actions = vec![
                "Create new scenario",
                "Edit scenario",
                "Delete scenario",
                "Back to use case menu",
            ];

            let choice = Select::new("What would you like to do?", actions).prompt()?;

            match choice {
                "Create new scenario" => {
                    Self::create_scenario(use_case_id)?;
                }
                "Edit scenario" => {
                    Self::edit_scenario(use_case_id)?;
                }
                "Delete scenario" => {
                    Self::delete_scenario(use_case_id)?;
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

    /// Create a new scenario interactively
    fn create_scenario(use_case_id: &str) -> Result<()> {
        UI::show_section_header("Create Scenario", "➕")?;

        let title = Text::new("Scenario title:")
            .with_help_message("Brief, descriptive title (e.g., 'User successfully logs in', 'Invalid password error')")
            .prompt()?;

        let scenario_types = vec!["main", "alternative", "exception"];
        let scenario_type = Select::new("Scenario type:", scenario_types).prompt()?;

        let description = Text::new("Description (optional):")
            .with_help_message("Describe what this scenario covers. Press Enter to skip.")
            .prompt()
            .ok();

        // Collect preconditions
        let preconditions = Self::collect_conditions("preconditions", use_case_id)?;

        // Collect postconditions
        let postconditions = Self::collect_conditions("postconditions", use_case_id)?;

        // Note: Actors are automatically derived from scenario steps, not manually assigned
        // Create the scenario
        let mut controller = ScenarioController::new()?;
        let result = controller.create_scenario(
            use_case_id.to_string(),
            title.clone(),
            scenario_type.to_string(),
            description,
            None, // persona_id removed from interactive workflow
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

        let mut conditions = Vec::new();
        loop {
            let condition = Text::new(&format!(
                "  {} (or press Enter to finish):",
                condition_type.trim_end_matches('s')
            ))
            .with_help_message(&format!(
                "Enter a text description of the {}",
                condition_type.trim_end_matches('s')
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
            println!("    Type: {}", scenario.scenario_type);
            println!("    Description: {}", scenario.description);
            println!("    Status: {}", scenario.status);
            println!("    Steps: {}", scenario.steps.len());
            println!("    Preconditions: {}", scenario.preconditions.len());
            println!("    Postconditions: {}", scenario.postconditions.len());
            if let Some(ref p) = scenario.persona {
                println!("    Persona: {}", p);
            }
            println!();

            let fields = vec![
                "Edit title",
                "Edit description",
                "Edit type",
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
                "Edit type" => {
                    let types = vec!["main", "alternative", "exception"];
                    let new_type = Select::new("New type:", types).prompt()?;

                    controller.edit_scenario(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        None,
                        None,
                        Some(new_type.to_string()),
                        None,
                    )?;

                    UI::show_success("✓ Type updated")?;
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
                    Self::manage_all_conditions_inline(use_case_id, scenario_id, &mut controller)?;
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

            let actions = vec![
                "Add step",
                "Insert step",
                "Edit step",
                "Remove step",
                "Reorder step",
                "Back",
            ];
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
                    let condition = Text::new("Enter precondition:")
                        .with_help_message("Describe what must be true before this scenario starts")
                        .prompt()?;

                    let result = controller.add_precondition(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        condition,
                    )?;

                    UI::show_success(&result.message)?;
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
                    let condition = Text::new("Enter postcondition:")
                        .with_help_message(
                            "Describe what must be true after this scenario completes",
                        )
                        .prompt()?;

                    let result = controller.add_postcondition(
                        use_case_id.to_string(),
                        scenario_id.to_string(),
                        condition,
                    )?;

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
}
