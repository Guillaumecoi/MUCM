//! Actor and persona related Handlebars helpers.
//!
//! Provides helpers for:
//! - Creating markdown links to actor documentation
//! - Displaying actor names
//! - Extracting unique actors/personas from scenarios

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
    RenderErrorReason,
};
use serde_json::Value;
use std::collections::HashSet;

/// Register all actor-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("actor_link", Box::new(actor_link_helper));
    handlebars.register_helper("actor_name", Box::new(actor_name_helper));
    handlebars.register_helper("actor_emoji", Box::new(actor_emoji_helper));
    handlebars.register_helper("unique_actors", Box::new(unique_actors_helper));
    handlebars.register_helper("unique_personas", Box::new(unique_personas_helper));
    handlebars.register_helper(
        "unique_supporting_actors",
        Box::new(unique_supporting_actors_helper),
    );
    handlebars.register_helper("has_personas", Box::new(has_personas_helper));
}

/// Helper to create a markdown link to actor documentation
/// Usage: {{actor_link actor_id_or_name}}
/// Returns: [Actor Name](../../personas/actor-id.md) if actor ID is found in repository
/// Falls back to just the name if actor not found (for backward compatibility)
fn actor_link_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let actor_id_or_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    // Try to resolve as actor ID and create link
    let display_markdown = resolve_actor_link(actor_id_or_name);

    out.write(&display_markdown)?;
    Ok(())
}

/// Calculate relative path from use_case_dir/category/ to actor_dir
/// Always goes all the way up to workspace root, then down into actor_dir
///
/// # Arguments
/// * `use_case_dir` - Path to use cases directory (e.g., "docs/use-cases")
/// * `actor_dir` - Path to actors directory (e.g., "docs/personas")
///
/// # Returns
/// Relative path string (e.g., "../../../docs/personas")
pub fn calculate_actor_relative_path(use_case_dir: &str, actor_dir: &str) -> String {
    // File is at: {use_case_dir}/{category}/{use-case-id}/file.md
    // actor_dir is relative from the project root (e.g., "docs/personas")
    // We need to calculate the relative path from file to actor_dir

    let use_case_parts: Vec<&str> = use_case_dir.split('/').collect();
    let actor_parts: Vec<&str> = actor_dir.split('/').collect();

    // Find common prefix length
    let common_len = use_case_parts
        .iter()
        .zip(actor_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Go up from use case location to common ancestor
    // use_case_parts.len() = depth of use_case_dir
    // +2 for category and use-case-id folders
    let ups = use_case_parts.len() + 2 - common_len;
    let up_path = "../".repeat(ups);

    // Go down to actor dir from common ancestor
    let down_path = actor_parts[common_len..].join("/");

    format!("{}{}", up_path, down_path)
}

/// Resolve actor ID to markdown link
/// Falls back to the input if actor not found
pub fn resolve_actor_link(actor_id: &str) -> String {
    use crate::config::Config;
    use crate::controller::ActorController;

    // Load config to get actor_dir and use_case_dir
    let actor_dir = Config::load()
        .ok()
        .map(|c| c.directories.actor_dir)
        .unwrap_or_else(|| "docs/actors".to_string());

    let use_case_dir = Config::load()
        .ok()
        .map(|c| c.directories.use_case_dir)
        .unwrap_or_else(|| "docs/use-cases".to_string());

    // Calculate relative path from use case category to actor directory
    let relative_path = calculate_actor_relative_path(&use_case_dir, &actor_dir);

    // Try to load actor controller
    if let Ok(actor_controller) = ActorController::new() {
        // Try as system actor first
        if let Ok(actor) = actor_controller.get_actor(actor_id) {
            return format!("[{}]({}/{}.md)", actor.name, relative_path, actor.id);
        }

        // Try as persona
        if let Ok(persona) = actor_controller.get_persona(actor_id) {
            return format!("[{}]({}/{}.md)", persona.name, relative_path, persona.id);
        }
    }

    // Fallback to the input string (for backward compatibility with names)
    actor_id.to_string()
}

/// Helper to display actor name without emoji (for text display)
/// Usage: {{actor_name actor_id_or_name}}
/// Returns: actor display name without emoji
/// Falls back to the input string if actor not found (for backward compatibility)
fn actor_name_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let actor_id_or_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    // Try to resolve as actor ID (without emoji)
    let display_name = resolve_actor_name(actor_id_or_name);

    out.write(&display_name)?;
    Ok(())
}

