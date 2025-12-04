//! # Use Case Workflow
//!
//! Interactive use case management for creating and managing use cases.
//! Provides guided workflows for use case operations.

use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;

use crate::cli::interactive::{
    field_helpers::FieldHelpers, prompts, runner::InteractiveRunner, ui::UI,
    workflows::operations::use_case_operations,
};

/// Use case workflow handler
pub struct UseCaseWorkflow;

impl UseCaseWorkflow {
    /// Interactive use case creation workflow
    pub fn create_use_case() -> Result<()> {
        UI::show_section_header("Create Use Case", "🔄")?;

        let mut runner = InteractiveRunner::new();
        let methodologies = runner.get_installed_methodologies()?;

        if methodologies.is_empty() {
            UI::show_error(
                "No methodologies available. Please configure methodologies in your project.",
            )?;
            UI::pause_for_input()?;
            return Ok(());
        }

        // Step 1: Prompt for title and category first
        UI::show_info("\n📋 Required Fields")?;

        let title = Text::new("Title:")
            .with_help_message("A clear, descriptive title for the use case")
            .prompt()?;

        // Get category with abbreviation using the new workflow
        let (category, category_abbreviation) = use_case_operations::create_or_select_category()?;

        // Step 2: Collect views
        let views = match use_case_operations::collect_methodology_views(&runner) {
            Ok(views) => views,
            Err(e) => {
                UI::show_error(&format!("Failed to collect views: {}", e))?;
                UI::pause_for_input()?;
                return Ok(());
            }
        };

        // Always use interactive form for additional fields
        Self::fill_use_case_form(
            &mut runner,
            title,
            category,
            category_abbreviation,
            None,
            views,
        )?;

        UI::pause_for_input()?;
        Ok(())
    }

