//! Unified persistence layer tests
//!
//! This module tests both TOML and SQLite backends with identical test suites
//! to ensure feature parity and correctness.

use markdown_use_case_manager::core::{SqliteUseCaseRepository, UseCase, UseCaseRepository};
use serial_test::serial;
use std::env;
use tempfile::TempDir;

/// Test helper: Create a test use case
fn create_test_use_case() -> UseCase {
    UseCase::new(
        "UC-TEST-001".to_string(),
        "Test Use Case".to_string(),
        "test".to_string(),
        "TES".to_string(),
        "A test use case for persistence testing".to_string(),
        "medium".to_string(),
    )
    .unwrap()
}

/// Test helper: Create a test use case with extra fields
fn create_test_use_case_with_extra() -> UseCase {
    let mut use_case = create_test_use_case();
    use_case.extra.insert(
        "custom_field".to_string(),
        serde_json::json!("custom_value"),
    );
    use_case
        .extra
        .insert("number_field".to_string(), serde_json::json!(42));
    use_case
}

/// Test helper: Create TOML repository for testing
fn create_toml_repository() -> (TempDir, Box<dyn UseCaseRepository>) {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    // Initialize basic config structure using the same approach as config tests
    use markdown_use_case_manager::config::ConfigFileManager;
    let config = markdown_use_case_manager::config::Config::default();
    ConfigFileManager::save_in_dir(&config, ".").unwrap();

    let config = markdown_use_case_manager::config::Config::load().unwrap();
    let repo = Box::new(markdown_use_case_manager::core::TomlUseCaseRepository::new(
        config,
    )) as Box<dyn UseCaseRepository>;

    (temp_dir, repo)
}

/// Test helper: Create SQLite repository for testing
fn create_sqlite_repository() -> (TempDir, Box<dyn UseCaseRepository>) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let repo =
        Box::new(SqliteUseCaseRepository::new(&db_path).unwrap()) as Box<dyn UseCaseRepository>;

    // Create markdown directory that the SQLite repository expects
    std::fs::create_dir_all(temp_dir.path().join("markdown")).unwrap();

    (temp_dir, repo)
}

/// Run all persistence tests against a repository
fn run_all_tests(repo: &dyn UseCaseRepository) {
    test_save_and_load(repo);
    test_save_with_extra_fields(repo);
    test_load_all(repo);
    test_save_markdown(repo);
}

#[test]
#[serial]
fn test_toml_backend() {
    let (_temp_dir, repo) = create_toml_repository();
    run_all_tests(&*repo);
}

#[test]
#[serial]
fn test_sqlite_backend() {
    let (_temp_dir, repo) = create_sqlite_repository();
    run_all_tests(&*repo);
}

// ===== Individual Test Functions =====

fn test_save_and_load(repo: &dyn UseCaseRepository) {
    let use_case = create_test_use_case();

    // Save
    repo.save(&use_case).expect("Save should succeed");

    // Load by ID
    let loaded = repo
        .load_by_id(&use_case.id)
        .expect("Load by ID should succeed");
    assert!(loaded.is_some(), "Use case should be found");
    let loaded = loaded.unwrap();

    // Verify fields
    assert_eq!(loaded.id, use_case.id);
    assert_eq!(loaded.title, use_case.title);
    assert_eq!(loaded.category, use_case.category);
    assert_eq!(loaded.description, use_case.description);
    assert_eq!(loaded.priority, use_case.priority);
    assert_eq!(loaded.status(), use_case.status());
}

fn test_save_with_extra_fields(repo: &dyn UseCaseRepository) {
    let use_case = create_test_use_case_with_extra();

    // Save
    repo.save(&use_case)
        .expect("Save with extra fields should succeed");

    // Load and verify extra fields
    let loaded = repo
        .load_by_id(&use_case.id)
        .expect("Load should succeed")
        .unwrap();

    assert_eq!(
        loaded.extra.get("custom_field"),
        Some(&serde_json::json!("custom_value"))
    );
    assert_eq!(
        loaded.extra.get("number_field"),
        Some(&serde_json::json!(42))
    );
}

// Deleted tests (methods removed in PR #11):
// - test_delete (delete method)
// - test_exists (exists method)
// - test_find_by_category (find_by_category method)
// - test_find_by_priority (find_by_priority method)
// - test_search_by_title (search_by_title method)
// - test_save_batch (save_batch method)
// - test_delete_batch (delete_batch method)

fn test_load_all(repo: &dyn UseCaseRepository) {
    let use_cases = vec![
        UseCase::new(
            "UC-ALL-001".to_string(),
            "Load All Test 1".to_string(),
            "load".to_string(),
            "LOA".to_string(),
            "".to_string(),
            "medium".to_string(),
        )
        .unwrap(),
        UseCase::new(
            "UC-ALL-002".to_string(),
            "Load All Test 2".to_string(),
            "load".to_string(),
            "LOA".to_string(),
            "".to_string(),
            "medium".to_string(),
        )
        .unwrap(),
    ];

    // Save all
    for use_case in &use_cases {
        repo.save(use_case).unwrap();
    }

    // Load all
    let all = repo.load_all().unwrap();
    assert!(all.len() >= 2, "Should have at least 2 use cases");

    // Verify our test cases are included
    let ids: Vec<String> = all.iter().map(|uc| uc.id.clone()).collect();
    assert!(ids.contains(&"UC-ALL-001".to_string()));
    assert!(ids.contains(&"UC-ALL-002".to_string()));
}

fn test_save_markdown(repo: &dyn UseCaseRepository) {
    let use_case = create_test_use_case();
    let markdown_content = "# Test Markdown\n\nThis is a test use case.".to_string();

    // Save use case first
    repo.save(&use_case).unwrap();

    // Save markdown
    repo.save_markdown(&use_case.id, &markdown_content).unwrap();

    // Note: We can't easily verify markdown content without backend-specific code
    // This test mainly ensures the method doesn't panic
    // Note: No cleanup since delete() was removed in PR #11
}
