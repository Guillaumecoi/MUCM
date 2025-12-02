//! Integration tests for scenario references
//!
//! Tests scenario references functionality across both TOML and SQLite backends

use mucm::core::{
    Actor, ReferenceType, Scenario, ScenarioReference, ScenarioType, SqliteUseCaseRepository,
    UseCase, UseCaseRepository,
};
use serial_test::serial;
use std::env;
use tempfile::TempDir;

mod common;

/// Test helper: Create a test use case with scenarios
fn create_test_use_case_with_scenarios() -> UseCase {
    let mut use_case = UseCase::new(
        "UC-TEST-001".to_string(),
        "Test Use Case".to_string(),
        "test".to_string(),
        "TES".to_string(),
        "A test use case for scenario reference testing".to_string(),
        "medium".to_string(),
    )
    .unwrap();

    // Add three scenarios
    let scenario1 = Scenario::new(
        "UC-TEST-001-S01".to_string(),
        "Happy Path".to_string(),
        "Main success scenario".to_string(),
        ScenarioType::HappyPath,
        Actor::User,
    );

    let scenario2 = Scenario::new(
        "UC-TEST-001-S02".to_string(),
        "Alternative Flow".to_string(),
        "Alternative path scenario".to_string(),
        ScenarioType::AlternativeFlow,
        Actor::User,
    );

    let scenario3 = Scenario::new(
        "UC-TEST-001-S03".to_string(),
        "Exception Flow".to_string(),
        "Error handling scenario".to_string(),
        ScenarioType::ExceptionFlow,
        Actor::User,
    );

    use_case.add_scenario(scenario1);
    use_case.add_scenario(scenario2);
    use_case.add_scenario(scenario3);

    use_case
}

/// Test helper: Create TOML repository for testing
fn create_toml_repository() -> (TempDir, Box<dyn UseCaseRepository>) {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    // Create minimal source-templates for testing
    common::create_minimal_source_templates(std::path::Path::new(".")).unwrap();

    use mucm::config::ConfigFileManager;
    let config = mucm::config::Config::default();
    ConfigFileManager::save_in_dir(&config, ".").unwrap();

    let config = mucm::config::Config::load().unwrap();
    let repo =
        Box::new(mucm::core::TomlUseCaseRepository::new(config)) as Box<dyn UseCaseRepository>;

    (temp_dir, repo)
}

/// Test helper: Create SQLite repository for testing
fn create_sqlite_repository() -> (TempDir, Box<dyn UseCaseRepository>) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let repo =
        Box::new(SqliteUseCaseRepository::new(&db_path).unwrap()) as Box<dyn UseCaseRepository>;

    (temp_dir, repo)
}

#[test]
#[serial]
fn test_toml_save_and_load_scenario_with_references() {
    let (_temp_dir, repo) = create_toml_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Add references to scenario 1
    let reference1 = ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S02".to_string(),
        "extends".to_string(),
    )
    .with_description("Extends the alternative flow".to_string());

    let reference2 = ScenarioReference::new(
        ReferenceType::UseCase,
        "UC-AUTH-001".to_string(),
        "depends_on".to_string(),
    );

    use_case.scenarios[0].add_reference(reference1.clone());
    use_case.scenarios[0].add_reference(reference2.clone());

    // Save the use case
    repo.save(&use_case).unwrap();

    // Load it back
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify references were preserved
    assert_eq!(loaded.scenarios[0].references.len(), 2);
    assert_eq!(loaded.scenarios[0].references[0], reference1);
    assert_eq!(loaded.scenarios[0].references[1], reference2);
}

#[test]
#[serial]
fn test_sqlite_save_and_load_scenario_with_references() {
    let (_temp_dir, repo) = create_sqlite_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Add references to scenario 1
    let reference1 = ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S02".to_string(),
        "extends".to_string(),
    )
    .with_description("Extends the alternative flow".to_string());

    let reference2 = ScenarioReference::new(
        ReferenceType::UseCase,
        "UC-AUTH-001".to_string(),
        "depends_on".to_string(),
    );

    use_case.scenarios[0].add_reference(reference1.clone());
    use_case.scenarios[0].add_reference(reference2.clone());

    // Save the use case
    repo.save(&use_case).unwrap();

    // Load it back
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify references were preserved
    assert_eq!(loaded.scenarios[0].references.len(), 2);
    assert_eq!(loaded.scenarios[0].references[0], reference1);
    assert_eq!(loaded.scenarios[0].references[1], reference2);
}

