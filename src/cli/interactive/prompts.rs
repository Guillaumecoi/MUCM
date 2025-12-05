//! # Reusable Prompt Functions
//!
//! This module provides a library of reusable prompt functions for interactive CLI workflows.
//! These functions serve as UI primitives, handling common input patterns and eliminating
//! code duplication across workflow modules.
//!
//! ## Design Principles
//!
//! - **Pure UI primitives**: Focus on gathering user input, minimal business logic
//! - **Type-safe**: Use generics for reusable patterns
//! - **Consistent UX**: Standardized messaging and interaction patterns
//! - **Error handling**: Consistent Result-based error propagation
//!
//! ## Usage
//!
//! Workflow modules should use these functions instead of directly calling `inquire`
//! prompts. This provides:
//! - Single source of truth for common prompts
//! - Consistent user experience
//! - Easier maintenance and updates
//! - Better testability (parsing/validation logic can be tested)

use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};

use crate::cli::interactive::runner::InteractiveRunner;

/// Select an actor for a scenario step
///
/// Presents a list of available actors with emoji formatting, plus special options
/// like "User", "System", and "Default (Actor)".
///
/// # Arguments
/// * `prompt` - The prompt text to display
/// * `include_defaults` - Whether to include "User", "System", "Default (Actor)" options
///
/// # Returns
/// * `Ok(Some(String))` - Selected actor ID ("user", "system", or actor/persona ID)
/// * `Ok(None)` - User selected "Default (Actor)" (no specific actor)
/// * `Err` - User cancelled or error occurred
///
/// # Examples
/// ```ignore
/// let actor = select_actor("Select actor for this step:", true)?;
/// ```
pub fn select_actor(prompt: &str, include_defaults: bool) -> Result<Option<String>> {
    use crate::controller::ActorController;

    let runner = InteractiveRunner::new();
    let available_actors_display = runner.get_available_actors()?;

    // Build a mapping of display strings to IDs
    let actor_controller = ActorController::new()?;
    let personas = actor_controller.list_personas()?;
    let system_actors = actor_controller.list_actors(Some(crate::core::ActorType::System))?;
    let database_actors = actor_controller.list_actors(Some(crate::core::ActorType::Database))?;
    let external_actors =
        actor_controller.list_actors(Some(crate::core::ActorType::ExternalService))?;

    let mut display_to_id = std::collections::HashMap::new();

    // Map personas: display format is "emoji name - function"
    for p in personas {
        let emoji = p
            .extra
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("👤");
        let display = format!("{} {} - {}", emoji, p.name, p.function);
        display_to_id.insert(display, p.id);
    }

    // Map actors: display format is "emoji name - id"
    for a in system_actors
        .iter()
        .chain(database_actors.iter())
        .chain(external_actors.iter())
    {
        let display = format!("{} {} - {}", a.emoji, a.name, a.id);
        display_to_id.insert(display, a.id.clone());
    }

    let mut all_options = Vec::new();

    if include_defaults {
        all_options.push("User".to_string());
        all_options.push("System".to_string());
        all_options.push("Default (Actor)".to_string());
    }

    all_options.extend(available_actors_display);

    let choice = Select::new(prompt, all_options).prompt()?;

    if choice == "Default (Actor)" {
        Ok(None)
    } else if choice == "User" || choice == "System" {
        Ok(Some(choice.to_lowercase()))
    } else {
        // Look up ID from display string
        if let Some(id) = display_to_id.get(&choice) {
            Ok(Some(id.clone()))
        } else {
            // Fallback for any unmatched case
            Ok(Some(choice))
        }
    }
}

