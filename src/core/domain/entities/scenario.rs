use super::{Condition, ScenarioReference, ScenarioStep, ScenarioType, Status};
use crate::core::domain::entities::scenario_step::StepOrder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a repeatable section of steps in a scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepeatBlock {
    /// Starting step of the repeat block (e.g., "2")
    pub from_step: String,
    /// Ending step of the repeat block (e.g., "8")
    pub to_step: String,
    /// Condition for repeating (e.g., "until authentication succeeds")
    pub condition: String,
}

impl RepeatBlock {
    pub fn new(from_step: String, to_step: String, condition: String) -> Self {
        Self {
            from_step,
            to_step,
            condition,
        }
    }

    /// Validate that the repeat block has a valid range
    pub fn validate(&self) -> Result<(), String> {
        if StepOrder::compare(&self.from_step, &self.to_step) != std::cmp::Ordering::Less {
            return Err(format!(
                "Invalid repeat block: from_step '{}' must be less than to_step '{}'",
                self.from_step, self.to_step
            ));
        }
        Ok(())
    }

    /// Check if this repeat block contains the given step
    pub fn contains_step(&self, step: &str) -> bool {
        StepOrder::compare(&self.from_step, step) != std::cmp::Ordering::Greater
            && StepOrder::compare(step, &self.to_step) != std::cmp::Ordering::Greater
    }

    /// Check if this repeat block overlaps with another (not properly nested)
    pub fn overlaps_with(&self, other: &RepeatBlock) -> bool {
        // Two blocks overlap if one starts inside the other but doesn't end inside it
        let self_contains_other_start = self.contains_step(&other.from_step);
        let self_contains_other_end = self.contains_step(&other.to_step);
        let other_contains_self_start = other.contains_step(&self.from_step);
        let other_contains_self_end = other.contains_step(&self.to_step);

        // If one fully contains the other, they don't overlap (properly nested)
        if (self_contains_other_start && self_contains_other_end)
            || (other_contains_self_start && other_contains_self_end)
        {
            return false;
        }

        // If they have any containment relationship, they overlap
        self_contains_other_start
            || self_contains_other_end
            || other_contains_self_start
            || other_contains_self_end
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario ID (e.g., "UC-AUTH-001-S01")
    pub id: String,

    pub title: String,
    pub description: String,
    pub scenario_type: ScenarioType,
    pub status: Status,

    /// Whether this is a main scenario (true) or an extension/alternate (false)
    #[serde(default = "default_is_main")]
    pub is_main: bool,

    /// Primary actor ID for this scenario (required) - used as default for steps
    pub primary_actor: String,

    /// Supporting actor IDs used in this scenario
    #[serde(default)]
    pub supporting_actors: Vec<String>,

    /// Persona this scenario is designed for (optional, in addition to primary_actor)
    #[serde(default)]
    pub persona: Option<String>,

    /// For extension scenarios: ID of the main scenario this extends
    #[serde(default)]
    pub extends_scenario_id: Option<String>,

    /// For extension scenarios: step in main scenario where this diverges
    #[serde(default)]
    pub extends_at_step: Option<String>,

    /// For extension scenarios: optional step in main scenario where this returns
    #[serde(default)]
    pub returns_at_step: Option<String>,

    /// Ordered steps in the scenario flow
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,

    /// Repeatable sections of steps
    #[serde(default)]
    pub repeat_blocks: Vec<RepeatBlock>,

    /// Scenario-specific preconditions (in addition to use case level, can reference use cases/scenarios)
    #[serde(default)]
    pub preconditions: Vec<Condition>,

    /// Scenario-specific postconditions (in addition to use case level, can reference use cases/scenarios)
    #[serde(default)]
    pub postconditions: Vec<Condition>,

    /// References to other scenarios or use cases
    #[serde(default)]
    pub references: Vec<ScenarioReference>,

    /// Flexible extra fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_is_main() -> bool {
    true
}

impl Scenario {
    pub fn new(
        id: String,
        title: String,
        description: String,
        scenario_type: ScenarioType,
        primary_actor: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            scenario_type,
            status: Status::Planned,
            is_main: true,
            primary_actor,
            supporting_actors: Vec::new(),
            persona: None,
            extends_scenario_id: None,
            extends_at_step: None,
            returns_at_step: None,
            steps: Vec::new(),
            repeat_blocks: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            references: Vec::new(),
            extra: HashMap::new(),
        }
    }

