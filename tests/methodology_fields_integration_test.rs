/// Integration tests for methodology-specific custom fields system
///
/// Tests the complete methodology field workflow from collection to cleanup.
use anyhow::Result;
use mucm::config::{Config, ConfigFileManager};
use mucm::core::{MethodologyFieldCollector, UseCaseCoordinator};
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use tempfile::TempDir;

mod common;

/// Helper to initialize test environment with methodologies
fn init_test_environment(methodologies: Vec<String>) -> Result<Config> {
    // Create minimal source-templates for testing
    common::create_minimal_source_templates(std::path::Path::new("."))?;

    let mut config = Config::default();
    config.templates.methodologies = methodologies.clone();
    if let Some(first) = methodologies.first() {
        config.templates.default_methodology = first.clone();
    }
    ConfigFileManager::save_in_dir(&config, ".")?;

    // Copy templates to config directory
    Config::copy_templates_to_config_with_language(None)?;

    Ok(config)
}

/// Test that methodology fields are collected and stored correctly
#[test]
#[serial]
fn test_methodology_fields_storage() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["business".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create use case with business view
    let result = service.create_use_case_with_views(
        "Test Business Use Case".to_string(),
        "testing".to_string(),
        Some("Testing methodology fields".to_string()),
        "business:advanced",
    )?;

    assert!(result.contains("UC-TES-001"));

    // Verify methodology_fields HashMap
    let use_case = service
        .get_all_use_cases()
        .iter()
        .find(|uc| uc.id == "UC-TES-001")
        .expect("Use case should exist");

    assert!(
        use_case.methodology_fields.contains_key("business"),
        "methodology_fields should contain business key (even if empty)"
    );

    // Verify the structure exists (fields may be empty if no values provided)
    assert!(
        use_case.methodology_fields.contains_key("business"),
        "Business methodology entry should exist in methodology_fields HashMap"
    );

    Ok(())
}

/// Test cleanup removes orphaned methodology fields
#[test]
#[serial]
fn test_cleanup_orphaned_fields() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["business".to_string(), "feature".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create use case with both views
    service.create_use_case_with_views(
        "Test Cleanup".to_string(),
        "testing".to_string(),
        None,
        "business:normal,feature:normal",
    )?;

    // Manually add orphaned field
    let use_cases = service.get_all_use_cases();
    let mut use_case = use_cases[0].clone();

    let mut orphaned_fields = HashMap::new();
    orphaned_fields.insert("orphaned_field".to_string(), serde_json::json!("test"));
    use_case
        .methodology_fields
        .insert("developer".to_string(), orphaned_fields);

    // Save modified use case
    let config = mucm::config::Config::load()?;
    let repository = mucm::core::RepositoryFactory::create(&config)?;
    repository.save(&use_case)?;

    // Reload and run cleanup
    let mut service = UseCaseCoordinator::load()?;
    let (cleaned_count, _total, details) =
        service.cleanup_methodology_fields(Some("UC-TES-001".to_string()), false)?;

    assert_eq!(cleaned_count, 1);
    assert_eq!(details[0].1, vec!["developer".to_string()]);

    // Verify cleanup
    let service = UseCaseCoordinator::load()?;
    let use_case = service.get_all_use_cases().first().unwrap();

    assert!(!use_case.methodology_fields.contains_key("developer"));
    assert!(use_case.methodology_fields.contains_key("business"));
    assert!(use_case.methodology_fields.contains_key("feature"));

    Ok(())
}

/// Test dry-run cleanup doesn't modify files
#[test]
#[serial]
fn test_cleanup_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["business".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create use case
    service.create_use_case_with_views(
        "Test Dry Run".to_string(),
        "testing".to_string(),
        None,
        "business:normal",
    )?;

    // Add orphaned field
    let use_cases = service.get_all_use_cases();
    let mut use_case = use_cases[0].clone();

    let mut orphaned_fields = HashMap::new();
    orphaned_fields.insert("orphaned".to_string(), serde_json::json!("value"));
    use_case
        .methodology_fields
        .insert("feature".to_string(), orphaned_fields);

    let config = mucm::config::Config::load()?;
    let repository = mucm::core::RepositoryFactory::create(&config)?;
    repository.save(&use_case)?;

    // Reload and run dry-run
    let mut service = UseCaseCoordinator::load()?;
    let (cleaned_count, _, details) = service.cleanup_methodology_fields(None, true)?;

    assert_eq!(cleaned_count, 1);
    assert_eq!(details[0].1, vec!["feature".to_string()]);

    // Verify nothing removed
    let service = UseCaseCoordinator::load()?;
    let use_case = service.get_all_use_cases().first().unwrap();

    assert!(
        use_case.methodology_fields.contains_key("feature"),
        "Dry run should not remove fields"
    );

    Ok(())
}

/// Test field inheritance (simple -> normal -> detailed)
#[test]
#[serial]
fn test_field_inheritance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["business".to_string()])?;
    let collector = MethodologyFieldCollector::new()?;

    // Normal level
    let normal_fields =
        collector.collect_fields_for_views(&[("business".to_string(), "normal".to_string())])?;
    let normal_count = normal_fields.fields.len();

    // Advanced level (inherits from normal)
    let advanced_fields =
        collector.collect_fields_for_views(&[("business".to_string(), "advanced".to_string())])?;
    let advanced_count = advanced_fields.fields.len();

    assert!(
        advanced_count > normal_count,
        "Advanced should have more fields than normal"
    );

    Ok(())
}

