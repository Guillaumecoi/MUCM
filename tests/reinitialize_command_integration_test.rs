/// Integration tests for issue #29: Reinitialize command behavior
///
/// Tests the expected behaviors of field initialization and management.
/// These tests verify that the domain model supports the reinitialize command's requirements.
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

/// Test that methodology fields can be added to use cases
#[test]
#[serial]
fn test_add_methodology_fields_to_use_case() {
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

    // Add methodology fields
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("High value"));
    business_fields.insert("stakeholders".to_string(), json!(["CEO", "CTO"]));
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository.save(&use_case).unwrap();

    // Reload and verify
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-001").unwrap();

    assert!(uc.methodology_fields.contains_key("business"));
    let fields = uc.methodology_fields.get("business").unwrap();
    assert_eq!(fields["business_value"], json!("High value"));
    assert_eq!(fields["stakeholders"], json!(["CEO", "CTO"]));
}

/// Test that empty fields can be initialized
#[test]
#[serial]
fn test_initialize_empty_fields() {
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

    // Initialize with empty values
    let mut dev_fields = HashMap::new();
    dev_fields.insert("implementation_notes".to_string(), json!(""));
    dev_fields.insert("technical_dependencies".to_string(), json!([]));
    dev_fields.insert("complexity_score".to_string(), json!(0));
    use_case
        .methodology_fields
        .insert("developer".to_string(), dev_fields);

    repository.save(&use_case).unwrap();

    // Verify empty values persist correctly
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-002").unwrap();

    let fields = uc.methodology_fields.get("developer").unwrap();
    assert_eq!(fields["implementation_notes"], json!(""));
    assert_eq!(fields["technical_dependencies"], json!([]));
    assert_eq!(fields["complexity_score"], json!(0));
}

/// Test that fields can be updated without losing existing values
#[test]
#[serial]
fn test_update_fields_preserves_existing() {
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
        .push(MethodologyView::new("business", "normal"));

    // Add initial fields
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("Original value"));
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    repository.save(&use_case).unwrap();

    // Load, add a new field, save again
    let mut loaded = repository.load_all().unwrap();
    let uc = loaded.iter_mut().find(|u| u.id == "UC-003").unwrap();

    let fields = uc.methodology_fields.get_mut("business").unwrap();
    fields.insert("stakeholders".to_string(), json!(["PM"]));

    repository.save(uc).unwrap();

    // Reload and verify both fields exist
    let reloaded = repository.load_all().unwrap();
    let uc = reloaded.iter().find(|u| u.id == "UC-003").unwrap();
    let fields = uc.methodology_fields.get("business").unwrap();

    assert_eq!(fields["business_value"], json!("Original value"));
    assert_eq!(fields["stakeholders"], json!(["PM"]));
}

/// Test multiple views with separate field sets
#[test]
#[serial]
fn test_multiple_views_separate_fields() {
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

    // Add two views
    use_case
        .views
        .push(MethodologyView::new("business", "simple"));
    use_case
        .views
        .push(MethodologyView::new("developer", "normal"));

    // Add fields for both methodologies
    let mut business_fields = HashMap::new();
    business_fields.insert("business_value".to_string(), json!("High"));
    use_case
        .methodology_fields
        .insert("business".to_string(), business_fields);

    let mut dev_fields = HashMap::new();
    dev_fields.insert("implementation_notes".to_string(), json!("Use Redis"));
    use_case
        .methodology_fields
        .insert("developer".to_string(), dev_fields);

    repository.save(&use_case).unwrap();

    // Verify both methodology fields persist
    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-004").unwrap();

    assert!(uc.methodology_fields.contains_key("business"));
    assert!(uc.methodology_fields.contains_key("developer"));
}

/// Test use case with no methodology fields
#[test]
#[serial]
fn test_use_case_without_methodology_fields() {
    let (_temp_dir, repository) = create_test_environment();

    let use_case = UseCase::new(
        "UC-005".to_string(),
        "Test Use Case".to_string(),
        "Test".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    // Use case without explicit methodology fields
    repository.save(&use_case).unwrap();

    let loaded = repository.load_all().unwrap();
    let uc = loaded.iter().find(|u| u.id == "UC-005").unwrap();

    // Use case is created successfully
    assert_eq!(uc.id, "UC-005");
    assert_eq!(uc.title, "Test Use Case");
}
