//! # Interactive Runner
//!
//! Business logic layer for interactive CLI workflows.
//! Similar to CliRunner but handles interactive user workflows,
//! coordinating between presentation layer and controllers.
//!
//! ## Responsibilities
//!
//! - Interactive workflow orchestration (initialization, menu navigation)
//! - User input validation and processing
//! - Coordination between UI presentation and business logic
//! - State management for interactive sessions
//!
//! ## Architecture
//!
//! The InteractiveRunner serves as the business logic coordinator for
//! interactive CLI operations, delegating to controllers for core business
//! logic while handling the interactive workflow state and user experience.

use anyhow::Result;

/// Parameters for initializing a project
pub struct InitProjectParams {
    pub language: Option<String>,
    pub methodologies: Vec<String>,
    pub storage: String,
    pub use_case_dir: String,
    pub test_dir: String,
    pub persona_dir: String,
    pub data_dir: String,
    pub scenario_template: Option<String>,
}

/// Parameters for creating a use case with views and fields
pub struct CreateUseCaseWithViewsParams {
    pub title: String,
    pub category: String,
    pub category_abbreviation: String,
    pub description: Option<String>,
    pub priority: String,
    pub views: Vec<(String, String)>,
    pub extra_fields: std::collections::HashMap<String, String>,
}

use crate::controller::{PersonaController, ProjectController, UseCaseController};
use crate::core::{FieldCollection, MethodologyFieldCollector};

/// Interactive runner that coordinates interactive CLI workflows
pub struct InteractiveRunner {
    use_case_controller: Option<UseCaseController>,
    persona_controller: Option<PersonaController>,
    actor_controller: Option<crate::controller::ActorController>,
}

impl InteractiveRunner {
    /// Create a new interactive runner instance
    pub fn new() -> Self {
        Self {
            use_case_controller: None,
            persona_controller: None,
            actor_controller: None,
        }
    }

    /// Sanitize an optional string input by trimming whitespace and filtering empty strings.
    ///
    /// Returns None if the input is None or contains only whitespace.
    /// Returns Some(trimmed_string) if the input contains non-whitespace characters.
    fn sanitize_optional_string(input: Option<String>) -> Option<String> {
        input
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
    }

    /// Sanitize a string input by trimming whitespace.
    ///
    /// Returns None if the input contains only whitespace, Some(trimmed_string) otherwise.
    fn sanitize_string(input: String) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Ensure the use case controller is loaded
    fn ensure_use_case_controller(&mut self) -> Result<&mut UseCaseController> {
        if self.use_case_controller.is_none() {
            self.use_case_controller = Some(UseCaseController::new()?);
        }
        Ok(self
            .use_case_controller
            .as_mut()
            .expect("controller was just initialized"))
    }

    /// Get available programming languages for selection
    pub fn get_available_languages(&self) -> Result<Vec<String>> {
        let selection_options = ProjectController::get_available_languages()?;
        Ok(selection_options.items)
    }

    /// Get available methodologies with descriptions
    /// Get all available methodologies from source templates (for initialization)
    pub fn get_available_methodologies(&self) -> Result<Vec<MethodologyInfo>> {
        ProjectController::get_available_methodologies()
    }

    /// Collect methodology-specific field definitions for the given views
    pub fn collect_methodology_fields(
        &self,
        views: &[(String, String)],
    ) -> Result<FieldCollection> {
        let collector = MethodologyFieldCollector::new()?;
        collector.collect_fields_for_views(views)
    }

    /// Get installed/configured methodologies in the project (for creating use cases)
    pub fn get_installed_methodologies(&self) -> Result<Vec<MethodologyInfo>> {
        ProjectController::get_installed_methodologies()
    }

    /// Get available levels for a specific methodology
    pub fn get_methodology_levels(
        &self,
        methodology_name: &str,
    ) -> Result<Vec<crate::core::DocumentationLevel>> {
        crate::controller::ProjectController::get_methodology_levels(methodology_name)
    }

    /// Initialize project for interactive mode.
    pub fn initialize_project(&mut self, params: InitProjectParams) -> Result<String> {
        // Sanitize inputs: trim whitespace and filter out empty strings
        let sanitized_language = Self::sanitize_optional_string(params.language);
        let sanitized_methodologies: Vec<String> = params
            .methodologies
            .into_iter()
            .filter_map(Self::sanitize_string)
            .collect();

        // Use first methodology as default, or "feature" if none provided
        let default_methodology = sanitized_methodologies
            .first()
            .cloned()
            .unwrap_or_else(|| "feature".to_string());

        // Call the complete initialization method
        let controller_params = crate::controller::InitProjectParams {
            language: sanitized_language,
            methodologies: Some(sanitized_methodologies),
            storage: Some(params.storage),
            default_methodology: Some(default_methodology.to_string()),
            use_case_dir: Some(params.use_case_dir),
            test_dir: Some(params.test_dir),
            actor_dir: Some(params.persona_dir),
            data_dir: Some(params.data_dir),
            default_scenario_template: params.scenario_template,
        };
        let result = crate::controller::ProjectController::init_project(controller_params)?;

        Ok(result.message)
    }

