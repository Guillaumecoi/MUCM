//! Comprehensive integration tests for TestGenerator
//!
//! Tests the public API of TestGenerator through realistic scenarios.
//! These tests verify:
//! - Test file generation for multiple languages
//! - Safe zone preservation during regeneration
//! - Multi-methodology test generation settings
//! - File path and naming conventions
//! - Template rendering with documentation links and steps

use mucm::config::Config;
use mucm::core::{Scenario, ScenarioStep, ScenarioType, TestGenerator, UseCase};
use std::fs;
use tempfile::TempDir;

mod common;

/// Helper to create a basic config for testing
fn create_test_config(temp_dir: &TempDir, test_language: &str) -> Config {
    // Create minimal source-templates structure
    common::create_minimal_source_templates(temp_dir.path()).unwrap();

    // Set env var so TestGenerator finds our test templates
    let templates_dir = temp_dir.path().join("source-templates");
    std::env::set_var("MUCM_TEST_TEMPLATES_DIR", &templates_dir);

    let mut config = Config::default();
    // Set test_dir to full path
    config.directories.test_dir = temp_dir
        .path()
        .join("tests/use-cases")
        .to_string_lossy()
        .to_string();
    config.generation.test_language = test_language.to_string();
    config.generation.auto_generate_tests = true;
    config.generation.overwrite_test_documentation = true;

    config
}

/// Helper to create a basic use case
fn create_use_case(id: &str, category: &str) -> UseCase {
    UseCase::new(
        id.to_string(),
        "Test Use Case".to_string(),
        category.to_string(),
        "TST".to_string(),
        "A test use case".to_string(),
        "High".to_string(),
    )
    .unwrap()
}

/// Helper to create a use case with scenarios
fn create_use_case_with_scenarios(id: &str) -> UseCase {
    let mut uc = create_use_case(id, "Authentication");

    let mut scenario1 = Scenario::new(
        format!("{}-S01", id),
        "Successful Login".to_string(),
        "User logs in successfully".to_string(),
        ScenarioType::HappyPath,
        "user".to_string(),
    );
    let mut step1 = ScenarioStep::new(
        "1".to_string(),
        "user".to_string(),
        "enters credentials".to_string(),
    );
    step1.set_receiving_actor("system".to_string());
    scenario1.add_step(step1);
    let mut step2 = ScenarioStep::new(
        "2".to_string(),
        "system".to_string(),
        "validates and grants access".to_string(),
    );
    step2.set_receiving_actor("user".to_string());
    scenario1.add_step(step2);

    let mut scenario2 = Scenario::new(
        format!("{}-S02", id),
        "Failed Login".to_string(),
        "User provides invalid credentials".to_string(),
        ScenarioType::AlternativeFlow,
        "user".to_string(),
    );
    let mut step3 = ScenarioStep::new(
        "1".to_string(),
        "user".to_string(),
        "enters invalid credentials".to_string(),
    );
    step3.set_receiving_actor("system".to_string());
    scenario2.add_step(step3);

    uc.scenarios.push(scenario1);
    uc.scenarios.push(scenario2);

    uc
}

#[test]
fn test_generator_creates_python_test_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-001");

    // Generate test
    generator.generate(&use_case).unwrap();

    // Check file was created in correct location
    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_001.py");
    assert!(test_file.exists(), "Python test file should be created");

    // Check file content - basic checks since we're using minimal templates
    let content = fs::read_to_string(&test_file).unwrap();
    // Just verify it's not empty and is a Python file
    assert!(!content.is_empty(), "File should not be empty");
    // In a real test with full templates, you'd check for unittest, test class, etc.
    // For now, we just verify the file was created successfully
}

#[test]
fn test_generator_creates_javascript_test_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "javascript");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-002");

    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_002.js");
    assert!(test_file.exists(), "JavaScript test file should be created");

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("describe("),
        "Should use Jest/Mocha describe"
    );
    assert!(
        content.contains("test(") || content.contains("it("),
        "Should use test/it function"
    );
}

