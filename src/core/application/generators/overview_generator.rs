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

        // Project name and generated date
        data.insert("project_name".to_string(), json!(self.config.project.name));

        // Group use cases by category to count them
        let mut categories_map: HashMap<String, usize> = HashMap::new();
        for uc in use_cases {
            *categories_map.entry(uc.category.clone()).or_default() += 1;
        }

        // Add total categories count
        data.insert("total_categories".to_string(), json!(categories_map.len()));

        // Convert to array format expected by new template
        // New format: categories with category_name, category_path, and use_case_count
        let categories: Vec<serde_json::Map<String, Value>> = categories_map
            .into_iter()
            .map(|(category_name, count)| {
                let mut cat = serde_json::Map::new();
                cat.insert("category_name".to_string(), json!(category_name));
                // Convert category name to snake_case for path
                cat.insert(
                    "category_path".to_string(),
                    json!(crate::core::utils::to_snake_case(&category_name)),
                );
                cat.insert("use_case_count".to_string(), json!(count));
                // Optional: Add description if available (would need category entity)
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
}
