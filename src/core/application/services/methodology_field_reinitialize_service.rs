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
                let fields = use_case
                    .methodology_fields
                    .entry(methodology.clone())
                    .or_default();

                let mut added_fields = Vec::new();

                // Add missing fields
                for (field_name, field_def) in &field_collection.fields {
                    if field_def.methodologies.contains(methodology)
                        && !fields.contains_key(field_name)
                    {
                        let default_value = match field_def.field_type.as_str() {
                            "array" => serde_json::Value::Array(vec![]),
                            "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                            "boolean" => serde_json::Value::Bool(false),
                            _ => serde_json::Value::String(String::new()),
                        };

                        fields.insert(field_name.clone(), default_value);
                        added_fields.push(field_name.clone());
                        use_case_updated = true;
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