/// Parse actor ID from formatted display string
///
/// Select methodology and level
///
/// Two-step selection: first methodology (with description), then level (with description).
/// Handles display formatting and parsing automatically.
///
/// # Arguments
/// * `runner` - Reference to InteractiveRunner for fetching methodologies
/// * `context` - Context string for prompts (e.g., "use case", "project")
///
/// # Returns
/// * `Ok((methodology_name, level))` - Selected methodology name and level
/// * `Err` - User cancelled or error occurred
pub fn select_methodology_and_level(
    runner: &InteractiveRunner,
    context: &str,
) -> Result<(String, String)> {
    // Step 1: Select methodology
    let methodologies = runner.get_installed_methodologies()?;

    if methodologies.is_empty() {
        anyhow::bail!("No methodologies installed. Please run initialization first.");
    }

    let methodology_options: Vec<String> = methodologies
        .iter()
        .map(|m| format!("{} - {}", m.display_name, m.description))
        .collect();

    let selected_methodology_display = Select::new(
        &format!("Select methodology for {}:", context),
        methodology_options,
    )
    .prompt()?;

    // Parse selected methodology
    let selected_methodology = methodologies
        .iter()
        .find(|m| format!("{} - {}", m.display_name, m.description) == selected_methodology_display)
        .context("Selected methodology not found")?;

    let methodology_name = selected_methodology.name.clone();

    // Step 2: Select level
    let level = select_level(runner, &methodology_name)?;

    Ok((methodology_name, level))
}

/// Select a level for a given methodology
///
/// Presents available levels with capitalized names and descriptions.
///
/// # Arguments
/// * `runner` - Reference to InteractiveRunner
/// * `methodology_name` - The methodology to get levels for
///
/// # Returns
/// * `Ok(String)` - Selected level name (lowercase)
/// * `Err` - User cancelled or error occurred
pub fn select_level(runner: &InteractiveRunner, methodology_name: &str) -> Result<String> {
    let available_levels = runner.get_methodology_levels(methodology_name)?;

    let level_options: Vec<String> = available_levels
        .iter()
        .map(|level| {
            let display_name = capitalize_first(&level.name);
            format!("{} - {}", display_name, level.description)
        })
        .collect();

    let selected_level_display = Select::new("Select level:", level_options).prompt()?;

    let level = selected_level_display
        .split(" - ")
        .next()
        .context("Failed to parse level name")?
        .to_lowercase();

    Ok(level)
}

/// Capitalize the first character of a string
///
/// # Examples
/// ```ignore
/// assert_eq!(capitalize_first("hello"), "Hello");
/// assert_eq!(capitalize_first("HELLO"), "HELLO");
/// ```
pub fn capitalize_first(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().next().unwrap()
            } else {
                c
            }
        })
        .collect()
}

/// Confirm a deletion action
///
/// Standardized confirmation prompt for delete operations.
///
/// # Arguments
/// * `item_name` - The name/ID of the item to delete (shown in prompt)
/// * `item_type` - The type of item (e.g., "actor", "scenario", "use case")
///
/// # Returns
/// * `Ok(true)` - User confirmed deletion
/// * `Ok(false)` - User cancelled deletion
/// * `Err` - Error occurred during prompt
///
/// # Examples
/// ```ignore
/// if confirm_delete("user-123", "actor")? {
///     // perform deletion
/// }
/// ```
pub fn confirm_delete(item_name: &str, item_type: &str) -> Result<bool> {
    let confirmed = Confirm::new(&format!(
        "Are you sure you want to delete {} '{}'?",
        item_type, item_name
    ))
    .with_default(false)
    .prompt()?;

    if !confirmed {
        println!("\n✓ Deletion cancelled.");
    }

    Ok(confirmed)
}

/// Generic selection with cancel option
///
/// Type-safe selection from a list of items with automatic "[Cancel]" option.
///
/// # Arguments
/// * `prompt` - The prompt text to display
/// * `items` - Vector of items to choose from
/// * `formatter` - Function to convert item to display string
///
/// # Returns
/// * `Ok(Some(T))` - User selected an item
/// * `Ok(None)` - User selected "[Cancel]"
/// * `Err` - Error occurred during prompt
///
/// # Examples
/// ```ignore
/// let actors = vec![actor1, actor2, actor3];
/// if let Some(actor) = select_or_cancel("Select actor:", actors, |a| a.name.clone())? {
///     // use selected actor
/// }
/// ```
pub fn select_or_cancel<T: Clone>(
    prompt: &str,
    items: Vec<T>,
    formatter: fn(&T) -> String,
) -> Result<Option<T>> {
    if items.is_empty() {
        anyhow::bail!("No items available to select from");
    }

    let mut options: Vec<String> = items.iter().map(formatter).collect();
    options.push("[Cancel]".to_string());

    let selection = Select::new(prompt, options).prompt()?;

    if selection == "[Cancel]" {
        return Ok(None);
    }

    // Find the matching item
    let selected_item = items
        .into_iter()
        .find(|item| formatter(item) == selection)
        .context("Selected item not found")?;

    Ok(Some(selected_item))
}

