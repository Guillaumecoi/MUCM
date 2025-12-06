/// Test to trace where actor names get lost during TOML -> Markdown conversion  
use mucm::core::{Scenario, UseCase};
use std::fs;
use std::path::PathBuf;

// Helper to create a scenario step with receiver
fn create_step_with_receiver(
    order: &str,
    acting: &str,
    receiving: &str,
    action: &str,
) -> serde_json::Value {
    serde_json::json!({
        "order": order,
        "acting_actor": acting,
        "receiving_actor": receiving,
        "action": action
    })
}

#[test]
fn test_actor_names_preserved_in_use_case_serialization() {
    // Create a use case with a scenario that has actor names
    let mut use_case = UseCase::new(
        "UC-TEST-001".to_string(),
        "Test Actor Rendering".to_string(),
        "Testing".to_string(),
        "TST".to_string(),
        "Test description".to_string(),
        "High".to_string(),
    )
    .unwrap();

    // Create a scenario with steps using JSON
    let scenario_json = serde_json::json!({
        "id": "UC-TEST-001-S01",
        "title": "Test Scenario",
        "description": "Test description",
        "scenario_type": "happy_path",
        "status": "deployed",
        "is_main": true,
        "primary_actor": "Guest User",
        "supporting_actors": ["System", "Database"],
        "persona": null,
        "extends_scenario_id": null,
        "extends_at_step": null,
        "returns_at_step": null,
        "repeat_blocks": [],
        "steps": [
            create_step_with_receiver("1", "Guest User", "System", "navigates to page"),
            create_step_with_receiver("2", "System", "Database", "queries data"),
            create_step_with_receiver("3", "Database", "System", "returns results")
        ],
        "preconditions": [],
        "postconditions": [],
        "references": [],
        "extra": {}
    });

    let scenario: Scenario =
        serde_json::from_value(scenario_json).expect("Failed to create scenario");
    use_case.scenarios.push(scenario);

    // Serialize to JSON (simulating what happens before template rendering)
    let json_value = serde_json::to_value(&use_case).expect("Failed to serialize to JSON");

    // Check that scenarios exist
    assert!(
        json_value.get("scenarios").is_some(),
        "scenarios field is missing"
    );

    let scenarios = json_value["scenarios"]
        .as_array()
        .expect("scenarios should be an array");
    assert_eq!(scenarios.len(), 1, "Should have 1 scenario");

    let scenario_json_out = &scenarios[0];

    // Check scenario has steps
    assert!(
        scenario_json_out.get("steps").is_some(),
        "steps field is missing"
    );

    let steps = scenario_json_out["steps"]
        .as_array()
        .expect("steps should be an array");
    assert_eq!(steps.len(), 3, "Should have 3 steps");

    // Check that actor names are preserved in step 1
    let step1 = &steps[0];
    println!(
        "Step 1 JSON: {}",
        serde_json::to_string_pretty(step1).unwrap()
    );

    assert_eq!(
        step1["acting_actor"].as_str(),
        Some("Guest User"),
        "Step 1 acting_actor should be 'Guest User', got: {:?}",
        step1["acting_actor"]
    );
    assert_eq!(
        step1["receiving_actor"].as_str(),
        Some("System"),
        "Step 1 receiving_actor should be 'System', got: {:?}",
        step1["receiving_actor"]
    );

    // Check step 2
    let step2 = &steps[1];
    assert_eq!(
        step2["acting_actor"].as_str(),
        Some("System"),
        "Step 2 acting_actor should be 'System'"
    );
    assert_eq!(
        step2["receiving_actor"].as_str(),
        Some("Database"),
        "Step 2 receiving_actor should be 'Database'"
    );

    // Check step 3
    let step3 = &steps[2];
    assert_eq!(
        step3["acting_actor"].as_str(),
        Some("Database"),
        "Step 3 acting_actor should be 'Database'"
    );
    assert_eq!(
        step3["receiving_actor"].as_str(),
        Some("System"),
        "Step 3 receiving_actor should be 'System'"
    );
}

