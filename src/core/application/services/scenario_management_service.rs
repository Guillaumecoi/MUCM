use crate::core::application::creators::ScenarioCreator;
use crate::core::utils::suggest_alternatives;
use crate::core::{
    domain::{ScenarioReference, ScenarioType},
    ReferenceType, ScenarioReferenceValidator, Status, UseCase, UseCaseRepository,
};
use anyhow::Result;

/// Service for managing scenarios within use cases
///
/// This service handles CRUD operations for scenarios, scenario steps,
/// and scenario references.
pub struct ScenarioManagementService<'a> {
    repository: &'a Box<dyn UseCaseRepository>,
    use_cases: &'a mut Vec<UseCase>,
    scenario_creator: &'a ScenarioCreator,
}

impl<'a> ScenarioManagementService<'a> {
    pub fn new(
        repository: &'a Box<dyn UseCaseRepository>,
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
        title: String,
        scenario_type: ScenarioType,
        description: Option<String>,
        preconditions: Vec<String>,
        postconditions: Vec<String>,
        actors: Vec<String>,
    ) -> Result<String> {
        let index = self.find_use_case_index(use_case_id)?;
        let use_case = &self.use_cases[index];

        // Use first actor as primary actor, default to User if none provided
        let primary_actor = if let Some(first_actor) = actors.first() {
            first_actor.clone().into()
        } else {
            crate::core::domain::Actor::User
        };

        let scenario = self.scenario_creator.create_scenario(
            use_case,
            title,
            scenario_type,
            description,
            primary_actor,
            preconditions,
            postconditions,
            actors,
        );

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
        order: String,
        actor: String,
        receiver: Option<String>,
        action: String,
        expected_result: Option<String>,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();

        let step = self.scenario_creator.create_scenario_step(
            order,
            actor,
            receiver,
            action,
            expected_result,
        );

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
