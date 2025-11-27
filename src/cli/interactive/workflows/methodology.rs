//! # Methodology Workflow
//!
//! Interactive methodology management for project configuration.
//! Handles UI for adding and removing methodologies, delegating business logic to the controller.

use anyhow::Result;
use inquire::MultiSelect;

use crate::cli::interactive::prompts;
use crate::cli::interactive::ui::UI;
use crate::controller::ProjectController;

/// Methodology workflow handler
pub struct MethodologyWorkflow;

impl MethodologyWorkflow {
    /// Add methodologies to project
    pub fn add_methodologies() -> Result<()> {
        UI::show_section_header("Add Methodologies", "➕")?;

        let available = ProjectController::get_available_methodologies()?;

        if available.is_empty() {
            UI::show_warning("No additional methodologies available.")?;
            return Ok(());
        }

        // Get currently installed to filter them out
        let installed = ProjectController::get_installed_methodologies()?;
        let installed_names: Vec<String> = installed.iter().map(|m| m.name.clone()).collect();

        // Filter out already configured methodologies
        let new_methodologies: Vec<String> = available
            .into_iter()
            .filter(|m| !installed_names.contains(&m.name))
            .map(|m| m.name)
            .collect();

        if new_methodologies.is_empty() {
            UI::show_warning("All available methodologies are already configured.")?;
            return Ok(());
        }

        let selected = MultiSelect::new("Select methodologies to add:", new_methodologies)
            .with_help_message("Use Space to select/deselect, Enter to confirm")
            .prompt()?;

        if selected.is_empty() {
            println!("\n  No methodologies selected.\n");
            return Ok(());
        }

        // Delegate to controller
        let result = ProjectController::add_methodologies(selected)?;
        println!("\n{}", result.message);

        Ok(())
    }

    /// Remove methodologies from project
    pub fn remove_methodologies() -> Result<()> {
        UI::show_section_header("Remove Methodologies", "➖")?;

        let installed = ProjectController::get_installed_methodologies()?;

        if installed.is_empty() {
            UI::show_warning("No methodologies configured to remove.")?;
            return Ok(());
        }

        let methodologies: Vec<String> = installed.iter().map(|m| m.name.clone()).collect();

        let selected = MultiSelect::new("Select methodologies to remove:", methodologies)
            .with_help_message("Use Space to select/deselect, Enter to confirm")
            .prompt()?;

        if selected.is_empty() {
            println!("\n  No methodologies selected for removal.\n");
            return Ok(());
        }

        // Confirm removal
        let methodology_list = selected.join(", ");
        if !prompts::confirm_delete(&methodology_list, "methodologies")? {
            return Ok(());
        }

        // Delegate to controller (validates default methodology)
        let result = ProjectController::remove_methodologies(selected)?;

        if result.is_success() {
            println!("\n{}", result.message);
        } else {
            UI::show_warning(&result.message)?;
        }

        Ok(())
    }
}