    /// Add a step to the scenario
    pub fn add_step(&mut self, step: ScenarioStep) {
        // Collect actors from this step into supporting_actors
        let acting = step.acting_actor().to_string();
        // Add acting_actor if it's not the primary actor
        if acting != self.primary_actor && !self.supporting_actors.contains(&acting) {
            self.supporting_actors.push(acting);
        }

        // Add receiving_actor if present and not already tracked
        if let Some(receiver) = step.receiving_actor() {
            let recv_str = receiver.to_string();
            if recv_str != self.primary_actor && !self.supporting_actors.contains(&recv_str) {
                self.supporting_actors.push(recv_str);
            }
        }

        self.steps.push(step);
        self.steps
            .sort_by(|a, b| StepOrder::compare(&a.order, &b.order));
    }

    /// Add a precondition
    pub fn add_precondition(&mut self, condition: Condition) {
        // Check for duplicates based on text and target
        if !self.preconditions.iter().any(|c| {
            c.text == condition.text
                && c.target_id == condition.target_id
                && c.target_type == condition.target_type
        }) {
            self.preconditions.push(condition);
        }
    }

    /// Add a postcondition
    pub fn add_postcondition(&mut self, condition: Condition) {
        // Check for duplicates based on text and target
        if !self.postconditions.iter().any(|c| {
            c.text == condition.text
                && c.target_id == condition.target_id
                && c.target_type == condition.target_type
        }) {
            self.postconditions.push(condition);
        }
    }

    /// Remove a precondition by text
    pub fn remove_precondition(&mut self, text: &str) {
        self.preconditions.retain(|c| c.text != text);
    }

    /// Remove a postcondition by text
    pub fn remove_postcondition(&mut self, text: &str) {
        self.postconditions.retain(|c| c.text != text);
    }

    /// Update scenario status
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    /// Remove a step by order
    pub fn remove_step(&mut self, step_order: &str) {
        self.steps.retain(|step| step.order != step_order);
    }

    /// Add a reference to another scenario or use case
    pub fn add_reference(&mut self, reference: ScenarioReference) {
        // Prevent duplicate references
        if !self.references.iter().any(|r| {
            r.ref_type == reference.ref_type
                && r.target_id == reference.target_id
                && r.relationship == reference.relationship
        }) {
            self.references.push(reference);
        }
    }

    /// Check if this scenario references another scenario
    pub fn references_scenario(&self, scenario_id: &str) -> bool {
        self.references.iter().any(|r| {
            matches!(r.ref_type, super::ReferenceType::Scenario) && r.target_id == scenario_id
        })
    }

    /// Check if this scenario depends on a use case
    pub fn depends_on_use_case(&self, use_case_id: &str) -> bool {
        self.references.iter().any(|r| {
            matches!(r.ref_type, super::ReferenceType::UseCase)
                && r.target_id == use_case_id
                && r.is_dependency()
        })
    }

    /// Remove a reference
    pub fn remove_reference(&mut self, target_id: &str, relationship: &str) {
        self.references
            .retain(|r| !(r.target_id == target_id && r.relationship == relationship));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scenario_creation() {
        let scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in with valid credentials".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        assert_eq!(scenario.id, "UC-AUTH-001-S01");
        assert_eq!(scenario.title, "Successful login");
        assert_eq!(
            scenario.description,
            "User successfully logs in with valid credentials"
        );
        assert_eq!(scenario.scenario_type, ScenarioType::HappyPath);
        assert_eq!(scenario.status, Status::Planned);
        assert!(scenario.is_main);
        assert_eq!(scenario.primary_actor, "user");
        assert!(scenario.supporting_actors.is_empty());
        assert!(scenario.persona.is_none());
        assert!(scenario.extends_scenario_id.is_none());
        assert!(scenario.steps.is_empty());
        assert!(scenario.repeat_blocks.is_empty());
        assert!(scenario.preconditions.is_empty());
        assert!(scenario.postconditions.is_empty());
        assert!(scenario.extra.is_empty());
    }

    #[test]
    fn test_scenario_add_step() {
        let mut scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        let step1 = ScenarioStep::new(
            "2".to_string(),
            "user".to_string(),
            "enters credentials".to_string(),
        );
        let step2 = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "navigates to login page".to_string(),
        );

        scenario.add_step(step1);
        scenario.add_step(step2);

        // Steps should be sorted by order
        assert_eq!(scenario.steps.len(), 2);
        assert_eq!(scenario.steps[0].order, "1");
        assert_eq!(scenario.steps[1].order, "2");
    }

