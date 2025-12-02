//! Integration tests for use case editing workflows
//!
//! Tests the complete end-to-end flow of editing use cases, including:
//! - Updating basic fields (title, description, category, priority)
//! - Updating methodology-specific fields
//! - Adding and removing methodology views
//! - Markdown regeneration after edits
//! - TOML persistence verification

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

    // Set CARGO_MANIFEST_DIR for template discovery
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    env::set_var("CARGO_MANIFEST_DIR", project_root);

    // Create minimal source-templates for testing
    common::create_minimal_source_templates(std::path::Path::new(".")).unwrap();
    
    // Create default config
    let config = Config::default();
    ConfigFileManager::save_in_dir(&config, ".").unwrap();

    // Copy templates to config directory
    Config::copy_templates_to_config_with_language(None).unwrap();

    let controller = UseCaseController::new().unwrap();
    (temp_dir, controller)
}

/// Test helper: Extract use case ID from controller result message
fn extract_use_case_id(message: &str) -> String {
    message
        .split_whitespace()
        .find(|s| s.starts_with("UC-"))
        .expect("Should have a use case ID in the message")
        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-')
        .to_string()
}

/// Test helper: Create use case params for testing
fn create_test_params(
    title: &str,
    category: &str,
    abbrev: &str,
    description: Option<String>,
    methodology: Option<String>,
    views: Option<String>,
    priority: Option<String>,
    extra_fields: Option<HashMap<String, String>>,
) -> CreateUseCaseParams {
    CreateUseCaseParams {
        title: title.to_string(),
        category: category.to_string(),
        category_abbreviation: abbrev.to_string(),
        description,
        methodology,
        views,
        priority,
        extra_fields,
    }
}

/// Test helper: Read TOML file content
fn read_toml_file(temp_dir: &TempDir, use_case_id: &str, category: &str) -> String {
    // TOML files are stored in data_dir/category/UC-XXX-nnn.toml
    let toml_path = temp_dir
        .path()
        .join("use-cases-data")
        .join(category)
        .join(format!("{}.toml", use_case_id));
    fs::read_to_string(toml_path).expect("Failed to read TOML file")
}

/// Test helper: Read markdown file content
/// Files are named UC-XXX-nnn-methodology-level.md (e.g., UC-TES-001-business-normal.md)
fn read_markdown_file(
    temp_dir: &TempDir,
    use_case_id: &str,
    category: &str,
    methodology: &str,
    level: &str,
) -> String {
    let md_path = temp_dir
        .path()
        .join("docs/use-cases")
        .join(category)
        .join(format!("{}-{}-{}.md", use_case_id, methodology, level));
    fs::read_to_string(md_path).expect("Failed to read markdown file")
}

#[test]
#[serial]
fn test_complete_use_case_edit_workflow() {
    let (temp_dir, mut controller) = setup_test_env();

    // Step 1: Create initial use case
    let params = create_test_params(
        "Original Title",
        "authentication",
        "AUT",
        Some("User login workflow".to_string()),
        Some("business".to_string()),
        None,
        Some("low".to_string()),
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();

    assert!(create_result.is_success());
    let use_case_id = extract_use_case_id(&create_result.message);

    // Verify initial state
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.title, "Original Title");
    assert_eq!(use_case.category, "authentication");
    assert_eq!(use_case.description, "User login workflow");
    assert_eq!(use_case.priority.to_string(), "LOW");

    // Step 2: Update basic fields
    let update_result = controller
        .update_use_case(
            use_case_id.clone(),
            Some("Updated Login Title".to_string()),
            Some("security".to_string()),
            Some("Enhanced user authentication workflow".to_string()),
            Some("high".to_string()),
        )
        .unwrap();

    assert!(update_result.is_success());

    // Verify updates in memory
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.title, "Updated Login Title");
    assert_eq!(use_case.category, "security");
    assert_eq!(
        use_case.description,
        "Enhanced user authentication workflow"
    );
    assert_eq!(use_case.priority.to_string(), "HIGH");

    // Verify TOML persistence
    let toml_content = read_toml_file(&temp_dir, &use_case_id, "security");
    assert!(toml_content.contains("Updated Login Title"));
    assert!(toml_content.contains("security"));
    assert!(toml_content.contains("Enhanced user authentication workflow"));
    assert!(toml_content.contains("priority = \"High\""));

    // Step 3: Update methodology fields
    let mut fields = HashMap::new();
    fields.insert("estimated_effort".to_string(), "8".to_string());
    fields.insert("complexity".to_string(), "high".to_string());

    let fields_result = controller
        .update_use_case_methodology_fields(use_case_id.clone(), "business".to_string(), fields)
        .unwrap();

    assert!(fields_result.is_success());

    // Verify methodology fields in TOML
    let toml_content = read_toml_file(&temp_dir, &use_case_id, "security");
    assert!(toml_content.contains("estimated_effort"));
    assert!(toml_content.contains("complexity"));

    println!("✅ Complete use case edit workflow test passed");
}

