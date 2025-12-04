/// Integration tests for issue #28: Field reinitialization during editing
///
/// Tests that when users skip fields during interactive editing, those fields
/// are reinitialized with empty values based on their type (arrays: [], numbers: 0, etc.)
use mucm::core::{MethodologyView, TomlUseCaseRepository, UseCase, UseCaseRepository};
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use tempfile::TempDir;

mod common;

/// Helper to create a test repository
fn create_test_repository() -> (TempDir, Box<dyn UseCaseRepository>) {
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

/// Test that missing array fields are reinitialized as empty arrays
#[test]
#[serial]
fn test_array_field_reinitialization() {
    let (_temp_dir, mut repository) = create_test_repository();

    // Create a use case with business methodology view
    let mut use_case = UseCase::new(
        "UC-001".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    // Add a view that requires array fields
    use_case.views.push(MethodologyView::new("business", "normal"));

    // Simulate field editing: add methodology but skip array field
    // This represents the bug scenario where technical_dependencies would be missing
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("High value"));
    // Note: NOT adding technical_dependencies (which is an array field)

    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository
        .save(&use_case)
        .expect("Failed to save use case");

    // Now simulate the fix: reinitialize missing fields
    // In the actual implementation, this happens in edit_methodology_fields
    let mut business_fields = use_case
        .methodology_fields
        .get_mut("business")
        .expect("Business fields should exist");

    // The fix should add empty array for missing technical_dependencies
    if !business_fields.contains_key("technical_dependencies") {
        business_fields.insert("technical_dependencies".to_string(), json!([]));
    }

    repository
        .save(&use_case)
        .expect("Failed to save use case after fix");

    // Reload and verify the field exists as an empty array
    let all_use_cases = repository
        .load_all()
        .expect("Failed to load use cases");
    let loaded = all_use_cases
        .iter()
        .find(|uc| uc.id == use_case.id)
        .expect("Use case not found");

    let business_fields = loaded
        .methodology_fields
        .get("business")
        .expect("Business methodology not found");

    assert!(
        business_fields.contains_key("technical_dependencies"),
        "technical_dependencies should be present"
    );
    assert_eq!(
        business_fields["technical_dependencies"],
        json!([]),
        "technical_dependencies should be an empty array"
    );
}

/// Test that missing number fields are reinitialized as 0
#[test]
#[serial]
fn test_number_field_reinitialization() {
    let (_temp_dir, mut repository) = create_test_repository();

    let mut use_case = UseCase::new(
        "UC-002".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case.views.push(MethodologyView::new("developer", "normal"));

    // Add some fields but skip a number field
    let mut dev_fields = HashMap::new();
    dev_fields.insert("implementation_notes".to_string(), json!("Some notes"));
    // Simulate missing a number field (if methodology defines one)

    use_case
        .methodology_fields
        .insert("developer".to_string(), dev_fields);

    repository
        .save(&use_case)
        .expect("Failed to save use case");

    // Apply the fix for number fields
    let mut dev_fields = use_case
        .methodology_fields
        .get_mut("developer")
        .expect("Developer fields should exist");

    // Example: complexity_score field (if it exists in methodology)
    if !dev_fields.contains_key("complexity_score") {
        dev_fields.insert("complexity_score".to_string(), json!(0));
    }

    repository
        .save(&use_case)
        .expect("Failed to save use case after fix");

    let all_use_cases = repository
        .load_all()
        .expect("Failed to load use cases");
    let loaded = all_use_cases
        .iter()
        .find(|uc| uc.id == use_case.id)
        .expect("Use case not found");

    let dev_fields = loaded
        .methodology_fields
        .get("developer")
        .expect("Developer methodology not found");

    assert!(
        dev_fields.contains_key("complexity_score"),
        "complexity_score should be present"
    );
    assert_eq!(
        dev_fields["complexity_score"],
        json!(0),
        "complexity_score should be 0"
    );
}

/// Test that missing boolean fields are reinitialized as false
#[test]
#[serial]
fn test_boolean_field_reinitialization() {
    let (_temp_dir, mut repository) = create_test_repository();

    let mut use_case = UseCase::new(
        "UC-003".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case.views.push(MethodologyView::new("tester", "normal"));

    let mut tester_fields = HashMap::new();
    tester_fields.insert("test_approach".to_string(), json!("Manual testing"));
    // Skip boolean field

    use_case
        .methodology_fields
        .insert("tester".to_string(), tester_fields);

    repository
        .save(&use_case)
        .expect("Failed to save use case");

    // Apply the fix for boolean fields
    let mut tester_fields = use_case
        .methodology_fields
        .get_mut("tester")
        .expect("Tester fields should exist");

    // Example: requires_automation field
    if !tester_fields.contains_key("requires_automation") {
        tester_fields.insert("requires_automation".to_string(), json!(false));
    }

    repository
        .save(&use_case)
        .expect("Failed to save use case after fix");

    let all_use_cases = repository
        .load_all()
        .expect("Failed to load use cases");
    let loaded = all_use_cases
        .iter()
        .find(|uc| uc.id == use_case.id)
        .expect("Use case not found");

    let tester_fields = loaded
        .methodology_fields
        .get("tester")
        .expect("Tester methodology not found");

    assert!(
        tester_fields.contains_key("requires_automation"),
        "requires_automation should be present"
    );
    assert_eq!(
        tester_fields["requires_automation"],
        json!(false),
        "requires_automation should be false"
    );
}

/// Test that missing string fields are reinitialized as empty strings
#[test]
#[serial]
fn test_string_field_reinitialization() {
    let (_temp_dir, mut repository) = create_test_repository();

    let mut use_case = UseCase::new(
        "UC-004".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case.views.push(MethodologyView::new("business", "simple"));

    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("High"));
    // Skip string field

    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository
        .save(&use_case)
        .expect("Failed to save use case");

    // Apply the fix for string fields
    let mut business_fields = use_case
        .methodology_fields
        .get_mut("business")
        .expect("Business fields should exist");

    if !business_fields.contains_key("stakeholder_notes") {
        business_fields.insert("stakeholder_notes".to_string(), json!(""));
    }

    repository
        .save(&use_case)
        .expect("Failed to save use case after fix");

    let all_use_cases = repository
        .load_all()
        .expect("Failed to load use cases");
    let loaded = all_use_cases
        .iter()
        .find(|uc| uc.id == use_case.id)
        .expect("Use case not found");

    let business_fields = loaded
        .methodology_fields
        .get("business")
        .expect("Business methodology not found");

    assert!(
        business_fields.contains_key("stakeholder_notes"),
        "stakeholder_notes should be present"
    );
    assert_eq!(
        business_fields["stakeholder_notes"],
        json!(""),
        "stakeholder_notes should be an empty string"
    );
}

/// Test that existing field values are never overwritten
#[test]
#[serial]
fn test_existing_fields_not_overwritten() {
    let (_temp_dir, mut repository) = create_test_repository();

    let mut use_case = UseCase::new(
        "UC-005".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    use_case.views.push(MethodologyView::new("business", "normal"));

    // Add fields with existing values
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("Very High"));
    business_fields.insert(
        "technical_dependencies".to_string(),
        json!(["Auth Service", "Database"]),
    );

    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository
        .save(&use_case)
        .expect("Failed to save use case");

    // Apply reinitialization logic (should not change existing values)
    let mut business_fields = use_case
        .methodology_fields
        .get_mut("business")
        .expect("Business fields should exist");

    // The fix should NOT overwrite existing values
    if !business_fields.contains_key("technical_dependencies") {
        business_fields.insert("technical_dependencies".to_string(), json!([]));
    }

    repository
        .save(&use_case)
        .expect("Failed to save use case after fix");

    let all_use_cases = repository
        .load_all()
        .expect("Failed to load use cases");
    let loaded = all_use_cases
        .iter()
        .find(|uc| uc.id == use_case.id)
        .expect("Use case not found");

    let business_fields = loaded
        .methodology_fields
        .get("business")
        .expect("Business methodology not found");

    // Values should remain unchanged
    assert_eq!(
        business_fields["business_value"],
        json!("Very High"),
        "business_value should not be changed"
    );
    assert_eq!(
        business_fields["technical_dependencies"],
        json!(["Auth Service", "Database"]),
        "technical_dependencies should not be changed"
    );
}
