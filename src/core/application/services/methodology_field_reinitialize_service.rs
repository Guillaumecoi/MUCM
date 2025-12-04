use crate::core::application::MethodologyFieldCollector;
use crate::core::{UseCase, UseCaseRepository};
use anyhow::Result;

/// Type alias for reinitialize result: (updated_count, total_checked, details)
/// Details is a Vec of (use_case_id, methodology, added_fields)
pub type ReinitializeResult = (usize, usize, Vec<(String, String, Vec<String>)>);

/// Service for reinitializing missing methodology fields
pub struct MethodologyFieldReinitializeService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a mut Vec<UseCase>,
}

impl<'a> MethodologyFieldReinitializeService<'a> {
    pub fn new(repository: &'a dyn UseCaseRepository, use_cases: &'a mut Vec<UseCase>) -> Self {
        Self {
            repository,
            use_cases,
        }
    }

    /// Reinitialize missing methodology fields in use cases
    ///
    /// Scans each use case's enabled methodology views and ensures all fields
    /// defined in those methodologies are present in the TOML file.
    /// Missing fields are initialized with empty values based on their type.
    /// Existing field values are never overwritten.
    pub fn reinitialize_methodology_fields(
        &mut self,
        use_case_id: Option<String>,
        dry_run: bool,
    ) -> Result<ReinitializeResult> {
        let collector = MethodologyFieldCollector::new()?;
        let mut updated_count = 0;
        let mut total_checked = 0;
        let mut details = Vec::new();

        let use_case_ids: Vec<String> = if let Some(id) = use_case_id {
            if !self.use_cases.iter().any(|uc| uc.id == id) {
                anyhow::bail!("Use case '{}' not found", id);
            }
            vec![id]
        } else {
            self.use_cases.iter().map(|uc| uc.id.clone()).collect()
        };

        for uc_id in use_case_ids {
            total_checked += 1;
            let index = self.find_use_case_index(&uc_id)?;
            let use_case = &mut self.use_cases[index];

            // Get views for this use case
            let view_pairs: Vec<_> = use_case
                .enabled_views()
                .map(|v| (v.methodology.clone(), v.level.clone()))
                .collect();

            if view_pairs.is_empty() {
                continue;
            }

            // Collect field definitions for all views
            let field_collection = collector.collect_fields_for_views(&view_pairs)?;

            let mut use_case_updated = false;
            let mut use_case_added_fields = Vec::new();

            // For each methodology in views
            for (methodology, _level) in &view_pairs {
                let mut added_fields = Vec::new();

                // First pass: identify missing fields
                let missing_fields: Vec<(String, serde_json::Value)> = {
                    let existing_fields = use_case.methodology_fields.get(methodology);

                    field_collection
                        .fields
                        .iter()
                        .filter(|(field_name, field_def)| {
                            field_def.methodologies.contains(methodology)
                                && existing_fields
                                    .map(|fields| !fields.contains_key(*field_name))
                                    .unwrap_or(true)
                        })
                        .map(|(field_name, field_def)| {
                            let default_value = match field_def.field_type.as_str() {
                                "array" => serde_json::Value::Array(vec![]),
                                "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                                "boolean" => serde_json::Value::Bool(false),
                                _ => serde_json::Value::String(String::new()),
                            };
                            (field_name.clone(), default_value)
                        })
                        .collect()
                };

                // Track missing fields for reporting
                if !missing_fields.is_empty() {
                    for (field_name, _) in &missing_fields {
                        added_fields.push(field_name.clone());
                        use_case_updated = true;
                    }
                }

                // Second pass: actually add missing fields (skip if dry_run)
                if !(dry_run || missing_fields.is_empty()) {
                    for (field_name, default_value) in missing_fields {
                        use_case
                            .methodology_fields
                            .entry(methodology.clone())
                            .or_default()
                            .insert(field_name, default_value);
                    }
                }

                if !added_fields.is_empty() {
                    use_case_added_fields.push((methodology.clone(), added_fields));
                }
            }

            if use_case_updated {
                updated_count += 1;

                for (methodology, added_fields) in use_case_added_fields {
                    details.push((uc_id.clone(), methodology, added_fields));
                }

                if !dry_run {
                    self.repository.save(use_case)?;
                }
            }
        }

        Ok((updated_count, total_checked, details))
    }