#[test]
#[serial]
fn test_multi_view_management() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create use case with business methodology
    let params = create_test_params(
        "Multi-View Test",
        "test",
        "TES",
        Some("Testing multi-view support".to_string()),
        Some("business".to_string()),
        None,
        None,
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();

    let use_case_id = extract_use_case_id(&create_result.message);

    // Initial state: one view (business)
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.views.len(), 1);
    assert!(use_case.views.iter().any(|v| v.methodology == "business"));

    // Add developer view
    let add_result = controller
        .add_view(
            use_case_id.clone(),
            "developer".to_string(),
            "normal".to_string(),
        )
        .unwrap();

    assert!(add_result.is_success());

    // Verify both views exist
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.views.len(), 2);
    assert!(use_case.views.iter().any(|v| v.methodology == "business"));
    assert!(use_case.views.iter().any(|v| v.methodology == "developer"));

    // Add tester view
    let add_result = controller
        .add_view(
            use_case_id.clone(),
            "tester".to_string(),
            "advanced".to_string(),
        )
        .unwrap();

    assert!(add_result.is_success());

    // Verify three views
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.views.len(), 3);

    // Remove business view
    let remove_result = controller
        .remove_view(use_case_id.clone(), "business".to_string())
        .unwrap();

    assert!(remove_result.is_success());

    // Verify business view removed, others remain
    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.views.len(), 2);
    assert!(!use_case.views.iter().any(|v| v.methodology == "business"));
    assert!(use_case.views.iter().any(|v| v.methodology == "developer"));
    assert!(use_case.views.iter().any(|v| v.methodology == "tester"));

    // Verify TOML persistence - views are stored as objects with methodology and level fields
    let toml_content = read_toml_file(&temp_dir, &use_case_id, "test");
    assert!(toml_content.contains("methodology = \"developer\""));
    assert!(toml_content.contains("level = \"normal\""));
    assert!(toml_content.contains("methodology = \"tester\""));
    assert!(toml_content.contains("level = \"advanced\""));
    assert!(!toml_content.contains("methodology = \"business\""));

    println!("✅ Multi-view management test passed");
}

