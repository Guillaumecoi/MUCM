use crate::core::domain::{Scenario, ScenarioStep, ScenarioType, StepOrder, UseCase};
use anyhow::{Context, Result};

/// Parameters for creating a scenario
pub struct ScenarioParams {
    pub title: String,
    pub scenario_type: ScenarioType,
    pub description: Option<String>,
    pub primary_actor: String,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    /// Supporting actors involved in this scenario (actor IDs)
    pub supporting_actors: Vec<String>,
}

/// Parameters for creating an extension scenario
pub struct ExtensionScenarioParams {
    pub parent_scenario_id: String,
    pub extends_at_step: String,
    pub title: String,
    pub description: Option<String>,
    pub primary_actor: String,
    pub scenario_type: ScenarioType,
}

/// Parameters for creating a scenario step
pub struct StepParams {
    pub order: String,
    pub actor: String,
    pub receiver: Option<String>,
    pub action: String,
    pub expected_result: Option<String>,
}

/// Handles scenario creation and management
pub struct ScenarioCreator;

impl ScenarioCreator {
    pub fn new() -> Self {
        Self
    }

    /// Create a new scenario for a use case
    pub fn create_scenario(&self, use_case: &UseCase, params: ScenarioParams) -> Scenario {
        let scenario_id = use_case.next_scenario_id();

        let mut scenario = Scenario::new(
            scenario_id,
            params.title,
            params.description.unwrap_or_default(),
            params.scenario_type,
            params.primary_actor,
        );

        // Add supporting actors
        scenario.supporting_actors = params.supporting_actors;

        // Add preconditions and postconditions
        for precondition in params.preconditions {
            scenario.add_precondition(precondition.into());
        }
        for postcondition in params.postconditions {
            scenario.add_postcondition(postcondition.into());
        }

        scenario
    }

    /// Create an extension scenario that diverges from a main scenario
    pub fn create_extension_scenario(
        &self,
        use_case: &UseCase,
        params: ExtensionScenarioParams,
        returns_at_step: Option<String>,
    ) -> Result<Scenario> {
        // Validate parent scenario exists and is a main scenario
        let parent = use_case
            .scenarios
            .iter()
            .find(|s| s.id == params.parent_scenario_id)
            .context(format!(
                "Parent scenario '{}' not found",
                params.parent_scenario_id
            ))?;

        if !parent.is_main {
            anyhow::bail!(
                "Cannot extend non-main scenario '{}'. Extensions can only extend main scenarios.",
                params.parent_scenario_id
            );
        }

        // Validate extends_at_step exists in parent
        if !parent
            .steps
            .iter()
            .any(|s| s.order == params.extends_at_step)
        {
            anyhow::bail!(
                "Step '{}' does not exist in parent scenario '{}'",
                params.extends_at_step,
                params.parent_scenario_id
            );
        }

        // Validate returns_at_step if specified
        if let Some(ref return_step) = returns_at_step {
            if !parent.steps.iter().any(|s| s.order == *return_step) {
                anyhow::bail!(
                    "Return step '{}' does not exist in parent scenario '{}'",
                    return_step,
                    params.parent_scenario_id
                );
            }

            // Allow return to any step, including before divergence (loop-back)
            // This enables validation retry patterns where errors return to earlier steps
        }

        let scenario_id = use_case.next_scenario_id();

        let mut scenario = Scenario::new(
            scenario_id,
            params.title,
            params.description.unwrap_or_default(),
            params.scenario_type,
            params.primary_actor,
        );

        // Mark as extension and link to parent
        scenario.is_main = false;
        scenario.extends_scenario_id = Some(params.parent_scenario_id.clone());
        scenario.extends_at_step = Some(params.extends_at_step);
        scenario.returns_at_step = returns_at_step;

        Ok(scenario)
    }

    /// Suggest the next step order after inserting between existing steps
    /// Returns a letter suffix (e.g., "3a") if inserting between "3" and "4"
    pub fn suggest_next_step_order(
        &self,
        existing_steps: &[ScenarioStep],
        after_step: &str,
    ) -> String {
        // Find the step after the insertion point
        let current_pos = existing_steps.iter().position(|s| s.order == after_step);

        match current_pos {
            Some(pos) if pos < existing_steps.len() - 1 => {
                // There's a step after this one, need to insert between
                // Parse the current step to get base number
                if let Ok(order) = StepOrder::parse(after_step) {
                    // If current step has no suffix, suggest letter suffix
                    if order.suffix.is_none() {
                        format!("{}a", order.base)
                    } else {
                        // Current step already has suffix, increment it
                        let mut new_suffix = order.suffix.unwrap();
                        if let Some(last_char) = new_suffix.chars().last() {
                            if last_char.is_ascii_lowercase() {
                                new_suffix.pop();
                                new_suffix
                                    .push(char::from_u32(last_char as u32 + 1).unwrap_or('a'));
                                format!("{}{}", order.base, new_suffix)
                            } else {
                                format!("{}a1", after_step)
                            }
                        } else {
                            format!("{}a", after_step)
                        }
                    }
                } else {
                    // Fallback: append 'a'
                    format!("{}a", after_step)
                }
            }
            _ => {
                // Last step or not found, suggest next numeric
                if let Ok(order) = StepOrder::parse(after_step) {
                    if order.suffix.is_none() {
                        (order.base + 1).to_string()
                    } else {
                        // Has suffix, suggest next base number
                        (order.base + 1).to_string()
                    }
                } else {
                    "1".to_string()
                }
            }
        }
    }

    /// Renumber steps starting from a given position, shifting by increment
    /// This is used when inserting a step in the middle forces renumbering
    pub fn renumber_steps_from(
        &self,
        scenario: &mut Scenario,
        from_step: &str,
        increment: i32,
    ) -> Result<()> {
        if let Ok(from_order) = StepOrder::parse(from_step) {
            // Only renumber numeric steps (main scenario steps)
            for step in &mut scenario.steps {
                if let Ok(step_order) = StepOrder::parse(&step.order) {
                    // Only renumber if it's a numeric step >= from_step and has no suffix
                    if step_order.suffix.is_none() && step_order.base >= from_order.base {
                        let new_order = (step_order.base as i32 + increment) as u32;
                        step.order = new_order.to_string();
                    }
                }
            }

            // Update repeat blocks to reflect new step numbers
            for block in &mut scenario.repeat_blocks {
                if let Ok(from_block_order) = StepOrder::parse(&block.from_step) {
                    if from_block_order.suffix.is_none() && from_block_order.base >= from_order.base
                    {
                        let new_order = (from_block_order.base as i32 + increment) as u32;
                        block.from_step = new_order.to_string();
                    }
                }
                if let Ok(to_block_order) = StepOrder::parse(&block.to_step) {
                    if to_block_order.suffix.is_none() && to_block_order.base >= from_order.base {
                        let new_order = (to_block_order.base as i32 + increment) as u32;
                        block.to_step = new_order.to_string();
                    }
                }
            }

            // Re-sort steps
            scenario
                .steps
                .sort_by(|a, b| StepOrder::compare(&a.order, &b.order));

            Ok(())
        } else {
            anyhow::bail!("Invalid step order format: {}", from_step);
        }
    }

    /// Create a scenario step with optional receiver
    pub fn create_scenario_step(&self, params: StepParams) -> ScenarioStep {
        if let Some(receiver) = params.receiver {
            ScenarioStep::with_receiver(params.order, params.actor, receiver, params.action)
        } else {
            ScenarioStep::new(params.order, params.actor, params.action)
        }
    }
}