    // Helper methods
    fn find_use_case_index(&self, use_case_id: &str) -> Result<usize> {
        self.use_cases
            .iter()
            .position(|uc| uc.id == use_case_id)
            .ok_or_else(|| anyhow::anyhow!("Use case '{}' not found", use_case_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MethodologyView, UseCase};
    use std::collections::HashMap;

    // Mock repository for testing
    struct MockRepository;

    impl UseCaseRepository for MockRepository {
        fn save(&self, _use_case: &UseCase) -> Result<()> {
            Ok(())
        }

        fn load_all(&self) -> Result<Vec<UseCase>> {
            Ok(vec![])
        }

        fn load_by_id(&self, _id: &str) -> Result<Option<UseCase>> {
            Ok(None)
        }

        fn save_markdown(&self, _use_case_id: &str, _content: &str) -> Result<()> {
            Ok(())
        }

        fn save_markdown_with_filename(
            &self,
            _use_case: &UseCase,
            _filename: &str,
            _content: &str,
        ) -> Result<()> {
            Ok(())
        }
    }

    fn create_test_use_case(id: &str) -> UseCase {
        let mut use_case = UseCase::new(
            id.to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TST".to_string(),
            "Test description".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case
            .views
            .push(MethodologyView::new("business", "normal"));
        use_case
    }

    #[test]
    #[ignore] // Requires template files to be present
    fn test_reinitialize_with_single_use_case() {
        let repository = MockRepository;
        let mut use_cases = vec![create_test_use_case("UC-TEST-001")];
        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        let result = service.reinitialize_methodology_fields(
            Some("UC-TEST-001".to_string()),
            true, // dry run
        );

        assert!(result.is_ok());
        let (updated_count, total_checked, _details) = result.unwrap();
        assert_eq!(total_checked, 1);
        // Updated count depends on whether fields were missing
        assert!(updated_count <= 1);
    }

    #[test]
    #[ignore] // Requires template files to be present
    fn test_reinitialize_all_use_cases() {
        let repository = MockRepository;
        let mut use_cases = vec![
            create_test_use_case("UC-TEST-001"),
            create_test_use_case("UC-TEST-002"),
        ];
        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        let result = service.reinitialize_methodology_fields(None, true);

        assert!(result.is_ok());
        let (_updated_count, total_checked, _details) = result.unwrap();
        assert_eq!(total_checked, 2);
    }

    #[test]
    fn test_reinitialize_nonexistent_use_case() {
        let repository = MockRepository;
        let mut use_cases = vec![create_test_use_case("UC-TEST-001")];
        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        let result =
            service.reinitialize_methodology_fields(Some("UC-NONEXISTENT".to_string()), true);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[ignore] // Requires template files to be present
    fn test_reinitialize_dry_run_vs_actual() {
        let repository = MockRepository;
        let mut use_cases = vec![create_test_use_case("UC-TEST-001")];

        // Dry run
        {
            let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
            let result = service.reinitialize_methodology_fields(None, true);
            assert!(result.is_ok());
        }

        // Actual run
        {
            let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
            let result = service.reinitialize_methodology_fields(None, false);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_find_use_case_index() {
        let repository = MockRepository;
        let mut use_cases = vec![
            create_test_use_case("UC-TEST-001"),
            create_test_use_case("UC-TEST-002"),
        ];
        let service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        let index1 = service.find_use_case_index("UC-TEST-001");
        assert!(index1.is_ok());
        assert_eq!(index1.unwrap(), 0);

        let index2 = service.find_use_case_index("UC-TEST-002");
        assert!(index2.is_ok());
        assert_eq!(index2.unwrap(), 1);

        let index_none = service.find_use_case_index("UC-NONEXISTENT");
        assert!(index_none.is_err());
    }

    #[test]
    #[ignore] // Requires template files to be present
    fn test_reinitialize_with_existing_fields() {
        let repository = MockRepository;
        let mut use_case = create_test_use_case("UC-TEST-001");

        // Add some existing fields
        let mut business_fields = HashMap::new();
        business_fields.insert(
            "existing_field".to_string(),
            serde_json::json!("existing value"),
        );
        use_case
            .methodology_fields
            .insert("business".to_string(), business_fields);

        let mut use_cases = vec![use_case];
        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        let result = service.reinitialize_methodology_fields(None, true);
        assert!(result.is_ok());

        // Verify existing field wasn't overwritten
        let use_case = &use_cases[0];
        if let Some(business_fields) = use_case.methodology_fields.get("business") {
            if let Some(value) = business_fields.get("existing_field") {
                assert_eq!(value, &serde_json::json!("existing value"));
            }
        }
    }

    /// Regression test for the dry-run mutation bug
    ///
    /// This test verifies that calling reinitialize with dry_run=true does NOT
    /// mutate the use case data in memory. This was a critical bug where:
    /// 1. Interactive mode would call with dry_run=true to preview changes
    /// 2. User confirms to apply changes
    /// 3. Call with dry_run=false would find 0 updates (state was already mutated)
    ///
    /// The bug was caused by .entry().or_default() creating empty HashMaps
    /// even during read-only dry-run operations.
    #[test]
    fn test_dry_run_does_not_mutate_state() {
        let repository = MockRepository;
        let mut use_case = create_test_use_case("UC-TEST-DRY");

        // Start with only 1 field in business methodology
        let mut business_fields = HashMap::new();
        business_fields.insert("field1".to_string(), serde_json::json!("value1"));
        use_case
            .methodology_fields
            .insert("business".to_string(), business_fields);

        let mut use_cases = vec![use_case];

        // Count fields before dry-run
        let fields_before = use_cases[0]
            .methodology_fields
            .get("business")
            .map(|f| f.len())
            .unwrap_or(0);

        // First call: dry-run (should NOT mutate)
        {
            let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
            let _result = service.reinitialize_methodology_fields(None, true);
            // We don't check the result because it might fail without templates
            // We only care that it doesn't mutate the data
        }

        // Count fields after dry-run
        let fields_after_dry = use_cases[0]
            .methodology_fields
            .get("business")
            .map(|f| f.len())
            .unwrap_or(0);

        // CRITICAL: Field count should be unchanged after dry-run
        assert_eq!(
            fields_before, fields_after_dry,
            "Dry-run must NOT add fields to use case data. Before: {}, After: {}",
            fields_before, fields_after_dry
        );

        // Verify no empty methodology entries were created
        for (methodology, _fields) in &use_cases[0].methodology_fields {
            if methodology != "business" {
                panic!(
                    "Dry-run created unexpected methodology entry: {}",
                    methodology
                );
            }
        }

        // Second call: actual run (should mutate if fields need to be added)
        {
            let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
            let _result = service.reinitialize_methodology_fields(None, false);
            // Result might fail without templates, we only verify no mutation happened during dry-run
        }
    }
}
