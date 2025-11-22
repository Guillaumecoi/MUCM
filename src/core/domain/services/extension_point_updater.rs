use crate::core::domain::{Scenario, StepOrder, UseCase};
use anyhow::Result;
use std::collections::HashMap;

/// Service for automatically updating extension scenario references when main scenarios change
pub struct ExtensionPointUpdater;

impl ExtensionPointUpdater {
    /// Update all extension scenarios when a main scenario's steps are reordered
    /// This handles the case where step numbers change (e.g., step 3 becomes step 4)
    /// and updates all extensions that reference those steps
    pub fn update_after_reorder(
        main_scenario: &Scenario,
        step_mapping: &HashMap<String, String>,
        use_case: &mut UseCase,
    ) -> Result<()> {
        if !main_scenario.is_main {
            anyhow::bail!(
                "Cannot update extension points for non-main scenario '{}'",
                main_scenario.id
            );
        }

        // Find all extensions of this main scenario
        for scenario in &mut use_case.scenarios {
            if let Some(ref parent_id) = scenario.extends_scenario_id {
                if parent_id == &main_scenario.id {
                    // Update extends_at_step
                    if let Some(ref extends_at) = scenario.extends_at_step {
                        if let Some(new_step) = step_mapping.get(extends_at) {
                            scenario.extends_at_step = Some(new_step.clone());
                        }
                    }

                    // Update returns_at_step
                    if let Some(ref returns_at) = scenario.returns_at_step {
                        if let Some(new_step) = step_mapping.get(returns_at) {
                            scenario.returns_at_step = Some(new_step.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update extension references when a step is inserted into a main scenario
    /// Steps at or after the insertion point need to be adjusted
    pub fn update_after_insert(
        main_scenario: &Scenario,
        inserted_at: &str,
        use_case: &mut UseCase,
    ) -> Result<()> {
        if !main_scenario.is_main {
            anyhow::bail!(
                "Cannot update extension points for non-main scenario '{}'",
                main_scenario.id
            );
        }

        let inserted_order = StepOrder::parse(inserted_at).map_err(|e| {
            anyhow::anyhow!("Invalid step order '{}': {}", inserted_at, e)
        })?;

        // Find all extensions of this main scenario
        for scenario in &mut use_case.scenarios {
            if let Some(ref parent_id) = scenario.extends_scenario_id {
                if parent_id == &main_scenario.id {
                    // Update extends_at_step if it's at or after the insertion
                    if let Some(ref extends_at) = scenario.extends_at_step {
                        if let Ok(extends_order) = StepOrder::parse(extends_at) {
                            if extends_order.base >= inserted_order.base {
                                let new_base = extends_order.base + 1;
                                scenario.extends_at_step = Some(new_base.to_string());
                            }
                        }
                    }

                    // Update returns_at_step if it's at or after the insertion
                    if let Some(ref returns_at) = scenario.returns_at_step {
                        if let Ok(returns_order) = StepOrder::parse(returns_at) {
                            if returns_order.base >= inserted_order.base {
                                let new_base = returns_order.base + 1;
                                scenario.returns_at_step = Some(new_base.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update extension references when a step is deleted from a main scenario
    /// Returns a list of extensions that became invalid (referenced deleted step)
    pub fn update_after_delete(
        main_scenario: &Scenario,
        deleted_step: &str,
        use_case: &mut UseCase,
    ) -> Result<Vec<String>> {
        if !main_scenario.is_main {
            anyhow::bail!(
                "Cannot update extension points for non-main scenario '{}'",
                main_scenario.id
            );
        }

        let deleted_order = StepOrder::parse(deleted_step).map_err(|e| {
            anyhow::anyhow!("Invalid step order '{}': {}", deleted_step, e)
        })?;

        let mut invalid_extensions = Vec::new();

        // Find all extensions of this main scenario
        for scenario in &mut use_case.scenarios {
            if let Some(ref parent_id) = scenario.extends_scenario_id {
                if parent_id == &main_scenario.id {
                    let mut is_invalid = false;

                    // Check if extension references the deleted step
                    if let Some(ref extends_at) = scenario.extends_at_step {
                        if extends_at == deleted_step {
                            invalid_extensions.push(scenario.id.clone());
                            is_invalid = true;
                        }
                    }

                    if let Some(ref returns_at) = scenario.returns_at_step {
                        if returns_at == deleted_step {
                            if !is_invalid {
                                invalid_extensions.push(scenario.id.clone());
                            }
                            is_invalid = true;
                        }
                    }

                    // If not invalid, update step numbers after deletion point
                    if !is_invalid {
                        if let Some(ref extends_at) = scenario.extends_at_step {
                            if let Ok(extends_order) = StepOrder::parse(extends_at) {
                                if extends_order.base > deleted_order.base {
                                    let new_base = extends_order.base - 1;
                                    scenario.extends_at_step = Some(new_base.to_string());
                                }
                            }
                        }

                        if let Some(ref returns_at) = scenario.returns_at_step {
                            if let Ok(returns_order) = StepOrder::parse(returns_at) {
                                if returns_order.base > deleted_order.base {
                                    let new_base = returns_order.base - 1;
                                    scenario.returns_at_step = Some(new_base.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(invalid_extensions)
    }

    /// Validate that a step order change won't break extension references
    /// Returns list of affected extensions that would need manual review
    pub fn validate_step_change(
        main_scenario: &Scenario,
        old_order: &str,
        new_order: &str,
        use_case: &UseCase,
    ) -> Result<Vec<String>> {
        if !main_scenario.is_main {
            anyhow::bail!(
                "Cannot validate step changes for non-main scenario '{}'",
                main_scenario.id
            );
        }

        let old_step = StepOrder::parse(old_order).map_err(|e| {
            anyhow::anyhow!("Invalid old step order '{}': {}", old_order, e)
        })?;

        let new_step = StepOrder::parse(new_order).map_err(|e| {
            anyhow::anyhow!("Invalid new step order '{}': {}", new_order, e)
        })?;

        let mut affected_extensions = Vec::new();

        // If the step order is just being renumbered (same position), extensions are fine
        if old_step.base == new_step.base {
            return Ok(affected_extensions);
        }

        // Find extensions that reference this step
        for scenario in &use_case.scenarios {
            if let Some(ref parent_id) = scenario.extends_scenario_id {
                if parent_id == &main_scenario.id {
                    let mut is_affected = false;

                    if let Some(ref extends_at) = scenario.extends_at_step {
                        if extends_at == old_order {
                            affected_extensions.push(format!(
                                "{} (extends at step {})",
                                scenario.id, extends_at
                            ));
                            is_affected = true;
                        }
                    }

                    if let Some(ref returns_at) = scenario.returns_at_step {
                        if returns_at == old_order && !is_affected {
                            affected_extensions.push(format!(
                                "{} (returns at step {})",
                                scenario.id, returns_at
                            ));
                        }
                    }
                }
            }
        }

        Ok(affected_extensions)
    }

    /// Recalculate all extension points after bulk changes to a main scenario
    /// This is useful after operations like renumbering or restructuring
    pub fn recalculate_all_extensions(
        main_scenario: &Scenario,
        use_case: &mut UseCase,
    ) -> Result<Vec<String>> {
        if !main_scenario.is_main {
            anyhow::bail!(
                "Cannot recalculate extensions for non-main scenario '{}'",
                main_scenario.id
            );
        }

        let mut invalid_extensions = Vec::new();

        // Find all extensions and validate their references
        for scenario in &mut use_case.scenarios {
            if let Some(ref parent_id) = scenario.extends_scenario_id {
                if parent_id == &main_scenario.id {
                    // Validate extends_at_step exists
                    if let Some(ref extends_at) = scenario.extends_at_step {
                        if !main_scenario.steps.iter().any(|s| &s.order == extends_at) {
                            invalid_extensions.push(format!(
                                "{} (extends at missing step {})",
                                scenario.id, extends_at
                            ));
                        }
                    }

                    // Validate returns_at_step exists
                    if let Some(ref returns_at) = scenario.returns_at_step {
                        if !main_scenario.steps.iter().any(|s| &s.order == returns_at) {
                            invalid_extensions.push(format!(
                                "{} (returns at missing step {})",
                                scenario.id, returns_at
                            ));
                        }
                    }
                }
            }
        }

        Ok(invalid_extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{Actor, Metadata, Priority, ScenarioStep, ScenarioType};

    fn create_test_use_case_with_extensions() -> UseCase {
        let mut use_case = UseCase {
            id: "UC-TEST-001".to_string(),
            title: "Test Use Case".to_string(),
            category: "Test".to_string(),
            description: "Test".to_string(),
            priority: Priority::Medium,
            metadata: Metadata::new(),
            views: vec![],
            preconditions: vec![],
            postconditions: vec![],
            use_case_references: vec![],
            scenarios: vec![],
            methodology_fields: std::collections::HashMap::new(),
            extra: std::collections::HashMap::new(),
        };

        // Create main scenario
        let mut main_scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Main".to_string(),
            "Test".to_string(),
            ScenarioType::HappyPath,
            Actor::User,
        );
        for i in 1..=5 {
            main_scenario.add_step(ScenarioStep::new(
                i.to_string(),
                Actor::User,
                "action".to_string(),
                format!("Step {}", i),
            ));
        }

        // Create extension scenario diverging at step 2, returning at step 4
        let mut extension = Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Extension".to_string(),
            "Test".to_string(),
            ScenarioType::Extension,
            Actor::User,
        );
        extension.is_main = false;
        extension.extends_scenario_id = Some("UC-TEST-001-S01".to_string());
        extension.extends_at_step = Some("2".to_string());
        extension.returns_at_step = Some("4".to_string());

        use_case.add_scenario(main_scenario);
        use_case.add_scenario(extension);

        use_case
    }

    #[test]
    fn test_update_after_reorder() {
        let mut use_case = create_test_use_case_with_extensions();
        let main_scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S01")
            .unwrap()
            .clone();

        // Simulate reordering: step 2->3, step 3->2, step 4->5, step 5->4
        let mut mapping = HashMap::new();
        mapping.insert("2".to_string(), "3".to_string());
        mapping.insert("3".to_string(), "2".to_string());
        mapping.insert("4".to_string(), "5".to_string());
        mapping.insert("5".to_string(), "4".to_string());

        ExtensionPointUpdater::update_after_reorder(&main_scenario, &mapping, &mut use_case)
            .unwrap();

        let extension = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S02")
            .unwrap();

        // Extension should now point to step 3 and 5
        assert_eq!(extension.extends_at_step, Some("3".to_string()));
        assert_eq!(extension.returns_at_step, Some("5".to_string()));
    }

    #[test]
    fn test_update_after_insert() {
        let mut use_case = create_test_use_case_with_extensions();
        let main_scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S01")
            .unwrap()
            .clone();

        // Insert step at position 2 (between 1 and 2)
        ExtensionPointUpdater::update_after_insert(&main_scenario, "2", &mut use_case).unwrap();

        let extension = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S02")
            .unwrap();

        // Extension should now point to step 3 and 5 (shifted by 1)
        assert_eq!(extension.extends_at_step, Some("3".to_string()));
        assert_eq!(extension.returns_at_step, Some("5".to_string()));
    }

    #[test]
    fn test_update_after_delete_valid() {
        let mut use_case = create_test_use_case_with_extensions();
        let main_scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S01")
            .unwrap()
            .clone();

        // Delete step 1 (before extension point)
        let invalid =
            ExtensionPointUpdater::update_after_delete(&main_scenario, "1", &mut use_case)
                .unwrap();

        assert!(invalid.is_empty());

        let extension = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S02")
            .unwrap();

        // Extension should now point to step 1 and 3 (shifted by -1)
        assert_eq!(extension.extends_at_step, Some("1".to_string()));
        assert_eq!(extension.returns_at_step, Some("3".to_string()));
    }

    #[test]
    fn test_update_after_delete_invalid() {
        let mut use_case = create_test_use_case_with_extensions();
        let main_scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S01")
            .unwrap()
            .clone();

        // Delete step 2 (extension divergence point)
        let invalid =
            ExtensionPointUpdater::update_after_delete(&main_scenario, "2", &mut use_case)
                .unwrap();

        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], "UC-TEST-001-S02");
    }

    #[test]
    fn test_validate_step_change() {
        let use_case = create_test_use_case_with_extensions();
        let main_scenario = use_case
            .scenarios
            .iter()
            .find(|s| s.id == "UC-TEST-001-S01")
            .unwrap();

        // Changing step 2 should affect the extension
        let affected = ExtensionPointUpdater::validate_step_change(
            main_scenario,
            "2",
            "6",
            &use_case,
        )
        .unwrap();

        assert_eq!(affected.len(), 1);
        assert!(affected[0].contains("UC-TEST-001-S02"));
    }
}