/// Test multi-methodology field storage
#[test]
#[serial]
fn test_multi_methodology_storage() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["business".to_string(), "feature".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create with multiple views
    service.create_use_case_with_views(
        "Multi-View".to_string(),
        "testing".to_string(),
        None,
        "business:normal,feature:normal",
    )?;

    let use_case = service.get_all_use_cases().first().unwrap();

    // Both methodologies present in the HashMap
    assert!(use_case.methodology_fields.contains_key("business"));
    assert!(use_case.methodology_fields.contains_key("feature"));

    // Verify structure exists (even if empty when no field values provided)
    assert!(use_case.methodology_fields.contains_key("business"));
    assert!(use_case.methodology_fields.contains_key("feature"));

    Ok(())
}

/// Test that ALL methodology-specific fields are present in TOML, even when empty
/// This ensures the TOML file structure is complete for all defined fields
#[test]
#[serial]
fn test_all_fields_present_in_toml_when_empty() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["developer".to_string(), "feature".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create use case with developer:normal and feature:normal WITHOUT filling any fields
    // This simulates the interactive mode where user skips optional fields
    service.create_use_case_with_views(
        "Test Empty Fields".to_string(),
        "testing".to_string(),
        Some("Testing that all fields are created".to_string()),
        "developer:normal,feature:normal",
    )?;

    let use_case = service
        .get_all_use_cases()
        .iter()
        .find(|uc| uc.id == "UC-TES-001")
        .expect("Use case should exist");

    // Verify developer methodology fields
    let developer_fields = use_case
        .methodology_fields
        .get("developer")
        .expect("Developer methodology should be present");

    // Developer:normal should have these fields defined in methodology.toml
    assert!(
        developer_fields.contains_key("api_endpoint"),
        "api_endpoint field should be present (even if empty)"
    );
    assert!(
        developer_fields.contains_key("database_tables"),
        "database_tables field should be present (even if empty)"
    );

    // Verify the fields are empty (no value provided)
    assert_eq!(
        developer_fields.get("api_endpoint"),
        Some(&serde_json::Value::String(String::new())),
        "api_endpoint should be empty string"
    );
    assert_eq!(
        developer_fields.get("database_tables"),
        Some(&serde_json::Value::Array(vec![])),
        "database_tables should be empty array"
    );

    // Verify feature methodology fields
    let feature_fields = use_case
        .methodology_fields
        .get("feature")
        .expect("Feature methodology should be present");

    // Feature:normal should have these fields defined in methodology.toml
    assert!(
        feature_fields.contains_key("user_segment"),
        "user_segment field should be present (even if empty)"
    );
    assert!(
        feature_fields.contains_key("success_metrics"),
        "success_metrics field should be present (even if empty)"
    );
    assert!(
        feature_fields.contains_key("hypothesis"),
        "hypothesis field should be present (even if empty)"
    );

    // Verify the fields are empty
    assert_eq!(
        feature_fields.get("user_segment"),
        Some(&serde_json::Value::String(String::new())),
        "user_segment should be empty string"
    );
    assert_eq!(
        feature_fields.get("success_metrics"),
        Some(&serde_json::Value::Array(vec![])),
        "success_metrics should be empty array"
    );
    assert_eq!(
        feature_fields.get("hypothesis"),
        Some(&serde_json::Value::String(String::new())),
        "hypothesis should be empty string"
    );

    Ok(())
}

/// Test that advanced level includes all inherited fields plus its own fields
#[test]
#[serial]
fn test_advanced_level_has_all_fields_including_inherited() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_current_dir(&temp_dir)?;

    init_test_environment(vec!["developer".to_string()])?;
    let mut service = UseCaseCoordinator::load()?;

    // Create use case with developer:advanced (inherits from normal)
    service.create_use_case_with_views(
        "Test Advanced Fields".to_string(),
        "testing".to_string(),
        None,
        "developer:advanced",
    )?;

    let use_case = service
        .get_all_use_cases()
        .iter()
        .find(|uc| uc.id == "UC-TES-001")
        .expect("Use case should exist");

    let developer_fields = use_case
        .methodology_fields
        .get("developer")
        .expect("Developer methodology should be present");

    // Check inherited fields from normal level
    assert!(
        developer_fields.contains_key("api_endpoint"),
        "Inherited field api_endpoint should be present"
    );
    assert!(
        developer_fields.contains_key("database_tables"),
        "Inherited field database_tables should be present"
    );

    // Check advanced-level specific fields
    assert!(
        developer_fields.contains_key("performance_requirements"),
        "Advanced field performance_requirements should be present"
    );
    assert!(
        developer_fields.contains_key("security_considerations"),
        "Advanced field security_considerations should be present"
    );
    assert!(
        developer_fields.contains_key("technical_dependencies"),
        "Advanced field technical_dependencies should be present"
    );
    assert!(
        developer_fields.contains_key("error_handling"),
        "Advanced field error_handling should be present"
    );

    // Verify all are empty (no values provided)
    assert_eq!(
        developer_fields.get("performance_requirements"),
        Some(&serde_json::Value::String(String::new()))
    );
    assert_eq!(
        developer_fields.get("technical_dependencies"),
        Some(&serde_json::Value::Array(vec![]))
    );

    Ok(())
}