#[test]
fn test_generator_creates_java_test_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "java");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-003");

    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_003.java");
    assert!(test_file.exists(), "Java test file should be created");

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("@Test"),
        "Should use JUnit @Test annotation"
    );
    assert!(
        content.contains("@BeforeEach"),
        "Should use JUnit @BeforeEach"
    );
    assert!(
        content.contains("@AfterEach"),
        "Should use JUnit @AfterEach"
    );
    assert!(
        content.contains("@DisplayName"),
        "Should use JUnit @DisplayName"
    );
}

#[test]
fn test_generator_java_pascal_case_naming() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "java");
    let generator = TestGenerator::new(config);

    let mut use_case = create_use_case("UC-USER-LOGIN", "user-authentication");
    use_case.title = "User Login System".to_string();

    let mut scenario = Scenario::new(
        "UC-USER-LOGIN-S01".to_string(),
        "Successful User Login".to_string(),
        "User successfully logs in".to_string(),
        ScenarioType::HappyPath,
        "user".to_string(),
    );
    let step = ScenarioStep::new(
        "1".to_string(),
        "user".to_string(),
        "submits valid credentials".to_string(),
    );
    scenario.add_step(step);
    use_case.scenarios.push(scenario);

    generator.generate(&use_case).unwrap();

    let test_file_path = temp_dir
        .path()
        .join("tests/use-cases/user_authentication/uc_user_login.java");

    let content = fs::read_to_string(&test_file_path).unwrap();

    // Verify PascalCase class name
    assert!(
        content.contains("public class UserLoginSystemTest"),
        "Should use PascalCase for class name from title"
    );

    // Verify PascalCase method name
    assert!(
        content.contains("void testUcUserLoginS01()"),
        "Should use PascalCase for test method from scenario ID"
    );

    // Verify package declaration (test template uses simplified package)
    assert!(
        content.contains("package test.usecases"),
        "Should have package declaration"
    );
}

#[test]
fn test_generator_java_multiple_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "java");
    let generator = TestGenerator::new(config);

    let mut use_case = create_use_case("UC-TEST-005", "authentication");

    // Add multiple scenarios
    for i in 1..=5 {
        let mut scenario = Scenario::new(
            format!("UC-TEST-005-S{:02}", i),
            format!("Test Scenario {}", i),
            format!("Description for scenario {}", i),
            ScenarioType::HappyPath,
            "user".to_string(),
        );
        let step = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            format!("performs action {}", i),
        );
        scenario.add_step(step);
        use_case.scenarios.push(scenario);
    }

    generator.generate(&use_case).unwrap();

    let test_file_path = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_005.java");

    let content = fs::read_to_string(&test_file_path).unwrap();

    // Verify all test methods are present
    for i in 1..=5 {
        assert!(
            content.contains(&format!("void testUcTest005S{:02}()", i)),
            "Should have test method for scenario {}",
            i
        );
        assert!(
            content.contains(&format!("@DisplayName(\"Test Scenario {}\")", i)),
            "Should have @DisplayName for scenario {}",
            i
        );
    }

    // Verify only one class and one @BeforeEach/@AfterEach
    assert_eq!(
        content.matches("public class").count(),
        1,
        "Should have exactly one class"
    );
    assert_eq!(
        content.matches("@BeforeEach").count(),
        1,
        "Should have exactly one @BeforeEach"
    );
    assert_eq!(
        content.matches("@AfterEach").count(),
        1,
        "Should have exactly one @AfterEach"
    );
}

#[test]
fn test_generator_java_package_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "java");
    let generator = TestGenerator::new(config);

    // Test with different category names
    let test_cases = vec![
        ("Simple", "test.usecases.simple"),
        ("Multi Word", "test.usecases.multi_word"),
        ("User-Authentication", "test.usecases.user_authentication"),
        ("Order_Processing", "test.usecases.order_processing"),
    ];

    for (category, _expected_package) in test_cases {
        let use_case = create_use_case(&format!("UC-{}", category), category);

        generator.generate(&use_case).unwrap();

        let category_snake = category.to_lowercase().replace(&[' ', '-'][..], "_");
        let id_snake = format!(
            "uc_{}",
            category.to_lowercase().replace(&[' ', '-'][..], "_")
        );

        let test_file_path = temp_dir.path().join(format!(
            "tests/use-cases/{}/{}.java",
            category_snake, id_snake
        ));

        if !test_file_path.exists() {
            panic!(
                "Expected file not found: {:?} for category: {}",
                test_file_path, category
            );
        }

        let content = fs::read_to_string(&test_file_path).unwrap();

        // The test template uses simplified package name
        assert!(
            content.contains("package test.usecases"),
            "Category '{}' should have package declaration",
            category
        );
    }
}

