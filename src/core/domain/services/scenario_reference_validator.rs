#[cfg(test)]
use crate::core::domain::entities::Actor;
use crate::core::domain::entities::{ReferenceType, UseCase};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Validates scenario references and prevents circular dependencies
pub struct ScenarioReferenceValidator;

impl ScenarioReferenceValidator {
    /// Validate that adding a reference won't create a circular dependency
    pub fn validate_no_circular_reference(
        use_case: &UseCase,
        from_scenario_id: &str,
        to_scenario_id: &str,
    ) -> Result<()> {
        // Build dependency graph
        let graph = Self::build_dependency_graph(use_case);

        // Check if adding this edge would create a cycle
        if Self::would_create_cycle(&graph, from_scenario_id, to_scenario_id) {
            anyhow::bail!(
                "Adding reference from {} to {} would create a circular dependency",
                from_scenario_id,
                to_scenario_id
            );
        }

        Ok(())
    }

    /// Build a dependency graph from scenario references
    fn build_dependency_graph(use_case: &UseCase) -> HashMap<String, Vec<String>> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for scenario in &use_case.scenarios {
            let dependencies: Vec<String> = scenario
                .references
                .iter()
                .filter(|r| matches!(r.ref_type, ReferenceType::Scenario))
                .map(|r| r.target_id.clone())
                .collect();

            graph.insert(scenario.id.clone(), dependencies);
        }

        graph
    }

    /// Check if adding an edge would create a cycle using DFS
    fn would_create_cycle(graph: &HashMap<String, Vec<String>>, from: &str, to: &str) -> bool {
        // If we can reach 'from' starting from 'to', adding from->to creates a cycle
        Self::can_reach(graph, to, from)
    }

    /// Check if we can reach 'target' starting from 'start' using DFS
    fn can_reach(graph: &HashMap<String, Vec<String>>, start: &str, target: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start.to_string()];

        while let Some(current) = stack.pop() {
            if current == target {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::entities::{Scenario, ScenarioReference, ScenarioType};

    fn create_test_use_case_with_scenarios() -> UseCase {
        let mut uc = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        // Add three scenarios
        uc.add_scenario(Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Scenario 1".to_string(),
            "First scenario".to_string(),
            ScenarioType::HappyPath,
            Actor::User,
        ));

        uc.add_scenario(Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Scenario 2".to_string(),
            "Second scenario".to_string(),
            ScenarioType::AlternativeFlow,
            Actor::User,
        ));

        uc.add_scenario(Scenario::new(
            "UC-TEST-001-S03".to_string(),
            "Scenario 3".to_string(),
            "Third scenario".to_string(),
            ScenarioType::ExceptionFlow,
            Actor::System,
        ));

        uc
    }

    #[test]
    fn test_no_cycle_with_linear_dependencies() {
        let mut uc = create_test_use_case_with_scenarios();

        // S01 -> S02 is fine
        uc.scenarios[0].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            "UC-TEST-001-S02".to_string(),
            "extends".to_string(),
        ));

        // S02 -> S03 is fine
        let result = ScenarioReferenceValidator::validate_no_circular_reference(
            &uc,
            "UC-TEST-001-S02",
            "UC-TEST-001-S03",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_direct_cycle_detected() {
        let mut uc = create_test_use_case_with_scenarios();

        // S01 -> S02
        uc.scenarios[0].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            "UC-TEST-001-S02".to_string(),
            "extends".to_string(),
        ));

        // S02 -> S01 would create a cycle
        let result = ScenarioReferenceValidator::validate_no_circular_reference(
            &uc,
            "UC-TEST-001-S02",
            "UC-TEST-001-S01",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_indirect_cycle_detected() {
        let mut uc = create_test_use_case_with_scenarios();

        // S01 -> S02
        uc.scenarios[0].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            "UC-TEST-001-S02".to_string(),
            "extends".to_string(),
        ));

        // S02 -> S03
        uc.scenarios[1].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            "UC-TEST-001-S03".to_string(),
            "extends".to_string(),
        ));

        // S03 -> S01 would create an indirect cycle
        let result = ScenarioReferenceValidator::validate_no_circular_reference(
            &uc,
            "UC-TEST-001-S03",
            "UC-TEST-001-S01",
        );

        assert!(result.is_err());
    }
}
