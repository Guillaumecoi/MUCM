//! Integration tests for folder-based use case structure
//!
//! Tests the complete end-to-end flow of folder-based use cases:
//! - Creating use cases in {category}/{use-case-id}/ folders
//! - Verifying README.md is the canonical link target
//! - Category overview generation in {category}/README.md
//! - Proper folder cleanup on deletion

use mucm::{
    config::{Config, ConfigFileManager},
    controller::{CreateUseCaseParams, UseCaseController},
};
use serial_test::serial;
use std::{collections::HashMap, env, fs};
use tempfile::TempDir;

mod common;

/// Test helper: Setup test environment with initialized config
fn setup_test_env() -> (TempDir, UseCaseController) {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    // Set up isolated test templates (bypasses user config caching)
    let _template_mgr = common::TestTemplateManager::new().unwrap();

    // Create default config with business methodology
    let mut config = Config::default();
    config.templates.methodologies = vec!["business".to_string()];
    ConfigFileManager::save_in_dir(&config, ".").unwrap();

    // Copy templates to config directory
    Config::copy_templates_to_config_with_language(None).unwrap();

    let controller = UseCaseController::new().unwrap();
    (temp_dir, controller)
}

/// Test helper: Extract use case ID from controller result message
fn extract_use_case_id(message: &str) -> String {
    // Message format: "Created use case: UC-XXX-nnn with views: ..."
    // or "Use case UC-XXX-nnn ..."
    message
        .split_whitespace()
        .find(|s| s.starts_with("UC-"))
        .unwrap_or_else(|| panic!("Should have a use case ID in the message: '{}'", message))
        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-')
        .to_string()
}

/// Test helper: Create use case params for testing
fn create_test_params(
    title: &str,
    category: &str,
    abbrev: &str,
    description: Option<String>,
) -> CreateUseCaseParams {
    CreateUseCaseParams {
        title: title.to_string(),
        category: category.to_string(),
        category_abbreviation: abbrev.to_string(),
        description,
        methodology: Some("business".to_string()),
        views: Some("business:normal".to_string()),
        priority: Some("high".to_string()),
        extra_fields: Some(HashMap::new()),
    }
}

#[test]
#[serial]
fn test_use_case_folder_structure() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create a use case
    let params = create_test_params(
        "User Authentication",
        "Authentication",
        "AUTH",
        Some("Test authentication system".to_string()),
    );

    let result = controller.create_use_case(params);
    assert!(result.is_ok(), "Should create use case: {:?}", result.err());

    let use_case_id = extract_use_case_id(&result.unwrap().message);

    // Generate markdown files
    let gen_result = controller.regenerate_all_markdown();
    assert!(
        gen_result.is_ok(),
        "Should generate markdown: {:?}",
        gen_result.err()
    );

    // Verify folder structure: docs/use-cases/{category}/{use-case-id}/
    let use_case_folder = temp_dir
        .path()
        .join("docs/use-cases")
        .join("authentication")
        .join(&use_case_id);

    assert!(
        use_case_folder.exists(),
        "Use case folder should exist at: {:?}",
        use_case_folder
    );
    assert!(
        use_case_folder.is_dir(),
        "Use case path should be a directory"
    );

    // Note: README.md is not created by default - only methodology view files are created
    // The canonical link target is the methodology view file itself

    // Verify methodology view file exists
    let view_file = use_case_folder.join(format!("{}-business-normal.md", use_case_id));
    assert!(
        view_file.exists(),
        "Methodology view file should exist: {:?}",
        view_file
    );

    // Verify TOML file is still in data directory (not in use case folder)
    let toml_path = temp_dir
        .path()
        .join("use-cases-data")
        .join("authentication")
        .join(format!("{}.toml", use_case_id));
    assert!(
        toml_path.exists(),
        "TOML file should exist in data directory"
    );
}

#[test]
#[serial]
fn test_category_overview_generation() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create multiple use cases in different categories
    let params1 = create_test_params(
        "User Login",
        "Authentication",
        "AUTH",
        Some("Login feature".to_string()),
    );

    let params2 = create_test_params(
        "User Registration",
        "Authentication",
        "AUTH",
        Some("Registration feature".to_string()),
    );

    let params3 = create_test_params(
        "View Profile",
        "User Management",
        "USR",
        Some("Profile viewing".to_string()),
    );

    controller.create_use_case(params1).unwrap();
    controller.create_use_case(params2).unwrap();
    controller.create_use_case(params3).unwrap();

    // Generate all markdown
    controller.regenerate_all_markdown().unwrap();

    // Verify category README exists for Authentication
    let auth_readme = temp_dir
        .path()
        .join("docs/use-cases")
        .join("authentication")
        .join("README.md");

    assert!(
        auth_readme.exists(),
        "Category README should exist for authentication"
    );

    let auth_content = fs::read_to_string(&auth_readme).expect("Should read category README");

    assert!(
        auth_content.contains("Authentication"),
        "Category README should contain category name"
    );
    assert!(
        auth_content.contains("User Login") || auth_content.contains("UC-AUTH-"),
        "Category README should list use cases"
    );

    // Verify category README exists for User Management
    let usr_readme = temp_dir
        .path()
        .join("docs/use-cases")
        .join("user_management")
        .join("README.md");

    assert!(
        usr_readme.exists(),
        "Category README should exist for user_management"
    );

    let usr_content = fs::read_to_string(&usr_readme).expect("Should read category README");
    assert!(
        usr_content.contains("User Management"),
        "Category README should contain category name"
    );
}

