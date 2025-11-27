//! Integration tests for persona management workflows
//!
//! Tests the complete end-to-end flow of persona CRUD operations using the
//! unified actor system. Personas are now a specialized actor type stored
//! in the actors repository.
//!
//! Tests cover:
//! - Creating personas with minimal and full field sets
//! - Updating persona names and custom fields
//! - Listing and retrieving personas
//! - Deleting personas
//! - TOML persistence verification in data/actors/ directory
//! - Custom field type handling (string, number, boolean, array)
//! - Sommerville-aligned persona fields (via config)

use mucm::controller::PersonaController;
use serial_test::serial;
use std::{collections::HashMap, env, fs};
use tempfile::TempDir;

/// Test helper: Setup test environment with persona configuration
fn setup_test_env() -> (TempDir, PersonaController) {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    // Create config with persona custom fields
    let config_content = r#"
[project]
name = "Test Project"
description = "Test project for persona integration tests"

[directories]
use_case_dir = "docs/use-cases"
test_dir = "tests"
persona_dir = "docs/personas"
data_dir = "data"

[templates]
methodologies = ["business", "developer"]
default_methodology = "business"

[generation]
test_language = "rust"
auto_generate_tests = false
overwrite_test_documentation = false

[storage]
backend = "toml"

[metadata]
created = true
last_updated = true

[actor.persona_fields]
department = { type = "string", required = false }
role = { type = "string", required = false }
experience_level = { type = "string", required = false }
years_experience = { type = "number", required = false }
is_manager = { type = "boolean", required = false }
skills = { type = "array", required = false }
goals = { type = "array", required = false }
pain_points = { type = "array", required = false }
"#;

    fs::create_dir_all(temp_dir.path().join(".config/.mucm")).unwrap();
    fs::write(
        temp_dir.path().join(".config/.mucm/mucm.toml"),
        config_content,
    )
    .unwrap();

    // Copy templates to config directory (even though personas don't use methodologies)
    // This ensures the full environment is set up correctly
    use mucm::config::Config;
    Config::copy_templates_to_config_with_language(None).unwrap();

    let controller = PersonaController::new().unwrap();
    (temp_dir, controller)
}

/// Test helper: Read persona TOML file
fn read_persona_toml(temp_dir: &TempDir, persona_id: &str) -> String {
    // With unified actor system, personas are stored in {data_dir}/actors/{persona_id}.toml
    let toml_path = temp_dir
        .path()
        .join("data")
        .join("actors")
        .join(format!("{}.toml", persona_id));
    fs::read_to_string(toml_path).expect("Failed to read persona TOML file")
}

#[test]
#[serial]
fn test_complete_persona_lifecycle() {
    let (temp_dir, controller) = setup_test_env();

    // Step 1: Create persona with minimal fields
    let create_result = controller
        .create_persona(
            "john_dev".to_string(),
            "John Developer".to_string(),
            "Software Developer".to_string(),
        )
        .unwrap();

    assert!(create_result.success);
    assert!(create_result.message.contains("Created persona"));

    // Verify persona exists
    let persona = controller.get_persona("john_dev").unwrap();
    assert_eq!(persona.id, "john_dev");
    assert_eq!(persona.name, "John Developer");
    // Note: extra may contain config-defined default fields

    // Step 2: Update persona name
    let update_result = controller
        .update_persona(
            "john_dev".to_string(),
            Some("John Senior Developer".to_string()),
        )
        .unwrap();

    assert!(update_result.success);

    let persona = controller.get_persona("john_dev").unwrap();
    assert_eq!(persona.name, "John Senior Developer");

    // Step 3: Add custom fields
    let mut fields = HashMap::new();
    fields.insert("department".to_string(), "Engineering".to_string());
    fields.insert("role".to_string(), "Senior Developer".to_string());
    fields.insert("years_experience".to_string(), "8".to_string());
    fields.insert("is_manager".to_string(), "false".to_string());
    fields.insert("skills".to_string(), "Rust\nPython\nDocker".to_string());

    let update_fields_result = controller
        .update_persona_fields("john_dev".to_string(), fields)
        .unwrap();

    assert!(update_fields_result.success);

    // Verify custom fields
    let persona = controller.get_persona("john_dev").unwrap();
    assert_eq!(
        persona.extra.get("department"),
        Some(&serde_json::Value::String("Engineering".to_string()))
    );
    assert_eq!(
        persona.extra.get("years_experience"),
        Some(&serde_json::json!(8.0))
    );
    assert_eq!(
        persona.extra.get("is_manager"),
        Some(&serde_json::Value::Bool(false))
    );

    // Verify array field
    let skills = persona.extra.get("skills").unwrap();
    assert!(skills.is_array());

    // Verify TOML persistence
    let toml_content = read_persona_toml(&temp_dir, "john_dev");
    assert!(toml_content.contains("John Senior Developer"));
    assert!(toml_content.contains("Engineering"));
    assert!(toml_content.contains("years_experience"));

    // Step 4: Delete persona
    let delete_result = controller.delete_persona("john_dev".to_string()).unwrap();
    assert!(delete_result.success);

    // Verify deletion
    assert!(controller.get_persona("john_dev").is_err());

    println!("✅ Complete persona lifecycle test passed");
}

