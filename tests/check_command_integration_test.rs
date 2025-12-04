/// Integration tests for issue #30: Check/validate command behavior
///
/// Tests the expected behaviors of field validation.
/// These tests verify that the domain model supports the check command's requirements.
use mucm::core::{MethodologyView, TomlUseCaseRepository, UseCase, UseCaseRepository};
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use tempfile::TempDir;

mod common;

/// Helper to create a test repository and config
fn create_test_environment() -> (TempDir, Box<dyn UseCaseRepository>) {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    // Set up isolated test templates
    let _template_mgr = common::TestTemplateManager::new().unwrap();

    // Initialize config
    use mucm::config::ConfigFileManager;
    let config = mucm::config::Config::default();
    ConfigFileManager::save_in_dir(&config, ".").unwrap();

    let config = mucm::config::Config::load().unwrap();
    let repo = Box::new(TomlUseCaseRepository::new(config)) as Box<dyn UseCaseRepository>;

    (temp_dir, repo)
}

/// Test that use cases can store empty field values
#[test]
#[serial]
fn test_use_case_with_empty_fields() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-001".to_string(),
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

    // Add fields with empty values (which would be flagged by validation)
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("")); // Empty string
    business_fields.insert("stakeholders".to_string(), json!([])); // Empty array
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository.save(&use_case).unwrap();

    // Verify empty values persist
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-001").unwrap();

    let fields = uc.methodology_fields.get("business").unwrap();
    assert_eq!(fields["business_value"], json!(""));
    assert_eq!(fields["stakeholders"], json!([]));
}

/// Test that use cases can have non-standard fields
#[test]
#[serial]
fn test_use_case_with_custom_fields() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-002".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case
        .views
        .push(MethodologyView::new("developer", "normal"));

    // Add fields that might not be in methodology definition (irrelevant fields)
    let mut dev_fields = HashMap::new();
    dev_fields.insert("custom_field_xyz".to_string(), json!("Custom value"));
    dev_fields.insert("nonstandard_field".to_string(), json!("Another value"));
    use_case
        .methodology_fields
        .insert("developer".to_string(), dev_fields);

    repository.save(&use_case).unwrap();

    // Verify custom fields persist
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-002").unwrap();

    let fields = uc.methodology_fields.get("developer").unwrap();
    assert!(fields.contains_key("custom_field_xyz"));
    assert!(fields.contains_key("nonstandard_field"));
}

/// Test that use cases with proper field values can be validated
#[test]
#[serial]
fn test_use_case_with_valid_fields() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-003".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case
        .views
        .push(MethodologyView::new("business", "simple"));

    // Add properly filled fields
    let mut business_fields = HashMap::new();
    business_fields.insert(
        "business_value".to_string(),
        json!("Enables faster customer onboarding"),
    );
    business_fields.insert(
        "stakeholders".to_string(),
        json!(["Product Manager", "CEO"]),
    );
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository.save(&use_case).unwrap();

    // Verify fields are properly stored
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-003").unwrap();

    let fields = uc.methodology_fields.get("business").unwrap();
    assert!(!fields["business_value"].as_str().unwrap().is_empty());
    assert!(!fields["stakeholders"].as_array().unwrap().is_empty());
}

/// Test detecting missing methodology sections
#[test]
#[serial]
fn test_use_case_missing_methodology_section() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-004".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    // Has view but no methodology fields
    use_case
        .views
        .push(MethodologyView::new("tester", "normal"));
    // Not adding any methodology fields

    repository.save(&use_case).unwrap();

    // Verify the section is missing
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-004").unwrap();

    assert!(!uc.methodology_fields.contains_key("tester"));
}

/// Test use case with multiple views and varying field completeness
#[test]
#[serial]
fn test_use_case_multiple_views_partial_fields() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-005".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    // Two views
    use_case
        .views
        .push(MethodologyView::new("business", "simple"));
    use_case
        .views
        .push(MethodologyView::new("developer", "normal"));

    // Only add fields for business, not developer
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("High"));
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository.save(&use_case).unwrap();

    // Verify: has business but missing developer
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-005").unwrap();

    assert!(uc.methodology_fields.contains_key("business"));
    assert!(!uc.methodology_fields.contains_key("developer"));
}

/// Test that field values of different types are handled
#[test]
#[serial]
fn test_field_types_validation() {
    let (_temp_dir, repository) = create_test_environment();

    let mut use_case = UseCase::new(
        "UC-006".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case
        .views
        .push(MethodologyView::new("developer", "normal"));

    // Add fields of different types
    let mut dev_fields = HashMap::new();
    dev_fields.insert("string_field".to_string(), json!("text"));
    dev_fields.insert("number_field".to_string(), json!(42));
    dev_fields.insert("boolean_field".to_string(), json!(true));
    dev_fields.insert("array_field".to_string(), json!(["item1", "item2"]));
    use_case
        .methodology_fields
        .insert("developer".to_string(), dev_fields);

    repository.save(&use_case).unwrap();

    // Verify different types persist
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-006").unwrap();

    let fields = uc.methodology_fields.get("developer").unwrap();
    assert!(fields["string_field"].is_string());
    assert!(fields["number_field"].is_number());
    assert!(fields["boolean_field"].is_boolean());
    assert!(fields["array_field"].is_array());
}