#[test]
fn test_actor_names_preserved_through_toml_conversion() {
    // Simulate the TOML -> JSON conversion that happens in load_all()
    let toml_content = r#"
id = "UC-TEST-002"
title = "Test TOML Conversion"
category = "Testing"
category_abbreviation = "TST"
description = "Test if actor names survive TOML conversion"
priority = "High"

[metadata]
created_at = "2025-12-06T00:00:00Z"
updated_at = "2025-12-06T00:00:00Z"

[methodology_fields]

[[scenarios]]
id = "UC-TEST-002-S01"
title = "Test Scenario"
description = ""
scenario_type = "happy_path"
status = "deployed"
is_main = true
primary_actor = "Guest User"
supporting_actors = ["System", "Database"]

[[scenarios.steps]]
order = "1"
acting_actor = "Guest User"
receiving_actor = "System"
action = "navigates to page"

[[scenarios.steps]]
order = "2"
acting_actor = "System"
receiving_actor = "Database"
action = "queries data"
"#;

    // Parse TOML
    let toml_value: toml::Value = toml::from_str(toml_content).expect("Failed to parse TOML");

    println!("TOML value: {:#?}", toml_value);

    // Convert to JSON (this is what the repository does)
    let json_str = serde_json::to_string(&toml_value).expect("Failed to convert TOML to JSON");

    println!("JSON string: {}", json_str);

    // Parse as UseCase
    let use_case: UseCase =
        serde_json::from_str(&json_str).expect("Failed to deserialize UseCase from JSON");

    // Check scenarios exist
    assert_eq!(use_case.scenarios.len(), 1, "Should have 1 scenario");

    let scenario = &use_case.scenarios[0];
    assert_eq!(scenario.steps.len(), 2, "Should have 2 steps");

    // Check actor names in steps
    let step1 = &scenario.steps[0];
    println!(
        "Step 1: acting_actor='{}', receiving_actor='{:?}'",
        step1.acting_actor, step1.receiving_actor
    );

    assert_eq!(
        step1.acting_actor, "Guest User",
        "Step 1 acting_actor should be 'Guest User', got: '{}'",
        step1.acting_actor
    );
    assert_eq!(
        step1.receiving_actor.as_deref(),
        Some("System"),
        "Step 1 receiving_actor should be 'System', got: {:?}",
        step1.receiving_actor
    );

    let step2 = &scenario.steps[1];
    assert_eq!(
        step2.acting_actor, "System",
        "Step 2 acting_actor should be 'System', got: '{}'",
        step2.acting_actor
    );
    assert_eq!(
        step2.receiving_actor.as_deref(),
        Some("Database"),
        "Step 2 receiving_actor should be 'Database', got: {:?}",
        step2.receiving_actor
    );
}

#[test]
fn test_real_uc_auth_001_actor_names() {
    // Load the actual UC-AUTH-001.toml file
    let toml_path =
        PathBuf::from("examples/ecommerce-demo/use-cases-data/authentication/UC-AUTH-001.toml");

    let toml_content = fs::read_to_string(&toml_path).expect("Failed to read UC-AUTH-001.toml");

    // Parse TOML
    let toml_value: toml::Value = toml::from_str(&toml_content).expect("Failed to parse TOML");

    // Convert to JSON (this is what the repository does)
    let json_str = serde_json::to_string(&toml_value).expect("Failed to convert TOML to JSON");

    // Parse as UseCase
    let use_case: UseCase =
        serde_json::from_str(&json_str).expect("Failed to deserialize UseCase from JSON");

    // Check the first scenario (happy path registration)
    assert!(
        !use_case.scenarios.is_empty(),
        "Should have at least one scenario"
    );

    let scenario = &use_case.scenarios[0];
    assert!(
        !scenario.steps.is_empty(),
        "First scenario should have steps"
    );

    // Check the first step - should have actor names
    let step1 = &scenario.steps[0];
    println!("\nUC-AUTH-001 Step 1:");
    println!("  acting_actor: '{}'", step1.acting_actor);
    println!("  receiving_actor: {:?}", step1.receiving_actor);
    println!("  action: '{}'", step1.action);

    // Actor names should NOT be empty
    assert!(
        !step1.acting_actor.is_empty(),
        "Step 1 acting_actor should not be empty"
    );

    // Check if we have multiple steps with actors
    for (i, step) in scenario.steps.iter().enumerate() {
        println!("\nStep {}:", i + 1);
        println!("  acting_actor: '{}'", step.acting_actor);
        println!("  receiving_actor: {:?}", step.receiving_actor);

        if step.acting_actor.is_empty() {
            panic!("Step {} has empty acting_actor!", i + 1);
        }
    }

    // Now convert to JSON for template rendering (simulating markdown generator)
    let use_case_json = serde_json::to_value(&use_case).expect("Failed to convert UseCase to JSON");

    println!("\n\n=== JSON FOR TEMPLATE ===");
    println!("{}", serde_json::to_string_pretty(&use_case_json).unwrap());

    // Check scenarios array in JSON
    let scenarios_array = use_case_json
        .get("scenarios")
        .expect("No scenarios in JSON")
        .as_array()
        .expect("scenarios not an array");

    let first_scenario = &scenarios_array[0];
    let steps_array = first_scenario
        .get("steps")
        .expect("No steps in first scenario")
        .as_array()
        .expect("steps not an array");

    let first_step = &steps_array[0];
    println!("\n\n=== FIRST STEP JSON ===");
    println!("{}", serde_json::to_string_pretty(&first_step).unwrap());

    // Verify actor fields exist in JSON
    assert!(
        first_step.get("acting_actor").is_some(),
        "JSON missing acting_actor field"
    );
    assert_eq!(
        first_step.get("acting_actor").unwrap().as_str().unwrap(),
        "Guest User",
        "JSON acting_actor should be 'Guest User'"
    );
}