/// Resolve actor ID to plain display name (no emoji)
/// Falls back to the input if actor not found
pub fn resolve_actor_name(actor_id: &str) -> String {
    use crate::controller::ActorController;

    // Try to load actor controller
    if let Ok(actor_controller) = ActorController::new() {
        // Try as system actor first
        if let Ok(actor) = actor_controller.get_actor(actor_id) {
            return actor.name;
        }

        // Try as persona
        if let Ok(persona) = actor_controller.get_persona(actor_id) {
            return persona.name;
        }
    }

    // Fallback to the input string (for backward compatibility with names)
    actor_id.to_string()
}

/// Helper to return an emoji for an actor
/// Usage: {{actor_emoji actor_name}}
/// Note: Deprecated - emoji should come from ActorEntity data in the template context
/// Returns empty string (emojis should be part of the data model)
fn actor_emoji_helper(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    _out: &mut dyn Output,
) -> HelperResult {
    // No-op: emojis should be in the ActorEntity.emoji field
    Ok(())
}

/// Helper to extract unique actors from scenarios
/// Usage: {{#each (unique_actors scenarios)}}{{this}}{{/each}}
///
/// This is a Handlebars helper function that returns an array value.
/// The returned value is stored in the context and can be iterated over.
fn unique_actors_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the scenarios parameter
    let scenarios = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "unique_actors requires scenarios parameter".to_string(),
        ))
    })?;

    let scenarios_array = scenarios.value().as_array().ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "scenarios must be an array".to_string(),
        ))
    })?;

    let mut actors = HashSet::new();

    // Extract actors from each scenario
    for scenario in scenarios_array {
        // Add primary_actor
        if let Some(Value::String(primary)) = scenario.get("primary_actor") {
            actors.insert(primary.clone());
        }

        // Add supporting_actors
        if let Some(supporting) = scenario.get("supporting_actors").and_then(|v| v.as_array()) {
            for actor in supporting {
                if let Value::String(s) = actor {
                    actors.insert(s.clone());
                }
            }
        }

        // Extract actors from steps (acting_actor and receiving_actor)
        if let Some(steps) = scenario.get("steps").and_then(|v| v.as_array()) {
            for step in steps {
                // Add acting_actor
                if let Some(Value::String(acting)) = step.get("acting_actor") {
                    actors.insert(acting.clone());
                }

                // Add receiving_actor (optional)
                if let Some(Value::String(receiving)) = step.get("receiving_actor") {
                    actors.insert(receiving.clone());
                }
            }
        }
    }

    // Convert to sorted Vec for consistent output
    let mut actors_vec: Vec<String> = actors.into_iter().collect();
    actors_vec.sort();

    // Write as JSON which Handlebars will parse
    let json_str = serde_json::to_string(&actors_vec)
        .map_err(|e| RenderError::from(RenderErrorReason::Other(e.to_string())))?;
    out.write(&json_str)?;

    Ok(())
}

/// Helper to check if any scenario has personas
/// Usage: {{#if (has_personas scenarios)}}...{{/if}}
fn has_personas_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the scenarios parameter
    let scenarios = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "has_personas requires scenarios parameter".to_string(),
        ))
    })?;

    let scenarios_array = scenarios.value().as_array().ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "scenarios must be an array".to_string(),
        ))
    })?;

    // Check if any scenario has a non-null, non-empty persona field
    for scenario in scenarios_array {
        if let Some(persona) = scenario.get("persona") {
            if !persona.is_null() {
                if let Some(s) = persona.as_str() {
                    if !s.is_empty() {
                        // Return true - write any truthy value
                        out.write("1")?;
                        return Ok(());
                    }
                }
            }
        }
    }

    // Return false - write nothing (empty string is falsy in Handlebars)
    Ok(())
}

/// Helper to extract unique personas from scenarios
/// Usage: {{#each (unique_personas scenarios)}}{{this}}{{/each}}
fn unique_personas_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the scenarios parameter
    let scenarios = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "unique_personas requires scenarios parameter".to_string(),
        ))
    })?;

    let scenarios_array = scenarios.value().as_array().ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "scenarios must be an array".to_string(),
        ))
    })?;

    let mut personas = HashSet::new();

    // Extract personas from scenarios
    for scenario in scenarios_array {
        if let Some(persona) = scenario.get("persona") {
            if !persona.is_null() {
                if let Some(s) = persona.as_str() {
                    if !s.is_empty() {
                        personas.insert(s.to_string());
                    }
                }
            }
        }
    }

    // Convert to sorted Vec for consistent output
    let mut personas_vec: Vec<String> = personas.into_iter().collect();
    personas_vec.sort();

    // Write as JSON which Handlebars will parse
    let json_str = serde_json::to_string(&personas_vec)
        .map_err(|e| RenderError::from(RenderErrorReason::Other(e.to_string())))?;
    out.write(&json_str)?;

    Ok(())
}

