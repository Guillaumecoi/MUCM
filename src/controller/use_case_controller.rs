//! # Use Case Controller
//!
//! This module provides the controller for use case management operations.
//! It handles the coordination between CLI commands and the use case application
//! services, providing a clean interface for creating, listing, and regenerating
//! use case documentation.
//!
//! ## Responsibilities
//!
//! - Use case creation with methodology selection
//! - Use case listing and status display
//! - Markdown regeneration for individual or all use cases
//! - Methodology switching and regeneration
//! - Data retrieval for interactive selection prompts
//!
//! ## Use Case Lifecycle
//!
//! 1. **Creation**: Use cases are created with a title, category, and optional methodology
//! 2. **Storage**: Raw data is stored as TOML files, generated markdown in separate directory
//! 3. **Regeneration**: Markdown can be regenerated when templates or data change
//! 4. **Methodology Changes**: Use cases can be regenerated with different methodologies

use crate::config::Config;
use crate::controller::dto::{DisplayResult, SelectionOptions};
use crate::core::{
    ReferenceType, ScenarioReference, ScenarioType, Status, UseCase, UseCaseCoordinator,
};
use crate::presentation::{StatusFormatter, UseCaseFormatter};
use anyhow::Result;
use std::collections::HashMap;

/// Parameters for creating a use case
pub struct CreateUseCaseParams {
    pub title: String,
    pub category: String,
    pub category_abbreviation: String,
    pub description: Option<String>,
    pub methodology: Option<String>,
    pub views: Option<String>,
    pub priority: Option<String>,
    pub extra_fields: Option<HashMap<String, String>>,
}

/// Controller for use case operations and management.
///
/// Manages all use case-related operations including creation, listing,
/// regeneration, and status reporting. Acts as the coordination layer
/// between CLI commands and the use case application services.
pub struct UseCaseController {
    /// Application service for use case business logic
    app_service: UseCaseCoordinator,
}

impl UseCaseController {
    /// Create a new use case controller instance.
    ///
    /// Initializes the controller with a loaded use case application service.
    /// The application service handles all the core business logic for use cases.
    ///
    /// # Returns
    /// A new UseCaseController instance ready for use
    ///
    /// # Errors
    /// Returns error if the application service cannot be loaded
    pub fn new() -> Result<Self> {
        let app_service = UseCaseCoordinator::load()?;
        Ok(Self { app_service })
    }