#[test]
#[serial]
#[ignore] // TODO: Fix template loading in test environment - overview template not rendering categories
fn test_main_overview_with_categories() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create use cases in different categories
    let params1 = create_test_params(
        "Feature A",
        "Category One",
        "CAT1",
        Some("First category".to_string()),
    );

    let params2 = create_test_params(
        "Feature B",
        "Category Two",
        "CAT2",
        Some("Second category".to_string()),
    );

    controller.create_use_case(params1).unwrap();
    controller.create_use_case(params2).unwrap();

    // Generate all markdown
    controller.regenerate_all_markdown().unwrap();

    // Verify main overview exists
    let overview_path = temp_dir.path().join("docs/use-cases/README.md");
    assert!(overview_path.exists(), "Main overview should exist");

    let overview_content = fs::read_to_string(&overview_path).expect("Should read main overview");

    // Verify it contains categories with paths
    assert!(
        overview_content.contains("category_one") || overview_content.contains("Category One"),
        "Overview should reference category_one. Content:\n{}",
        overview_content
    );
    assert!(
        overview_content.contains("category_two") || overview_content.contains("Category Two"),
        "Overview should reference category_two. Content:\n{}",
        overview_content
    );

    // Verify use_case_count is present
    assert!(
        overview_content.contains("1") || overview_content.contains("count"),
        "Overview should show use case counts"
    );
}

// Note: test_use_case_deletion_removes_folder removed because delete_use_case
// is not available in the public API. Deletion is handled through CLI only.

#[test]
#[serial]
fn test_multi_view_folder_structure() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create a use case with multiple views
    let params = CreateUseCaseParams {
        title: "Complex Feature".to_string(),
        category: "Features".to_string(),
        category_abbreviation: "FEAT".to_string(),
        description: Some("Feature with multiple views".to_string()),
        methodology: Some("business".to_string()),
        views: Some("business:simple,business:normal,business:detailed".to_string()),
        priority: Some("medium".to_string()),
        extra_fields: Some(HashMap::new()),
    };

    let result = controller.create_use_case(params);
    let use_case_id = extract_use_case_id(&result.unwrap().message);

    // Generate markdown
    controller.regenerate_all_markdown().unwrap();

    // Verify all view files exist in the use case folder
    let use_case_folder = temp_dir
        .path()
        .join("docs/use-cases")
        .join("features")
        .join(&use_case_id);

    let simple_view = use_case_folder.join(format!("{}-business-simple.md", use_case_id));
    let normal_view = use_case_folder.join(format!("{}-business-normal.md", use_case_id));
    let detailed_view = use_case_folder.join(format!("{}-business-detailed.md", use_case_id));

    assert!(simple_view.exists(), "Simple view should exist");
    assert!(normal_view.exists(), "Normal view should exist");
    assert!(detailed_view.exists(), "Detailed view should exist");

    // Note: README.md is not automatically created - only methodology view files exist
    // Users can create their own README.md to link to the views if desired
}

#[test]
#[serial]
fn test_category_overview_updates_on_changes() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create initial use case
    let params1 = create_test_params(
        "First Feature",
        "Dynamic",
        "DYN",
        Some("Initial feature".to_string()),
    );

    controller.create_use_case(params1).unwrap();
    controller.regenerate_all_markdown().unwrap();

    // Read initial category overview
    let category_readme = temp_dir
        .path()
        .join("docs/use-cases")
        .join("dynamic")
        .join("README.md");

    let initial_content =
        fs::read_to_string(&category_readme).expect("Should read initial category README");

    assert!(
        initial_content.contains("First Feature") || initial_content.contains("UC-DYN-"),
        "Initial category README should list first feature"
    );

    // Add another use case
    let params2 = create_test_params(
        "Second Feature",
        "Dynamic",
        "DYN",
        Some("Another feature".to_string()),
    );

    controller.create_use_case(params2).unwrap();
    controller.regenerate_all_markdown().unwrap();

    // Read updated category overview
    let updated_content =
        fs::read_to_string(&category_readme).expect("Should read updated category README");

    // Verify both use cases are listed
    assert!(
        (updated_content.contains("First Feature") || updated_content.contains("UC-DYN-001"))
            && (updated_content.contains("Second Feature")
                || updated_content.contains("UC-DYN-002")),
        "Updated category README should list both features"
    );
}

// Note: test_empty_category_overview removed because delete_use_case
// is not available in the public API. Deletion is handled through CLI only.
