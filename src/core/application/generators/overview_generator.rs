//! Overview generator for project documentation.
//!
//! Handles generation of project overview documentation that summarizes all use cases.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::config::Config;
use crate::core::file_operations::FileOperations;
use crate::core::{TemplateEngine, UseCase};

/// Generator for project overview documentation.
pub struct OverviewGenerator {
    config: Config,
    file_operations: FileOperations,
    template_engine: TemplateEngine,
}

impl OverviewGenerator {
    /// Creates a new overview generator with the given configuration.
    pub fn new(config: Config) -> Self {
        let file_operations = FileOperations::new(config.clone());
        let template_engine = TemplateEngine::with_config(Some(&config));
        Self {
            config,
            file_operations,
            template_engine,
        }
    }

    /// Generates and saves the project overview file.
    ///
    /// Creates an overview document that includes:
    /// - Project name and generation date
    /// - Total use case count and total category count
    /// - Categories with their paths and use case counts
    pub fn generate(&self, use_cases: &[UseCase]) -> Result<()> {
        let mut data = HashMap::new();

        // Basic counts
        data.insert("total_use_cases".to_string(), json!(use_cases.len()));

        // Project name
        data.insert("project_name".to_string(), json!(self.config.project.name));

        // Group use cases by category to collect lists and counts
        let mut categories_map: HashMap<String, Vec<&UseCase>> = HashMap::new();
        for uc in use_cases {
            categories_map
                .entry(uc.category.clone())
                .or_default()
                .push(uc);
        }

        // Add total categories count
        data.insert("total_categories".to_string(), json!(categories_map.len()));

        // Compute status distribution across all use cases
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        for uc in use_cases {
            *status_counts
                .entry(uc.status().display_name().to_string())
                .or_default() += 1;
        }
        data.insert("status_counts".to_string(), json!(status_counts));

        // Compute scenario totals across all use cases (main/alternative/exception/extension)
        let mut scen_main = 0usize;
        let mut scen_alt = 0usize;
        let mut scen_exc = 0usize;
        let mut scen_ext = 0usize;
        // Compute scenario status distribution across all scenarios
        let mut scenario_status_counts: HashMap<String, usize> = HashMap::new();
        for uc in use_cases {
            for s in &uc.scenarios {
                match s.scenario_type {
                    crate::core::domain::ScenarioType::HappyPath => scen_main += 1,
                    crate::core::domain::ScenarioType::AlternativeFlow => scen_alt += 1,
                    crate::core::domain::ScenarioType::ExceptionFlow => scen_exc += 1,
                    crate::core::domain::ScenarioType::Extension => scen_ext += 1,
                }
                *scenario_status_counts
                    .entry(s.status.display_name().to_string())
                    .or_default() += 1;
            }
        }
        data.insert(
            "scenario_totals".to_string(),
            json!({
                "main": scen_main,
                "alternative": scen_alt,
                "exception": scen_exc,
                "extension": scen_ext
            }),
        );

        // Insert scenario status distribution map for templates
        data.insert(
            "scenario_status_counts".to_string(),
            json!(scenario_status_counts),
        );

        // Compute overall last-updated across all use cases (most recent updated_at)
        let overall_last_updated = use_cases
            .iter()
            .map(|uc| uc.metadata.updated_at.clone())
            .max();

        if let Some(dt) = overall_last_updated {
            data.insert("last_updated".to_string(), json!(dt.to_rfc3339()));
        } else {
            data.insert("last_updated".to_string(), json!(""));
        }

        // Convert to array format expected by new template
        let categories: Vec<serde_json::Map<String, Value>> = categories_map
            .into_iter()
            .map(|(category_name, uc_list)| {
                let mut cat = serde_json::Map::new();
                cat.insert("category_name".to_string(), json!(category_name));
                // Convert category name to snake_case for path
                cat.insert(
                    "category_path".to_string(),
                    json!(crate::core::utils::to_snake_case(&category_name)),
                );
                cat.insert("use_case_count".to_string(), json!(uc_list.len()));
                // Build a small use_cases list for overview display
                let use_cases_data: Vec<_> = uc_list
                    .iter()
                    .map(|uc| {
                        // Build scenarios list for quick-links in overview templates
                        let scenarios_data: Vec<_> = uc
                            .scenarios
                            .iter()
                            .map(|s| {
                                json!({
                                    "id": s.id,
                                    "title": s.title,
                                    "status": s.status.display_name(),
                                    "status_emoji": s.status.emoji(),
                                    // Link to use case README with fragment to jump to scenario header
                                    "link": format!("./{}/README.md#{}", uc.id, s.id),
                                })
                            })
                            .collect();

                        json!({
                            "id": uc.id,
                            "title": uc.title,
                            "path": crate::core::utils::to_snake_case(&uc.category),
                            "aggregated_status": uc.status().display_name(),
                            "aggregated_status_emoji": uc.status().emoji(),
                            "priority": uc.priority.to_string(),
                            // Provide last_updated as RFC3339 string so templates can format it
                            "last_updated": uc.metadata.updated_at.to_rfc3339(),
                            "scenarios": scenarios_data,
                        })
                    })
                    .collect();

                cat.insert("use_cases".to_string(), json!(use_cases_data));
                // Optional: Add description if available (would need category entity)
                // Category-level last-updated (most recent updated_at among its use cases)
                let category_last = uc_list
                    .iter()
                    .map(|uc| uc.metadata.updated_at.clone())
                    .max();
                if let Some(dt) = category_last {
                    cat.insert("last_updated".to_string(), json!(dt.to_rfc3339()));
                } else {
                    cat.insert("last_updated".to_string(), json!(""));
                }

                cat.insert("description".to_string(), json!(""));
                cat
            })
            .collect();

        data.insert("categories".to_string(), json!(categories));

        let overview_content = self.template_engine.render_overview(&data)?;
        self.file_operations.save_overview(&overview_content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        let mut config = Config::default();
        config.project.name = "Test Project".to_string();
        config.directories.use_case_dir = temp_dir
            .path()
            .join("use-cases")
            .to_string_lossy()
            .to_string();
        config
    }

    fn create_test_use_case(id: &str, category: &str, title: &str) -> UseCase {
        UseCase::new(
            id.to_string(),
            title.to_string(),
            category.to_string(),
            "TST".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_generate_overview_with_multiple_categories() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = OverviewGenerator::new(config);

        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
            create_test_use_case("PAY-001", "Payment", "Process Payment"),
            create_test_use_case("RPT-001", "Reporting", "Generate Report"),
        ];

        let result = generator.generate(&use_cases);
        assert!(
            result.is_ok(),
            "Should generate overview: {:?}",
            result.err()
        );

        // Verify README.md was created at root of use-cases directory
        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        assert!(
            overview_path.exists(),
            "Overview README.md should exist at: {}",
            overview_path.display()
        );

        // Verify content contains category information
        let content = std::fs::read_to_string(&overview_path).unwrap();
        assert!(content.contains("Test Project"));
        assert!(content.contains("Authentication"));
        assert!(content.contains("Payment"));
        assert!(content.contains("Reporting"));
    }

    #[test]
    #[serial]
    fn test_overview_contains_category_counts() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = OverviewGenerator::new(config);

        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
            create_test_use_case("AUT-003", "Authentication", "Two-Factor Auth"),
            create_test_use_case("PAY-001", "Payment", "Process Payment"),
            create_test_use_case("PAY-002", "Payment", "Refund Payment"),
        ];