    /// Interactive form for filling use case fields
    fn fill_use_case_form(
        runner: &mut InteractiveRunner,
        title: String,
        category: String,
        category_abbreviation: String,
        description: Option<String>,
        views: Vec<(String, String)>,
    ) -> Result<()> {
        // Ask if user wants to fill additional fields
        let fill_additional = Confirm::new("Fill in additional fields now?")
            .with_default(false)
            .with_help_message("You can add description, author, reviewer, and other custom fields")
            .prompt()?;

        if !fill_additional {
            // Collect preconditions/postconditions before creating use case
            let preconditions = prompts::collect_conditions(
                "preconditions",
                false, // No references for simpler flow
                vec![],
            )?;

            let postconditions = prompts::collect_conditions("postconditions", false, vec![])?;

            // Create use case with just the basic fields and default priority
            let params = crate::cli::interactive::runner::CreateUseCaseWithViewsParams {
                title,
                category,
                category_abbreviation: category_abbreviation.clone(),
                description,
                priority: "Medium".to_string(),
                views: views.clone(),
                extra_fields: HashMap::new(),
            };
            let (use_case_id, message) = runner.create_use_case_with_views_and_fields(params)?;

            UI::show_success(&message)?;

            // Add preconditions if any
            if !preconditions.is_empty() {
                use crate::controller::UseCaseController;
                let mut uc_controller = UseCaseController::new()?;

                for condition in preconditions {
                    uc_controller.add_precondition(use_case_id.clone(), condition)?;
                }
            }

            // Add postconditions if any
            if !postconditions.is_empty() {
                use crate::controller::UseCaseController;
                let mut uc_controller = UseCaseController::new()?;

                for condition in postconditions {
                    uc_controller.add_postcondition(use_case_id.clone(), condition)?;
                }
            }

            // Show summary of created views
            UI::show_info("\n📄 Generated files:")?;
            for (methodology, level) in &views {
                println!("   • {}-{}.md", methodology, level);
            }

            // Ask if they want to create a scenario
            let create_scenario = Confirm::new("Create a scenario for this use case?")
                .with_default(false)
                .with_help_message("You can add scenarios to describe specific workflows")
                .prompt()?;

            if create_scenario {
                super::scenario::ScenarioWorkflow::create_scenario_for_use_case(&use_case_id)?;
            }

            UI::show_info("\n💡 You can edit the TOML files directly to add additional fields like author, reviewer, and custom methodology fields.")?;
            return Ok(());
        }

        UI::show_section_header("Additional Fields", "📝")?;

        // Priority (with default)
        let priority_options = vec!["Low", "Medium", "High", "Critical"];
        let priority = Select::new("Priority:", priority_options)
            .with_starting_cursor(1) // Default to "Medium"
            .with_help_message("Priority level for this use case")
            .prompt()?;

        // Description (if not already provided)
        let final_description = if description.is_some() {
            description
        } else {
            Text::new("Description:")
                .with_help_message("Brief description of what this use case accomplishes")
                .prompt_skippable()?
        };

        // Author (optional)
        let author = Text::new("Author (optional):")
            .with_help_message("Person who created this use case")
            .prompt_skippable()?;

        // Reviewer (optional)
        let reviewer = Text::new("Reviewer (optional):")
            .with_help_message("Person responsible for reviewing this use case")
            .prompt_skippable()?;

        // Collect preconditions
        let preconditions = prompts::collect_conditions(
            "preconditions",
            false, // No use case references for now (simpler flow)
            vec![],
        )?;

        // Collect postconditions
        let postconditions = prompts::collect_conditions(
            "postconditions",
            false, // No use case references for postconditions
            vec![],
        )?;

        // Collect methodology-specific field values
        let methodology_field_values =
            use_case_operations::prompt_methodology_fields(runner, &views)?;

        // Create the use case with additional fields (only truly extra fields)
        let mut extra_fields = HashMap::new();

        if let Some(auth) = author {
            if !auth.is_empty() {
                extra_fields.insert("author".to_string(), auth);
            }
        }

        if let Some(rev) = reviewer {
            if !rev.is_empty() {
                extra_fields.insert("reviewer".to_string(), rev);
            }
        }

        // Merge methodology field values into extra_fields
        extra_fields.extend(methodology_field_values);

        let params = crate::cli::interactive::runner::CreateUseCaseWithViewsParams {
            title,
            category,
            category_abbreviation,
            description: final_description,
            priority: priority.to_string(),
            views: views.clone(),
            extra_fields,
        };
        let (use_case_id, message) = runner.create_use_case_with_views_and_fields(params)?;

        UI::show_success(&message)?;

        // Add preconditions if any
        if !preconditions.is_empty() {
            use crate::controller::UseCaseController;
            let mut uc_controller = UseCaseController::new()?;

            for condition_text in preconditions {
                uc_controller.add_precondition(use_case_id.clone(), condition_text)?;
            }
        }

        // Add postconditions if any
        if !postconditions.is_empty() {
            use crate::controller::UseCaseController;
            let mut uc_controller = UseCaseController::new()?;

            for condition_text in postconditions {
                uc_controller.add_postcondition(use_case_id.clone(), condition_text)?;
            }
        }

        // Show summary of created views
        UI::show_info("\n📄 Generated files:")?;
        for (methodology, level) in &views {
            println!("   • {}-{}.md", methodology, level);
        }

        // Ask if they want to create a scenario
        let create_scenario = Confirm::new("Create a scenario for this use case?")
            .with_default(false)
            .with_help_message("You can add scenarios to describe specific workflows")
            .prompt()?;

        if create_scenario {
            super::scenario::ScenarioWorkflow::create_scenario_for_use_case(&use_case_id)?;
        }

        Ok(())
    }

    /// List all use cases
    pub fn list_use_cases() -> Result<()> {
        UI::show_section_header("Use Cases", "📋")?;

        let mut runner = InteractiveRunner::new();
        runner.list_use_cases()?;

        UI::pause_for_input()?;
        Ok(())
    }