    /// Create a multi-view use case
    #[cfg(test)]
    pub fn create_use_case_with_views(
        &mut self,
        title: String,
        category: String,
        description: Option<String>,
        views: Vec<(String, String)>, // Vec of (methodology, level) pairs
    ) -> Result<String> {
        let controller = self.ensure_use_case_controller()?;

        // Format views as "methodology:level,methodology:level"
        let views_string = views
            .iter()
            .map(|(methodology, level)| format!("{}:{}", methodology, level))
            .collect::<Vec<_>>()
            .join(",");

        let category_abbreviation = category
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        let params = crate::controller::CreateUseCaseParams {
            title,
            category,
            category_abbreviation,
            description,
            methodology: None,
            priority: None,
            views: Some(views_string),
            extra_fields: None,
        };
        let result = controller.create_use_case(params)?;
        Ok(result.message)
    }

    /// Create use case with multiple views and additional fields
    /// Returns (use_case_id, message)
    pub fn create_use_case_with_views_and_fields(
        &mut self,
        params: CreateUseCaseWithViewsParams,
    ) -> Result<(String, String)> {
        let controller = self.ensure_use_case_controller()?;

        // Format views as "methodology:level,methodology:level"
        let views_string = params
            .views
            .iter()
            .map(|(methodology, level)| format!("{}:{}", methodology, level))
            .collect::<Vec<_>>()
            .join(",");

        let uc_params = crate::controller::CreateUseCaseParams {
            title: params.title.clone(),
            category: params.category,
            category_abbreviation: params.category_abbreviation,
            description: params.description,
            methodology: None,
            views: Some(views_string),
            priority: Some(params.priority),
            extra_fields: Some(params.extra_fields),
        };
        let result = controller.create_use_case(uc_params)?;

        // Extract use case ID from message (format: "Created use case: UC-XXX-XXX with views: ...")
        let use_case_id = if let Some(id_part) = result.message.split("Created use case: ").nth(1) {
            id_part
                .split(" with")
                .next()
                .unwrap_or(&params.title)
                .to_string()
        } else {
            params.title.clone()
        };

        Ok((use_case_id, result.message))
    }