/// Edit a field with change detection
///
/// Prompts to edit a field, showing current value as default.
/// Returns Some(new_value) only if the value changed.
///
/// # Arguments
/// * `label` - Field label for prompt
/// * `current_value` - Current value to show as default
/// * `help` - Optional help message
///
/// # Returns
/// * `Ok(Some(String))` - Field was changed to new value
/// * `Ok(None)` - Field was not changed (user kept default)
/// * `Err` - Error occurred during prompt
///
/// # Examples
/// ```ignore
/// if let Some(new_name) = edit_field_with_change("Name:", &actor.name, None)? {
///     actor.name = new_name;
/// }
/// ```
pub fn edit_field_with_change(
    label: &str,
    current_value: &str,
    help: Option<&str>,
) -> Result<Option<String>> {
    let mut text_prompt = Text::new(label).with_default(current_value);

    if let Some(help_text) = help {
        text_prompt = text_prompt.with_help_message(help_text);
    }

    let new_value = text_prompt.prompt()?;

    if new_value != current_value {
        Ok(Some(new_value))
    } else {
        Ok(None)
    }
}

/// Collect conditions (preconditions or postconditions)
///
/// Unified conditions collection supporting both text-only and use case reference modes.
/// Loops until user enters empty input or declines to add more.
///
/// # Arguments
/// * `condition_type` - "preconditions" or "postconditions" (for display)
/// * `allow_references` - If true, allows referencing use cases; if false, text-only
/// * `available_use_cases` - List of use case IDs for reference (format: "ID - Title")
///
/// # Returns
/// * `Ok(Vec<String>)` - Collected conditions, potentially with references in format:
///   - Text-only: "condition text"
///   - With reference: "condition text||UC:target_id:relationship"
/// * `Err` - Error occurred during prompts
///
/// # Examples
/// ```ignore
/// // Text-only postconditions
/// let postconditions = collect_conditions("postconditions", false, vec![])?;
///
/// // Preconditions with use case references
/// let use_cases = vec!["UC-001 - Login".to_string(), "UC-002 - Register".to_string()];
/// let preconditions = collect_conditions("preconditions", true, use_cases)?;
/// ```
pub fn collect_conditions(
    condition_type: &str,
    allow_references: bool,
    available_use_cases: Vec<String>,
) -> Result<Vec<String>> {
    let add_conditions = Confirm::new(&format!("Add {}?", condition_type))
        .with_default(false)
        .prompt()?;

    if !add_conditions {
        return Ok(Vec::new());
    }

    let mut conditions = Vec::new();

    loop {
        if allow_references && !available_use_cases.is_empty() {
            println!("\n  💡 Tip: You can reference other use cases to show dependencies\n");
        }

        // Get condition text
        let condition_text = Text::new(&format!(
            "  {} (or press Enter to finish):",
            condition_type.trim_end_matches('s')
        ))
        .with_help_message(if allow_references {
            "Enter a text description (e.g., 'User must be logged in')"
        } else {
            "Enter a text description of the resulting state"
        })
        .prompt()?;

        if condition_text.trim().is_empty() {
            break;
        }

        // If references allowed, ask if they want to add one
        let final_condition = if allow_references && !available_use_cases.is_empty() {
            let reference_options = vec!["None - Just text", "Use Case"];
            let ref_choice = Select::new("Add a reference?", reference_options).prompt()?;

            if ref_choice == "Use Case" {
                // Select use case to reference
                let uc_selection =
                    Select::new("Select use case to reference:", available_use_cases.clone())
                        .prompt()?;

                // Parse use case ID
                let target_id = uc_selection
                    .split(" - ")
                    .next()
                    .context("Failed to parse use case ID")?;

                // Hardcode relationship type to 'depend' for preconditions
                let relationship = "depend";

                format!("{}||UC:{}:{}", condition_text, target_id, relationship)
            } else {
                condition_text
            }
        } else {
            condition_text
        };

        conditions.push(final_condition);

        // Ask if they want to add more
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

    Ok(conditions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("world"), "World");
        assert_eq!(capitalize_first("HELLO"), "HELLO");
        assert_eq!(capitalize_first("h"), "H");
        assert_eq!(capitalize_first(""), "");
    }
}
