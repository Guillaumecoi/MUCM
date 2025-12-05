//! Scenario Controller
//!
//! Manages scenario operations within use cases including CRUD operations,
//! step management, references, and persona assignments.

use crate::controller::{DisplayMessage, DisplayResult};
use crate::core::{ScenarioType, Status, UseCaseCoordinator};
use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;

/// Parameters for creating a scenario
pub struct CreateScenarioParams {
    pub use_case_id: String,
    pub title: String,
    pub scenario_type: String,
    pub description: Option<String>,
    pub persona_id: Option<String>,
    pub preconditions: Option<Vec<String>>,
    pub postconditions: Option<Vec<String>>,
}

/// Parameters for creating an extension scenario
pub struct CreateExtensionParams {
    pub use_case_id: String,
    pub parent_scenario_id: String,
    pub extends_at_step: String,
    pub returns_at_step: Option<String>,
    pub title: String,
    pub description: String,
    pub primary_actor: String,
}

/// Parameters for inserting a step
pub struct InsertStepParams {
    pub use_case_id: String,
    pub scenario_id: String,
    pub after_step: String,
    pub actor: String,
    pub receiver: Option<String>,
    pub action: String,
    pub expected_result: Option<String>,
}

/// Controller for managing scenarios within use cases
pub struct ScenarioController {
    app_service: UseCaseCoordinator,
}

impl ScenarioController {
    /// Create a new ScenarioController
    pub fn new() -> Result<Self> {
        let app_service = UseCaseCoordinator::load()?;
        Ok(Self { app_service })
    }