    #[test]
    fn test_scenario_add_precondition() {
        let mut scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        scenario.add_precondition(Condition::new("User has account".to_string()));
        scenario.add_precondition(Condition::new("User has account".to_string())); // duplicate

        assert_eq!(scenario.preconditions.len(), 1);
        assert_eq!(scenario.preconditions[0].text, "User has account");
    }

    #[test]
    fn test_scenario_add_postcondition() {
        let mut scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        scenario.add_postcondition(Condition::new("User is authenticated".to_string()));
        assert_eq!(scenario.postconditions.len(), 1);
        assert_eq!(scenario.postconditions[0].text, "User is authenticated");
    }

    #[test]
    fn test_scenario_set_status() {
        let mut scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        assert_eq!(scenario.status, Status::Planned);

        scenario.set_status(Status::Implemented);
        assert_eq!(scenario.status, Status::Implemented);
    }

    #[test]
    fn test_scenario_serialization() {
        let mut scenario = Scenario::new(
            "UC-AUTH-001-S01".to_string(),
            "Successful login".to_string(),
            "User successfully logs in".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        scenario.add_step(ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "enters credentials".to_string(),
        ));
        scenario.add_precondition(Condition::new("Valid account".to_string()));
        scenario
            .extra
            .insert("test_field".to_string(), json!("test_value"));

        let serialized = serde_json::to_string(&scenario).unwrap();
        let deserialized: Scenario = serde_json::from_str(&serialized).unwrap();

        assert_eq!(scenario.id, deserialized.id);
        assert_eq!(scenario.title, deserialized.title);
        assert_eq!(scenario.scenario_type, deserialized.scenario_type);
        assert_eq!(scenario.steps.len(), deserialized.steps.len());
        assert_eq!(scenario.preconditions, deserialized.preconditions);
        assert_eq!(scenario.extra["test_field"], json!("test_value"));
    }

    #[test]
    fn test_repeat_block_validation() {
        let valid_block = RepeatBlock::new(
            "2".to_string(),
            "5".to_string(),
            "until success".to_string(),
        );
        assert!(valid_block.validate().is_ok());

        let invalid_block =
            RepeatBlock::new("5".to_string(), "2".to_string(), "invalid".to_string());
        assert!(invalid_block.validate().is_err());
    }

    #[test]
    fn test_repeat_block_contains_step() {
        let block = RepeatBlock::new("2".to_string(), "8".to_string(), "repeat".to_string());

        assert!(!block.contains_step("1"));
        assert!(block.contains_step("2"));
        assert!(block.contains_step("5"));
        assert!(block.contains_step("8"));
        assert!(!block.contains_step("9"));
    }

    #[test]
    fn test_repeat_block_overlap_detection() {
        let block1 = RepeatBlock::new("2".to_string(), "8".to_string(), "repeat".to_string());
        let block2 = RepeatBlock::new("5".to_string(), "6".to_string(), "nested".to_string());
        let block3 = RepeatBlock::new("4".to_string(), "10".to_string(), "overlapping".to_string());

        // block2 is nested within block1 - no overlap
        assert!(!block1.overlaps_with(&block2));
        assert!(!block2.overlaps_with(&block1));

        // block3 overlaps with block1 - starts inside but ends outside
        assert!(block1.overlaps_with(&block3));
        assert!(block3.overlaps_with(&block1));
    }
}