    /// Show project status
    pub fn show_status() -> Result<()> {
        UI::show_section_header("Project Status", "📊")?;

        let mut runner = InteractiveRunner::new();
        runner.show_status()?;

        UI::pause_for_input()?;
        Ok(())
    }

    /// Interactive use case editing workflow
    pub fn edit_use_case() -> Result<()> {
        UI::show_section_header("Edit Use Case", "✏️")?;

        let mut runner = InteractiveRunner::new();

        // Get list of use cases
        let mut use_case_ids = runner.get_use_case_ids()?;

        if use_case_ids.is_empty() {
            UI::show_error("No use cases found. Please create a use case first.")?;
            UI::pause_for_input()?;
            return Ok(());
        }

        // Add cancel option
        use_case_ids.push("[Cancel]".to_string());

        // Let user select which use case to edit
        let selected_id = Select::new("Select use case to edit:", use_case_ids)
            .with_help_message("Choose the use case you want to modify")
            .prompt()?;

        if selected_id == "[Cancel]" {
            return Ok(());
        }

        // Load use case details
        let use_case = runner.get_use_case_details(&selected_id)?;

        // Show edit menu
        loop {
            UI::clear_screen()?;
            UI::show_section_header(&format!("Editing: {}", use_case.title), "✏️")?;
            UI::show_info(&format!("ID: {}", use_case.id))?;
            UI::show_info(&format!("Category: {}", use_case.category))?;

            let edit_options = vec![
                "Edit Basic Info (title, category, description, priority)",
                "Edit Methodology Fields",
                "Manage Views (add/remove)",
                "Manage Conditions (pre/post)",
                "Manage Scenarios",
                "Back to Menu",
            ];

            let choice = Select::new("What would you like to edit?", edit_options).prompt()?;

            match choice {
                "Edit Basic Info (title, category, description, priority)" => {
                    Self::edit_basic_info(&mut runner, &selected_id, &use_case)?
                }
                "Edit Methodology Fields" => {
                    Self::edit_methodology_fields(&mut runner, &selected_id, &use_case)?
                }
                "Manage Views (add/remove)" => {
                    Self::manage_views(&mut runner, &selected_id, &use_case)?
                }
                "Manage Conditions (pre/post)" => {
                    super::conditions::ConditionsWorkflow::manage_conditions(&selected_id)?
                }
                "Manage Scenarios" => {
                    super::scenario::ScenarioWorkflow::manage_scenarios(&selected_id)?
                }
                "Back to Menu" => break,
                _ => {}
            }

            // Reload use case after edits
            let _use_case = runner.get_use_case_details(&selected_id)?;
        }

        Ok(())
    }