#[test]
#[serial]
fn test_multiple_personas_management() {
    let (_temp_dir, controller) = setup_test_env();

    // Create multiple personas
    controller
        .create_persona(
            "alice_pm".to_string(),
            "Alice Manager".to_string(),
            "Project Manager".to_string(),
        )
        .unwrap();

    controller
        .create_persona(
            "bob_dev".to_string(),
            "Bob Developer".to_string(),
            "Backend Developer".to_string(),
        )
        .unwrap();

    controller
        .create_persona(
            "carol_test".to_string(),
            "Carol Tester".to_string(),
            "QA Tester".to_string(),
        )
        .unwrap();

    // List all personas
    let personas = controller.list_personas().unwrap();
    assert_eq!(personas.len(), 3);

    let names: Vec<String> = personas.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"Alice Manager".to_string()));
    assert!(names.contains(&"Bob Developer".to_string()));
    assert!(names.contains(&"Carol Tester".to_string()));

    // Get persona IDs
    let ids = controller.get_persona_ids().unwrap();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"alice_pm".to_string()));
    assert!(ids.contains(&"bob_dev".to_string()));
    assert!(ids.contains(&"carol_test".to_string()));

    // Update one persona
    let mut fields = HashMap::new();
    fields.insert("department".to_string(), "Quality Assurance".to_string());
    controller
        .update_persona_fields("carol_test".to_string(), fields)
        .unwrap();

    // Verify update worked on target persona
    let carol = controller.get_persona("carol_test").unwrap();
    assert_eq!(
        carol.extra.get("department"),
        Some(&serde_json::Value::String("Quality Assurance".to_string()))
    );

    println!("✅ Multiple personas management test passed");
}

#[test]
#[serial]
fn test_custom_field_type_handling() {
    let (_temp_dir, controller) = setup_test_env();

    controller
        .create_persona(
            "type_test".to_string(),
            "Type Test User".to_string(),
            "Test User".to_string(),
        )
        .unwrap();

    // Test all field types
    let mut fields = HashMap::new();

    // String fields
    fields.insert("department".to_string(), "Sales".to_string());
    fields.insert("role".to_string(), "Account Manager".to_string());

    // Number field
    fields.insert("years_experience".to_string(), "12".to_string());

    // Boolean fields
    fields.insert("is_manager".to_string(), "true".to_string());

    // Array field (newline-separated)
    fields.insert(
        "skills".to_string(),
        "Negotiation\nCustomer Relations\nSales Strategy".to_string(),
    );

    fields.insert(
        "goals".to_string(),
        "Increase revenue\nExpand client base".to_string(),
    );

    controller
        .update_persona_fields("type_test".to_string(), fields)
        .unwrap();

    // Verify type conversions
    let persona = controller.get_persona("type_test").unwrap();

    // String types
    assert!(persona.extra.get("department").unwrap().is_string());
    assert!(persona.extra.get("role").unwrap().is_string());

    // Number type
    assert!(persona.extra.get("years_experience").unwrap().is_number());
    assert_eq!(
        persona.extra.get("years_experience"),
        Some(&serde_json::json!(12.0))
    );

    // Boolean type
    assert!(persona.extra.get("is_manager").unwrap().is_boolean());
    assert_eq!(
        persona.extra.get("is_manager"),
        Some(&serde_json::Value::Bool(true))
    );

    // Array types
    assert!(persona.extra.get("skills").unwrap().is_array());
    assert!(persona.extra.get("goals").unwrap().is_array());

    let skills_array = persona.extra.get("skills").unwrap().as_array().unwrap();
    assert_eq!(skills_array.len(), 3);
    assert_eq!(skills_array[0], "Negotiation");

    println!("✅ Custom field type handling test passed");
}

