//! Category overview generator for category-level documentation.
//!
//! Handles generation of category README.md files that list all use cases in a category.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;

use crate::config::Config;
use crate::core::domain::Category;
use crate::core::file_operations::FileOperations;
use crate::core::utils::to_snake_case;
use crate::core::{TemplateEngine, UseCase};

/// Generator for category overview documentation.
pub struct CategoryOverviewGenerator {
    config: Config,
    file_operations: FileOperations,
    template_engine: TemplateEngine,
}

impl CategoryOverviewGenerator {
    /// Creates a new category overview generator with the given configuration.
    pub fn new(config: Config) -> Self {
        let file_operations = FileOperations::new(config.clone());
        let template_engine = TemplateEngine::with_config(Some(&config));
        Self {
            config,
            file_operations,
            template_engine,
        }
    }

    /// Generates and saves category overview files for all categories.
    ///
    /// Creates a README.md file in each category folder that lists all use cases
    /// in that category with their titles, statuses, and priorities.
    ///
    /// # Arguments
    /// * `use_cases` - All use cases in the project
    /// * `categories` - All categories defined in the project
    pub fn generate_all(&self, use_cases: &[UseCase], categories: &[Category]) -> Result<()> {
        // Group use cases by category
        let mut category_use_cases: HashMap<String, Vec<&UseCase>> = HashMap::new();
        for uc in use_cases {
            category_use_cases
                .entry(uc.category.clone())
                .or_default()
                .push(uc);
        }

        // Generate overview for each category
        for category in categories {
            let use_cases_in_category = category_use_cases
                .get(&category.full_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            self.generate_for_category(category, use_cases_in_category)?;
        }

        Ok(())
    }

    /// Generates and saves a category overview file for a specific category.
    ///
    /// # Arguments
    /// * `category` - The category to generate overview for
    /// * `use_cases` - Use cases belonging to this category
    pub fn generate_for_category(&self, category: &Category, use_cases: &[&UseCase]) -> Result<()> {
        let mut data = HashMap::new();

        // Category information
        data.insert("category_name".to_string(), json!(category.full_name));
        data.insert(
            "category_abbreviation".to_string(),
            json!(category.abbreviation),
        );
        data.insert("use_case_count".to_string(), json!(use_cases.len()));

        // Project name and generated date
        data.insert("project_name".to_string(), json!(self.config.project.name));
        data.insert(
            "generated_date".to_string(),
            json!(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        );

        // Use cases data
        let use_cases_data: Vec<_> = use_cases
            .iter()
            .map(|uc| {
                json!({
                    "id": uc.id,
                    "title": uc.title,
                    "aggregated_status": uc.status().display_name(),
                    "priority": uc.priority.to_string(),
                })
            })
            .collect();

        data.insert("use_cases".to_string(), json!(use_cases_data));

        // Render template
        let content = self.template_engine.render_category_overview(&data)?;

        // Save to category folder
        let category_path = to_snake_case(&category.full_name);
        self.file_operations
            .save_category_overview(&category_path, &content)?;

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
    #[serial]
    fn test_generate_category_overview() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
        ];
        let use_case_refs: Vec<&UseCase> = use_cases.iter().collect();

        let result = generator.generate_for_category(&category, &use_case_refs);
        assert!(
            result.is_ok(),
            "Should generate category overview: {:?}",
            result.err()
        );
    }

    #[test]
    #[serial]
    fn test_generate_all_categories() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let categories = vec![
            Category::new("Authentication".to_string(), "AUT".to_string()).unwrap(),
            Category::new("Payment".to_string(), "PAY".to_string()).unwrap(),
        ];

        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
            create_test_use_case("PAY-001", "Payment", "Process Payment"),
        ];

        let result = generator.generate_all(&use_cases, &categories);
        assert!(
            result.is_ok(),
            "Should generate all category overviews: {:?}",
            result.err()
        );
    }

    #[test]
    #[serial]
    fn test_generate_empty_category() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Empty Category".to_string(), "EMP".to_string()).unwrap();
        let use_cases: Vec<&UseCase> = vec![];

        let result = generator.generate_for_category(&category, &use_cases);
        assert!(
            result.is_ok(),
            "Should handle empty category: {:?}",
            result.err()
        );
    }

    #[test]
    #[serial]
    fn test_category_overview_file_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        let use_cases = vec![create_test_use_case(
            "AUT-001",
            "Authentication",
            "User Login",
        )];
        let use_case_refs: Vec<&UseCase> = use_cases.iter().collect();

        generator
            .generate_for_category(&category, &use_case_refs)
            .unwrap();

        // Verify file was created at correct path: {use_case_dir}/authentication/README.md
        let expected_path = temp_dir
            .path()
            .join("use-cases")
            .join("authentication")
            .join("README.md");
        assert!(
            expected_path.exists(),
            "Category README.md should exist at: {}",
            expected_path.display()
        );

        // Verify content contains category information
        let content = std::fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("Authentication"));
        assert!(content.contains("AUT-001"));
        assert!(content.contains("User Login"));
    }

    #[test]
    #[serial]
    fn test_category_overview_multiple_use_cases() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Payment".to_string(), "PAY".to_string()).unwrap();
        let use_cases = vec![
            create_test_use_case("PAY-001", "Payment", "Process Payment"),
            create_test_use_case("PAY-002", "Payment", "Refund Payment"),
            create_test_use_case("PAY-003", "Payment", "Payment History"),
        ];
        let use_case_refs: Vec<&UseCase> = use_cases.iter().collect();

        generator
            .generate_for_category(&category, &use_case_refs)
            .unwrap();

        let readme_path = temp_dir
            .path()
            .join("use-cases")
            .join("payment")
            .join("README.md");
        let content = std::fs::read_to_string(&readme_path).unwrap();

        // Verify all use cases are listed
        assert!(content.contains("PAY-001"));
        assert!(content.contains("PAY-002"));
        assert!(content.contains("PAY-003"));
        assert!(content.contains("Process Payment"));
        assert!(content.contains("Refund Payment"));
        assert!(content.contains("Payment History"));
    }

    #[test]
    #[serial]
    fn test_generate_all_creates_multiple_category_readmes() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let categories = vec![
            Category::new("Authentication".to_string(), "AUT".to_string()).unwrap(),
            Category::new("Payment".to_string(), "PAY".to_string()).unwrap(),
            Category::new("Reporting".to_string(), "RPT".to_string()).unwrap(),
        ];

        let use_cases = vec![
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("PAY-001", "Payment", "Process Payment"),
            create_test_use_case("RPT-001", "Reporting", "Generate Report"),
        ];

        generator.generate_all(&use_cases, &categories).unwrap();

        // Verify each category has its README.md
        let auth_readme = temp_dir
            .path()
            .join("use-cases")
            .join("authentication")
            .join("README.md");
        let payment_readme = temp_dir
            .path()
            .join("use-cases")
            .join("payment")
            .join("README.md");
        let reporting_readme = temp_dir
            .path()
            .join("use-cases")
            .join("reporting")
            .join("README.md");

        assert!(auth_readme.exists(), "Authentication README should exist");
        assert!(payment_readme.exists(), "Payment README should exist");
        assert!(reporting_readme.exists(), "Reporting README should exist");
    }
}
