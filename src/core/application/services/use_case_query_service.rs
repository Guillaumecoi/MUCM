use crate::core::utils::suggest_alternatives;
use crate::core::UseCase;
use anyhow::Result;

/// Service for querying use case data
///
/// This service provides read-only query operations over the use case collection.
/// It handles filtering, searching, and finding use cases by various criteria.
pub struct UseCaseQueryService<'a> {
    use_cases: &'a [UseCase],
}

impl<'a> UseCaseQueryService<'a> {
    pub fn new(use_cases: &'a [UseCase]) -> Self {
        Self { use_cases }
    }

    /// Find scenario ID by its title within a use case
    pub fn find_scenario_id_by_title(
        &self,
        use_case_id: &str,
        scenario_title: &str,
    ) -> Result<String> {
        let use_case = self.find_use_case_by_id(use_case_id)?;

        use_case
            .scenarios
            .iter()
            .find(|s| s.title == scenario_title)
            .map(|s| s.id.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Scenario with title '{}' not found in use case '{}'",
                    scenario_title,
                    use_case_id
                )
            })
    }

    /// Get all use case info that uses a specific persona
    /// Returns a list of tuples (use_case_id, title, scenario_count) where at least one scenario uses the given persona
    pub fn get_use_cases_for_persona(
        &self,
        persona_id: &str,
    ) -> Result<Vec<(String, String, usize)>> {
        let mut matching_use_cases = Vec::new();

        // Scan all loaded use cases for scenarios that use this persona
        for use_case in self.use_cases {
            let scenario_count = use_case
                .scenarios
                .iter()
                .filter(|scenario| scenario.persona.as_deref() == Some(persona_id))
                .count();

            if scenario_count > 0 {
                matching_use_cases.push((
                    use_case.id.clone(),
                    use_case.title.clone(),
                    scenario_count,
                ));
            }
        }

        Ok(matching_use_cases)
    }

    /// Helper to find a use case index by ID
    pub fn find_use_case_index(&self, use_case_id: &str) -> Result<usize> {
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

    /// Helper to find a use case by ID (immutable)
    pub fn find_use_case_by_id(&self, use_case_id: &str) -> Result<&UseCase> {
        let index = self.find_use_case_index(use_case_id)?;
        Ok(&self.use_cases[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::Metadata;

    fn create_test_use_case(id: &str, title: &str) -> UseCase {
        UseCase {
            id: id.to_string(),
            title: title.to_string(),
            category: "test".to_string(),
            category_abbreviation: "TES".to_string(),
            description: "Test description".to_string(),
            priority: "Medium".parse().unwrap(),
            status: crate::core::domain::Status::Planned,
            metadata: Metadata::default(),
            views: vec![],
            preconditions: vec![],
            postconditions: vec![],
            methodology_fields: std::collections::HashMap::new(),
            use_case_references: vec![],
            scenarios: vec![],
            extra: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_find_use_case_index() {
        let use_cases = vec![
            create_test_use_case("UC-001", "Test 1"),
            create_test_use_case("UC-002", "Test 2"),
        ];
        let service = UseCaseQueryService::new(&use_cases);

        assert_eq!(service.find_use_case_index("UC-001").unwrap(), 0);
        assert_eq!(service.find_use_case_index("UC-002").unwrap(), 1);
        assert!(service.find_use_case_index("UC-999").is_err());
    }

    #[test]
    fn test_find_use_case_by_id() {
        let use_cases = vec![
            create_test_use_case("UC-001", "Test 1"),
            create_test_use_case("UC-002", "Test 2"),
        ];
        let service = UseCaseQueryService::new(&use_cases);

        let uc = service.find_use_case_by_id("UC-001").unwrap();
        assert_eq!(uc.id, "UC-001");
        assert_eq!(uc.title, "Test 1");
    }
}
