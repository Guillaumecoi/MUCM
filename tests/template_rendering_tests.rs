use mucm::core::TemplateEngine;
use serde_json::{json, Value};
use serial_test::serial;
use std::collections::HashMap;
use tempfile::TempDir;

/// Setup test environment with initialized config
/// Note: Does NOT change directories - runs from workspace root where source-templates exists
fn setup_test_env() -> TempDir {
    // Point to project source templates to avoid loading from user config
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_templates = project_root.join("source-templates");
    unsafe {
        std::env::set_var("MUCM_TEST_TEMPLATES_DIR", source_templates);
    }

    // Create temp dir for test artifacts
    // Config files aren't needed for template rendering tests
    TempDir::new().unwrap()
}

/// Create comprehensive test data with all possible fields
fn create_full_test_data() -> HashMap<String, Value> {
    let mut data = HashMap::new();

    data.insert("id".to_string(), json!("UC-TST-001"));
    data.insert("title".to_string(), json!("Test Use Case"));
    data.insert("category".to_string(), json!("testing"));
    data.insert(
        "description".to_string(),
        json!("A comprehensive test use case"),
    );
    data.insert(
        "summary".to_string(),
        json!("Brief summary of the test use case"),
    );
    data.insert("status".to_string(), json!("draft"));
    data.insert("priority".to_string(), json!("high"));
    data.insert("created_date".to_string(), json!("2025-11-25"));
    data.insert("last_updated".to_string(), json!("2025-11-25"));
    data.insert("generated_at".to_string(), json!("2025-11-25 20:00:00"));
    data.insert("test_module_name".to_string(), json!("uc_tst_001"));

    data.insert(
        "preconditions".to_string(),
        json!(["User is authenticated", "System is online"]),
    );

    data.insert(
        "postconditions".to_string(),
        json!(["Data is saved", "User receives confirmation"]),
    );

    data.insert(
        "use_case_references".to_string(),
        json!([
            {
                "target_id": "UC-TST-002",
                "relationship": "depends_on",
                "description": "Requires authentication"
            }
        ]),
    );

    // Comprehensive scenarios with all possible fields
    data.insert(
        "scenarios".to_string(),
        json!([
            {
                "id": "SCN-001",
                "title": "Happy Path",
                "description": "Main success scenario",
                "scenario_type": "Main Scenario",
                "status": "implemented",
                "is_main": true,
                "persona": "Power User",
                "steps": [
                    {
                        "order": 1,
                        "actor": "User",
                        "action": "enters credentials",
                        "description": "User enters credentials",
                        "receiver": "System"
                    },
                    {
                        "order": 2,
                        "actor": "System",
                        "action": "validates credentials",
                        "description": "System validates credentials",
                        "receiver": null
                    },
                    {
                        "order": 3,
                        "actor": "System",
                        "action": "confirms login",
                        "description": "System confirms login",
                        "receiver": "User"
                    }
                ]
            },
            {
                "id": "SCN-002",
                "title": "Error Handling",
                "description": "Extension scenario for errors",
                "scenario_type": "Extension Scenario",
                "status": "draft",
                "is_main": false,
                "persona": null,
                "extends_scenario_id": "SCN-001",
                "extends_at_step": 2,
                "returns_at_step": 3,
                "steps": [
                    {
                        "order": 1,
                        "actor": "System",
                        "action": "shows error message",
                        "description": "System shows error message",
                        "receiver": "User"
                    },
                    {
                        "order": 2,
                        "actor": "User",
                        "action": "retries login",
                        "description": "User retries login",
                        "receiver": "System"
                    }
                ]
            }
        ]),
    );

    // Methodology-specific fields

    // Business methodology fields
    data.insert(
        "business_value".to_string(),
        json!("Increases user engagement by 25%"),
    );
    data.insert(
        "stakeholders".to_string(),
        json!(["Product Manager", "CTO"]),
    );
    data.insert(
        "success_criteria".to_string(),
        json!(["95% uptime", "< 2s response time"]),
    );

    // Developer methodology fields
    data.insert(
        "technical_requirements".to_string(),
        json!("RESTful API with OAuth2"),
    );
    data.insert(
        "api_endpoints".to_string(),
        json!(["/auth/login", "/auth/validate"]),
    );
    data.insert(
        "dependencies".to_string(),
        json!(["auth-service", "database"]),
    );
    data.insert(
        "error_handling".to_string(),
        json!("Return 401 for invalid credentials"),
    );

    // Feature methodology fields
    data.insert("user_segment".to_string(), json!("Enterprise users"));
    data.insert(
        "success_metrics".to_string(),
        json!(["NPS > 8", "Engagement +30%"]),
    );
    data.insert(
        "hypothesis".to_string(),
        json!("Users need faster authentication"),
    );
    data.insert(
        "feature_dependencies".to_string(),
        json!(["SSO integration"]),
    );
    data.insert(
        "design_assets".to_string(),
        json!(["https://figma.com/design/123"]),
    );

    // Tester methodology fields
    data.insert("test_type".to_string(), json!("Integration"));
    data.insert("test_priority".to_string(), json!("P0 - Critical"));
    data.insert("automation_status".to_string(), json!("Automated"));
    data.insert("test_environment".to_string(), json!("Staging"));
    data.insert(
        "regression_suite".to_string(),
        json!("Auth Regression Suite"),
    );
    data.insert(
        "performance_baseline".to_string(),
        json!("< 500ms response time"),
    );
    data.insert(
        "coverage_areas".to_string(),
        json!(["Authentication", "Authorization"]),
    );
    data.insert(
        "test_data_requirements".to_string(),
        json!("Test user accounts"),
    );

    data
}

