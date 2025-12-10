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

        // Use cases data
        let use_cases_data: Vec<_> = use_cases
            .iter()
            .map(|uc| {
                // Compute scenario type counts for this use case
                let mut main_count = 0usize;
                let mut alternative_count = 0usize;
                let mut exception_count = 0usize;
                let mut extension_count = 0usize;
                for s in &uc.scenarios {
                    match s.scenario_type {
                        crate::core::domain::ScenarioType::HappyPath => main_count += 1,
                        crate::core::domain::ScenarioType::AlternativeFlow => {
                            alternative_count += 1
                        }
                        crate::core::domain::ScenarioType::ExceptionFlow => exception_count += 1,
                        crate::core::domain::ScenarioType::Extension => extension_count += 1,
                    }
                }

                json!({
                    "id": uc.id,
                    "title": uc.title,
                    "aggregated_status": uc.status().display_name(),
                    "aggregated_status_emoji": uc.status().emoji(),
                    "priority": uc.priority.to_string(),
                    "description": uc.description,
                    "views": uc.views,
                    // last_updated for the use case (RFC3339)
                    "last_updated": uc.metadata.updated_at.to_rfc3339(),
                    // scenario counts split by type
                    "scenario_counts": {
                        "main": main_count,
                        "alternative": alternative_count,
                        "exception": exception_count,
                        "extension": extension_count
                    }
                })
            })
            .collect();

        data.insert("use_cases".to_string(), json!(use_cases_data));
        data.insert("total_use_cases".to_string(), json!(use_cases.len()));

        // Compute status distribution across use cases in this category
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        for uc in use_cases {
            *status_counts
                .entry(uc.status().display_name().to_string())
                .or_default() += 1;
        }
        data.insert("status_counts".to_string(), json!(status_counts));

        // Category-level last-updated (most recent use case updated_at)
        let category_last_updated = use_cases
            .iter()
            .map(|uc| uc.metadata.updated_at)
            .max();
        if let Some(dt) = category_last_updated {
            data.insert("last_updated".to_string(), json!(dt.to_rfc3339()));
        } else {
            data.insert("last_updated".to_string(), json!(""));
        }

        // Category-level scenario aggregates
        let mut cat_main = 0usize;
        let mut cat_alt = 0usize;
        let mut cat_exc = 0usize;
        let mut cat_ext = 0usize;
        for uc in use_cases {
            for s in &uc.scenarios {
                match s.scenario_type {
                    crate::core::domain::ScenarioType::HappyPath => cat_main += 1,
                    crate::core::domain::ScenarioType::AlternativeFlow => cat_alt += 1,
                    crate::core::domain::ScenarioType::ExceptionFlow => cat_exc += 1,
                    crate::core::domain::ScenarioType::Extension => cat_ext += 1,
                }
            }
        }
        data.insert(
            "scenario_totals".to_string(),
            json!({
                "main": cat_main,
                "alternative": cat_alt,
                "exception": cat_exc,
                "extension": cat_ext
            }),
        );

        // Compute scenario status distribution for this category
        let mut scenario_status_counts: HashMap<String, usize> = HashMap::new();
        for uc in use_cases {
            for s in &uc.scenarios {
                *scenario_status_counts
                    .entry(s.status.display_name().to_string())
                    .or_default() += 1;
            }
        }
        data.insert(
            "scenario_status_counts".to_string(),
            json!(scenario_status_counts),
        );

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
    use crate::core::domain::MethodologyView;
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
        let use_cases = [
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
        let use_cases = [create_test_use_case(
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
        let use_cases = [
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
    fn test_category_overview_scenario_counts_and_last_updated() {
        use chrono::DateTime;

        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("AuthX".to_string(), "AXX".to_string()).unwrap();

        // Create a use case with one main and one alternative scenario
        let mut uc = create_test_use_case("AX-001", "AuthX", "Sample UC");
        let s1 = crate::core::domain::Scenario::new(
            "AX-001-S01".to_string(),
            "Main Flow".to_string(),
            "Main happy path".to_string(),
            crate::core::domain::ScenarioType::HappyPath,
            "user".to_string(),
        );
        let s2 = crate::core::domain::Scenario::new(
            "AX-001-S02".to_string(),
            "Alt Flow".to_string(),
            "Alternative path".to_string(),
            crate::core::domain::ScenarioType::AlternativeFlow,
            "user".to_string(),
        );
        uc.scenarios.push(s1);
        uc.scenarios.push(s2);

        // Set a fixed updated_at so formatted date is deterministic
        let dt = DateTime::parse_from_rfc3339("2025-01-02T03:04:05+00:00").unwrap();
        uc.metadata.updated_at = dt.with_timezone(&chrono::Utc);

        let use_cases: Vec<&UseCase> = vec![&uc];

        // Generate and assert
        generator
            .generate_for_category(&category, &use_cases)
            .unwrap();

        let readme_path = temp_dir
            .path()
            .join("use-cases")
            .join("authx")
            .join("README.md");
        let content = std::fs::read_to_string(&readme_path).unwrap();

        // Scenario counts: 1 main, 1 alternative, 0 exception, 0 extension
        assert!(
            content.contains("| 1 | 1 | 0 | 0 |"),
            "scenario counts present: {}",
            content
        );

        // Date formatted according to default (%d/%m/%Y) -> 02/01/2025
        assert!(
            content.contains("02/01/2025"),
            "last updated formatted in content: {}",
            content
        );
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

    #[test]
    #[serial]
    fn test_category_overview_contains_total_use_cases() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        let use_cases = [
            create_test_use_case("AUT-001", "Authentication", "User Login"),
            create_test_use_case("AUT-002", "Authentication", "Password Reset"),
            create_test_use_case("AUT-003", "Authentication", "Two-Factor Auth"),
        ];
        let use_case_refs: Vec<&UseCase> = use_cases.iter().collect();

        generator
            .generate_for_category(&category, &use_case_refs)
            .unwrap();

        let readme_path = temp_dir
            .path()
            .join("use-cases")
            .join("authentication")
            .join("README.md");
        let content = std::fs::read_to_string(&readme_path).unwrap();

        // Verify total use cases count is correct
        assert!(
            content.contains("**3** use cases") || content.contains("**3** use case"),
            "Should contain total use cases count of 3, content: {}",
            content
        );
    }

    #[test]
    #[serial]
    fn test_category_overview_single_use_case_count() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Payment".to_string(), "PAY".to_string()).unwrap();
        let use_cases = [create_test_use_case(
            "PAY-001",
            "Payment",
            "Process Payment",
        )];
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

        // Verify singular form for 1 use case
        assert!(
            content.contains("**1** use case") && !content.contains("**1** use cases"),
            "Should use singular 'use case' for count of 1, content: {}",
            content
        );
    }

    #[test]
    #[serial]
    fn test_category_overview_contains_use_case_details() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();

        // Create use case with specific status and priority
        let mut use_case = create_test_use_case("AUT-001", "Authentication", "User Login");
        use_case.description = "Allows users to log in with credentials".to_string();

        let use_case_refs = vec![&use_case];

        generator
            .generate_for_category(&category, &use_case_refs)
            .unwrap();

        let readme_path = temp_dir
            .path()
            .join("use-cases")
            .join("authentication")
            .join("README.md");
        let content = std::fs::read_to_string(&readme_path).unwrap();

        // Verify use case details are included
        assert!(content.contains("AUT-001"), "Should contain use case ID");
        assert!(
            content.contains("User Login"),
            "Should contain use case title"
        );
        assert!(
            content.contains("MEDIUM") || content.contains("Medium"),
            "Should contain priority"
        );
        assert!(
            content.contains("Allows users to log in with credentials"),
            "Should contain description"
        );
    }

    #[test]
    #[serial]
    fn test_category_overview_with_multi_view_use_case() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let generator = CategoryOverviewGenerator::new(config);

        let category = Category::new("Testing".to_string(), "TST".to_string()).unwrap();

        // Create use case with multiple views
        let mut use_case = create_test_use_case("TST-001", "Testing", "Multi-View Feature");
        use_case.views = vec![
            MethodologyView::new("developer".to_string(), "advanced".to_string()),
            MethodologyView::new("tester".to_string(), "normal".to_string()),
        ];

        let use_case_refs = vec![&use_case];

        generator
            .generate_for_category(&category, &use_case_refs)
            .unwrap();

        let readme_path = temp_dir
            .path()
            .join("use-cases")
            .join("testing")
            .join("README.md");
        let content = std::fs::read_to_string(&readme_path).unwrap();

        // Verify views are listed
        assert!(
            content.contains("Available Views")
                || content.contains("developer-advanced")
                || content.contains("tester-normal"),
            "Should list available views, content: {}",
            content
        );
    }
}