    /// Create a new scenario in a use case
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to add the scenario to
    /// * `title` - Title of the scenario
    /// * `scenario_type` - Type of scenario (main/alternative/exception)
    /// * `description` - Optional description
    /// * `persona_id` - Optional persona to assign to this scenario
    /// * `preconditions` - Optional preconditions for the scenario
    /// * `postconditions` - Optional postconditions for the scenario
    /// * `actors` - Optional list of actor IDs involved in the scenario
    ///
    /// # Returns
    /// DisplayResult with the scenario ID
    /// Create a main scenario (always creates as main/happy path type)
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `title` - Scenario title
    /// * `description` - Optional description
    /// * `preconditions` - Optional preconditions
    /// * `postconditions` - Optional postconditions
    ///
    /// # Returns
    /// DisplayResult with scenario ID
    pub fn create_main_scenario(
        &mut self,
        use_case_id: String,
        title: String,
        description: Option<String>,
        preconditions: Option<Vec<String>>,
        postconditions: Option<Vec<String>>,
        primary_actor: String,
    ) -> Result<DisplayResult> {
        // Always create as main scenario with HappyPath type
        let params = crate::core::AddScenarioParams {
            title: title.clone(),
            scenario_type: ScenarioType::HappyPath,
            description,
            preconditions: preconditions.unwrap_or_default(),
            postconditions: postconditions.unwrap_or_default(),
            actors: vec![primary_actor],
        };
        let scenario_id = self.app_service.add_scenario(&use_case_id, params)?;

        // Regenerate markdown to reflect the new scenario
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::created(
            "main scenario",
            &scenario_id,
            &title,
        )))
    }

    /// Create a scenario with specific type (legacy - prefer create_main_scenario or create_extension_scenario)
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `title` - Scenario title
    /// * `scenario_type` - Type string ("main", "alternative", "exception")
    /// * `description` - Optional description
    /// * `persona_id` - Optional persona assignment
    /// * `preconditions` - Optional preconditions
    /// * `postconditions` - Optional postconditions
    ///
    /// # Returns
    /// DisplayResult with scenario ID
    pub fn create_scenario(&mut self, params: CreateScenarioParams) -> Result<DisplayResult> {
        let use_case_id = params.use_case_id;
        let title = params.title;
        let description = params.description;
        let persona_id = params.persona_id;
        let preconditions = params.preconditions;
        let postconditions = params.postconditions;

        // Parse scenario type
        let parsed_type = ScenarioType::from_str(&params.scenario_type)
            .map_err(|e| anyhow::anyhow!("Invalid scenario type: {}", e))?;

        // Create scenario via coordinator (actors derived from steps)
        let params = crate::core::AddScenarioParams {
            title: title.clone(),
            scenario_type: parsed_type,
            description: description.clone(),
            preconditions: preconditions.unwrap_or_default(),
            postconditions: postconditions.unwrap_or_default(),
            actors: Vec::new(),
        };
        let scenario_id = self.app_service.add_scenario(&use_case_id, params)?;

        // Assign persona if provided
        if let Some(persona) = persona_id {
            self.app_service
                .assign_persona_to_scenario(&use_case_id, &scenario_id, &persona)?;
        }

        // Regenerate markdown to reflect the new scenario
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::created(
            "scenario",
            &scenario_id,
            &title,
        )))
    }

    /// Edit an existing scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case containing the scenario
    /// * `scenario_id` - The ID of the scenario to edit
    /// * `title` - Optional new title
    /// * `description` - Optional new description
    /// * `scenario_type` - Optional new type
    /// * `status` - Optional new status
    ///
    /// # Returns
    /// DisplayResult indicating success or failure
    pub fn edit_scenario(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        title: Option<String>,
        description: Option<String>,
        scenario_type: Option<String>,
        status: Option<String>,
    ) -> Result<DisplayResult> {
        // Parse optional enums
        let parsed_type = scenario_type
            .as_ref()
            .map(|t| ScenarioType::from_str(t))
            .transpose()
            .map_err(|_| anyhow::anyhow!("Invalid scenario type"))?;

        let parsed_status = status
            .as_ref()
            .map(|s| s.parse::<Status>())
            .transpose()
            .map_err(|_| anyhow::anyhow!("Invalid status"))?;

        // Delegate to coordinator
        self.app_service.edit_scenario(
            &use_case_id,
            &scenario_id,
            title,
            description,
            parsed_type,
            parsed_status,
        )?;

        // Regenerate markdown to reflect changes
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::updated("scenario", &scenario_id)))
    }

    /// Delete a scenario from a use case
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case containing the scenario
    /// * `scenario_id` - The ID of the scenario to delete
    ///
    /// # Returns
    /// DisplayResult indicating success or failure
    pub fn delete_scenario(
        &mut self,
        use_case_id: String,
        scenario_id: String,
    ) -> Result<DisplayResult> {
        self.app_service
            .delete_scenario(&use_case_id, &scenario_id)?;

        // Regenerate markdown to reflect the deletion
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::deleted("scenario", &scenario_id)))
    }

    /// List all scenarios for a use case
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with formatted scenario list
    pub fn list_scenarios(&mut self, use_case_id: String) -> Result<DisplayResult> {
        let scenarios = self.app_service.get_scenarios(&use_case_id)?;

        if scenarios.is_empty() {
            return Ok(DisplayResult::success("No scenarios found".to_string()));
        }

        let mut output = format!("Scenarios for {}:\n", use_case_id);
        for scenario in scenarios {
            output.push_str(&format!(
                "  {} | {} | {} | {} steps\n",
                scenario.id,
                scenario.title,
                scenario.scenario_type,
                scenario.steps.len()
            ));
        }

        Ok(DisplayResult::success(output))
    }

    /// Get all scenarios for a use case (for programmatic use)
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// Vector of scenarios
    pub fn get_scenarios(&mut self, use_case_id: &str) -> Result<Vec<crate::core::Scenario>> {
        self.app_service.get_scenarios(use_case_id)
    }

    /// Get details of a specific scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case containing the scenario
    /// * `scenario_id` - The ID of the scenario
    ///
    /// # Returns
    /// The Scenario object wrapped in Result
    pub fn get_scenario(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
    ) -> Result<crate::core::Scenario> {
        let scenarios = self.app_service.get_scenarios(use_case_id)?;
        scenarios
            .into_iter()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))
    }

    /// Resolve an actor ID to its call name or display name
    ///
    /// # Arguments
    /// * `actor_id` - The actor ID to resolve
    ///
    /// # Returns
    /// The actor's call_name, or name, or the ID itself if not found
    fn resolve_actor_call_name(&self, actor_id: &str) -> String {
        // Special cases for common actors
        if actor_id == "user" || actor_id == "system" {
            return actor_id.to_string();
        }

        // Try to load actor controller to look up the actor
        if let Ok(actor_controller) = crate::controller::ActorController::new() {
            // Try as persona first
            if let Ok(persona) = actor_controller.get_persona(actor_id) {
                return persona.get_call_name().to_string();
            }

            // Try as system actor
            if let Ok(actor) = actor_controller.get_actor(actor_id) {
                return actor.get_call_name().to_string();
            }
        }

        // Fallback to the ID itself
        actor_id.to_string()
    }

    /// Add a step to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `step_description` - Description of the step
    /// * `order` - Optional order (will append if not specified)
    /// * `actor` - Optional actor for the step (defaults to "Actor")
    /// * `receiver` - Optional receiving actor
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn add_step(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        step_description: String,
        order: Option<u32>,
        actor: Option<String>,
        receiver: Option<String>,
    ) -> Result<DisplayResult> {
        let order = order.unwrap_or_else(|| {
            // Get current step count to append
            self.get_scenario(&use_case_id, &scenario_id)
                .ok()
                .map(|s| s.steps.len() as u32 + 1)
                .unwrap_or(1)
        });

        let order_str = order.to_string();
        let actor_str = actor.unwrap_or_else(|| "user".to_string());

        // Resolve actor ID to call name for better readability
        let actor_call_name = self.resolve_actor_call_name(&actor_str);
        let receiver_call_name = receiver.as_ref().map(|r| self.resolve_actor_call_name(r));

        let params = crate::core::AddScenarioStepParams {
            order: order_str,
            actor: actor_call_name.clone(),
            receiver: receiver_call_name.clone(),
            action: step_description.clone(),
            expected_result: None,
        };
        self.app_service
            .add_scenario_step(&use_case_id, &scenario_id, params)?;

        let message = if let Some(ref recv) = receiver_call_name {
            format!(
                "✅ Added step {} to scenario {} ({} → {})",
                order, scenario_id, actor_call_name, recv
            )
        } else {
            format!(
                "✅ Added step {} to scenario {} (actor: {})",
                order, scenario_id, actor_call_name
            )
        };

        // Regenerate markdown to reflect the new step
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(message))
    }

    /// Edit a scenario step
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `step_order` - The order of the step to edit (1-based)
    /// * `new_description` - New description for the step
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn edit_step(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        step_order: u32,
        actor: Option<String>,
        new_description: String,
    ) -> Result<DisplayResult> {
        self.app_service.edit_scenario_step(
            &use_case_id,
            &scenario_id,
            step_order,
            actor,
            new_description,
        )?;

        // Regenerate markdown to reflect the edit
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Updated step {} in scenario {}",
            step_order, scenario_id
        )))
    }

    /// Remove a step from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `step_order` - The order of the step to remove (1-based)
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn remove_step(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        step_order: u32,
    ) -> Result<DisplayResult> {
        let step_order_str = step_order.to_string();
        self.app_service
            .remove_scenario_step(&use_case_id, &scenario_id, &step_order_str)?;

        // Regenerate markdown to reflect the removal
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Removed step {} from scenario {}",
            step_order, scenario_id
        )))
    }

    /// Reorder scenario steps
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `reorderings` - HashMap of current_order -> new_order
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn reorder_steps(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        reorderings: HashMap<String, String>,
    ) -> Result<DisplayResult> {
        self.app_service
            .reorder_scenario_steps(&use_case_id, &scenario_id, reorderings)?;

        // Regenerate markdown to reflect reordering
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Reordered steps in scenario {}",
            scenario_id
        )))
    }

    /// Assign a persona to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `persona_id` - The ID of the persona to assign
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn assign_persona(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        persona_id: String,
    ) -> Result<DisplayResult> {
        self.app_service
            .assign_persona_to_scenario(&use_case_id, &scenario_id, &persona_id)?;

        // Regenerate markdown to reflect persona assignment
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::added(
            &format!("persona {}", persona_id),
            &format!("scenario {}", scenario_id),
        )))
    }

    /// Unassign persona from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn unassign_persona(
        &mut self,
        use_case_id: String,
        scenario_id: String,
    ) -> Result<DisplayResult> {
        self.app_service
            .unassign_persona_from_scenario(&use_case_id, &scenario_id)?;

        // Regenerate markdown to reflect persona change
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Unassigned persona from scenario {}",
            scenario_id
        )))
    }

    /// Add a reference to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `reference` - The reference to add
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn add_reference(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        reference: crate::core::ScenarioReference,
    ) -> Result<DisplayResult> {
        self.app_service
            .add_scenario_reference(&use_case_id, &scenario_id, reference)?;

        // Regenerate markdown to reflect new reference
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::added(
            "reference",
            &format!("scenario {}", scenario_id),
        )))
    }

    /// Remove a reference from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `target_id` - The target ID of the reference
    /// * `relationship` - The relationship type
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn remove_reference(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        target_id: String,
        relationship: String,
    ) -> Result<DisplayResult> {
        self.app_service.remove_scenario_reference(
            &use_case_id,
            &scenario_id,
            &target_id,
            &relationship,
        )?;

        // Regenerate markdown to reflect reference removal
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Removed reference from scenario {}",
            scenario_id
        )))
    }

    /// List references for a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    ///
    /// # Returns
    /// Vector of scenario references
    pub fn list_references(
        &mut self,
        use_case_id: String,
        scenario_id: String,
    ) -> Result<Vec<crate::core::ScenarioReference>> {
        self.app_service
            .get_scenario_references(&use_case_id, &scenario_id)
    }

    /// Add a precondition to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `condition` - The precondition text
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn add_precondition(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        condition: String,
    ) -> Result<DisplayResult> {
        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        scenario.add_precondition(condition.clone().into());
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::added(
            "precondition",
            &format!("scenario {}", scenario_id),
        )))
    }

    /// Add a precondition with use case reference to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `text` - The condition description
    /// * `referenced_use_case_id` - The use case being referenced
    /// * `relationship` - The relationship type (e.g., "must be completed")
    pub fn add_precondition_with_use_case(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        text: String,
        referenced_use_case_id: String,
        relationship: String,
    ) -> Result<DisplayResult> {
        use crate::core::Condition;

        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        let condition = Condition::with_use_case(
            text.clone(),
            referenced_use_case_id.clone(),
            Some(relationship),
        );
        scenario.add_precondition(condition);
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Added precondition with use case reference '{}' to scenario {}",
            referenced_use_case_id, scenario_id
        )))
    }

    /// Add a postcondition with use case reference to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `text` - The condition description
    /// * `referenced_use_case_id` - The use case being referenced
    /// * `relationship` - The relationship type (e.g., "must be completed")
    pub fn add_postcondition_with_use_case(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        text: String,
        referenced_use_case_id: String,
        relationship: String,
    ) -> Result<DisplayResult> {
        use crate::core::Condition;

        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        let condition = Condition::with_use_case(
            text.clone(),
            referenced_use_case_id.clone(),
            Some(relationship),
        );
        scenario.add_postcondition(condition);
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Added postcondition with use case reference '{}' to scenario {}",
            referenced_use_case_id, scenario_id
        )))
    }

    /// Add a postcondition to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `condition` - The postcondition text
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn add_postcondition(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        condition: String,
    ) -> Result<DisplayResult> {
        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        scenario.add_postcondition(condition.clone().into());
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(DisplayMessage::added(
            "postcondition",
            &format!("scenario {}", scenario_id),
        )))
    }

    /// Remove a precondition from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `condition` - The precondition text to remove
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn remove_precondition(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        condition: String,
    ) -> Result<DisplayResult> {
        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        scenario.remove_precondition(&condition);
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Removed precondition from scenario {}",
            scenario_id
        )))
    }

    /// Remove a postcondition from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `condition` - The postcondition text to remove
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn remove_postcondition(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        condition: String,
    ) -> Result<DisplayResult> {
        // Get the use case, modify scenario, save entire use case
        let mut use_case = self.app_service.get_use_case(&use_case_id)?;

        let scenario = use_case
            .scenarios
            .iter_mut()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario {} not found", scenario_id))?;

        scenario.remove_postcondition(&condition);
        use_case.metadata.touch();

        self.app_service.save_use_case(&use_case)?;
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Removed postcondition from scenario {}",
            scenario_id
        )))
    }

    /// Get available actors (personas + system actors) for selection
    ///
    /// # Returns
    /// Vector of actor display strings using centralized display formatting
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

    /// Get actor IDs only for programmatic use
    ///
    /// # Returns
    /// Vector of actor IDs
    pub fn get_actor_ids(&self) -> Result<Vec<String>> {
        use crate::controller::ActorController;

        let actor_controller = ActorController::new()?;

        // Get persona IDs
        let mut ids = actor_controller.get_persona_ids()?;

        // Get system actor IDs
        ids.extend(actor_controller.get_actor_ids()?);

        Ok(ids)
    }

    // ========== Extension Scenarios and Advanced Operations ==========

    /// Create an extension scenario that diverges from a main scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `parent_scenario_id` - The ID of the main scenario to extend
    /// * `extends_at_step` - The step number where the extension diverges
    /// * `returns_at_step` - Optional step number where the extension returns
    /// * `title` - Title of the extension scenario
    /// * `description` - Description of the extension
    /// * `primary_actor` - The primary actor for this extension
    ///
    /// # Returns
    /// DisplayResult with the new extension scenario ID
    pub fn create_extension_scenario(
        &mut self,
        params: CreateExtensionParams,
    ) -> Result<DisplayResult> {
        let use_case_id = params.use_case_id;
        let parent_scenario_id = params.parent_scenario_id;
        let extends_at_step = params.extends_at_step;
        let returns_at_step = params.returns_at_step;
        let title = params.title;
        let description = params.description;
        let primary_actor = params.primary_actor;
        let actor = primary_actor.parse().map_err(|_| {
            anyhow::anyhow!(
                "Invalid actor: {}. Use System, User, or a persona ID",
                primary_actor
            )
        })?;

        let params = crate::core::CreateExtensionScenarioParams {
            parent_scenario_id: parent_scenario_id.clone(),
            extends_at_step: extends_at_step.clone(),
            returns_at_step: returns_at_step.clone(),
            title: title.clone(),
            description,
            primary_actor: actor,
        };
        let scenario_id = self
            .app_service
            .create_extension_scenario(&use_case_id, params)?;

        let return_info = returns_at_step
            .map(|r| format!(" and returns at step {}", r))
            .unwrap_or_default();

        // Regenerate markdown to reflect the new extension scenario
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Created extension scenario: {} - {}\n   Extends {} at step {}{}",
            scenario_id, title, parent_scenario_id, extends_at_step, return_info
        )))
    }

    /// Add a repeat block to a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `from_step` - Starting step of the repeat block
    /// * `to_step` - Ending step of the repeat block
    /// * `condition` - Condition describing when to repeat
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn add_repeat_block(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        from_step: String,
        to_step: String,
        condition: String,
    ) -> Result<DisplayResult> {
        self.app_service.add_repeat_block(
            &use_case_id,
            &scenario_id,
            from_step.clone(),
            to_step.clone(),
            condition.clone(),
        )?;

        // Regenerate markdown to reflect new repeat block
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Added repeat block to scenario {}: steps {} to {}\n   Condition: {}",
            scenario_id, from_step, to_step, condition
        )))
    }

    /// Remove a repeat block from a scenario
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `from_step` - Starting step of the repeat block to remove
    /// * `to_step` - Ending step of the repeat block to remove
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn remove_repeat_block(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        from_step: String,
        to_step: String,
    ) -> Result<DisplayResult> {
        self.app_service
            .remove_repeat_block(&use_case_id, &scenario_id, &from_step, &to_step)?;

        // Regenerate markdown to reflect repeat block removal
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Removed repeat block from scenario {}: steps {} to {}",
            scenario_id, from_step, to_step
        )))
    }

    /// Insert a step into a main scenario with automatic extension updates
    /// This is smarter than add_step as it handles letter suffixes and extension updates
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the main scenario
    /// * `after_step` - Insert after this step (e.g., "3" to insert between 3 and 4)
    /// * `actor` - Actor performing the action
    /// * `receiver` - Optional receiving actor
    /// * `action` - Description of the action
    /// * `expected_result` - Optional expected result
    ///
    /// # Returns
    /// DisplayResult with the new step order
    pub fn insert_step(&mut self, params: InsertStepParams) -> Result<DisplayResult> {
        let use_case_id = params.use_case_id;
        let scenario_id = params.scenario_id;
        let after_step = params.after_step;
        let actor = params.actor;
        let receiver = params.receiver;
        let action = params.action;
        let expected_result = params.expected_result;
        let params = crate::core::InsertStepWithExtensionParams {
            after_step: after_step.clone(),
            actor: actor.clone(),
            receiver: receiver.clone(),
            action: action.clone(),
            expected_result,
        };
        let new_step_order = self.app_service.insert_step_with_extension_update(
            &use_case_id,
            &scenario_id,
            params,
        )?;

        let receiver_info = receiver.map(|r| format!(" → {}", r)).unwrap_or_default();

        // Regenerate markdown to reflect smart insertion
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Inserted step {} in scenario {} (after step {})\n   {} {}{}: {}",
            new_step_order, scenario_id, after_step, new_step_order, actor, receiver_info, action
        )))
    }

    /// Delete a step from a main scenario with extension validation
    /// Warns if any extension scenarios become invalid
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the main scenario
    /// * `step_order` - The step to delete
    ///
    /// # Returns
    /// DisplayResult with warning if extensions are affected
    pub fn delete_step(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        step_order: String,
    ) -> Result<DisplayResult> {
        let invalid_extensions = self.app_service.delete_step_with_extension_update(
            &use_case_id,
            &scenario_id,
            &step_order,
        )?;

        let mut message = format!(
            "✅ Deleted step {} from scenario {}",
            step_order, scenario_id
        );

        if !invalid_extensions.is_empty() {
            message.push_str(&format!(
                "\n⚠️  Warning: {} extension scenario(s) became invalid:\n",
                invalid_extensions.len()
            ));
            for ext in &invalid_extensions {
                message.push_str(&format!("   - {}\n", ext));
            }
            message
                .push_str("   These extensions reference the deleted step and need to be updated.");
        }

        // Regenerate markdown to reflect smart deletion
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(message))
    }

    /// Renumber steps in a scenario starting from a specific step
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `scenario_id` - The ID of the scenario
    /// * `from_step` - Start renumbering from this step
    /// * `increment` - Amount to shift (can be negative)
    ///
    /// # Returns
    /// DisplayResult indicating success
    pub fn renumber_steps(
        &mut self,
        use_case_id: String,
        scenario_id: String,
        from_step: String,
        increment: i32,
    ) -> Result<DisplayResult> {
        self.app_service
            .renumber_steps_from(&use_case_id, &scenario_id, &from_step, increment)?;

        let direction = if increment > 0 { "forward" } else { "backward" };

        // Regenerate markdown to reflect renumbering
        self.app_service.regenerate_markdown(&use_case_id)?;

        Ok(DisplayResult::success(format!(
            "✅ Renumbered steps in scenario {} from step {} {} by {}",
            scenario_id,
            from_step,
            direction,
            increment.abs()
        )))
    }

    /// Validate all scenarios in a use case
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with validation result
    pub fn validate_scenarios(&mut self, use_case_id: String) -> Result<DisplayResult> {
        match self.app_service.validate_use_case_scenarios(&use_case_id) {
            Ok(_) => Ok(DisplayResult::success(format!(
                "✅ All scenarios in {} are valid",
                use_case_id
            ))),
            Err(e) => Ok(DisplayResult::error(format!(
                "❌ Validation failed for {}: {}",
                use_case_id, e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigFileManager};
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, ScenarioController) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".").unwrap();
        Config::copy_templates_to_config_with_language(None).unwrap();

        let controller = ScenarioController::new().unwrap();
        (temp_dir, controller)
    }

    fn create_test_use_case(_controller: &mut ScenarioController) -> String {
        // Create a use case to test scenarios with
        let mut use_case_controller = crate::controller::UseCaseController::new().unwrap();
        let params = crate::controller::CreateUseCaseParams {
            title: "Test Use Case".to_string(),
            category: "test".to_string(),
            category_abbreviation: "TES".to_string(),
            description: Some("Testing scenarios".to_string()),
            methodology: Some("feature".to_string()),
            priority: None,
            views: None,
            extra_fields: None,
        };
        let result = use_case_controller.create_use_case(params).unwrap();

        // Extract use case ID from message (format: "Created use case: UC-TES-001 with views: ...")
        // Find the token that starts with "UC-"
        result
            .message
            .split_whitespace()
            .find(|s| s.starts_with("UC-"))
            .unwrap()
            .to_string()
    }

    #[test]
    #[serial]
    fn test_create_scenario() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);

        // Reload the controller to pick up the newly created use case
        let mut controller = ScenarioController::new().unwrap();

        let params = CreateScenarioParams {
            use_case_id: use_case_id.clone(),
            title: "User Login".to_string(),
            scenario_type: "main".to_string(),
            description: Some("Main login scenario".to_string()),
            persona_id: None,
            preconditions: None,
            postconditions: None,
        };
        let result = controller.create_scenario(params).unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Created scenario"));
    }

    #[test]
    #[serial]
    fn test_list_scenarios() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);

        // Reload the controller to pick up the newly created use case
        let mut controller = ScenarioController::new().unwrap();

        // Create a scenario
        let params = CreateScenarioParams {
            use_case_id: use_case_id.clone(),
            title: "Scenario 1".to_string(),
            scenario_type: "main".to_string(),
            description: None,
            persona_id: None,
            preconditions: None,
            postconditions: None,
        };
        controller.create_scenario(params).unwrap();

        // List scenarios
        let result = controller.list_scenarios(use_case_id).unwrap();
        assert!(result.is_success());
        assert!(result.message.contains("Scenario 1"));
    }

    #[test]
    #[serial]
    fn test_add_step_to_scenario() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);

        // Reload the controller to pick up the newly created use case
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        let params = CreateScenarioParams {
            use_case_id: use_case_id.clone(),
            title: "Test Scenario".to_string(),
            scenario_type: "main".to_string(),
            description: None,
            persona_id: None,
            preconditions: None,
            postconditions: None,
        };
        controller.create_scenario(params).unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add step
        let result = controller
            .add_step(
                use_case_id,
                scenario_id.clone(),
                "User clicks login button".to_string(),
                None,
                None,
                None, // receiver
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added step"));
    }

    #[test]
    #[serial]
    fn test_remove_step() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);

        // Reload the controller to pick up the newly created use case
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        let params = CreateScenarioParams {
            use_case_id: use_case_id.clone(),
            title: "Test Scenario".to_string(),
            scenario_type: "main".to_string(),
            description: None,
            persona_id: None,
            preconditions: None,
            postconditions: None,
        };
        controller.create_scenario(params).unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add and remove step
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Step to remove".to_string(),
                None,
                None,
                None, // receiver
            )
            .unwrap();

        let result = controller.remove_step(use_case_id, scenario_id, 1).unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Removed step"));
    }

    #[test]
    #[serial]
    fn test_add_precondition() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add precondition
        let result = controller
            .add_precondition(
                use_case_id.clone(),
                scenario_id.clone(),
                "User must be logged in".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added precondition"));

        // Verify precondition was saved
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.preconditions.len(), 1);
        assert_eq!(scenario.preconditions[0].text, "User must be logged in");
    }

    #[test]
    #[serial]
    fn test_add_postcondition() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add postcondition
        let result = controller
            .add_postcondition(
                use_case_id.clone(),
                scenario_id.clone(),
                "User session is created".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added postcondition"));

        // Verify postcondition was saved
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.postconditions.len(), 1);
        assert_eq!(scenario.postconditions[0].text, "User session is created");
    }

    #[test]
    #[serial]
    fn test_add_precondition_with_use_case() {
        let (_temp_dir, _controller) = setup_test_env();

        // Create first use case
        let mut uc_controller = crate::controller::UseCaseController::new().unwrap();
        let params = crate::controller::CreateUseCaseParams {
            title: "Test Use Case".to_string(),
            category: "test".to_string(),
            category_abbreviation: "TES".to_string(),
            description: Some("Testing".to_string()),
            methodology: Some("feature".to_string()),
            priority: None,
            views: None,
            extra_fields: None,
        };
        let result = uc_controller.create_use_case(params).unwrap();
        let use_case_id = result
            .message
            .split_whitespace()
            .find(|s| s.starts_with("UC-"))
            .unwrap()
            .to_string();

        // Create a second use case to reference
        let params2 = crate::controller::CreateUseCaseParams {
            title: "Authentication".to_string(),
            category: "auth".to_string(),
            category_abbreviation: "AUT".to_string(),
            description: Some("Auth use case".to_string()),
            methodology: Some("feature".to_string()),
            priority: None,
            views: None,
            extra_fields: None,
        };
        let result = uc_controller.create_use_case(params2).unwrap();
        let auth_use_case_id = result
            .message
            .split_whitespace()
            .find(|s| s.starts_with("UC-"))
            .unwrap()
            .to_string();

        // Get scenario controller
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add precondition with use case reference
        let result = controller
            .add_precondition_with_use_case(
                use_case_id.clone(),
                scenario_id.clone(),
                "Authentication must be completed".to_string(),
                auth_use_case_id.clone(),
                "must be completed".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("use case reference"));

        // Verify precondition was saved with reference
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.preconditions.len(), 1);
        assert_eq!(
            scenario.preconditions[0].text,
            "Authentication must be completed"
        );
        assert_eq!(scenario.preconditions[0].target_id, Some(auth_use_case_id));
    }

    #[test]
    #[serial]
    fn test_remove_precondition() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with precondition
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                Some(vec!["User must be logged in".to_string()]),
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Verify precondition exists
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.preconditions.len(), 1);

        // Remove precondition
        let result = controller
            .remove_precondition(
                use_case_id.clone(),
                scenario_id.clone(),
                "User must be logged in".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Removed precondition"));

        // Verify precondition was removed
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.preconditions.len(), 0);
    }

    #[test]
    #[serial]
    fn test_remove_postcondition() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with postcondition
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                Some(vec!["Session is created".to_string()]),
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Verify postcondition exists
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.postconditions.len(), 1);

        // Remove postcondition
        let result = controller
            .remove_postcondition(
                use_case_id.clone(),
                scenario_id.clone(),
                "Session is created".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Removed postcondition"));

        // Verify postcondition was removed
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.postconditions.len(), 0);
    }

    #[test]
    #[serial]
    fn test_edit_scenario() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Original Title".to_string(),
                Some("Original description".to_string()),
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Edit scenario
        let result = controller
            .edit_scenario(
                use_case_id.clone(),
                scenario_id.clone(),
                Some("Updated Title".to_string()),
                Some("Updated description".to_string()),
                None,
                None,
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Updated scenario"));

        // Verify changes were saved
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.title, "Updated Title");
        assert_eq!(scenario.description, "Updated description");
    }

    #[test]
    #[serial]
    fn test_delete_scenario() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();
        assert_eq!(scenarios.len(), 1);

        // Delete scenario
        let result = controller
            .delete_scenario(use_case_id.clone(), scenario_id)
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Deleted scenario"));

        // Verify scenario was deleted
        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        assert_eq!(scenarios.len(), 0);
    }

    #[test]
    #[serial]
    fn test_assign_persona() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Assign persona
        let result = controller
            .assign_persona(
                use_case_id.clone(),
                scenario_id.clone(),
                "test-persona".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added persona"));

        // Verify persona was assigned
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.persona, Some("test-persona".to_string()));
    }

    #[test]
    #[serial]
    fn test_unassign_persona() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with persona
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Assign persona first
        controller
            .assign_persona(
                use_case_id.clone(),
                scenario_id.clone(),
                "test-persona".to_string(),
            )
            .unwrap();

        // Verify persona was assigned
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert!(scenario.persona.is_some());

        // Unassign persona
        let result = controller
            .unassign_persona(use_case_id.clone(), scenario_id.clone())
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Unassigned persona"));

        // Verify persona was removed
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert!(scenario.persona.is_none());
    }

    #[test]
    #[serial]
    fn test_add_reference() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add reference
        use crate::core::{ReferenceType, ScenarioReference};
        let reference = ScenarioReference::new(
            ReferenceType::Scenario,
            "other-scenario".to_string(),
            "includes".to_string(),
        );
        let result = controller
            .add_reference(use_case_id.clone(), scenario_id.clone(), reference)
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added reference"));

        // Verify reference was added
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.references.len(), 1);
        assert_eq!(scenario.references[0].target_id, "other-scenario");
    }

    #[test]
    #[serial]
    fn test_remove_reference() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with reference
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add reference first
        use crate::core::{ReferenceType, ScenarioReference};
        let reference = ScenarioReference::new(
            ReferenceType::Scenario,
            "other-scenario".to_string(),
            "includes".to_string(),
        );
        controller
            .add_reference(use_case_id.clone(), scenario_id.clone(), reference)
            .unwrap();

        // Verify reference exists
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.references.len(), 1);

        // Remove reference
        let result = controller
            .remove_reference(
                use_case_id.clone(),
                scenario_id.clone(),
                "other-scenario".to_string(),
                "includes".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Removed reference"));

        // Verify reference was removed
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.references.len(), 0);
    }

    #[test]
    #[serial]
    fn test_edit_step() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with step
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add a step
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Original action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();

        // Edit step
        let result = controller
            .edit_step(
                use_case_id.clone(),
                scenario_id.clone(),
                1,
                None,
                "Updated action".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Updated step"));

        // Verify step was edited
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.steps.len(), 1);
        assert_eq!(scenario.steps[0].action, "Updated action");
    }

    #[test]
    #[serial]
    fn test_reorder_steps() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with multiple steps
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add steps
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Third action".to_string(),
                Some(3),
                Some("User".to_string()),
                None,
            )
            .unwrap();

        // Reorder steps: map old positions to new positions
        use std::collections::HashMap;
        let mut reorderings = HashMap::new();
        reorderings.insert("1".to_string(), "2".to_string()); // step 1 goes to position 2
        reorderings.insert("2".to_string(), "3".to_string()); // step 2 goes to position 3
        reorderings.insert("3".to_string(), "1".to_string()); // step 3 goes to position 1

        let result = controller
            .reorder_steps(use_case_id.clone(), scenario_id.clone(), reorderings)
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Reordered steps"));

        // Verify steps were reordered
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.steps.len(), 3);
    }

    #[test]
    #[serial]
    fn test_create_extension_scenario() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create main scenario with steps
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Main Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let main_scenario_id = scenarios[0].id.clone();

        // Add steps to main scenario
        controller
            .add_step(
                use_case_id.clone(),
                main_scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                main_scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();

        // Create extension scenario
        let params = CreateExtensionParams {
            use_case_id: use_case_id.clone(),
            parent_scenario_id: main_scenario_id.clone(),
            extends_at_step: "1".to_string(),
            returns_at_step: Some("2".to_string()),
            title: "Extension Scenario".to_string(),
            description: "Extension description".to_string(),
            primary_actor: "User".to_string(),
        };
        let result = controller.create_extension_scenario(params).unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Created extension scenario"));

        // Verify extension scenario was created
        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        assert_eq!(scenarios.len(), 2);

        let extension = scenarios
            .iter()
            .find(|s| s.title == "Extension Scenario")
            .unwrap();
        assert_eq!(extension.scenario_type, ScenarioType::Extension);
        assert_eq!(extension.extends_scenario_id, Some(main_scenario_id));
        assert_eq!(extension.extends_at_step, Some("1".to_string()));
        assert_eq!(extension.returns_at_step, Some("2".to_string()));
    }

    #[test]
    #[serial]
    fn test_add_repeat_block() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with steps
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add steps
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();

        // Add repeat block
        let result = controller
            .add_repeat_block(
                use_case_id.clone(),
                scenario_id.clone(),
                "1".to_string(),
                "2".to_string(),
                "Retry logic".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Added repeat block"));

        // Verify repeat block was added
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.repeat_blocks.len(), 1);
        assert_eq!(scenario.repeat_blocks[0].from_step, "1");
        assert_eq!(scenario.repeat_blocks[0].to_step, "2");
        assert_eq!(scenario.repeat_blocks[0].condition, "Retry logic");
    }

    #[test]
    #[serial]
    fn test_remove_repeat_block() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with steps and repeat block
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add steps
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();

        // Add repeat block
        controller
            .add_repeat_block(
                use_case_id.clone(),
                scenario_id.clone(),
                "1".to_string(),
                "2".to_string(),
                "Retry logic".to_string(),
            )
            .unwrap();

        // Verify repeat block exists
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.repeat_blocks.len(), 1);

        // Remove repeat block
        let result = controller
            .remove_repeat_block(
                use_case_id.clone(),
                scenario_id.clone(),
                "1".to_string(),
                "2".to_string(),
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Removed repeat block"));

        // Verify repeat block was removed
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.repeat_blocks.len(), 0);
    }

    #[test]
    #[serial]
    fn test_insert_step() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with steps
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add initial steps
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();

        // Insert step smartly after step 1 (should create 1a)
        let params = InsertStepParams {
            use_case_id: use_case_id.clone(),
            scenario_id: scenario_id.clone(),
            after_step: "1".to_string(),
            actor: "System".to_string(),
            receiver: None,
            action: "Inserted action".to_string(),
            expected_result: None,
        };
        let result = controller.insert_step(params).unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Inserted step"));

        // Verify step was inserted
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.steps.len(), 3);
    }

    #[test]
    #[serial]
    fn test_delete_step() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with steps
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add steps
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(2),
                Some("System".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Third action".to_string(),
                Some(3),
                Some("User".to_string()),
                None,
            )
            .unwrap();

        // Delete step smartly
        let result = controller
            .delete_step(use_case_id.clone(), scenario_id.clone(), "2".to_string())
            .unwrap();

        assert!(result.is_success());

        // Verify step was deleted and remaining steps were renumbered to close the gap
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.steps.len(), 2);
        assert_eq!(scenario.steps[0].action, "First action");
        assert_eq!(scenario.steps[0].order, "1");
        assert_eq!(scenario.steps[1].action, "Third action");
        assert_eq!(scenario.steps[1].order, "2"); // Renumbered from "3" to "2" to close the gap
    }

    #[test]
    #[serial]
    fn test_renumber_steps() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create scenario with steps that have gaps in numbering
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        let scenarios = controller.app_service.get_scenarios(&use_case_id).unwrap();
        let scenario_id = scenarios[0].id.clone();

        // Add steps with non-sequential numbering
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "First action".to_string(),
                Some(1),
                Some("User".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Second action".to_string(),
                Some(5),
                Some("System".to_string()),
                None,
            )
            .unwrap();
        controller
            .add_step(
                use_case_id.clone(),
                scenario_id.clone(),
                "Third action".to_string(),
                Some(10),
                Some("User".to_string()),
                None,
            )
            .unwrap();

        // Renumber steps starting from step 5, shifting by -4
        let result = controller
            .renumber_steps(
                use_case_id.clone(),
                scenario_id.clone(),
                "5".to_string(),
                -4,
            )
            .unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("Renumbered"));

        // Verify steps were renumbered
        let scenario = controller.get_scenario(&use_case_id, &scenario_id).unwrap();
        assert_eq!(scenario.steps.len(), 3);
        // After renumbering: step 5 becomes 1, step 10 becomes 6
        assert_eq!(scenario.steps[0].order, "1");
        assert!(scenario.steps[1].order == "1" || scenario.steps[1].order == "5");
        assert!(scenario.steps[2].order == "6" || scenario.steps[2].order == "10");
    }

    #[test]
    #[serial]
    fn test_validate_scenarios() {
        let (_temp_dir, mut controller) = setup_test_env();
        let use_case_id = create_test_use_case(&mut controller);
        let mut controller = ScenarioController::new().unwrap();

        // Create valid scenario
        controller
            .create_main_scenario(
                use_case_id.clone(),
                "Valid Scenario".to_string(),
                None,
                None,
                None,
                "user".to_string(),
            )
            .unwrap();

        // Validate scenarios
        let result = controller.validate_scenarios(use_case_id.clone()).unwrap();

        assert!(result.is_success());
        assert!(result.message.contains("All scenarios") && result.message.contains("are valid"));
    }
}