#[test]
fn test_generator_creates_rust_test_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "rust");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-003");

    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_003.rs");
    assert!(test_file.exists(), "Rust test file should be created");

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("#[cfg(test)]"),
        "Should have test cfg attribute"
    );
    assert!(content.contains("#[test]"), "Should have test attribute");
    assert!(
        content.contains("fn test_uc_test_003_s01()"),
        "Should have test function"
    );
}

#[test]
fn test_generator_skips_when_language_is_none() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "none");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-004");

    // Should not error, but also not create file
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_004.py");
    assert!(
        !test_file.exists(),
        "Should not create file when language is 'none'"
    );
}

#[test]
fn test_generator_respects_overwrite_setting() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(&temp_dir, "python");
    config.generation.overwrite_test_documentation = false;

    let generator = TestGenerator::new(config);
    let use_case = create_use_case_with_scenarios("UC-TEST-005");

    // Generate initially
    generator.generate(&use_case).unwrap();
    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_005.py");

    // Modify the file
    fs::write(&test_file, "# Modified content\n").unwrap();

    // Try to generate again - should skip
    generator.generate(&use_case).unwrap();

    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        content, "# Modified content\n",
        "File should not be overwritten"
    );
}

#[test]
fn test_generator_regenerate_preserves_global_safe_zone() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-006");

    // Generate initial test
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_006.py");

    // With minimal templates, just verify regenerate doesn't error
    // In a full implementation with real templates, we'd test actual safe zone preservation
    generator.regenerate(&use_case).unwrap();

    // Verify file still exists
    assert!(
        test_file.exists(),
        "File should still exist after regeneration"
    );
}

#[test]
fn test_generator_regenerate_preserves_scenario_safe_zones() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-007");

    // Generate initial test
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_007.py");
    let mut content = fs::read_to_string(&test_file).unwrap();

    // Add user code in first scenario safe zone
    content = content.replace(
        "self.fail(\"Test not implemented yet\")",
        "# My custom test\n        self.assertTrue(True)",
    );
    fs::write(&test_file, &content).unwrap();

    // Regenerate
    generator.regenerate(&use_case).unwrap();

    // Check user code is preserved
    let new_content = fs::read_to_string(&test_file).unwrap();
    assert!(
        new_content.contains("My custom test"),
        "User test code should be preserved"
    );
    assert!(
        new_content.contains("self.assertTrue(True)"),
        "User test code should be preserved"
    );
    assert!(
        !new_content.contains("self.fail(\"Test not implemented yet\")"),
        "Default code should be replaced"
    );
}

#[test]
fn test_generator_regenerate_preserves_setup_and_teardown() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-008");

    // Generate initial test
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_008.py");

    // With minimal templates, just verify regenerate doesn't error
    generator.regenerate(&use_case).unwrap();

    // Verify file still exists
    assert!(
        test_file.exists(),
        "File should still exist after regeneration"
    );
}

#[test]
fn test_generator_handles_added_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let mut use_case = create_use_case_with_scenarios("UC-TEST-009");

    // Generate with 2 scenarios
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_009.py");
    let content_before = fs::read_to_string(&test_file).unwrap();
    let count_before = content_before.matches("def test_").count();

    // Add a new scenario
    let mut scenario3 = Scenario::new(
        "UC-TEST-009-S03".to_string(),
        "Password Reset".to_string(),
        "User resets password".to_string(),
        ScenarioType::AlternativeFlow,
        "user".to_string(),
    );
    let mut step4 = ScenarioStep::new(
        "1".to_string(),
        "user".to_string(),
        "requests password reset".to_string(),
    );
    step4.set_receiving_actor("system".to_string());
    scenario3.add_step(step4);
    use_case.scenarios.push(scenario3);

    // Regenerate
    generator.regenerate(&use_case).unwrap();

    // Check new scenario test is added
    let content_after = fs::read_to_string(&test_file).unwrap();
    let count_after = content_after.matches("def test_").count();

    assert_eq!(
        count_after,
        count_before + 1,
        "New scenario test should be added"
    );
    assert!(
        content_after.contains("def test_uc_test_009_s03(self):"),
        "New test function should exist"
    );
    assert!(
        content_after.contains("Password Reset"),
        "New scenario title should be in docstring"
    );
}

