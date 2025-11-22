use crate::core::domain::{Actor, Scenario, ScenarioStep, ScenarioType, UseCase};

/// Handles scenario creation and management
pub struct ScenarioCreator;

impl ScenarioCreator {
    pub fn new() -> Self {
        Self
    }

    /// Create a new scenario for a use case
    pub fn create_scenario(
        &self,
        use_case: &UseCase,
        title: String,
        scenario_type: ScenarioType,
        description: Option<String>,
        primary_actor: Actor,
        preconditions: Vec<String>,
        postconditions: Vec<String>,
        _actors: Vec<String>,
    ) -> Scenario {
        let scenario_id = use_case.next_scenario_id();

        let mut scenario = Scenario::new(
            scenario_id,
            title,
            description.unwrap_or_default(),
            scenario_type,
            primary_actor,
        );

        // Add preconditions and postconditions
        for precondition in preconditions {
            scenario.add_precondition(precondition.into());
        }
        for postcondition in postconditions {
            scenario.add_postcondition(postcondition.into());
        }

        scenario
    }

    /// Create a scenario step with optional receiver
    pub fn create_scenario_step(
        &self,
        order: String,
        actor: String,
        receiver: Option<String>,
        action: String,
        expected_result: Option<String>,
    ) -> ScenarioStep {
        let actor_enum: Actor = actor.into(); // Convert String to Actor using From<String>
        let receiver_enum: Option<Actor> = receiver.map(|r| r.into());

        let description = expected_result.unwrap_or_else(|| {
            if let Some(ref recv) = receiver_enum {
                format!("{} {} to {}", actor_enum.name(), action, recv.name())
            } else {
                format!("{} {}", actor_enum.name(), action)
            }
        });

        let mut step = ScenarioStep::new(order, actor_enum, action, description);
        if let Some(recv) = receiver_enum {
            step.set_receiver(recv);
        }
        step
    }
}