    /// Create a new use case with flexible options.
    ///
    /// Creates a use case with optional methodology, views, priority, and custom fields.
    /// If no methodology or views are specified, uses the project's default methodology.
    ///
    /// # Arguments
    /// * `title` - The title of the use case
    /// * `category` - The category under which to organize the use case
    /// * `description` - Optional detailed description of the use case
    /// * `methodology` - Optional methodology to use (ignored if views are specified)
    /// * `views` - Optional comma-separated methodology:level pairs (e.g., "feature:simple,business:normal")
    /// * `priority` - Optional priority for multi-view use cases
    /// * `extra_fields` - Optional additional field values (priority, status, author, etc.)
    ///
    /// # Returns
    /// DisplayResult with success message and use case information
    ///
    /// # Errors
    /// Returns error if use case creation fails or parameters are invalid
    pub fn create_use_case(&mut self, params: CreateUseCaseParams) -> Result<DisplayResult> {
        // Determine views string - either from views parameter or from methodology parameter
        let views_str = if let Some(views_str) = params.views {
            views_str
        } else {
            // Convert methodology to views format (methodology:normal)
            let methodology_str = params.methodology.unwrap_or_else(|| {
                Config::load()
                    .map(|c| c.templates.default_methodology.clone())
                    .unwrap_or_else(|_| "feature".to_string())
            });
            format!("{}:normal", methodology_str)
        };

        // All use cases now use the views-based API
        let result = if params.priority.is_some() || params.extra_fields.is_some() {
            let prio = params.priority.unwrap_or_else(|| "medium".to_string());
            let fields = params.extra_fields.unwrap_or_default();
            let coord_params = crate::core::CreateUseCaseWithViewsParams {
                title: params.title,
                category: params.category,
                category_abbreviation: params.category_abbreviation,
                description: params.description,
                priority: prio,
                views: views_str.clone(),
                extra_fields: fields,
            };
            self.app_service
                .create_use_case_with_views_and_fields(coord_params)
        } else {
            let coord_params = crate::core::CreateUseCaseWithViewsParams {
                title: params.title,
                category: params.category,
                category_abbreviation: params.category_abbreviation,
                description: params.description,
                priority: "medium".to_string(),
                views: views_str.clone(),
                extra_fields: HashMap::new(),
            };
            self.app_service
                .create_use_case_with_views_and_fields(coord_params)
        }
        .map(|use_case_id| {
            UseCaseFormatter::display_created(&use_case_id, &views_str);
            (
                use_case_id.clone(),
                format!(
                    "Created use case: {} with views: {}",
                    use_case_id, views_str
                ),
            )
        });

        match result {
            Ok((_, message)) => Ok(DisplayResult::success(message)),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// List all use cases in the project.
    ///
    /// Retrieves and displays a formatted list of all existing use cases
    /// in the project, including their titles, categories, and current status.
    ///
    /// # Returns
    /// Ok(()) on successful display
    ///
    /// # Errors
    /// Returns error if use case retrieval fails
    pub fn list_use_cases(&mut self) -> Result<()> {
        let use_cases = self.app_service.get_all_use_cases();
        UseCaseFormatter::display_list(use_cases);
        Ok(())
    }

    /// Show project status and statistics.
    ///
    /// Displays comprehensive project status including use case counts,
    /// categories, methodologies used, and other project metrics.
    ///
    /// # Returns
    /// Ok(()) on successful display
    ///
    /// # Errors
    /// Returns error if status retrieval fails
    pub fn show_status(&mut self) -> Result<()> {
        let use_cases = self.app_service.get_all_use_cases();
        StatusFormatter::display_project_status(use_cases);
        Ok(())
    }

    /// Get all categories currently in use.
    ///
    /// Retrieves a list of all categories that have use cases, useful for
    /// filtering and organization displays.
    ///
    /// # Returns
    /// SelectionOptions containing all category names
    ///
    /// # Errors
    /// Returns error if category retrieval fails
    pub fn get_categories(&mut self) -> Result<SelectionOptions> {
        // Inline: get_all_categories (PR #13)
        let mut categories: Vec<String> = self
            .app_service
            .get_all_use_cases()
            .iter()
            .map(|uc| uc.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        Ok(SelectionOptions::new(categories))
    }

    /// Regenerate use case with different methodology.
    ///
    /// Changes the methodology of an existing use case and regenerates its
    /// documentation using the new methodology's templates.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to regenerate
    /// * `methodology` - The new methodology to apply
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if regeneration fails or use case doesn't exist
    pub fn regenerate_use_case_with_methodology(
        &mut self,
        use_case_id: String,
        methodology: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .regenerate_use_case_with_methodology(&use_case_id, &methodology)
        {
            Ok(_) => {
                // Display using formatter
                UseCaseFormatter::display_regenerated(&use_case_id, &methodology);

                Ok(DisplayResult::success(format!(
                    "Regenerated use case {} with {} methodology",
                    use_case_id, methodology
                )))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Regenerate markdown for a single use case.
    ///
    /// Regenerates the markdown documentation for a specific use case using
    /// its current methodology and data.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to regenerate
    ///
    /// # Returns
    /// Ok(()) on successful regeneration
    ///
    /// # Errors
    /// Returns error if regeneration fails or use case doesn't exist
    pub fn regenerate_use_case(&mut self, use_case_id: &str) -> Result<()> {
        self.app_service.regenerate_markdown(use_case_id)?;
        UseCaseFormatter::display_markdown_regenerated(use_case_id);
        Ok(())
    }

    /// Regenerate markdown for all use cases.
    ///
    /// Regenerates the markdown documentation for all use cases in the project.
    /// Useful after template changes or bulk updates.
    ///
    /// # Returns
    /// Ok(()) on successful regeneration
    ///
    /// # Errors
    /// Returns error if any regeneration fails
    pub fn regenerate_all_use_cases(&mut self) -> Result<()> {
        let count = self.app_service.get_all_use_cases().len();
        self.app_service.regenerate_all_markdown()?;
        UseCaseFormatter::display_all_regenerated(count);
        Ok(())
    }

    /// Regenerate markdown for all use cases and return a DisplayResult.
    ///
    /// Wrapper around `regenerate_all_use_cases` that returns a DisplayResult
    /// for use in interactive workflows.
    ///
    /// # Returns
    /// DisplayResult with success message and regeneration count
    ///
    /// # Errors
    /// Returns error if any regeneration fails
    pub fn regenerate_all_markdown(&mut self) -> Result<DisplayResult> {
        let count = self.app_service.get_all_use_cases().len();
        self.app_service.regenerate_all_markdown()?;
        Ok(DisplayResult::success(format!(
            "✅ Regenerated documentation for {} use case(s)",
            count
        )))
    }

    /// Add a precondition to a use case.
    ///
    /// Adds a new precondition to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `precondition` - The precondition text to add
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or precondition cannot be added
    pub fn add_precondition(
        &mut self,
        use_case_id: String,
        precondition: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .add_precondition(&use_case_id, precondition)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Added precondition to use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// List preconditions for a use case.
    ///
    /// Retrieves and displays all preconditions for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with preconditions list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_preconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.get_preconditions(&use_case_id) {
            Ok(preconditions) => {
                let mut result = format!("Preconditions for {}:\n", use_case_id);
                if preconditions.is_empty() {
                    result.push_str("  No preconditions found.");
                } else {
                    for (i, precondition) in preconditions.iter().enumerate() {
                        result.push_str(&format!("  {}. {}\n", i + 1, precondition));
                    }
                }
                Ok(DisplayResult::success(result))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a precondition from a use case.
    ///
    /// Removes the precondition at the specified index from the use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the precondition to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn remove_precondition(
        &mut self,
        use_case_id: String,
        index: usize,
    ) -> Result<DisplayResult> {
        match self.app_service.remove_precondition(&use_case_id, index) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Removed precondition {} from use case: {}",
                index, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Edit a precondition in a use case.
    ///
    /// Updates the text of an existing precondition at the specified index.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the precondition to edit
    /// * `new_text` - The new text for the precondition
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn edit_precondition(
        &mut self,
        use_case_id: String,
        index: usize,
        new_text: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .edit_precondition(&use_case_id, index, new_text)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Updated precondition {} in use case: {}",
                index, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Reorder preconditions in a use case.
    ///
    /// Moves a precondition from one position to another.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `from_index` - The 1-based index of the precondition to move
    /// * `to_index` - The 1-based index of the destination position
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or indices are invalid
    pub fn reorder_preconditions(
        &mut self,
        use_case_id: String,
        from_index: usize,
        to_index: usize,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .reorder_preconditions(&use_case_id, from_index, to_index)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Reordered preconditions in use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Clear all preconditions from a use case.
    ///
    /// Removes all preconditions from the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn clear_preconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.clear_preconditions(&use_case_id) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Cleared all preconditions from use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Add a postcondition to a use case.
    ///
    /// Adds a new postcondition to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `postcondition` - The postcondition text to add
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or postcondition cannot be added
    pub fn add_postcondition(
        &mut self,
        use_case_id: String,
        postcondition: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .add_postcondition(&use_case_id, postcondition)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Added postcondition to use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// List postconditions for a use case.
    ///
    /// Retrieves and displays all postconditions for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with postconditions list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_postconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.get_postconditions(&use_case_id) {
            Ok(postconditions) => {
                let mut result = format!("Postconditions for {}:\n", use_case_id);
                if postconditions.is_empty() {
                    result.push_str("  No postconditions found.");
                } else {
                    for (i, postcondition) in postconditions.iter().enumerate() {
                        result.push_str(&format!("  {}. {}\n", i + 1, postcondition));
                    }
                }
                Ok(DisplayResult::success(result))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a postcondition from a use case.
    ///
    /// Removes the postcondition at the specified index from the use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the postcondition to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn remove_postcondition(
        &mut self,
        use_case_id: String,
        index: usize,
    ) -> Result<DisplayResult> {
        match self.app_service.remove_postcondition(&use_case_id, index) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Removed postcondition {} from use case: {}",
                index, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Edit a postcondition in a use case.
    ///
    /// Updates the text of an existing postcondition at the specified index.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the postcondition to edit
    /// * `new_text` - The new text for the postcondition
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn edit_postcondition(
        &mut self,
        use_case_id: String,
        index: usize,
        new_text: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .edit_postcondition(&use_case_id, index, new_text)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Updated postcondition {} in use case: {}",
                index, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Reorder postconditions in a use case.
    ///
    /// Moves a postcondition from one position to another.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `from_index` - The 1-based index of the postcondition to move
    /// * `to_index` - The 1-based index of the destination position
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or indices are invalid
    pub fn reorder_postconditions(
        &mut self,
        use_case_id: String,
        from_index: usize,
        to_index: usize,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .reorder_postconditions(&use_case_id, from_index, to_index)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Reordered postconditions in use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Clear all postconditions from a use case.
    ///
    /// Removes all postconditions from the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn clear_postconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.clear_postconditions(&use_case_id) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Cleared all postconditions from use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Add a reference to a use case.
    ///
    /// Adds a new reference relationship to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `target_id` - The ID of the target use case
    /// * `relationship` - The type of relationship (dependency, extension, inclusion, alternative)
    /// * `description` - Optional description of the relationship
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or reference cannot be added
    pub fn add_reference(
        &mut self,
        use_case_id: String,
        target_id: String,
        relationship: String,
        description: Option<String>,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .add_reference(&use_case_id, target_id, relationship, description)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Added reference to use case: {}",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// List references for a use case.
    ///
    /// Retrieves and displays all references for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with references list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_references(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.get_references(&use_case_id) {
            Ok(references) => {
                let mut result = format!("References for {}:\n", use_case_id);
                if references.is_empty() {
                    result.push_str("  No references found.");
                } else {
                    for reference in &references {
                        result.push_str(&format!(
                            "  - {} ({})",
                            reference.target_id, reference.relationship
                        ));
                        if let Some(desc) = &reference.description {
                            result.push_str(&format!(": {}", desc));
                        }
                        result.push('\n');
                    }
                }
                Ok(DisplayResult::success(result))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a reference from a use case.
    ///
    /// Removes the reference to the specified target use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `target_id` - The ID of the target use case to remove reference to
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or reference doesn't exist
    pub fn remove_reference(
        &mut self,
        use_case_id: String,
        target_id: String,
    ) -> Result<DisplayResult> {
        match self.app_service.remove_reference(&use_case_id, &target_id) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Removed reference to {} from use case: {}",
                target_id, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Add a scenario to a use case.
    ///
    /// Adds a new scenario to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `title` - The title of the scenario
    /// * `scenario_type` - The type of scenario (main, alternative, exception)
    /// * `description` - Optional description of the scenario
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or scenario cannot be added
    pub fn add_scenario(
        &mut self,
        use_case_id: String,
        title: String,
        scenario_type: String,
        description: Option<String>,
    ) -> Result<DisplayResult> {
        let scenario_type_enum = match scenario_type.as_str() {
            "happy_path" | "happy" | "main" => ScenarioType::HappyPath,
            "alternative_flow" | "alternative" | "alt" => ScenarioType::AlternativeFlow,
            "exception_flow" | "exception" | "error" => ScenarioType::ExceptionFlow,
            "extension" | "ext" => ScenarioType::Extension,
            _ => return Ok(DisplayResult::error(format!("Invalid scenario type: {}. Must be 'main', 'alternative', 'exception', or 'extension'", scenario_type))),
        };

        let params = crate::core::AddScenarioParams {
            title,
            scenario_type: scenario_type_enum,
            description,
            preconditions: vec![],
            postconditions: vec![],
            actors: vec![],
        };
        match self.app_service.add_scenario(&use_case_id, params) {
            Ok(scenario_id) => Ok(DisplayResult::success(format!(
                "Added scenario '{}' to use case: {}",
                scenario_id, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Add a step to a scenario.
    ///
    /// Adds a new step to the specified scenario.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_title` - The title of the scenario
    /// * `step` - The step description to add
    /// * `order` - Optional 1-based order for the step
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case or scenario not found or step cannot be added
    pub fn add_scenario_step(
        &mut self,
        use_case_id: String,
        scenario_title: String,
        step: String,
        order: Option<u32>,
    ) -> Result<DisplayResult> {
        // For now, we'll use default values for the required parameters
        // In a real implementation, we'd need to get these from the user
        let order_val = order.unwrap_or(0).to_string(); // Convert to String
        let actor = "User".to_string(); // Default actor
        let receiver = None; // No receiver by default
        let action = step; // Use the step as the action
        let expected_result = None; // No expected result for now

        let params = crate::core::AddScenarioStepParams {
            order: order_val,
            actor,
            receiver,
            action,
            expected_result,
        };
        match self
            .app_service
            .add_scenario_step(&use_case_id, &scenario_title, params)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Added step to scenario '{}' in use case: {}",
                scenario_title, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Update scenario status.
    ///
    /// Updates the status of the specified scenario.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_title` - The title of the scenario
    /// * `status` - The new status for the scenario
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case or scenario not found or status update fails
    pub fn update_scenario_status(
        &mut self,
        use_case_id: String,
        scenario_title: String,
        status: String,
    ) -> Result<DisplayResult> {
        let status_enum = match status.parse::<Status>() {
            Ok(s) => s,
            Err(e) => return Ok(DisplayResult::error(e)),
        };

        match self
            .app_service
            .update_scenario_status(&use_case_id, &scenario_title, status_enum)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Updated status of scenario '{}' in use case: {}",
                scenario_title, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// List scenarios for a use case.
    ///
    /// Retrieves and displays all scenarios for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with scenarios list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_scenarios(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.get_scenarios(&use_case_id) {
            Ok(scenarios) => {
                let mut result = format!("Scenarios for {}:\n", use_case_id);
                if scenarios.is_empty() {
                    result.push_str("  No scenarios found.");
                } else {
                    for scenario in &scenarios {
                        result.push_str(&format!(
                            "  - {} ({}): {} - {} steps\n",
                            scenario.title,
                            scenario.scenario_type,
                            scenario.status,
                            scenario.steps.len()
                        ));
                        if !scenario.description.is_empty() {
                            result
                                .push_str(&format!("    Description: {}\n", scenario.description));
                        }
                        if !scenario.steps.is_empty() {
                            result.push_str("    Steps:\n");
                            for (i, step) in scenario.steps.iter().enumerate() {
                                result.push_str(&format!(
                                    "      {}. {}\n",
                                    i + 1,
                                    step.description
                                ));
                            }
                        }
                    }
                }
                Ok(DisplayResult::success(result))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a step from a scenario.
    ///
    /// Removes the step at the specified order from the scenario.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_title` - The title of the scenario
    /// * `order` - The 1-based order of the step to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case or scenario not found or step doesn't exist
    pub fn remove_scenario_step(
        &mut self,
        use_case_id: String,
        scenario_title: String,
        order: u32,
    ) -> Result<DisplayResult> {
        let order_str = order.to_string();
        match self
            .app_service
            .remove_scenario_step(&use_case_id, &scenario_title, &order_str)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Removed step {} from scenario '{}' in use case: {}",
                order, scenario_title, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    // ========== Scenario Reference Operations (PR #7) ==========

    /// Add a reference to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - Use case containing the source scenario
    /// * `scenario_title` - Title of the source scenario
    /// * `target_id` - Target ID (scenario or use case)
    /// * `ref_type` - Reference type ("scenario" or "usecase")
    /// * `relationship` - Relationship type ("includes", "extends", "depends-on", "alternative-to")
    /// * `description` - Optional description
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case or scenario not found, or invalid reference type
    pub fn add_scenario_reference(
        &mut self,
        use_case_id: String,
        scenario_title: String,
        target_id: String,
        ref_type: String,
        relationship: String,
        description: Option<String>,
    ) -> Result<DisplayResult> {
        // Parse reference type
        let reference_type = match ref_type.to_lowercase().as_str() {
            "scenario" => ReferenceType::Scenario,
            "usecase" => ReferenceType::UseCase,
            _ => {
                return Ok(DisplayResult::error(format!(
                    "Invalid reference type '{}'. Use 'scenario' or 'usecase'",
                    ref_type
                )))
            }
        };

        // Find scenario ID by title
        let scenario_id = match self
            .app_service
            .find_scenario_id_by_title(&use_case_id, &scenario_title)
        {
            Ok(id) => id,
            Err(e) => return Ok(DisplayResult::error(e.to_string())),
        };

        // Create the reference object
        let mut reference =
            ScenarioReference::new(reference_type, target_id.clone(), relationship.clone());
        if let Some(desc) = description {
            reference = reference.with_description(desc);
        }

        match self
            .app_service
            .add_scenario_reference(&use_case_id, &scenario_id, reference)
        {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Added {} reference to '{}' in scenario '{}' of use case {}",
                relationship, target_id, scenario_title, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a reference from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - Use case containing the scenario
    /// * `scenario_title` - Title of the scenario
    /// * `target_id` - Target ID to remove
    /// * `relationship` - Relationship type
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case, scenario, or reference not found
    pub fn remove_scenario_reference(
        &mut self,
        use_case_id: String,
        scenario_title: String,
        target_id: String,
        relationship: String,
    ) -> Result<DisplayResult> {
        // Find scenario ID by title
        let scenario_id = match self
            .app_service
            .find_scenario_id_by_title(&use_case_id, &scenario_title)
        {
            Ok(id) => id,
            Err(e) => return Ok(DisplayResult::error(e.to_string())),
        };

        match self.app_service.remove_scenario_reference(
            &use_case_id,
            &scenario_id,
            &target_id,
            &relationship,
        ) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "Removed {} reference to '{}' from scenario '{}' in use case {}",
                relationship, target_id, scenario_title, use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Get all references for a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - Use case containing the scenario
    /// * `scenario_title` - Title of the scenario
    ///
    /// # Returns
    /// Vector of ScenarioReference objects
    ///
    /// # Errors
    /// Returns error if use case or scenario not found
    pub fn list_scenario_references(
        &self,
        use_case_id: String,
        scenario_title: String,
    ) -> Result<Vec<ScenarioReference>> {
        // Find scenario ID by title
        let scenario_id = self
            .app_service
            .find_scenario_id_by_title(&use_case_id, &scenario_title)?;

        self.app_service
            .get_scenario_references(&use_case_id, &scenario_id)
    }

    /// Get all use cases that use a specific persona
    ///
    /// # Arguments
    /// * `persona_id` - The persona identifier to search for
    ///
    /// # Returns
    /// Vector of tuples (use_case_id, title, scenario_count) that have scenarios using this persona
    ///
    /// # Errors
    /// Returns error if repository access fails
    pub fn get_use_cases_for_persona(
        &self,
        persona_id: String,
    ) -> Result<Vec<(String, String, usize)>> {
        self.app_service.get_use_cases_for_persona(&persona_id)
    }

    /// Clean up orphaned methodology fields from use cases
    ///
    /// Scans methodology_fields and removes entries for methodologies not used by any enabled view.
    ///
    /// # Arguments
    /// * `use_case_id` - Optional specific use case to clean. If None, cleans all use cases.
    /// * `dry_run` - If true, shows what would be cleaned without making changes
    ///
    /// # Returns
    /// DisplayResult with summary of cleanup operation
    ///
    /// # Errors
    /// Returns error if use case not found or cleanup fails
    pub fn cleanup_methodology_fields(
        &mut self,
        use_case_id: Option<String>,
        dry_run: bool,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .cleanup_methodology_fields(use_case_id.clone(), dry_run)
        {
            Ok((cleaned_count, total_checked, details)) => {
                if dry_run {
                    let mut message = format!(
                        "🔍 Dry run: Would clean {} of {} use case(s)\n",
                        cleaned_count, total_checked
                    );

                    if !details.is_empty() {
                        message.push_str("\nOrphaned methodology fields found:\n");
                        for (uc_id, methodologies) in details {
                            message.push_str(&format!(
                                "  • {}: {}\n",
                                uc_id,
                                methodologies.join(", ")
                            ));
                        }
                        message.push_str("\nRun without --dry-run to remove these fields.");
                    } else {
                        message.push_str("\nNo orphaned methodology fields found.");
                    }

                    Ok(DisplayResult::success(message))
                } else {
                    let mut message = format!(
                        "✨ Cleaned {} of {} use case(s)\n",
                        cleaned_count, total_checked
                    );

                    if !details.is_empty() {
                        message.push_str("\nRemoved methodology fields:\n");
                        for (uc_id, methodologies) in details {
                            message.push_str(&format!(
                                "  • {}: {}\n",
                                uc_id,
                                methodologies.join(", ")
                            ));
                        }
                    } else {
                        message.push_str("\nNo orphaned methodology fields found.");
                    }

                    Ok(DisplayResult::success(message))
                }
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    // ========== Update Operations ==========

    /// Update basic use case information
    ///
    /// Updates the title, category, description, and/or priority of an existing use case.
    /// Only updates fields that are provided (Some). None values leave the field unchanged.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to update
    /// * `title` - Optional new title
    /// * `category` - Optional new category
    /// * `description` - Optional new description
    /// * `priority` - Optional new priority
    ///
    /// # Returns
    /// DisplayResult with success message and updated use case ID
    ///
    /// # Errors
    /// Returns error if use case not found or update fails
    pub fn update_use_case(
        &mut self,
        use_case_id: String,
        title: Option<String>,
        category: Option<String>,
        description: Option<String>,
        priority: Option<String>,
    ) -> Result<DisplayResult> {
        match self.app_service.update_use_case(
            &use_case_id,
            title.as_deref(),
            category.as_deref(),
            description.as_deref(),
            priority.as_deref(),
        ) {
            Ok(_) => {
                let message = format!("✅ Updated use case: {}", use_case_id);
                Ok(DisplayResult::success(message))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Update methodology-specific fields for a use case
    ///
    /// Updates the custom fields for a specific methodology view in the use case.
    /// The fields are merged with existing fields for that methodology.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to update
    /// * `methodology` - The methodology whose fields to update
    /// * `fields` - HashMap of field names to new values
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or methodology not in use case views
    pub fn update_use_case_methodology_fields(
        &mut self,
        use_case_id: String,
        methodology: String,
        fields: std::collections::HashMap<String, String>,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .update_methodology_fields(&use_case_id, &methodology, fields)
        {
            Ok(_) => {
                let message = format!(
                    "✅ Updated {} methodology fields for use case: {}",
                    methodology, use_case_id
                );
                Ok(DisplayResult::success(message))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Add a new methodology view to an existing use case
    ///
    /// Adds a new methodology:level view to the use case and initializes
    /// empty methodology fields for it.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `methodology` - The methodology to add
    /// * `level` - The level for the methodology
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or view already exists
    pub fn add_view(
        &mut self,
        use_case_id: String,
        methodology: String,
        level: String,
    ) -> Result<DisplayResult> {
        match self
            .app_service
            .add_view(&use_case_id, &methodology, &level)
        {
            Ok(_) => {
                let message = format!(
                    "✅ Added {}:{} view to use case: {}",
                    methodology, level, use_case_id
                );
                Ok(DisplayResult::success(message))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Remove a methodology view from a use case
    ///
    /// Removes the specified methodology view and cleans up its associated
    /// methodology fields.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `methodology` - The methodology to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or if it's the last view
    pub fn remove_view(
        &mut self,
        use_case_id: String,
        methodology: String,
    ) -> Result<DisplayResult> {
        match self.app_service.remove_view(&use_case_id, &methodology) {
            Ok(_) => {
                let message = format!(
                    "✅ Removed {} view from use case: {}",
                    methodology, use_case_id
                );
                Ok(DisplayResult::success(message))
            }
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    /// Get use case by ID for display/editing
    ///
    /// Retrieves a use case by its ID, useful for displaying current values
    /// when editing.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to retrieve
    ///
    /// # Returns
    /// The UseCase entity if found
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn get_use_case(&self, use_case_id: &str) -> Result<&UseCase> {
        self.app_service
            .get_all_use_cases()
            .iter()
            .find(|uc| uc.id == use_case_id)
            .ok_or_else(|| anyhow::anyhow!("Use case {} not found", use_case_id))
    }

    /// Get all use cases
    ///
    /// Retrieves all use cases in the project
    ///
    /// # Returns
    /// Vector of UseCase entities
    ///
    /// # Errors
    /// Returns error if retrieval fails
    pub fn get_all_use_cases(&self) -> Result<Vec<UseCase>> {
        Ok(self.app_service.get_all_use_cases().to_vec())
    }
}
