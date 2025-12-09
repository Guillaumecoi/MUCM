//! Scenario-related Handlebars helpers.
//!
//! Provides helpers for:
//! - Creating links to use cases
//! - Merging scenario steps for extension scenarios
//! - Checking scenario types

use crate::core::domain::StepOrder;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
    RenderErrorReason,
};
use serde_json::Value;
use std::cmp::Ordering;

/// Register all scenario-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("use_case_link", Box::new(use_case_link_helper));
    handlebars.register_helper("merged_scenario_steps", Box::new(merged_scenario_steps_helper));
    handlebars.register_helper("is_main_scenario", Box::new(is_main_scenario_helper));
    handlebars.register_helper("is_extension_scenario", Box::new(is_extension_scenario_helper));
}

/// Helper to create a markdown link to another use case
/// Usage: {{use_case_link target_use_case_id}}
/// Returns: ../../category/UC-XXX-001/README.md
/// Resolves the category from the use case repository to build correct cross-category links
fn use_case_link_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let target_id = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    if target_id.is_empty() {
        return Ok(());
    }

    let link_path = resolve_use_case_link(target_id);
    out.write(&link_path)?;
    Ok(())
}

/// Resolve use case ID to relative link path
/// Returns: ../../category/UC-XXX-001/README.md
fn resolve_use_case_link(target_id: &str) -> String {
    use crate::config::Config;
    use crate::core::to_snake_case;

    // Try to load use case coordinator to find the category
    if let Ok(_config) = Config::load() {
        // Try to load all use cases to find the target
        if let Ok(coordinator) = crate::core::UseCaseCoordinator::load() {
            if let Some(target_uc) = coordinator
                .get_all_use_cases()
                .iter()
                .find(|uc| uc.id == target_id)
            {
                let category_snake = to_snake_case(&target_uc.category);
                // Path from current use case folder: ../../category/use-case-id/README.md
                return format!("../../{}/{}/README.md", category_snake, target_id);
            }
        }
    }

    // Fallback: assume same category if we can't resolve
    format!("../{}/README.md", target_id)
}

/// Helper to merge scenario steps for extension scenarios
/// Usage: {{merged_scenario_steps scenario all_scenarios}}
///
/// For main scenarios: Returns the scenario's own steps unchanged.
/// For extension scenarios:
///   1. Gets parent steps BEFORE extends_at_step (exclusive)
///   2. Adds ALL steps from the extension scenario
///   3. Handles returns_at_step:
///      - If returns_at >= extends_at: Appends parent steps FROM returns_at onward
///      - If returns_at < extends_at (LOOP): Appends parent steps from returns_at to extends_at
fn merged_scenario_steps_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get scenario (first param)
    let scenario = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "merged_scenario_steps requires scenario parameter".to_string(),
        ))
    })?;

    // Get all_scenarios (second param)
    let all_scenarios = h.param(1).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "merged_scenario_steps requires all_scenarios parameter".to_string(),
        ))
    })?;

    let scenario_value = scenario.value();

    // Check if this is an extension scenario
    let extends_id = scenario_value
        .get("extends_scenario_id")
        .and_then(|v| v.as_str());

    if extends_id.is_none() {
        // Main scenario - just return its steps
        let steps = scenario_value
            .get("steps")
            .cloned()
            .unwrap_or(Value::Array(vec![]));
        let json_str = serde_json::to_string(&steps)
            .map_err(|e| RenderError::from(RenderErrorReason::Other(e.to_string())))?;
        out.write(&json_str)?;
        return Ok(());
    }

    let extends_id = extends_id.unwrap();
    let extends_at = scenario_value
        .get("extends_at_step")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(
                "Extension scenario missing extends_at_step".to_string(),
            ))
        })?;
    let returns_at = scenario_value
        .get("returns_at_step")
        .and_then(|v| v.as_str());

    // Find parent scenario
    let all_scenarios_array = all_scenarios.value().as_array().ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "all_scenarios must be an array".to_string(),
        ))
    })?;

    let parent = all_scenarios_array
        .iter()
        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(extends_id))
        .ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(format!(
                "Parent scenario '{}' not found",
                extends_id
            )))
        })?;

    let parent_steps = parent
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(
                "Parent scenario has no steps".to_string(),
            ))
        })?;

    let empty_steps = vec![];
    let current_steps = scenario_value
        .get("steps")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_steps);

    let mut merged: Vec<Value> = Vec::new();

    // Step 1: Parent steps BEFORE extends_at_step (exclusive)
    for step in parent_steps {
        let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
        if StepOrder::compare(order, extends_at) == Ordering::Less {
            merged.push(step.clone());
        }
    }

    // Step 2: ALL current scenario steps
    merged.extend(current_steps.iter().cloned());

    // Step 3: Handle returns_at_step
    if let Some(returns_at) = returns_at {
        if StepOrder::compare(returns_at, extends_at) != Ordering::Less {
            // Normal case: returns_at >= extends_at, get steps FROM returns_at onward
            for step in parent_steps {
                let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
                if StepOrder::compare(order, returns_at) != Ordering::Less {
                    merged.push(step.clone());
                }
            }
        } else {
            // LOOP case: returns_at < extends_at
            // Get steps from returns_at up to (but not including) extends_at
            for step in parent_steps {
                let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
                if StepOrder::compare(order, returns_at) != Ordering::Less
                    && StepOrder::compare(order, extends_at) == Ordering::Less
                {
                    merged.push(step.clone());
                }
            }
        }
    }

    // Write as JSON which Handlebars will parse
    let json_str = serde_json::to_string(&merged)
        .map_err(|e| RenderError::from(RenderErrorReason::Other(e.to_string())))?;
    out.write(&json_str)?;

    Ok(())
}