#[test]
#[serial]
fn test_partial_field_updates() {
    let (_temp_dir, mut controller) = setup_test_env();

    // Create use case with all fields
    let params = create_test_params(
        "Partial Update Test",
        "original_category",
        "ORI",
        Some("Original description".to_string()),
        Some("business".to_string()),
        None,
        Some("medium".to_string()),
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();

    let use_case_id = extract_use_case_id(&create_result.message);

    // Update only title
    controller
        .update_use_case(
            use_case_id.clone(),
            Some("New Title Only".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.title, "New Title Only");
    assert_eq!(use_case.category, "original_category");
    assert_eq!(use_case.description, "Original description");
    assert_eq!(use_case.priority.to_string(), "MEDIUM");

    // Update only category
    controller
        .update_use_case(
            use_case_id.clone(),
            None,
            Some("new_category".to_string()),
            None,
            None,
        )
        .unwrap();

    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.title, "New Title Only");
    assert_eq!(use_case.category, "new_category");
    assert_eq!(use_case.description, "Original description");

    // Update only priority
    controller
        .update_use_case(
            use_case_id.clone(),
            None,
            None,
            None,
            Some("critical".to_string()),
        )
        .unwrap();

    let use_case = controller.get_use_case(&use_case_id).unwrap();
    assert_eq!(use_case.priority.to_string(), "CRITICAL");
    assert_eq!(use_case.title, "New Title Only");
    assert_eq!(use_case.category, "new_category");

    println!("✅ Partial field updates test passed");
}

#[test]
#[serial]
fn test_methodology_fields_per_view() {
    let (_temp_dir, mut controller) = setup_test_env();

    // Create use case with business methodology
    let params = create_test_params(
        "Methodology Fields Test",
        "test",
        "TES",
        None,
        Some("business".to_string()),
        None,
        None,
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();

    let use_case_id = extract_use_case_id(&create_result.message);

    // Update business methodology fields
    let mut business_fields = HashMap::new();
    business_fields.insert("roi".to_string(), "high".to_string());
    business_fields.insert("estimated_effort".to_string(), "40".to_string());

    controller
        .update_use_case_methodology_fields(
            use_case_id.clone(),
            "business".to_string(),
            business_fields,
        )
        .unwrap();

    // Add developer view
    controller
        .add_view(
            use_case_id.clone(),
            "developer".to_string(),
            "normal".to_string(),
        )
        .unwrap();

    // Update developer methodology fields
    let mut dev_fields = HashMap::new();
    dev_fields.insert("complexity".to_string(), "medium".to_string());
    dev_fields.insert("technical_stack".to_string(), "rust".to_string());

    controller
        .update_use_case_methodology_fields(
            use_case_id.clone(),
            "developer".to_string(),
            dev_fields,
        )
        .unwrap();

    // Verify fields are stored per methodology
    let use_case = controller.get_use_case(&use_case_id).unwrap();

    // Business fields should exist
    assert!(use_case
        .methodology_fields
        .get("business")
        .and_then(|fields| fields.get("roi"))
        .is_some());

    // Developer fields should exist
    assert!(use_case
        .methodology_fields
        .get("developer")
        .and_then(|fields| fields.get("complexity"))
        .is_some());

    println!("✅ Methodology fields per view test passed");
}

#[test]
#[serial]
fn test_error_handling_invalid_operations() {
    let (_temp_dir, mut controller) = setup_test_env();

    // Create a use case that will trigger an error
    let params = create_test_params(
        "Error Test",
        "test",
        "TES",
        None,
        Some("business".to_string()),
        None,
        None,
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();

    let use_case_id = extract_use_case_id(&create_result.message);

    // Try to remove the only view - should fail
    let remove_result = controller
        .remove_view(use_case_id.clone(), "business".to_string())
        .unwrap();

    assert!(!remove_result.is_success());
    assert!(
        remove_result.message.contains("last")
            || remove_result.message.contains("only")
            || remove_result.message.contains("cannot remove")
    );

    // Try to update non-existent use case
    let update_result = controller.update_use_case(
        "UC-NONEXISTENT-999".to_string(),
        Some("Test".to_string()),
        None,
        None,
        None,
    );

    assert!(update_result.is_err() || !update_result.unwrap().is_success());

    // Try to remove non-existent view
    let remove_result = controller
        .remove_view(use_case_id.clone(), "nonexistent".to_string())
        .unwrap();

    assert!(!remove_result.is_success());

    println!("✅ Error handling test passed");
}

#[test]
#[serial]
fn test_edit_workflow_with_regeneration() {
    let (temp_dir, mut controller) = setup_test_env();

    // Create initial use case
    let params = create_test_params(
        "Regeneration Test",
        "test",
        "TES",
        Some("Testing regeneration".to_string()),
        Some("business".to_string()),
        None,
        None,
        None,
    );
    let create_result = controller.create_use_case(params).unwrap();
    let use_case_id = extract_use_case_id(&create_result.message);

    // Markdown should be automatically generated on creation
    // Get initial markdown content - file is UC-TES-001-business-normal.md
    let initial_md = read_markdown_file(&temp_dir, &use_case_id, "test", "business", "normal");
    assert!(initial_md.contains("Regeneration Test"));
    // Description may be in different sections depending on the template
    assert!(initial_md.len() > 100); // Just verify markdown was generated

    // Update use case
    controller
        .update_use_case(
            use_case_id.clone(),
            Some("Updated After Regen".to_string()),
            None,
            Some("New description for testing".to_string()),
            None,
        )
        .unwrap();

    // Regenerate markdown
    controller.regenerate_use_case(&use_case_id).unwrap();

    // Verify markdown reflects updates
    let updated_md = read_markdown_file(&temp_dir, &use_case_id, "test", "business", "normal");
    assert!(updated_md.contains("Updated After Regen"));
    // Description placement varies by template, just verify markdown was regenerated
    assert!(updated_md.len() > 100);
    assert!(!updated_md.contains("Regeneration Test")); // Old title should be gone

    println!("✅ Edit workflow with regeneration test passed");
}