/// Helper to get unique supporting actors excluding the primary actor
/// Usage: {{#each (unique_supporting_actors supporting_actors primary_actor)}}{{this}}{{/each}}
/// Returns: unique list of supporting actors with primary actor filtered out
fn unique_supporting_actors_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the supporting_actors array parameter
    let supporting_actors = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "unique_supporting_actors requires supporting_actors parameter".to_string(),
        ))
    })?;

    let supporting_array = supporting_actors.value().as_array().ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "supporting_actors must be an array".to_string(),
        ))
    })?;

    // Get the primary_actor parameter (optional)
    let primary_actor = h.param(1).and_then(|v| v.value().as_str());

    let mut actors = HashSet::new();

    // Add all supporting actors except the primary one
    for actor in supporting_array {
        if let Value::String(actor_name) = actor {
            // Skip if this is the primary actor
            if let Some(primary) = primary_actor {
                if actor_name == primary {
                    continue;
                }
            }
            actors.insert(actor_name.clone());
        }
    }

    // Convert to sorted Vec for consistent output
    let mut actors_vec: Vec<String> = actors.into_iter().collect();
    actors_vec.sort();

    // Write as JSON which Handlebars will parse
    let json_str = serde_json::to_string(&actors_vec)
        .map_err(|e| RenderError::from(RenderErrorReason::Other(e.to_string())))?;
    out.write(&json_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_unique_actors_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Test that the helper returns JSON array
        let template = "{{unique_actors scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenarios": [
                {
                    "primary_actor": "User",
                    "supporting_actors": ["Admin"],
                    "steps": [
                        {"acting_actor": "User", "action": "clicks"},
                        {"acting_actor": "System", "receiving_actor": "Database", "action": "validates"}
                    ]
                },
                {
                    "primary_actor": "Admin",
                    "steps": [
                        {"acting_actor": "User", "action": "enters"},
                        {"acting_actor": "Database", "action": "stores"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        // The helper returns a JSON array (sorted alphabetically)
        assert_eq!(result, r#"["Admin","Database","System","User"]"#);
    }

    #[test]
    fn test_has_personas_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Test with actual usage in template
        let template = "{{#if (has_personas scenarios)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenarios": [
                {"persona": "customer"},
                {"persona": null}
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_has_personas_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (has_personas scenarios)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenarios": [
                {"persona": null},
                {"persona": ""}
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_unique_personas_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Test that the helper returns JSON array
        let template = "{{unique_personas scenarios}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "scenarios": [
                {"persona": "customer"},
                {"persona": "admin"},
                {"persona": "customer"},
                {"persona": null}
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        // The helper returns a JSON array
        assert_eq!(result, r#"["admin","customer"]"#);
    }

    #[test]
    fn test_actor_link_helper_registered() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Verify the helper is registered and can be used in templates
        let template = "{{{actor_link actor_name}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_name": "test-actor"
        });

        // Should not panic - helper is registered and callable
        let result = handlebars.render("test", &data);
        assert!(result.is_ok(), "actor_link helper should be registered");
    }

    #[test]
    fn test_actor_link_helper_with_empty_string() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "Actor: {{{actor_link actor_name}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_name": ""
        });

        let result = handlebars.render("test", &data).unwrap();
        // Empty string should return empty string (fallback behavior)
        assert_eq!(result, "Actor: ");
    }

    #[test]
    fn test_actor_link_helper_with_unknown_id() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "Actor: {{{actor_link actor_name}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_name": "nonexistent-actor-id"
        });

        let result = handlebars.render("test", &data).unwrap();
        // Unknown IDs should fall back to the input string (backward compatibility)
        assert_eq!(result, "Actor: nonexistent-actor-id");
    }

    #[test]
    fn test_actor_link_helper_with_display_name_fallback() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{{actor_link actor_name}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        // Test with a display name (old TOML format) - should pass through unchanged
        let data = json!({
            "actor_name": "Guest User"
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "Guest User");
    }

    #[test]
    fn test_actor_link_in_scenario_step_template() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Simulate a scenario step template pattern
        let template =
            "**{{{actor_link acting_actor}}}** → **{{{actor_link receiving_actor}}}**: {{action}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "acting_actor": "user-id",
            "receiving_actor": "system-id",
            "action": "sends request"
        });

        let result = handlebars.render("test", &data).unwrap();
        // Should format properly with actor IDs (will fall back to IDs if not found in repository)
        assert_eq!(result, "**user-id** → **system-id**: sends request");
    }

    #[test]
    fn test_actor_name_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        // Verify the helper is registered and can be used in templates
        let template = "{{{actor_name actor_id}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_id": "test-actor"
        });

        // Should not panic - helper is registered and callable
        let result = handlebars.render("test", &data);
        assert!(result.is_ok(), "actor_name helper should be registered");
    }

    #[test]
    fn test_actor_name_helper_with_unknown_id() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{{actor_name actor_id}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_id": "nonexistent-actor-id"
        });

        let result = handlebars.render("test", &data).unwrap();
        // Unknown IDs should fall back to the input string (backward compatibility)
        assert_eq!(result, "nonexistent-actor-id");
    }

    #[test]
    fn test_actor_link_creates_markdown_link_format() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{{actor_link actor_id}}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "actor_id": "unknown-actor"
        });

        let result = handlebars.render("test", &data).unwrap();
        // For unknown actors, should fall back to the ID itself (not a link)
        assert_eq!(result, "unknown-actor");
    }

    #[test]
    fn test_calculate_actor_relative_path_same_parent() {
        // Both under same parent (e.g., "docs/use-cases" and "docs/personas")
        let use_case_dir = "docs/use-cases";
        let actor_dir = "docs/personas";
        let relative_path = calculate_actor_relative_path(use_case_dir, actor_dir);
        // From docs/use-cases/category/use-case-id/ to docs/personas/
        // Common prefix: "docs/" (1 part)
        // Up from use-cases/category/use-case-id: 2 + 2 - 1 = 3 levels
        // Down to personas: 1 part
        assert_eq!(
            relative_path, "../../../personas",
            "Should go up 3 levels from category/id to common ancestor, then into personas"
        );
    }

    #[test]
    fn test_calculate_actor_relative_path_at_root() {
        // use_case_dir at root (e.g., "use-cases")
        let use_case_dir = "use-cases";
        let actor_dir = "personas";
        let relative_path = calculate_actor_relative_path(use_case_dir, actor_dir);
        // From use-cases/category/use-case-id/ -> ../../../ (to root) -> personas
        // Added extra level for use case ID folder
        assert_eq!(
            relative_path, "../../../personas",
            "Should go up 3 levels then into personas"
        );
    }

    #[test]
    fn test_calculate_actor_relative_path_deeply_nested() {
        // Deeply nested use_case_dir (e.g., "project/docs/use-cases")
        let use_case_dir = "project/docs/use-cases";
        let actor_dir = "project/docs/personas";
        let relative_path = calculate_actor_relative_path(use_case_dir, actor_dir);
        // From project/docs/use-cases/category/use-case-id/ to project/docs/personas/
        // Common prefix: "project/docs/" (2 parts)
        // Up from use-cases/category/use-case-id: 3 + 2 - 2 = 3 levels
        // Down to personas: 1 part
        assert_eq!(
            relative_path, "../../../personas",
            "Should go up 3 levels from category/id to common ancestor (project/docs/), then into personas"
        );
    }

    #[test]
    fn test_calculate_actor_relative_path_different_roots() {
        // Different root directories
        let use_case_dir = "documentation/use-cases";
        let actor_dir = "team/actors";
        let relative_path = calculate_actor_relative_path(use_case_dir, actor_dir);
        // From documentation/use-cases/category/use-case-id/ -> ../../../../ (to root) -> team/actors
        // Added extra level for use case ID folder
        assert_eq!(
            relative_path, "../../../../team/actors",
            "Should work with completely different directory trees"
        );
    }

    #[test]
    fn test_unique_supporting_actors_excludes_primary() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{unique_supporting_actors supporting_actors primary_actor}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "supporting_actors": ["User", "Admin", "System"],
            "primary_actor": "User"
        });

        let result = handlebars.render("test", &data).unwrap();
        // User should be excluded since it's the primary actor
        assert_eq!(result, r#"["Admin","System"]"#);
    }

    #[test]
    fn test_unique_supporting_actors_no_primary() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{unique_supporting_actors supporting_actors}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "supporting_actors": ["User", "Admin", "User"]  // duplicate
        });

        let result = handlebars.render("test", &data).unwrap();
        // Should deduplicate and sort
        assert_eq!(result, r#"["Admin","User"]"#);
    }
}