    /// List use cases
    pub fn list_use_cases(&mut self) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.list_use_cases()
    }

    /// Show project status
    pub fn show_status(&mut self) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.show_status()
    }

    /// Create a persona interactively
    pub fn create_persona_interactive(
        &mut self,
        id: String,
        name: String,
        function: String,
    ) -> Result<String> {
        let controller = self.ensure_persona_controller()?;
        let result = controller.create_persona(id, name, function)?;
        Ok(result.message)
    }

    /// Create a system actor interactively
    pub fn create_system_actor_interactive(
        &mut self,
        id: String,
        name: String,
        actor_type: String,
        emoji: Option<String>,
    ) -> Result<String> {
        let controller = self.ensure_persona_controller()?;
        let result = controller.create_system_actor(id, name, actor_type, emoji)?;
        Ok(result.message)
    }

    /// List all actors
    pub fn list_actors(&self) -> Result<()> {
        use crate::cli::args::ActorCommands;
        use crate::cli::standard::handle_actor_command;
        let command = ActorCommands::List { actor_type: None };
        handle_actor_command(command)
    }

    /// Show actor details
    pub fn show_actor(&self, id: String) -> Result<()> {
        use crate::cli::args::ActorCommands;
        use crate::cli::standard::handle_actor_command;
        let command = ActorCommands::Show { id: id.to_string() };
        handle_actor_command(command)
    }

    /// Delete an actor
    pub fn delete_actor(&self, id: &str) -> Result<()> {
        use crate::cli::args::ActorCommands;
        use crate::cli::standard::handle_actor_command;
        let command = ActorCommands::Delete { id: id.to_string() };
        handle_actor_command(command)?;
        Ok(())
    }

    // ========== Persona Editing Methods ==========

    /// Ensure the persona controller is loaded
    fn ensure_persona_controller(&mut self) -> Result<&mut PersonaController> {
        if self.persona_controller.is_none() {
            self.persona_controller = Some(PersonaController::new()?);
        }
        Ok(self
            .persona_controller
            .as_mut()
            .expect("controller was just initialized"))
    }

    /// Ensure the actor controller is loaded
    fn ensure_actor_controller(&mut self) -> Result<&mut crate::controller::ActorController> {
        if self.actor_controller.is_none() {
            self.actor_controller = Some(crate::controller::ActorController::new()?);
        }
        Ok(self
            .actor_controller
            .as_mut()
            .expect("controller was just initialized"))
    }

    /// Get list of all actor IDs for selection (both personas and system actors)
    pub fn get_actor_ids(&mut self) -> Result<Vec<String>> {
        let controller = self.ensure_actor_controller()?;
        controller.get_actor_ids()
    }

    /// Get actor details for editing (persona-specific, includes custom fields)
    pub fn get_actor_details(&mut self, actor_id: &str) -> Result<crate::core::Persona> {
        let controller = self.ensure_persona_controller()?;
        controller.get_persona(actor_id)
    }

    /// Update actor custom fields
    pub fn update_actor_fields(
        &mut self,
        actor_id: String,
        fields: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let controller = self.ensure_persona_controller()?;
        let result = controller.update_persona_fields(actor_id, fields)?;
        Ok(result.message)
    }

    /// Get actor field configuration
    pub fn get_actor_field_config(
        &mut self,
    ) -> Result<std::collections::HashMap<String, crate::core::CustomFieldConfig>> {
        let controller = self.ensure_persona_controller()?;
        Ok(controller.get_persona_field_config())
    }

    /// Get current actor field values
    pub fn get_actor_field_values(
        &mut self,
        actor_id: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let controller = self.ensure_persona_controller()?;
        controller.get_persona_field_values(actor_id)
    }

    // ========== Actor Editing Methods (All Types) ==========

    /// Get actor entity details (works for all actor types)
    pub fn get_actor_entity(&mut self, actor_id: &str) -> Result<crate::core::ActorEntity> {
        let controller = self.ensure_actor_controller()?;
        controller.get_actor(actor_id)
    }

    /// Update actor name (works for all actor types)
    pub fn update_actor_entity_name(&mut self, actor_id: String, name: String) -> Result<String> {
        let controller = self.ensure_actor_controller()?;
        let result = controller.update_actor_name(actor_id, name)?;
        Ok(result.message)
    }

    /// Update actor emoji (works for all actor types)
    pub fn update_actor_entity_emoji(&mut self, actor_id: String, emoji: String) -> Result<String> {
        let controller = self.ensure_actor_controller()?;
        let result = controller.update_emoji(actor_id, emoji)?;
        Ok(result.message)
    }

    // ========== Use Case Editing Methods ==========

    /// Get list of all use case IDs for selection
    pub fn get_use_case_ids(&mut self) -> Result<Vec<String>> {
        let controller = self.ensure_use_case_controller()?;
        let use_cases = controller.get_all_use_cases()?;
        Ok(use_cases.iter().map(|uc| uc.id.clone()).collect())
    }

    /// Get use case details for editing
    pub fn get_use_case_details(&mut self, use_case_id: &str) -> Result<crate::core::UseCase> {
        let controller = self.ensure_use_case_controller()?;
        let use_case = controller.get_use_case(use_case_id)?;
        // Clone to return owned value
        Ok(use_case.clone())
    }

    /// Update basic use case fields
    pub fn update_use_case(
        &mut self,
        use_case_id: String,
        title: Option<String>,
        category: Option<String>,
        description: Option<String>,
        priority: Option<String>,
    ) -> Result<String> {
        let controller = self.ensure_use_case_controller()?;
        let result = controller.update_use_case(
            use_case_id.clone(),
            title,
            category,
            description,
            priority,
        )?;
        Ok(result.message)
    }

    /// Update methodology-specific fields for a use case
    pub fn update_methodology_fields(
        &mut self,
        use_case_id: &str,
        methodology: &str,
        fields: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let controller = self.ensure_use_case_controller()?;
        let result = controller.update_use_case_methodology_fields(
            use_case_id.to_string(),
            methodology.to_string(),
            fields,
        )?;
        Ok(result.message)
    }

    /// Add a new view to a use case
    pub fn add_view_to_use_case(
        &mut self,
        use_case_id: &str,
        methodology: &str,
        level: &str,
    ) -> Result<String> {
        let controller = self.ensure_use_case_controller()?;
        let result = controller.add_view(
            use_case_id.to_string(),
            methodology.to_string(),
            level.to_string(),
        )?;
        Ok(result.message)
    }

    /// Remove a view from a use case
    pub fn remove_view_from_use_case(
        &mut self,
        use_case_id: &str,
        methodology: &str,
    ) -> Result<String> {
        let controller = self.ensure_use_case_controller()?;
        let result = controller.remove_view(use_case_id.to_string(), methodology.to_string())?;
        Ok(result.message)
    }

    /// Get current methodology field values for a use case
    pub fn get_methodology_field_values(
        &mut self,
        use_case_id: &str,
        methodology: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let use_case = self.get_use_case_details(use_case_id)?;
        Ok(use_case
            .methodology_fields
            .get(methodology)
            .cloned()
            .unwrap_or_default())
    }

    // ========== Actor Selection Methods ==========

    /// Get available actors for selection (personas + system actors)
    ///
    /// # Returns
    /// Vector of actor display strings with emoji, call_name, and ID
    pub fn get_available_actors(&self) -> Result<Vec<String>> {
        use crate::controller::ActorController;

        let actor_controller = ActorController::new()?;

        // Get all actors and use centralized display formatting
        let all_actors = actor_controller.list_actors(None)?;
        let actors: Vec<String> = all_actors
            .iter()
            .map(|a| a.display_for_selection())
            .collect();

        Ok(actors)
    }

    /// Get all available use cases for referencing
    pub fn get_available_use_cases(&self) -> Result<Vec<crate::core::UseCase>> {
        use crate::controller::UseCaseController;

        let uc_controller = UseCaseController::new()?;
        uc_controller.get_all_use_cases()
    }
}

// Re-export for convenience
pub use crate::controller::MethodologyInfo;