/// Create minimal test data (only required fields)
fn create_minimal_test_data() -> HashMap<String, Value> {
    let mut data = HashMap::new();

    data.insert("id".to_string(), json!("UC-MIN-001"));
    data.insert("title".to_string(), json!("Minimal Use Case"));
    data.insert("category".to_string(), json!("test"));
    data.insert("description".to_string(), json!("A minimal use case"));
    data.insert("status".to_string(), json!("draft"));
    data.insert("priority".to_string(), json!("medium"));
    data.insert("generated_at".to_string(), json!("2025-11-25"));

    data
}

#[test]
#[serial]
fn test_business_normal_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "business", "normal");

    assert!(
        result.is_ok(),
        "Business normal template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_business_advanced_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "business", "advanced");

    assert!(
        result.is_ok(),
        "Business advanced template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_developer_normal_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "developer", "normal");

    assert!(
        result.is_ok(),
        "Developer normal template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_developer_advanced_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "developer", "advanced");

    assert!(
        result.is_ok(),
        "Developer advanced template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_feature_normal_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "feature", "normal");

    assert!(
        result.is_ok(),
        "Feature normal template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_feature_advanced_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "feature", "advanced");

    assert!(
        result.is_ok(),
        "Feature advanced template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_tester_normal_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "tester", "normal");

    assert!(
        result.is_ok(),
        "Tester normal template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_tester_advanced_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_use_case_with_methodology_and_level(&data, "tester", "advanced");

    assert!(
        result.is_ok(),
        "Tester advanced template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("Test Use Case"));
}

#[test]
#[serial]
fn test_all_templates_with_minimal_data() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_minimal_test_data();

    let methodologies = vec!["business", "developer", "feature", "tester"];
    let levels = ["normal", "advanced"];

    for methodology in methodologies {
        for level in levels.iter() {
            let result =
                engine.render_use_case_with_methodology_and_level(&data, methodology, level);
            assert!(
                result.is_ok(),
                "{}-{} template should handle minimal data: {:?}",
                methodology,
                level,
                result.err()
            );
        }
    }
}

#[test]
#[serial]
fn test_templates_render_with_aggregated_status_injected() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let mut data = create_full_test_data();

    // Inject aggregated_status as the generator would
    data.insert("aggregated_status".to_string(), json!("Implemented"));

    let result = engine.render_use_case_with_methodology_and_level(&data, "business", "normal");
    assert!(
        result.is_ok(),
        "Rendering should succeed: {:?}",
        result.err()
    );
    let rendered = result.unwrap();

    // The header should now contain the aggregated status text
    assert!(rendered.contains("Implemented"));
}

#[test]
#[serial]
fn test_rust_test_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_test("rust", &data);

    assert!(
        result.is_ok(),
        "Rust test template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("#[test]"));
}

#[test]
#[serial]
fn test_python_test_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_test("python", &data);

    assert!(
        result.is_ok(),
        "Python test template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("unittest"));
}

#[test]
#[serial]
fn test_javascript_test_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    let result = engine.render_test("javascript", &data);

    assert!(
        result.is_ok(),
        "JavaScript test template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();
    assert!(rendered.contains("UC-TST-001"));
    assert!(rendered.contains("describe"));
}

#[test]
#[serial]
fn test_scenario_partial_rendering() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let data = create_full_test_data();

    // Test that scenario partial is properly included in templates
    let result = engine.render_use_case_with_methodology_and_level(&data, "business", "normal");

    assert!(
        result.is_ok(),
        "Template with scenario partial should render"
    );
    let rendered = result.unwrap();

    // Check that scenario content is rendered
    assert!(rendered.contains("Happy Path") || rendered.contains("Scenario"));
}

#[test]
#[serial]
fn test_templates_with_empty_scenarios() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let mut data = create_minimal_test_data();

    // Add empty scenarios array
    data.insert("scenarios".to_string(), json!([]));

    let methodologies = vec!["business", "developer", "feature", "tester"];

    for methodology in methodologies {
        let result =
            engine.render_use_case_with_methodology_and_level(&data, methodology, "normal");
        assert!(
            result.is_ok(),
            "{} template should handle empty scenarios: {:?}",
            methodology,
            result.err()
        );
    }
}

#[test]
#[serial]
fn test_templates_with_missing_optional_fields() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();

    // Create data with only absolutely required fields
    let mut data = HashMap::new();
    data.insert("id".to_string(), json!("UC-MIN-001"));
    data.insert("title".to_string(), json!("Minimal"));

    let methodologies = vec!["business", "developer", "feature", "tester"];

    for methodology in methodologies {
        let result =
            engine.render_use_case_with_methodology_and_level(&data, methodology, "normal");
        assert!(
            result.is_ok(),
            "{} template should handle missing optional fields: {:?}",
            methodology,
            result.err()
        );
    }
}

#[test]
#[serial]
fn test_overview_template() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();

    // Updated data structure for new category-focused overview
    let data: HashMap<String, Value> = [
        ("project_name".to_string(), json!("Test Project")),
        ("total_use_cases".to_string(), json!(5)),
        ("total_categories".to_string(), json!(2)),
        (
            "categories".to_string(),
            json!([
                {
                    "category_name": "Authentication",
                    "category_path": "authentication",
                    "use_case_count": 3,
                    "description": "User authentication and authorization"
                },
                {
                    "category_name": "Payment",
                    "category_path": "payment",
                    "use_case_count": 2
                }
            ]),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let result = engine.render_overview(&data);

    assert!(
        result.is_ok(),
        "Overview template should render: {:?}",
        result.err()
    );
    let rendered = result.unwrap();

    // Verify the new category-focused overview format
    assert!(rendered.contains("Test Project"));
    assert!(rendered.contains("**Total Use Cases:** 5"));
    assert!(rendered.contains("**Total Categories:** 2"));
    assert!(rendered.contains("## Categories"));
    assert!(rendered.contains("### [Authentication](./authentication/README.md)"));
    assert!(rendered.contains("**Use Cases:** 3"));
    assert!(rendered.contains("User authentication and authorization"));
    assert!(rendered.contains("### [Payment](./payment/README.md)"));
    assert!(rendered.contains("**Use Cases:** 2"));
}

#[test]
#[serial]
fn test_all_templates_comprehensive() {
    let _temp_dir = setup_test_env();
    let engine = TemplateEngine::new().unwrap();
    let full_data = create_full_test_data();
    let minimal_data = create_minimal_test_data();

    let methodologies = vec!["business", "developer", "feature", "tester"];
    let levels = vec!["normal", "advanced"];
    let test_languages = vec!["rust", "python", "javascript"];

    // Test all methodology templates with full data
    for methodology in &methodologies {
        for level in &levels {
            let result =
                engine.render_use_case_with_methodology_and_level(&full_data, methodology, level);
            assert!(
                result.is_ok(),
                "{}-{} should render with full data: {:?}",
                methodology,
                level,
                result.err()
            );
        }
    }

    // Test all methodology templates with minimal data
    for methodology in &methodologies {
        for level in &levels {
            let result = engine.render_use_case_with_methodology_and_level(
                &minimal_data,
                methodology,
                level,
            );
            assert!(
                result.is_ok(),
                "{}-{} should render with minimal data: {:?}",
                methodology,
                level,
                result.err()
            );
        }
    }

    // Test all language test templates
    for language in &test_languages {
        let result = engine.render_test(language, &full_data);
        assert!(
            result.is_ok(),
            "{} test template should render: {:?}",
            language,
            result.err()
        );
    }
}