    /// Edit basic use case information
    fn edit_basic_info(
        runner: &mut InteractiveRunner,
        use_case_id: &str,
        use_case: &crate::core::UseCase,
    ) -> Result<()> {
        UI::show_section_header("Edit Basic Information", "📝")?;

        // Title
        let new_title = Text::new("Title:")
            .with_default(&use_case.title)
            .with_help_message("Press Enter to keep current value")
            .prompt()?;

        let title = if new_title != use_case.title {
            Some(new_title)
        } else {
            None
        };

        // Category
        let new_category = Text::new("Category:")
            .with_default(&use_case.category)
            .with_help_message("Press Enter to keep current value")
            .prompt()?;

        let category = if new_category != use_case.category {
            Some(new_category)
        } else {
            None
        };

        // Description
        let current_desc = use_case.description.clone();
        let new_description = Text::new("Description:")
            .with_default(&current_desc)
            .with_help_message("Press Enter to keep current value")
            .prompt()?;

        let description = if new_description != current_desc {
            Some(new_description)
        } else {
            None
        };

        // Priority
        let priority_options = vec!["Low", "Medium", "High", "Critical"];
        let current_priority = format!("{:?}", use_case.priority);
        let priority_idx = priority_options
            .iter()
            .position(|&p| p == current_priority)
            .unwrap_or(1);

        let new_priority = Select::new("Priority:", priority_options)
            .with_starting_cursor(priority_idx)
            .with_help_message("Select priority level")
            .prompt()?;

        let priority = if new_priority != current_priority {
            Some(new_priority.to_string())
        } else {
            None
        };

        // Only update if something changed
        if title.is_none() && category.is_none() && description.is_none() && priority.is_none() {
            UI::show_info("No changes made.")?;
            return Ok(());
        }

        let result = runner.update_use_case(
            use_case_id.to_string(),
            title,
            category,
            description,
            priority,
        )?;

        UI::show_success(&result)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Edit methodology-specific fields
    fn edit_methodology_fields(
        runner: &mut InteractiveRunner,
        use_case_id: &str,
        use_case: &crate::core::UseCase,
    ) -> Result<()> {
        UI::show_section_header("Edit Methodology Fields", "🎯")?;

        // Get list of methodologies in this use case
        let methodologies: Vec<String> = use_case
            .views
            .iter()
            .map(|v| format!("{}:{}", v.methodology, v.level))
            .collect();

        if methodologies.is_empty() {
            UI::show_error("No methodology views found in this use case.")?;
            UI::pause_for_input()?;
            return Ok(());
        }

        // Let user select which methodology to edit
        let selected = Select::new("Select methodology to edit:", methodologies)
            .with_help_message("Choose which view's fields to modify")
            .prompt()?;

        let (methodology, level) = selected
            .split_once(':')
            .context("Invalid methodology format")?;

        // Collect field definitions for this methodology
        let views = vec![(methodology.to_string(), level.to_string())];
        let field_collection = runner.collect_methodology_fields(&views)?;

        if field_collection.fields.is_empty() {
            UI::show_info("No custom fields defined for this methodology.")?;
            UI::pause_for_input()?;
            return Ok(());
        }

        // Get current values
        let current_values = runner.get_methodology_field_values(use_case_id, methodology)?;

        // Prompt for each field
        let mut updated_fields = HashMap::new();

        UI::show_info(&format!("\n📋 {} Fields:", methodology))?;
        UI::show_info("Press Enter to skip fields you don't want to change.\n")?;

        for (field_name, field_def) in &field_collection.fields {
            let current_json = current_values.get(field_name);

            // Check if field is empty/new
            let is_new_field = current_json.is_none()
                || matches!(current_json, Some(serde_json::Value::Null))
                || matches!(current_json, Some(serde_json::Value::String(s)) if s.is_empty())
                || matches!(current_json, Some(serde_json::Value::Array(a)) if a.is_empty());

            let new_value = if is_new_field {
                // Use prompt_by_type for new/empty fields (shows description and example)
                FieldHelpers::prompt_by_type(
                    &field_def.field_type,
                    &field_def.label,
                    field_def.required,
                    field_def.description.as_deref(),
                    field_def.example.as_deref(),
                )?
            } else {
                // Use edit_by_type for existing fields (shows current value)
                let help_msg = field_def
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{} field", field_def.field_type));

                FieldHelpers::edit_by_type(
                    &field_def.field_type,
                    &field_def.label,
                    current_json,
                    &help_msg,
                )?
            };

            if let Some(value) = new_value {
                updated_fields.insert(field_name.clone(), value);
            }
        }

        if updated_fields.is_empty() {
            UI::show_info("No changes made.")?;
            return Ok(());
        }

        let result = runner.update_methodology_fields(use_case_id, methodology, updated_fields)?;

        UI::show_success(&result)?;
        UI::pause_for_input()?;
        Ok(())
    }

    /// Manage views (add/remove)
    fn manage_views(
        runner: &mut InteractiveRunner,
        use_case_id: &str,
        use_case: &crate::core::UseCase,
    ) -> Result<()> {
        UI::show_section_header("Manage Views", "👁️")?;

        // Show current views
        UI::show_info("Current views:")?;
        for view in &use_case.views {
            println!(
                "  • {}:{} {}",
                view.methodology,
                view.level,
                if view.enabled { "" } else { "(disabled)" }
            );
        }

        let options = vec!["Add New View", "Remove View", "Back"];

        let choice = Select::new("What would you like to do?", options).prompt()?;

        match choice {
            "Add New View" => {
                let methodologies = runner.get_installed_methodologies()?;

                if methodologies.is_empty() {
                    UI::show_error("No methodologies available.")?;
                    UI::pause_for_input()?;
                    return Ok(());
                }

                // Select methodology
                let methodology_options: Vec<String> = methodologies
                    .iter()
                    .map(|m| format!("{} - {}", m.display_name, m.description))
                    .collect();

                let selected_idx =
                    Select::new("Select methodology:", methodology_options).prompt()?;

                let selected_methodology = &methodologies[methodologies
                    .iter()
                    .position(|m| format!("{} - {}", m.display_name, m.description) == selected_idx)
                    .context("Selected methodology not found")?];

                let methodology_name = selected_methodology.name.clone();

                // Get available levels
                let available_levels = runner.get_methodology_levels(&methodology_name)?;

                if available_levels.is_empty() {
                    UI::show_error(&format!(
                        "No levels available for methodology '{}'",
                        methodology_name
                    ))?;
                    UI::pause_for_input()?;
                    return Ok(());
                }

                // Select level
                let level_options: Vec<String> = available_levels
                    .iter()
                    .map(|level| {
                        let display_name = level
                            .name
                            .chars()
                            .enumerate()
                            .map(|(i, c)| {
                                if i == 0 {
                                    c.to_uppercase().next().unwrap()
                                } else {
                                    c
                                }
                            })
                            .collect::<String>();
                        format!("{} - {}", display_name, level.description)
                    })
                    .collect();

                let selected_level_display =
                    Select::new("Select level:", level_options).prompt()?;

                let level = selected_level_display
                    .split(" - ")
                    .next()
                    .context("Failed to parse level name")?
                    .to_lowercase();

                // Add the view
                let result = runner.add_view_to_use_case(use_case_id, &methodology_name, &level)?;

                UI::show_success(&result)?;
                UI::pause_for_input()?;
            }
            "Remove View" => {
                if use_case.views.len() == 1 {
                    UI::show_error("Cannot remove the last view from a use case.")?;
                    UI::pause_for_input()?;
                    return Ok(());
                }

                let view_options: Vec<String> = use_case
                    .views
                    .iter()
                    .map(|v| format!("{}:{}", v.methodology, v.level))
                    .collect();

                let selected = Select::new("Select view to remove:", view_options).prompt()?;

                let methodology = selected.split(':').next().context("Invalid view format")?;

                let confirm = Confirm::new(&format!("Remove view '{}'?", selected))
                    .with_default(false)
                    .prompt()?;

                if confirm {
                    let result = runner.remove_view_from_use_case(use_case_id, methodology)?;
                    UI::show_success(&result)?;
                } else {
                    UI::show_info("Removal cancelled.")?;
                }

                UI::pause_for_input()?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Interactive use case management menu
    pub fn manage_use_cases() -> Result<()> {
        UI::clear_screen()?;
        UI::show_section_header("Use Case Management", "📝")?;

        loop {
            let options = vec![
                "Create New Use Case",
                "Edit Use Case",
                "List All Use Cases",
                "Show Project Status",
                "Back to Main Menu",
            ];

            let choice = Select::new("What would you like to do?", options).prompt()?;

            match choice {
                "Create New Use Case" => Self::create_use_case()?,
                "Edit Use Case" => Self::edit_use_case()?,
                "List All Use Cases" => Self::list_use_cases()?,
                "Show Project Status" => Self::show_status()?,
                "Back to Main Menu" => break,
                _ => {}
            }
        }

        Ok(())
    }
}