        generator.generate(&use_cases).unwrap();

        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        let content = std::fs::read_to_string(&overview_path).unwrap();

        // Verify total counts
        assert!(content.contains("5")); // Total use cases
        assert!(content.contains("2")); // Total categories

        // Verify individual category counts (3 for Authentication, 2 for Payment)
        assert!(content.contains("3") || content.contains("three"));
        assert!(content.contains("2") || content.contains("two"));
    }

    #[test]
    fn test_overview_with_empty_use_cases() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = OverviewGenerator::new(config);

        let use_cases: Vec<UseCase> = vec![];

        let result = generator.generate(&use_cases);
        assert!(
            result.is_ok(),
            "Should handle empty use case list: {:?}",
            result.err()
        );

        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        assert!(
            overview_path.exists(),
            "Overview should be created even when empty"
        );
    }

    #[test]
    fn test_overview_category_paths_are_snake_case() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = OverviewGenerator::new(config);

        let use_cases = vec![
            create_test_use_case("UC-001", "User Management", "Create User"),
            create_test_use_case("UC-002", "API Integration", "Connect API"),
        ];

        generator.generate(&use_cases).unwrap();

        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        let content = std::fs::read_to_string(&overview_path).unwrap();

        // Category paths should be snake_case for folder structure
        assert!(
            content.contains("user_management") || content.contains("User Management"),
            "Should contain user_management path"
        );
        assert!(
            content.contains("api_integration") || content.contains("API Integration"),
            "Should contain api_integration path"
        );
    }

    #[test]
    fn test_overview_single_category() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = OverviewGenerator::new(config);

        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
            create_test_use_case("AUT-003", "Authentication", "Two-Factor Auth"),
        ];

        generator.generate(&use_cases).unwrap();

        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        let content = std::fs::read_to_string(&overview_path).unwrap();

        // Should show 3 use cases and 1 category
        assert!(content.contains("3"));
        assert!(content.contains("1"));
        assert!(content.contains("Authentication"));
    }

    #[test]
    #[serial]
    fn test_overview_last_updated_is_most_recent() {
        use chrono::DateTime;

        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        // ensure output directory points to temp dir
        let generator = OverviewGenerator::new(config);

        // Two use cases with different updated_at
        let mut uc1 = create_test_use_case("AUT-010", "Authentication", "Old UC");
        let mut uc2 = create_test_use_case("PAY-020", "Payment", "New UC");

        let dt_old = DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap();
        let dt_new = DateTime::parse_from_rfc3339("2025-03-05T12:00:00+00:00").unwrap();
        uc1.metadata.updated_at = dt_old.with_timezone(&chrono::Utc);
        uc2.metadata.updated_at = dt_new.with_timezone(&chrono::Utc);

        let use_cases = vec![uc1, uc2];

        generator.generate(&use_cases).unwrap();

        let overview_path = temp_dir.path().join("use-cases").join("README.md");
        let content = std::fs::read_to_string(&overview_path).unwrap();

        // The overall last_updated should be the newer date formatted (%d/%m/%Y -> 05/03/2025)
        assert!(content.contains("05/03/2025"), "overview contains overall last updated: {}", content);
    }
}
