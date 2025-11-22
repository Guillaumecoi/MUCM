//! Scenario Controller
//!
//! Manages scenario operations within use cases including CRUD operations,
//! step management, references, and persona assignments.

use crate::controller::DisplayResult;
use crate::core::{ScenarioType, Status, UseCaseCoordinator};
use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;

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
    pub fn create_scenario(
        &mut self,
        use_case_id: String,
        title: String,
        scenario_type: String,
        description: Option<String>,
        persona_id: Option<String>,
        preconditions: Option<Vec<String>>,
        postconditions: Option<Vec<String>>,
    ) -> Result<DisplayResult> {
        // Parse scenario type
        let parsed_type = ScenarioType::from_str(&scenario_type)
            .map_err(|_| anyhow::anyhow!("Invalid scenario type: {}", scenario_type))?;

        // Create scenario via coordinator (actors derived from steps)
        let scenario_id = self.app_service.add_scenario(
            &use_case_id,
            title.clone(),
            parsed_type,
            description.clone(),
            preconditions.unwrap_or_default(),
            postconditions.unwrap_or_default(),
            Vec::new(), // actors will be derived from steps
        )?;

        // Assign persona if provided
        if let Some(persona) = persona_id {
            self.app_service
                .assign_persona_to_scenario(&use_case_id, &scenario_id, &persona)?;
        }

        Ok(DisplayResult::success(format!(
            "✅ Created scenario: {} - {}",
            scenario_id, title
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
            .map(|s| Status::from_str(s))
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

        Ok(DisplayResult::success(format!(
            "✅ Updated scenario: {}",
            scenario_id
        )))
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

        Ok(DisplayResult::success(format!(
            "✅ Deleted scenario: {}",
            scenario_id
        )))
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
        let actor_name = actor.unwrap_or_else(|| "Actor".to_string());

        self.app_service.add_scenario_step(
            &use_case_id,
            &scenario_id,
            order_str,
            actor_name.clone(),
            receiver.clone(),
            step_description.clone(),
            None, // No expected result by default
        )?;

        let message = if let Some(ref recv) = receiver {
            format!(
                "✅ Added step {} to scenario {} ({} → {})",
                order, scenario_id, actor_name, recv
            )
        } else {
            format!(
                "✅ Added step {} to scenario {} (actor: {})",
                order, scenario_id, actor_name
            )
        };

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
        new_description: String,
    ) -> Result<DisplayResult> {
        self.app_service.edit_scenario_step(
            &use_case_id,
            &scenario_id,
            step_order,
            new_description,
        )?;

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

        Ok(DisplayResult::success(format!(
            "✅ Assigned persona {} to scenario {}",
            persona_id, scenario_id
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

        Ok(DisplayResult::success(format!(
            "✅ Added reference to scenario {}",
            scenario_id
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
        let mut scenario = self.get_scenario(&use_case_id, &scenario_id)?;
        scenario.add_precondition(condition.clone().into());

        // Save via coordinator
        self.app_service.edit_scenario(
            &use_case_id,
            &scenario_id,
            Some(scenario.title),
            Some(scenario.description),
            Some(scenario.scenario_type),
            Some(scenario.status),
        )?;

        Ok(DisplayResult::success(format!(
            "✅ Added precondition to scenario {}",
            scenario_id
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
        let mut scenario = self.get_scenario(&use_case_id, &scenario_id)?;
        scenario.add_postcondition(condition.clone().into());

        // Save via coordinator
        self.app_service.edit_scenario(
            &use_case_id,
            &scenario_id,
            Some(scenario.title),
            Some(scenario.description),
            Some(scenario.scenario_type),
            Some(scenario.status),
        )?;

        Ok(DisplayResult::success(format!(
            "✅ Added postcondition to scenario {}",
            scenario_id
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
        let mut scenario = self.get_scenario(&use_case_id, &scenario_id)?;
        scenario.remove_precondition(&condition);

        // Save via coordinator
        self.app_service.edit_scenario(
            &use_case_id,
            &scenario_id,
            Some(scenario.title),
            Some(scenario.description),
            Some(scenario.scenario_type),
            Some(scenario.status),
        )?;

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
        let mut scenario = self.get_scenario(&use_case_id, &scenario_id)?;
        scenario.remove_postcondition(&condition);

        // Save via coordinator
        self.app_service.edit_scenario(
            &use_case_id,
            &scenario_id,
            Some(scenario.title),
            Some(scenario.description),
            Some(scenario.scenario_type),
            Some(scenario.status),
        )?;

        Ok(DisplayResult::success(format!(
            "✅ Removed postcondition from scenario {}",
            scenario_id
        )))
    }

    /// Get available actors (personas + system actors) for selection
    ///
    /// # Returns
    /// Vector of actor display strings (emoji + name + id)
    pub fn get_available_actors(&self) -> Result<Vec<String>> {
        use crate::controller::ActorController;

        let actor_controller = ActorController::new()?;

        // Get personas
        let personas = actor_controller.list_personas()?;
        let mut actors: Vec<String> = personas
            .iter()
            .map(|p| {
                let emoji = p
                    .extra
                    .get("emoji")
                    .and_then(|v| v.as_str())
                    .unwrap_or("🙂");
                format!("{} {} ({})", emoji, p.name, p.id)
            })
            .collect();

        // Get system actors
        let system_actors = actor_controller.list_actors(None)?;
        actors.extend(
            system_actors
                .iter()
                .map(|a| format!("{} {} ({})", a.emoji, a.name, a.id)),
        );

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
        let result = use_case_controller
            .create_use_case(
                "Test Use Case".to_string(),
                "test".to_string(),
                Some("Testing scenarios".to_string()),
                Some("feature".to_string()),
                None,
                None,
                None,
            )
            .unwrap();

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

        let result = controller
            .create_scenario(
                use_case_id.clone(),
                "User Login".to_string(),
                "main".to_string(),
                Some("Main login scenario".to_string()),
                None,
                None,
                None,
            )
            .unwrap();

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
        controller
            .create_scenario(
                use_case_id.clone(),
                "Scenario 1".to_string(),
                "main".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();

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
        controller
            .create_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                "main".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();

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
        controller
            .create_scenario(
                use_case_id.clone(),
                "Test Scenario".to_string(),
                "main".to_string(),
                None,
                None,
                None,
                None,
            )
            .unwrap();

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
}
