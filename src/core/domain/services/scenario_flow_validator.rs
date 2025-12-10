#[cfg(test)]
use crate::core::domain::RepeatBlock;
use crate::core::domain::{Scenario, StepOrder, UseCase};
use anyhow::{Context, Result};

/// Validates scenario flow structures including extensions, step ordering, and repeat blocks
pub struct ScenarioFlowValidator;

impl ScenarioFlowValidator {
    /// Validate that a scenario's step orders are properly formatted
    /// Main scenarios should only have numeric steps, extensions can have letter suffixes
    pub fn validate_step_orders(scenario: &Scenario) -> Result<()> {
        for step in &scenario.steps {
            // Validate step order format
            StepOrder::parse(&step.order).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid step order '{}' in scenario '{}': {}",
                    step.order,
                    scenario.id,
                    e
                )
            })?;

            // Main scenarios should only have numeric steps (no suffixes)
            if scenario.is_main && !StepOrder::is_main_step(&step.order) {
                anyhow::bail!(
                    "Main scenario '{}' cannot have extension step '{}'. Only numeric steps are allowed in main scenarios.",
                    scenario.id,
                    step.order
                );
            }

            // Extension scenarios should have letter suffixes within their divergence range
            if !scenario.is_main && scenario.extends_at_step.is_some() {
                // Extension steps should use letter suffixes
                if StepOrder::is_main_step(&step.order) {
                    anyhow::bail!(
                        "Extension scenario '{}' should use letter suffixes for steps (e.g., '3a', '3b'). Found numeric step '{}'",
                        scenario.id,
                        step.order
                    );
                }
            }
        }

        Ok(())
    }

    /// Validate that an extension scenario properly references a main scenario
    pub fn validate_extension(scenario: &Scenario, use_case: &UseCase) -> Result<()> {
        if scenario.is_main {
            // Main scenarios should not have extension fields set
            if scenario.extends_scenario_id.is_some() {
                anyhow::bail!(
                    "Main scenario '{}' should not have extends_scenario_id set",
                    scenario.id
                );
            }
            return Ok(());
        }

        // Extension scenarios must reference a parent
        let parent_id = scenario.extends_scenario_id.as_ref().context(format!(
            "Extension scenario '{}' must have extends_scenario_id set",
            scenario.id
        ))?;

        // Find parent scenario
        let parent = use_case
            .scenarios
            .iter()
            .find(|s| &s.id == parent_id)
            .context(format!(
                "Parent scenario '{}' not found for extension '{}'",
                parent_id, scenario.id
            ))?;

        // Parent must be a main scenario
        if !parent.is_main {
            anyhow::bail!(
                "Extension '{}' references non-main scenario '{}'. Extensions can only extend main scenarios.",
                scenario.id,
                parent_id
            );
        }

        // Validate extends_at_step exists in parent
        let extends_at = scenario.extends_at_step.as_ref().context(format!(
            "Extension scenario '{}' must have extends_at_step set",
            scenario.id
        ))?;

        if !parent.steps.iter().any(|s| &s.order == extends_at) {
            anyhow::bail!(
                "Extension '{}' diverges at step '{}' which does not exist in parent scenario '{}'",
                scenario.id,
                extends_at,
                parent_id
            );
        }

        // Validate returns_at_step if specified
        if let Some(ref returns_at) = scenario.returns_at_step {
            if !parent.steps.iter().any(|s| &s.order == returns_at) {
                anyhow::bail!(
                    "Extension '{}' returns at step '{}' which does not exist in parent scenario '{}'",
                    scenario.id,
                    returns_at,
                    parent_id
                );
            }

            // Allow return to any step, including before divergence (loop-back)
            // This enables validation retry patterns where errors return to earlier steps
        }

        // Validate extension steps fall within divergence-return range
        if let Some(ref returns_at) = scenario.returns_at_step {
            for step in &scenario.steps {
                // Extension steps should be within the divergence range
                // Parse base number from step order
                if let Ok(step_order) = StepOrder::parse(&step.order) {
                    if let Ok(extends_order) = StepOrder::parse(extends_at) {
                        if let Ok(returns_order) = StepOrder::parse(returns_at) {
                            // For loop-back (return < divergence), extension steps should be after divergence
                            // For normal flow (return >= divergence), steps should be between divergence and return
                            if returns_order.base < extends_order.base {
                                // Loop-back case: steps must be after or at divergence point
                                if step_order.base < extends_order.base {
                                    anyhow::bail!(
                                        "Extension '{}' step '{}' is before divergence point '{}' (loop-back flow)",
                                        scenario.id,
                                        step.order,
                                        extends_at
                                    );
                                }
                            } else {
                                // Normal case: steps should be between extends and returns
                                if step_order.base < extends_order.base
                                    || step_order.base > returns_order.base
                                {
                                    anyhow::bail!(
                                        "Extension '{}' step '{}' is outside the divergence-return range ({}->{})",
                                        scenario.id,
                                        step.order,
                                        extends_at,
                                        returns_at
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate repeat blocks for proper nesting and non-overlapping ranges
    pub fn validate_repeat_blocks(scenario: &Scenario) -> Result<()> {
        // Validate each block individually
        for block in &scenario.repeat_blocks {
            block.validate().map_err(|e| {
                anyhow::anyhow!("Invalid repeat block in scenario '{}': {}", scenario.id, e)
            })?;

            // Validate that steps in the range exist
            let from_exists = scenario.steps.iter().any(|s| s.order == block.from_step);
            let to_exists = scenario.steps.iter().any(|s| s.order == block.to_step);

            if !from_exists {
                anyhow::bail!(
                    "Repeat block in scenario '{}' references non-existent from_step '{}'",
                    scenario.id,
                    block.from_step
                );
            }
            if !to_exists {
                anyhow::bail!(
                    "Repeat block in scenario '{}' references non-existent to_step '{}'",
                    scenario.id,
                    block.to_step
                );
            }
        }

        // Validate that blocks don't overlap (but nesting is allowed)
        for (i, block1) in scenario.repeat_blocks.iter().enumerate() {
            for block2 in scenario.repeat_blocks.iter().skip(i + 1) {
                if block1.overlaps_with(block2) {
                    anyhow::bail!(
                        "Repeat blocks in scenario '{}' overlap: ({}->{}) and ({}->{}). Blocks must be properly nested or separate.",
                        scenario.id,
                        block1.from_step,
                        block1.to_step,
                        block2.from_step,
                        block2.to_step
                    );
                }
            }
        }

        Ok(())
    }

    /// Validate that there are no circular extension chains
    /// Extensions can only extend main scenarios, not other extensions
    pub fn validate_no_circular_extensions(use_case: &UseCase) -> Result<()> {
        for scenario in &use_case.scenarios {
            if !scenario.is_main {
                if let Some(ref parent_id) = scenario.extends_scenario_id {
                    // Find parent
                    if let Some(parent) = use_case.scenarios.iter().find(|s| &s.id == parent_id) {
                        // Parent must be main (already checked in validate_extension, but double-check)
                        if !parent.is_main {
                            anyhow::bail!(
                                "Circular or multi-level extension detected: '{}' extends non-main scenario '{}'",
                                scenario.id,
                                parent_id
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate all aspects of a scenario's flow structure
    pub fn validate_scenario_flow(scenario: &Scenario, use_case: &UseCase) -> Result<()> {
        Self::validate_step_orders(scenario)?;
        Self::validate_extension(scenario, use_case)?;
        Self::validate_repeat_blocks(scenario)?;
        Ok(())
    }

    /// Validate all scenarios in a use case
    pub fn validate_use_case_scenarios(use_case: &UseCase) -> Result<()> {
        for scenario in &use_case.scenarios {
            Self::validate_scenario_flow(scenario, use_case)?;
        }
        Self::validate_no_circular_extensions(use_case)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{Metadata, Priority, ScenarioStep, ScenarioType, Status};

    fn create_test_use_case() -> UseCase {
        UseCase {
            id: "UC-TEST-001".to_string(),
            title: "Test Use Case".to_string(),
            category: "Test".to_string(),
            category_abbreviation: "TES".to_string(),
            description: "Test".to_string(),
            priority: Priority::Medium,
            status: Status::Planned,
            metadata: Metadata::new(),
            views: vec![],
            preconditions: vec![],
            postconditions: vec![],
            use_case_references: vec![],
            scenarios: vec![],
            methodology_fields: std::collections::HashMap::new(),
            extra: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_validate_main_scenario_numeric_steps_only() {
        let mut scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Main Scenario".to_string(),
            "Test".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        scenario.add_step(ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "desc".to_string(),
        ));
        scenario.add_step(ScenarioStep::new(
            "2".to_string(),
            "system".to_string(),
            "desc".to_string(),
        ));

        assert!(ScenarioFlowValidator::validate_step_orders(&scenario).is_ok());

        // Add a step with letter suffix - should fail for main scenario
        scenario.add_step(ScenarioStep::new(
            "3a".to_string(),
            "user".to_string(),
            "desc".to_string(),
        ));

        assert!(ScenarioFlowValidator::validate_step_orders(&scenario).is_err());
    }

    #[test]
    fn test_validate_extension_scenario() {
        let mut use_case = create_test_use_case();

        // Create main scenario
        let mut main_scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Main".to_string(),
            "Test".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );
        main_scenario.add_step(ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "desc".to_string(),
        ));
        main_scenario.add_step(ScenarioStep::new(
            "2".to_string(),
            "system".to_string(),
            "desc".to_string(),
        ));
        main_scenario.add_step(ScenarioStep::new(
            "3".to_string(),
            "user".to_string(),
            "desc".to_string(),
        ));

        use_case.add_scenario(main_scenario);

        // Create valid extension
        let mut extension = Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Extension".to_string(),
            "Test".to_string(),
            ScenarioType::Extension,
            "user".to_string(),
        );
        extension.is_main = false;
        extension.extends_scenario_id = Some("UC-TEST-001-S01".to_string());
        extension.extends_at_step = Some("2".to_string());
        extension.returns_at_step = Some("3".to_string());

        assert!(ScenarioFlowValidator::validate_extension(&extension, &use_case).is_ok());

        // Test loop-back: return before divergence is now allowed
        extension.returns_at_step = Some("1".to_string());
        assert!(ScenarioFlowValidator::validate_extension(&extension, &use_case).is_ok());
    }

    #[test]
    fn test_validate_loop_back_extension() {
        let mut use_case = create_test_use_case();

        // Create main scenario with more steps
        let mut main_scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Main".to_string(),
            "Test".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );
        for i in 1..=5 {
            main_scenario.add_step(ScenarioStep::new(
                i.to_string(),
                if i % 2 == 0 {
                    "system".to_string()
                } else {
                    "user".to_string()
                },
                format!("Step {}", i),
            ));
        }
        use_case.add_scenario(main_scenario);

        // Create loop-back extension (diverges at 4, returns to 2)
        let mut loop_extension = Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Loop Back".to_string(),
            "Test loop-back".to_string(),
            ScenarioType::ExceptionFlow,
            "user".to_string(),
        );
        loop_extension.is_main = false;
        loop_extension.extends_scenario_id = Some("UC-TEST-001-S01".to_string());
        loop_extension.extends_at_step = Some("4".to_string());
        loop_extension.returns_at_step = Some("2".to_string());

        // Add extension steps (should be after divergence point)
        loop_extension.add_step(ScenarioStep::new(
            "4a".to_string(),
            "system".to_string(),
            "Error handling".to_string(),
        ));
        loop_extension.add_step(ScenarioStep::new(
            "4b".to_string(),
            "system".to_string(),
            "Show error message".to_string(),
        ));

        assert!(ScenarioFlowValidator::validate_extension(&loop_extension, &use_case).is_ok());

        // Test invalid: extension step before divergence point in loop-back
        loop_extension.add_step(ScenarioStep::new(
            "3a".to_string(),
            "system".to_string(),
            "Invalid step".to_string(),
        ));
        assert!(ScenarioFlowValidator::validate_extension(&loop_extension, &use_case).is_err());
    }

    #[test]
    fn test_validate_repeat_blocks() {
        let mut scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        for i in 1..=10 {
            scenario.add_step(ScenarioStep::new(
                i.to_string(),
                "user".to_string(),
                "desc".to_string(),
            ));
        }

        // Valid nested blocks: 2-8 contains 5-6
        scenario.repeat_blocks.push(RepeatBlock::new(
            "2".to_string(),
            "8".to_string(),
            "outer".to_string(),
        ));
        scenario.repeat_blocks.push(RepeatBlock::new(
            "5".to_string(),
            "6".to_string(),
            "inner".to_string(),
        ));

        assert!(ScenarioFlowValidator::validate_repeat_blocks(&scenario).is_ok());

        // Invalid overlapping blocks: 2-5 and 4-7
        scenario.repeat_blocks.clear();
        scenario.repeat_blocks.push(RepeatBlock::new(
            "2".to_string(),
            "5".to_string(),
            "first".to_string(),
        ));
        scenario.repeat_blocks.push(RepeatBlock::new(
            "4".to_string(),
            "7".to_string(),
            "second".to_string(),
        ));

        assert!(ScenarioFlowValidator::validate_repeat_blocks(&scenario).is_err());
    }
}
