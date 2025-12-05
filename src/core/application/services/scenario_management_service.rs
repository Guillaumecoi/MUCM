use crate::core::application::creators::ScenarioCreator;
use crate::core::utils::suggest_alternatives;
use crate::core::{
    domain::{RepeatBlock, ScenarioReference, ScenarioType},
    ExtensionPointUpdater, ReferenceType, ScenarioFlowValidator, ScenarioReferenceValidator,
    Status, UseCase, UseCaseRepository,
};
use anyhow::Result;

/// Service for managing scenarios within use cases
///
/// This service handles CRUD operations for scenarios, scenario steps,
/// and scenario references.
pub struct ScenarioManagementService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a mut Vec<UseCase>,
    scenario_creator: &'a ScenarioCreator,
}

impl<'a> ScenarioManagementService<'a> {
    pub fn new(
        repository: &'a dyn UseCaseRepository,
        use_cases: &'a mut Vec<UseCase>,
        scenario_creator: &'a ScenarioCreator,
    ) -> Self {
        Self {
            repository,
            use_cases,
            scenario_creator,
        }
    }

    /// Add a scenario to a use case
    pub fn add_scenario(
        &mut self,
        use_case_id: &str,
        params: crate::core::application::creators::ScenarioParams,
    ) -> Result<String> {
        let index = self.find_use_case_index(use_case_id)?;
        let use_case = &self.use_cases[index];

        let scenario = self.scenario_creator.create_scenario(use_case, params);

        let mut updated_use_case = self.use_cases[index].clone();
        updated_use_case.add_scenario(scenario.clone());
        self.repository.save(&updated_use_case)?;
        self.use_cases[index] = updated_use_case;

        Ok(scenario.id)
    }