/// Helper to check if a scenario is a main scenario (not an extension)
/// Usage: {{#if (is_main_scenario scenario)}}...{{/if}}
fn is_main_scenario_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let scenario = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "is_main_scenario requires scenario parameter".to_string(),
        ))
    })?;

    let is_main = scenario
        .value()
        .get("is_main")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let has_extends = scenario
        .value()
        .get("extends_scenario_id")
        .map(|v| !v.is_null() && v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .unwrap_or(false);

    if is_main && !has_extends {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if a scenario is an extension scenario
/// Usage: {{#if (is_extension_scenario scenario)}}...{{/if}}
fn is_extension_scenario_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let scenario = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "is_extension_scenario requires scenario parameter".to_string(),
        ))
    })?;

    let has_extends = scenario
        .value()
        .get("extends_scenario_id")
        .map(|v| !v.is_null() && v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .unwrap_or(false);

    if has_extends {
        out.write("true")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_use_case_link_helper_registered() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Verify the helper is registered and can be used in templates
        let template = "{{use_case_link target_id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "target_id": "UC-TEST-001"
        });

        // Should not panic - helper is registered and callable
        let result = handlebars.render("test", &data);
        assert!(result.is_ok(), "use_case_link helper should be registered");
    }

    #[test]
    fn test_use_case_link_helper_with_empty_string() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "Link: {{use_case_link target_id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "target_id": ""
        });

        let result = handlebars.render("test", &data).unwrap();
        // Empty string should return empty string
        assert_eq!(result, "Link: ");
    }

    #[test]
    fn test_use_case_link_helper_with_unknown_id() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "See: {{use_case_link target_id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "target_id": "UC-UNKNOWN-999"
        });

        let result = handlebars.render("test", &data).unwrap();
        // Unknown IDs should fall back to same-category assumption
        assert_eq!(
            result, "See: ../UC-UNKNOWN-999/README.md",
            "Should assume same category for unknown use cases"
        );
    }

    #[test]
    fn test_use_case_link_helper_with_null_value() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "Link: {{use_case_link target_id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "target_id": null
        });

        let result = handlebars.render("test", &data).unwrap();
        // Null should be treated as empty string
        assert_eq!(result, "Link: ");
    }

    #[test]
    fn test_use_case_link_fallback_behavior() {
        // Test the fallback behavior when coordinator can't be loaded
        let link = resolve_use_case_link("UC-FALLBACK-001");
        // Should fall back to same-category assumption
        assert_eq!(
            link, "../UC-FALLBACK-001/README.md",
            "Should provide fallback link when coordinator unavailable"
        );
    }

    #[test]
    fn test_merged_scenario_steps_main_scenario() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{merged_scenario_steps scenario scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S01",
                "is_main": true,
                "steps": [
                    {"order": "1", "acting_actor": "user", "action": "step 1"},
                    {"order": "2", "acting_actor": "user", "action": "step 2"}
                ]
            },
            "scenarios": [
                {
                    "id": "UC-AUTH-001-S01",
                    "is_main": true,
                    "steps": [
                        {"order": "1", "acting_actor": "user", "action": "step 1"},
                        {"order": "2", "acting_actor": "user", "action": "step 2"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["order"], "1");
        assert_eq!(parsed[1]["order"], "2");
    }

    #[test]
    fn test_merged_scenario_steps_extension_returns_after() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{merged_scenario_steps scenario scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        // Parent: steps 1, 2, 3, 4, 5
        // Extension: extends_at=3, returns_at=4
        // Expected: 1, 2, [extension], 4, 5
        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S02",
                "extends_scenario_id": "UC-AUTH-001-S01",
                "extends_at_step": "3",
                "returns_at_step": "4",
                "steps": [
                    {"order": "3a", "acting_actor": "user", "action": "alternative"}
                ]
            },
            "scenarios": [
                {
                    "id": "UC-AUTH-001-S01",
                    "is_main": true,
                    "steps": [
                        {"order": "1", "acting_actor": "user", "action": "step 1"},
                        {"order": "2", "acting_actor": "user", "action": "step 2"},
                        {"order": "3", "acting_actor": "user", "action": "step 3"},
                        {"order": "4", "acting_actor": "user", "action": "step 4"},
                        {"order": "5", "acting_actor": "user", "action": "step 5"}
                    ]
                },
                {
                    "id": "UC-AUTH-001-S02",
                    "extends_scenario_id": "UC-AUTH-001-S01",
                    "extends_at_step": "3",
                    "returns_at_step": "4",
                    "steps": [
                        {"order": "3a", "acting_actor": "user", "action": "alternative"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();

        // Should be: 1, 2, 3a, 4, 5
        assert_eq!(parsed.len(), 5);
        assert_eq!(parsed[0]["order"], "1");
        assert_eq!(parsed[1]["order"], "2");
        assert_eq!(parsed[2]["order"], "3a");
        assert_eq!(parsed[3]["order"], "4");
        assert_eq!(parsed[4]["order"], "5");
    }

    #[test]
    fn test_merged_scenario_steps_loop_case() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{merged_scenario_steps scenario scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        // Parent: steps 1, 2, 3, 4, 5
        // Exception: extends_at=4, returns_at=2 (LOOP back!)
        // Expected: 1, 2, 3, [exception], 2, 3
        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-E01",
                "extends_scenario_id": "UC-AUTH-001-S01",
                "extends_at_step": "4",
                "returns_at_step": "2",
                "steps": [
                    {"order": "4a", "acting_actor": "system", "action": "shows error"}
                ]
            },
            "scenarios": [
                {
                    "id": "UC-AUTH-001-S01",
                    "is_main": true,
                    "steps": [
                        {"order": "1", "acting_actor": "user", "action": "step 1"},
                        {"order": "2", "acting_actor": "user", "action": "step 2"},
                        {"order": "3", "acting_actor": "user", "action": "step 3"},
                        {"order": "4", "acting_actor": "user", "action": "step 4"},
                        {"order": "5", "acting_actor": "user", "action": "step 5"}
                    ]
                },
                {
                    "id": "UC-AUTH-001-E01",
                    "extends_scenario_id": "UC-AUTH-001-S01",
                    "extends_at_step": "4",
                    "returns_at_step": "2",
                    "steps": [
                        {"order": "4a", "acting_actor": "system", "action": "shows error"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();

        // Should be: 1, 2, 3, 4a, 2, 3 (loops back)
        assert_eq!(parsed.len(), 6);
        assert_eq!(parsed[0]["order"], "1");
        assert_eq!(parsed[1]["order"], "2");
        assert_eq!(parsed[2]["order"], "3");
        assert_eq!(parsed[3]["order"], "4a");
        assert_eq!(parsed[4]["order"], "2"); // Loop back
        assert_eq!(parsed[5]["order"], "3"); // Loop continues
    }

    #[test]
    fn test_merged_scenario_steps_no_return() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{merged_scenario_steps scenario scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        // Parent: steps 1, 2, 3, 4, 5
        // Exception: extends_at=3, no returns_at (terminal)
        // Expected: 1, 2, [exception]
        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-E02",
                "extends_scenario_id": "UC-AUTH-001-S01",
                "extends_at_step": "3",
                "steps": [
                    {"order": "3a", "acting_actor": "system", "action": "terminates"}
                ]
            },
            "scenarios": [
                {
                    "id": "UC-AUTH-001-S01",
                    "is_main": true,
                    "steps": [
                        {"order": "1", "acting_actor": "user", "action": "step 1"},
                        {"order": "2", "acting_actor": "user", "action": "step 2"},
                        {"order": "3", "acting_actor": "user", "action": "step 3"},
                        {"order": "4", "acting_actor": "user", "action": "step 4"},
                        {"order": "5", "acting_actor": "user", "action": "step 5"}
                    ]
                },
                {
                    "id": "UC-AUTH-001-E02",
                    "extends_scenario_id": "UC-AUTH-001-S01",
                    "extends_at_step": "3",
                    "steps": [
                        {"order": "3a", "acting_actor": "system", "action": "terminates"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();

        // Should be: 1, 2, 3a (ends there)
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0]["order"], "1");
        assert_eq!(parsed[1]["order"], "2");
        assert_eq!(parsed[2]["order"], "3a");
    }

    #[test]
    fn test_is_main_scenario_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (is_main_scenario scenario)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S01",
                "is_main": true
            }
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_is_main_scenario_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (is_main_scenario scenario)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S02",
                "is_main": false,
                "extends_scenario_id": "UC-AUTH-001-S01"
            }
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_is_extension_scenario_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (is_extension_scenario scenario)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S02",
                "extends_scenario_id": "UC-AUTH-001-S01"
            }
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_is_extension_scenario_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (is_extension_scenario scenario)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S01",
                "is_main": true
            }
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_is_extension_scenario_helper_null_extends() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (is_extension_scenario scenario)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenario": {
                "id": "UC-AUTH-001-S01",
                "extends_scenario_id": null
            }
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }
}
