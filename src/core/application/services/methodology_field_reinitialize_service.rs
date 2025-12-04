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

    fn create_use_case_with_views(id: &str, views: Vec<(&str, &str)>) -> UseCase {
        let mut use_case = UseCase::new(
            id.to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TST".to_string(),
            "Test description".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        for (methodology, view) in views {
            use_case.views.push(MethodologyView::new(methodology, view));
        }
        use_case
    }

    fn create_use_case_with_fields(
        id: &str,
        methodology: &str,
        fields: Vec<(&str, serde_json::Value)>,
    ) -> UseCase {
        let mut use_case = create_test_use_case(id);
        let mut field_map = HashMap::new();
        for (key, value) in fields {
            field_map.insert(key.to_string(), value);
        }
        use_case
            .methodology_fields
            .insert(methodology.to_string(), field_map);
        use_case
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
        for methodology in use_cases[0].methodology_fields.keys() {
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

    #[test]
    fn test_reinitialize_with_multiple_views() {
        let repository = MockRepository;
        let mut use_cases = vec![create_use_case_with_views(
            "UC-TEST-MULTI",
            vec![
                ("business", "normal"),
                ("developer", "advanced"),
                ("tester", "normal"),
            ],
        )];

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        // This should try to process all three methodologies
        // It will fail because we don't have templates, but we're testing the logic flow
        let _result = service.reinitialize_methodology_fields(None, true);

        // The important part is that it processes all views without panicking
        // and that dry_run doesn't mutate the state
        assert_eq!(use_cases[0].methodology_fields.len(), 0);
    }

    #[test]
    fn test_reinitialize_preserves_existing_fields() {
        let repository = MockRepository;
        let mut use_cases = vec![create_use_case_with_fields(
            "UC-TEST-PRESERVE",
            "business",
            vec![
                ("existing_field1", serde_json::json!("value1")),
                ("existing_field2", serde_json::json!(42)),
                ("existing_field3", serde_json::json!(true)),
            ],
        )];

        let fields_before = use_cases[0]
            .methodology_fields
            .get("business")
            .unwrap()
            .clone();

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
        let _result = service.reinitialize_methodology_fields(None, true);

        // Existing fields should remain unchanged after dry-run
        let fields_after = use_cases[0].methodology_fields.get("business").unwrap();

        assert_eq!(fields_before.len(), fields_after.len());
        for (key, value) in fields_before.iter() {
            assert_eq!(
                fields_after.get(key),
                Some(value),
                "Field {} should be preserved",
                key
            );
        }
    }

    #[test]
    fn test_reinitialize_multiple_use_cases() {
        let repository = MockRepository;
        let mut use_cases = vec![
            create_test_use_case("UC-TEST-001"),
            create_test_use_case("UC-TEST-002"),
            create_test_use_case("UC-TEST-003"),
        ];

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        // Process all use cases
        let _result = service.reinitialize_methodology_fields(None, true);

        // Verify dry-run didn't mutate any of the use cases
        for uc in &use_cases {
            assert_eq!(
                uc.methodology_fields.len(),
                0,
                "Use case {} should have no fields after dry-run",
                uc.id
            );
        }
    }

    #[test]
    fn test_reinitialize_specific_use_case() {
        let repository = MockRepository;
        let mut use_cases = vec![
            create_test_use_case("UC-TEST-001"),
            create_test_use_case("UC-TEST-002"),
        ];

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);

        // Process only specific use case
        let _result =
            service.reinitialize_methodology_fields(Some("UC-TEST-002".to_string()), true);

        // Should attempt to process only the specified use case
        // (will fail without templates but we're testing the selection logic)
        assert_eq!(use_cases[0].methodology_fields.len(), 0);
        assert_eq!(use_cases[1].methodology_fields.len(), 0);
    }

    #[test]
    fn test_use_case_without_views() {
        let repository = MockRepository;
        let mut use_case = create_test_use_case("UC-NO-VIEWS");
        use_case.views.clear(); // Remove all views
        let mut use_cases = vec![use_case];

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
        let _result = service.reinitialize_methodology_fields(None, true);

        // Should handle gracefully when no views exist
        assert_eq!(use_cases[0].methodology_fields.len(), 0);
    }

    #[test]
    fn test_empty_use_case_list() {
        let repository = MockRepository;
        let mut use_cases: Vec<UseCase> = vec![];

        let mut service = MethodologyFieldReinitializeService::new(&repository, &mut use_cases);
        let result = service.reinitialize_methodology_fields(None, true);

        // Should handle empty list gracefully
        assert!(result.is_ok());
        let (use_cases_processed, fields_added, _details) = result.unwrap();
        assert_eq!(use_cases_processed, 0);
        assert_eq!(fields_added, 0);
    }
}