    /// Update the status of a scenario
    pub fn update_scenario_status(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        new_status: Status,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        use_case.update_scenario_status(scenario_id, new_status)?;
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Add a step to an existing scenario
    pub fn add_scenario_step(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        params: crate::core::application::creators::StepParams,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let step = self.scenario_creator.create_scenario_step(params);

        use_case.add_step_to_scenario(scenario_id, step)?;
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Remove a step from a scenario
    pub fn remove_scenario_step(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        step_order: &str,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        use_case.remove_step_from_scenario(scenario_id, step_order)?;
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Edit an existing scenario
    pub fn edit_scenario(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        title: Option<String>,
        description: Option<String>,
        scenario_type: Option<ScenarioType>,
        status: Option<Status>,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        // Update fields if provided
        if let Some(new_title) = title {
            use_case.scenarios[scenario_index].title = new_title;
        }
        if let Some(new_desc) = description {
            use_case.scenarios[scenario_index].description = new_desc;
        }
        if let Some(new_type) = scenario_type {
            use_case.scenarios[scenario_index].scenario_type = new_type;
        }
        if let Some(new_status) = status {
            use_case.scenarios[scenario_index].set_status(new_status);
        }

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Delete a scenario from a use case
    pub fn delete_scenario(&mut self, use_case_id: &str, scenario_id: &str) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        // Check if other scenarios reference this one
        let has_references = use_case
            .scenarios
            .iter()
            .any(|s| s.id != scenario_id && s.references_scenario(scenario_id));

        if has_references {
            return Err(anyhow::anyhow!(
                "Cannot delete scenario '{}': it is referenced by other scenarios",
                scenario_id
            ));
        }

        use_case.scenarios.retain(|s| s.id != scenario_id);
        use_case.metadata.touch();

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Edit a scenario step
    pub fn edit_scenario_step(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        step_order: u32,
        actor: Option<String>,
        new_description: String,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        let step_order_str = step_order.to_string();
        let step = use_case.scenarios[scenario_index]
            .steps
            .iter_mut()
            .find(|s| s.order == step_order_str)
            .ok_or_else(|| {
                anyhow::anyhow!("Step {} not found in scenario {}", step_order, scenario_id)
            })?;

        if let Some(actor_str) = actor {
            step.acting_actor = actor_str;
        }
        step.action = new_description;
        use_case.metadata.touch(); // Update use case metadata when scenario changes

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Reorder scenario steps
    pub fn reorder_scenario_steps(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        reorderings: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        // Apply reorderings
        for step in &mut use_case.scenarios[scenario_index].steps {
            if let Some(new_order) = reorderings.get(&step.order) {
                step.order = new_order.clone();
            }
        }

        // Re-sort steps using StepOrder comparison
        use_case.scenarios[scenario_index].steps.sort_by(|a, b| {
            use crate::core::domain::StepOrder;
            StepOrder::compare(&a.order, &b.order)
        });
        use_case.metadata.touch(); // Update use case metadata when scenario changes

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Assign a persona to a scenario
    pub fn assign_persona_to_scenario(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        persona_id: &str,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        use_case.scenarios[scenario_index].persona = Some(persona_id.to_string());
        use_case.metadata.touch(); // Update use case metadata when scenario changes

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Unassign persona from a scenario
    pub fn unassign_persona_from_scenario(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        use_case.scenarios[scenario_index].persona = None;
        use_case.metadata.touch(); // Update use case metadata when scenario changes

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Add a reference to a scenario
    pub fn add_scenario_reference(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        reference: ScenarioReference,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        // Validate no circular reference for scenario-to-scenario references
        if matches!(reference.ref_type, ReferenceType::Scenario) {
            ScenarioReferenceValidator::validate_no_circular_reference(
                &use_case,
                scenario_id,
                &reference.target_id,
            )?;
        }

        use_case.scenarios[scenario_index].add_reference(reference);
        use_case.metadata.touch();

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Remove a reference from a scenario
    pub fn remove_scenario_reference(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        target_id: &str,
        relationship: &str,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        use_case.scenarios[scenario_index].remove_reference(target_id, relationship);
        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Create an extension scenario that diverges from a main scenario
    pub fn create_extension_scenario(
        &mut self,
        use_case_id: &str,
        params: crate::core::application::creators::ExtensionScenarioParams,
        returns_at_step: Option<String>,
    ) -> Result<String> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario =
            self.scenario_creator
                .create_extension_scenario(&use_case, params, returns_at_step)?;

        // Validate the complete scenario flow
        ScenarioFlowValidator::validate_scenario_flow(&scenario, &use_case)?;

        let scenario_id = scenario.id.clone();
        use_case.add_scenario(scenario);
        use_case.metadata.touch();

        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(scenario_id)
    }

    /// Add a repeat block to a scenario
    pub fn add_repeat_block(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        from_step: String,
        to_step: String,
        condition: String,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        let repeat_block = RepeatBlock::new(from_step, to_step, condition);

        // Validate the block before adding
        repeat_block
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid repeat block: {}", e))?;

        use_case.scenarios[scenario_index]
            .repeat_blocks
            .push(repeat_block);

        // Validate all repeat blocks together
        ScenarioFlowValidator::validate_repeat_blocks(&use_case.scenarios[scenario_index])?;

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Remove a repeat block from a scenario
    pub fn remove_repeat_block(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        from_step: &str,
        to_step: &str,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        use_case.scenarios[scenario_index]
            .repeat_blocks
            .retain(|block| !(block.from_step == from_step && block.to_step == to_step));

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Insert a step into a main scenario and automatically update extension references
    pub fn insert_step_with_extension_update(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        after_step: &str,
        params: crate::core::application::creators::StepParams,
    ) -> Result<String> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        // Get the scenario and validate it's a main scenario
        let scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        if !scenario.is_main {
            anyhow::bail!("Can only insert steps into main scenarios");
        }

        // Suggest next step order
        let new_step_order = self
            .scenario_creator
            .suggest_next_step_order(&scenario.steps, after_step);

        // Create the step with the suggested order
        let step_params = crate::core::application::creators::StepParams {
            order: new_step_order.clone(),
            actor: params.actor,
            receiver: params.receiver,
            action: params.action,
            expected_result: params.expected_result,
        };

        let step = self.scenario_creator.create_scenario_step(step_params);

        // Clone scenario for extension update before mutation
        let scenario_for_update = use_case
            .scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .unwrap()
            .clone();

        // Add step to scenario
        use_case.add_step_to_scenario(scenario_id, step)?;

        // Update extension points
        ExtensionPointUpdater::update_after_insert(
            &scenario_for_update,
            &new_step_order,
            &mut use_case,
        )?;

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(new_step_order)
    }

    /// Delete a step from a main scenario and update extension references
    /// Returns list of extensions that became invalid
    pub fn delete_step_with_extension_update(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        step_order: &str,
    ) -> Result<Vec<String>> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        // Get the scenario and validate it's a main scenario
        let scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        if !scenario.is_main {
            anyhow::bail!("Can only delete steps from main scenarios using this method");
        }

        // Clone scenario for extension update before mutation
        let scenario_for_update = use_case
            .scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .unwrap()
            .clone();

        // Remove the step
        use_case.remove_step_from_scenario(scenario_id, step_order)?;

        // Auto-renumber remaining steps to close gaps
        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .unwrap();

        // Only renumber if it's a main scenario (numeric steps only)
        if use_case.scenarios[scenario_index].is_main {
            // Renumber steps after the deleted step to close the gap
            let deleted_order: u32 = step_order.parse().unwrap_or(0);
            if deleted_order > 0 {
                self.scenario_creator.renumber_steps_from(
                    &mut use_case.scenarios[scenario_index],
                    step_order,
                    -1, // Decrement by 1 to close the gap
                )?;
            }
        }

        // Update extension points and get list of invalid extensions
        let invalid_extensions = ExtensionPointUpdater::update_after_delete(
            &scenario_for_update,
            step_order,
            &mut use_case,
        )?;

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(invalid_extensions)
    }

    /// Renumber steps in a main scenario starting from a specific step
    pub fn renumber_steps_from(
        &mut self,
        use_case_id: &str,
        scenario_id: &str,
        from_step: &str,
        increment: i32,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let scenario_index = use_case
            .scenarios
            .iter()
            .position(|s| s.id == scenario_id)
            .ok_or_else(|| anyhow::anyhow!("Scenario with ID '{}' not found", scenario_id))?;

        // Renumber using the creator
        self.scenario_creator.renumber_steps_from(
            &mut use_case.scenarios[scenario_index],
            from_step,
            increment,
        )?;

        // Validate the scenario flow after renumbering
        ScenarioFlowValidator::validate_scenario_flow(
            &use_case.scenarios[scenario_index],
            &use_case,
        )?;

        use_case.metadata.touch();
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;

        Ok(())
    }

    /// Validate all scenarios in a use case
    pub fn validate_use_case_scenarios(&self, use_case_id: &str) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let use_case = &self.use_cases[index];
        ScenarioFlowValidator::validate_use_case_scenarios(use_case)
    }

    // Helper methods
    fn find_use_case_index(&self, use_case_id: &str) -> Result<usize> {
        self.use_cases
            .iter()
            .position(|uc| uc.id == use_case_id)
            .ok_or_else(|| {
                let available_ids: Vec<String> =
                    self.use_cases.iter().map(|uc| uc.id.clone()).collect();
                let error_msg = suggest_alternatives(use_case_id, &available_ids, "Use case");
                anyhow::anyhow!("{}", error_msg)
            })
    }
}
