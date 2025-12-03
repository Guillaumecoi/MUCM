//! # Use Case Operations
//!
//! Extracted operations for use case workflows.
//! Contains reusable logic for views collection, methodology fields, and category management.

use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;

use crate::cli::interactive::{
    field_helpers::FieldHelpers, runner::InteractiveRunner, ui::UI,
};
use crate::controller::CategoryController;

/// Collect methodology views from user
///
/// Prompts user to select methodology-level pairs for use case views.
/// Each view will generate a separate markdown file.
///
/// # Arguments
/// * `runner` - InteractiveRunner for fetching methodologies and levels
///
/// # Returns
/// * `Ok(Vec<(String, String)>)` - List of (methodology_name, level) pairs
/// * `Err` - User cancelled or error occurred
pub fn collect_methodology_views(runner: &InteractiveRunner) -> Result<Vec<(String, String)>> {
    use crate::cli::interactive::prompts;

    let methodologies = runner.get_installed_methodologies()?;

    if methodologies.is_empty() {
        anyhow::bail!(
            "No methodologies available. Please configure methodologies in your project."
        );
    }

    UI::show_section_header("Select Views", "👁️")?;
    UI::show_info("Add methodology views. Each view will generate a separate markdown file.")?;

    let mut views: Vec<(String, String)> = Vec::new();

    loop {
        let (methodology_name, level) =
            prompts::select_methodology_and_level(runner, &format!("view #{}", views.len() + 1))?;

        views.push((methodology_name.clone(), level.clone()));

        UI::show_success(&format!("✓ Added view: {}:{}", methodology_name, level))?;

        // Ask if user wants to add another view
        let add_another = Confirm::new("Add another view?")
            .with_default(false)
            .with_help_message("Each view will generate a separate markdown file")
            .prompt()?;

        if !add_another {
            break;
        }
    }

    if views.is_empty() {
        anyhow::bail!("No views selected.");
    }

    Ok(views)
}

/// Prompt for methodology-specific field values
///
/// Collects field definitions from methodologies and prompts user for values.
///
/// # Arguments
/// * `runner` - InteractiveRunner for collecting methodology fields
/// * `views` - List of (methodology_name, level) pairs
///
/// # Returns
/// * `Ok(HashMap<String, String>)` - Field name to value mapping
/// * `Err` - Error during field collection
pub fn prompt_methodology_fields(
    runner: &InteractiveRunner,
    views: &[(String, String)],
) -> Result<HashMap<String, String>> {
    // Collect field definitions
    let field_collection = match runner.collect_methodology_fields(views) {
        Ok(collection) => collection,
        Err(e) => {
            // If we can't collect fields (e.g., methodology not found in workspace),
            // just warn and continue without methodology fields
            UI::show_warning(&format!(
                "Could not collect methodology fields: {}. Continuing without methodology-specific fields.",
                e
            ))?;
            return Ok(HashMap::new());
        }
    };

    // Show any warnings
    for warning in &field_collection.warnings {
        UI::show_warning(warning)?;
    }

    if field_collection.fields.is_empty() {
        return Ok(HashMap::new());
    }

    UI::show_section_header("Methodology Fields", "🎯")?;
    UI::show_info("These fields are defined by the methodologies you selected. Press Enter to skip optional fields.")?;

    let mut field_values = HashMap::new();

    // Group fields by methodology for better UX
    let mut fields_by_methodology: HashMap<String, Vec<&crate::core::CollectedField>> =
        HashMap::new();
    for field in field_collection.fields.values() {
        for methodology in &field.methodologies {
            fields_by_methodology
                .entry(methodology.clone())
                .or_default()
                .push(field);
        }
    }

    // Sort methodologies for consistent ordering
    let mut methodology_names: Vec<_> = fields_by_methodology.keys().collect();
    methodology_names.sort();

    // Prompt for each methodology's fields
    for methodology_name in methodology_names {
        let fields = fields_by_methodology.get(methodology_name).unwrap();
        if !fields.is_empty() {
            UI::show_info(&format!("\n📋 {} Fields:", methodology_name))?;

            for field in fields {
                // Use FieldHelpers to handle different field types automatically
                let value = FieldHelpers::prompt_by_type(
                    &field.field_type,
                    &field.label,
                    field.required,
                    field.description.as_deref(),
                    field.example.as_deref(),
                )?;

                // Check if required field is missing
                if field.required && value.is_none() {
                    anyhow::bail!("Required field '{}' cannot be empty", field.label);
                }

                // Store the value if provided
                if let Some(v) = value {
                    field_values.insert(field.name.clone(), v);
                }
            }
        }
    }

    Ok(field_values)
}