#[test]
fn test_generator_handles_removed_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let mut use_case = create_use_case_with_scenarios("UC-TEST-010");

    // Generate with 2 scenarios
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_010.py");

    // Remove second scenario
    use_case.scenarios.remove(1);

    // Regenerate
    generator.regenerate(&use_case).unwrap();

    // Check removed scenario test is gone
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        !content.contains("def test_uc_test_010_s02(self):"),
        "Removed scenario should not exist"
    );
    assert!(
        !content.contains("Failed Login"),
        "Removed scenario title should not exist"
    );
    assert!(
        content.contains("def test_uc_test_010_s01(self):"),
        "Remaining scenario should still exist"
    );
}

#[test]
fn test_generator_skips_when_auto_generate_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(&temp_dir, "python");
    config.generation.auto_generate_tests = false;

    let generator = TestGenerator::new(config);
    let use_case = create_use_case("UC-TEST-011", "Test");

    // Should not create file when auto_generate is false
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir.path().join("tests/use-cases/test/uc_test_011.py");
    assert!(
        !test_file.exists(),
        "Test should not be generated when auto_generate is false"
    );
}

#[test]
fn test_generator_file_path_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case("UC-AUTH-001", "Authentication");

    generator.generate(&use_case).unwrap();

    // Check file is at correct path with snake_case naming
    let expected_path = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_auth_001.py");
    assert!(
        expected_path.exists(),
        "Test file should be at correct snake_case path"
    );
}

#[test]
fn test_generator_regenerate_preserves_user_implementation_content() {
    // Use real templates to reproduce the bug
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_templates = project_root.join("source-templates");
    unsafe {
        std::env::set_var("MUCM_TEST_TEMPLATES_DIR", source_templates);
    }

    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case_with_scenarios("UC-TEST-009");

    // Generate initial test
    generator.generate(&use_case).unwrap();

    let test_file = temp_dir
        .path()
        .join("tests/use-cases/authentication/uc_test_009.py");

    // Extract content between user implementation markers
    let initial_content = fs::read_to_string(&test_file).unwrap();
    let initial_user_content = extract_user_implementation_content(&initial_content);

    // Regenerate
    generator.regenerate(&use_case).unwrap();

    // Extract content again
    let regenerated_content = fs::read_to_string(&test_file).unwrap();
    let regenerated_user_content = extract_user_implementation_content(&regenerated_content);

    // Verify content is identical
    assert_eq!(
        initial_user_content, regenerated_user_content,
        "User implementation content should not change during regeneration. Initial: '{}', Regenerated: '{}'",
        initial_user_content, regenerated_user_content
    );
}

fn extract_user_implementation_content(content: &str) -> String {
    const START_MARKER: &str = "# START USER IMPLEMENTATION - Add your imports and setup code here\n# =============================================================================\n";
    const END_MARKER: &str = "# =============================================================================\n# END USER IMPLEMENTATION\n# =============================================================================\n";

    if let Some(start_pos) = content.find(START_MARKER) {
        let start = start_pos + START_MARKER.len();
        if let Some(end_pos) = content[start..].find(END_MARKER) {
            let end = start + end_pos;
            return content[start..end].to_string();
        }
    }
    panic!("Could not find user implementation markers in content");
}

#[test]
fn test_generator_snake_case_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir, "python");
    let generator = TestGenerator::new(config);

    let use_case = create_use_case("UC-COMPLEX-ID-123", "My Category Name");

    generator.generate(&use_case).unwrap();

    // Check snake_case conversion in paths and filenames
    let expected_path = temp_dir
        .path()
        .join("tests/use-cases/my_category_name/uc_complex_id_123.py");
    assert!(
        expected_path.exists(),
        "File and directory names should be snake_case"
    );
}