#[test]
#[serial]
fn test_persona_field_config_and_values() {
    let (_temp_dir, controller) = setup_test_env();

    // Get field configuration
    let field_config = controller.get_persona_field_config();

    // Verify expected fields are configured
    assert!(field_config.contains_key("department"));
    assert!(field_config.contains_key("role"));
    assert!(field_config.contains_key("experience_level"));
    assert!(field_config.contains_key("years_experience"));
    assert!(field_config.contains_key("is_manager"));
    assert!(field_config.contains_key("skills"));
    assert!(field_config.contains_key("goals"));
    assert!(field_config.contains_key("pain_points"));

    // Create persona with some fields
    controller
        .create_persona(
            "config_test".to_string(),
            "Config Test".to_string(),
            "Test Role".to_string(),
        )
        .unwrap();

    let mut fields = HashMap::new();
    fields.insert("department".to_string(), "Marketing".to_string());
    fields.insert("role".to_string(), "Content Writer".to_string());
    fields.insert("years_experience".to_string(), "3".to_string());

    controller
        .update_persona_fields("config_test".to_string(), fields)
        .unwrap();

    // Get field values - returns all fields (including config defaults)
    let field_values = controller.get_persona_field_values("config_test").unwrap();

    // Verify at least the fields we set are present with correct values
    assert!(field_values.len() >= 3);
    assert_eq!(
        field_values.get("department"),
        Some(&serde_json::Value::String("Marketing".to_string()))
    );
    assert_eq!(
        field_values.get("role"),
        Some(&serde_json::Value::String("Content Writer".to_string()))
    );
    assert_eq!(
        field_values.get("years_experience"),
        Some(&serde_json::json!(3.0))
    );

    println!("✅ Persona field config and values test passed");
}

#[test]
#[serial]
fn test_persona_duplicate_prevention() {
    let (_temp_dir, controller) = setup_test_env();

    // Create first persona
    let result1 = controller
        .create_persona(
            "duplicate_test".to_string(),
            "First User".to_string(),
            "User Role".to_string(),
        )
        .unwrap();
    assert!(result1.success);

    // Try to create duplicate
    let result2 = controller
        .create_persona(
            "duplicate_test".to_string(),
            "Second User".to_string(),
            "Another Role".to_string(),
        )
        .unwrap();
    assert!(!result2.success);
    assert!(result2.message.contains("already exists") || result2.message.contains("duplicate"));

    // Verify only one persona exists
    let personas = controller.list_personas().unwrap();
    let duplicates: Vec<_> = personas
        .iter()
        .filter(|p| p.id == "duplicate_test")
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].name, "First User");

    println!("✅ Persona duplicate prevention test passed");
}

#[test]
#[serial]
fn test_persona_field_updates_preserve_existing() {
    let (_temp_dir, controller) = setup_test_env();

    controller
        .create_persona(
            "update_test".to_string(),
            "Update Test".to_string(),
            "Update Role".to_string(),
        )
        .unwrap();

    // Add initial fields
    let mut fields1 = HashMap::new();
    fields1.insert("department".to_string(), "Engineering".to_string());
    fields1.insert("role".to_string(), "Developer".to_string());
    fields1.insert("years_experience".to_string(), "5".to_string());

    controller
        .update_persona_fields("update_test".to_string(), fields1)
        .unwrap();

    // Update only some fields
    let mut fields2 = HashMap::new();
    fields2.insert("role".to_string(), "Senior Developer".to_string());
    fields2.insert("is_manager".to_string(), "true".to_string());

    controller
        .update_persona_fields("update_test".to_string(), fields2)
        .unwrap();

    // Verify: role updated, department preserved, new field added
    let persona = controller.get_persona("update_test").unwrap();

    assert_eq!(
        persona.extra.get("department"),
        Some(&serde_json::Value::String("Engineering".to_string()))
    );
    assert_eq!(
        persona.extra.get("role"),
        Some(&serde_json::Value::String("Senior Developer".to_string()))
    );
    assert_eq!(
        persona.extra.get("years_experience"),
        Some(&serde_json::json!(5.0))
    );
    assert_eq!(
        persona.extra.get("is_manager"),
        Some(&serde_json::Value::Bool(true))
    );

    println!("✅ Persona field updates preserve existing test passed");
}

#[test]
#[serial]
fn test_persona_error_handling() {
    let (_temp_dir, controller) = setup_test_env();

    // Try to get non-existent persona
    let result = controller.get_persona("nonexistent");
    assert!(result.is_err());

    // Try to update non-existent persona
    let result = controller.update_persona("nonexistent".to_string(), Some("Test".to_string()));
    assert!(result.is_err() || !result.unwrap().success);

    // Try to delete non-existent persona
    let result = controller
        .delete_persona("nonexistent".to_string())
        .unwrap();
    assert!(!result.success);
    assert!(result.message.contains("not found") || result.message.contains("doesn't exist"));

    // Try to get field values for non-existent persona
    let result = controller.get_persona_field_values("nonexistent");
    assert!(result.is_err());

    println!("✅ Persona error handling test passed");
}