/// Create a new category or select an existing one
///
/// Handles category selection with abbreviation collision detection and resolution.
///
/// # Returns
/// * `Ok((category_name, abbreviation))` - Selected or created category
/// * `Err` - User cancelled or error occurred
pub fn create_or_select_category() -> Result<(String, String)> {
    let category_controller = CategoryController::new()?;

    // Get existing categories
    let existing_categories = category_controller.get_all_categories()?;

    // Build options: existing categories + "Create New Category"
    let mut options: Vec<String> = existing_categories
        .iter()
        .map(|(name, abbr)| format!("{} ({})", name, abbr))
        .collect();
    options.push("➕ Create New Category".to_string());

    let selection = Select::new("Category:", options)
        .with_help_message("Select an existing category or create a new one")
        .prompt()?;

    if selection == "➕ Create New Category" {
        // Create new category workflow
        UI::show_section_header("Create New Category", "📁")?;

        let full_name = Text::new("Category name:")
            .with_help_message("Full category name (e.g., 'Authentication', 'User Management')")
            .prompt()?;

        // Suggest abbreviation
        let suggested_abbr = category_controller.suggest_abbreviation(&full_name);

        let abbreviation = Text::new("Abbreviation:")
            .with_default(&suggested_abbr)
            .with_help_message("3+ uppercase letters for use case IDs (e.g., 'AUT', 'USR')")
            .prompt()?;

        // Check for collision
        if let Some(existing) = category_controller.detect_collision(&abbreviation)? {
            UI::show_warning(&format!(
                "⚠️  Abbreviation '{}' is already used by category '{}'",
                abbreviation, existing.full_name
            ))?;

            // Suggest auto-resolution
            let (new_abbr, existing_abbr) = category_controller
                .suggest_collision_resolution(&full_name, &existing)
                .ok_or_else(|| anyhow::anyhow!("Could not generate collision resolution"))?;

            UI::show_info(&format!(
                "💡 Suggested resolution:\n   • {}: {} → {}\n   • {}: {} → {}",
                full_name,
                abbreviation,
                new_abbr,
                existing.full_name,
                existing.abbreviation,
                existing_abbr
            ))?;

            let choices = vec![
                "Auto-resolve both categories",
                "Edit abbreviation manually",
                "Cancel",
            ];

            match Select::new("How would you like to proceed?", choices).prompt()? {
                "Auto-resolve both categories" => {
                    // Update existing category
                    category_controller
                        .update_abbreviation(&existing.abbreviation, &existing_abbr)?;

                    // Create new category with resolved abbreviation
                    let result =
                        category_controller.create_category(full_name.clone(), new_abbr.clone())?;
                    UI::show_success(&result.message)?;

                    Ok((full_name, new_abbr))
                }
                "Edit abbreviation manually" => {
                    // Let user enter a different abbreviation
                    let new_abbreviation = Text::new("Enter a different abbreviation:")
                        .with_help_message("Must be at least 3 characters, alphanumeric")
                        .prompt()?;

                    // Validate new abbreviation
                    if new_abbreviation.len() < 3
                        || !new_abbreviation.chars().all(char::is_alphanumeric)
                    {
                        anyhow::bail!("Abbreviation must be at least 3 alphanumeric characters");
                    }

                    // Check for collision again
                    if category_controller
                        .detect_collision(&new_abbreviation)?
                        .is_some()
                    {
                        anyhow::bail!(
                            "Abbreviation '{}' is still in use. Please try again.",
                            new_abbreviation
                        );
                    }

                    // Create with new abbreviation
                    let result = category_controller
                        .create_category(full_name.clone(), new_abbreviation.clone())?;
                    UI::show_success(&result.message)?;

                    Ok((full_name, new_abbreviation))
                }
                _ => {
                    anyhow::bail!("Category creation cancelled");
                }
            }
        } else {
            // No collision, create category
            let result =
                category_controller.create_category(full_name.clone(), abbreviation.clone())?;
            UI::show_success(&result.message)?;

            Ok((full_name, abbreviation))
        }
    } else {
        // Parse existing category selection
        let (full_name, abbr) = selection
            .rsplit_once(" (")
            .and_then(|(name, abbr_with_paren)| {
                abbr_with_paren
                    .strip_suffix(')')
                    .map(|abbr| (name.to_string(), abbr.to_string()))
            })
            .context("Failed to parse category selection")?;

        Ok((full_name, abbr))
    }
}