#[test]
#[serial]
fn test_toml_multiple_references_per_scenario() {
    let (_temp_dir, repo) = create_toml_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Add multiple references to different scenarios
    use_case.scenarios[0].add_reference(ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S02".to_string(),
        "includes".to_string(),
    ));

    use_case.scenarios[0].add_reference(ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S03".to_string(),
        "precedes".to_string(),
    ));

    use_case.scenarios[1].add_reference(ScenarioReference::new(
        ReferenceType::UseCase,
        "UC-AUTH-001".to_string(),
        "depends_on".to_string(),
    ));

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify all references preserved
    assert_eq!(loaded.scenarios[0].references.len(), 2);
    assert_eq!(loaded.scenarios[1].references.len(), 1);
    assert_eq!(loaded.scenarios[2].references.len(), 0);
}

#[test]
#[serial]
fn test_sqlite_multiple_references_per_scenario() {
    let (_temp_dir, repo) = create_sqlite_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Add multiple references to different scenarios
    use_case.scenarios[0].add_reference(ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S02".to_string(),
        "includes".to_string(),
    ));

    use_case.scenarios[0].add_reference(ScenarioReference::new(
        ReferenceType::Scenario,
        "UC-TEST-001-S03".to_string(),
        "precedes".to_string(),
    ));

    use_case.scenarios[1].add_reference(ScenarioReference::new(
        ReferenceType::UseCase,
        "UC-AUTH-001".to_string(),
        "depends_on".to_string(),
    ));

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify all references preserved
    assert_eq!(loaded.scenarios[0].references.len(), 2);
    assert_eq!(loaded.scenarios[1].references.len(), 1);
    assert_eq!(loaded.scenarios[2].references.len(), 0);
}

#[test]
#[serial]
fn test_toml_backward_compatibility_no_references() {
    let (_temp_dir, repo) = create_toml_repository();

    // Create use case without adding any references
    let use_case = create_test_use_case_with_scenarios();

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify scenarios have empty references
    assert!(loaded.scenarios[0].references.is_empty());
    assert!(loaded.scenarios[1].references.is_empty());
    assert!(loaded.scenarios[2].references.is_empty());
}

#[test]
#[serial]
fn test_sqlite_backward_compatibility_no_references() {
    let (_temp_dir, repo) = create_sqlite_repository();

    // Create use case without adding any references
    let use_case = create_test_use_case_with_scenarios();

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify scenarios have empty references
    assert!(loaded.scenarios[0].references.is_empty());
    assert!(loaded.scenarios[1].references.is_empty());
    assert!(loaded.scenarios[2].references.is_empty());
}

#[test]
#[serial]
fn test_toml_reference_with_all_relationship_types() {
    let (_temp_dir, repo) = create_toml_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Test all relationship types
    let relationships = [
        "includes",
        "extends",
        "depends_on",
        "precedes",
        "alternative_to",
    ];

    for (i, relationship) in relationships.iter().enumerate() {
        use_case.scenarios[0].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            format!("UC-TEST-001-S0{}", i + 2),
            relationship.to_string(),
        ));
    }

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify all relationship types preserved
    assert_eq!(loaded.scenarios[0].references.len(), 5);
    for (i, relationship) in relationships.iter().enumerate() {
        assert_eq!(
            loaded.scenarios[0].references[i].relationship,
            *relationship
        );
    }
}

#[test]
#[serial]
fn test_sqlite_reference_with_all_relationship_types() {
    let (_temp_dir, repo) = create_sqlite_repository();

    let mut use_case = create_test_use_case_with_scenarios();

    // Test all relationship types
    let relationships = [
        "includes",
        "extends",
        "depends_on",
        "precedes",
        "alternative_to",
    ];

    for (i, relationship) in relationships.iter().enumerate() {
        use_case.scenarios[0].add_reference(ScenarioReference::new(
            ReferenceType::Scenario,
            format!("UC-TEST-001-S0{}", i + 2),
            relationship.to_string(),
        ));
    }

    // Save and load
    repo.save(&use_case).unwrap();
    let loaded = repo.load_by_id(&use_case.id).unwrap().unwrap();

    // Verify all relationship types preserved
    assert_eq!(loaded.scenarios[0].references.len(), 5);
    for (i, relationship) in relationships.iter().enumerate() {
        assert_eq!(
            loaded.scenarios[0].references[i].relationship,
            *relationship
        );
    }
}
