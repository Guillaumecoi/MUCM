use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
    RenderErrorReason,
};
use serde_json::Value;
use std::collections::HashSet;

/// Register all custom Handlebars helpers for actor and persona support
pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("unique_actors", Box::new(unique_actors_helper));
    handlebars.register_helper("has_personas", Box::new(has_personas_helper));
    handlebars.register_helper("unique_personas", Box::new(unique_personas_helper));
    handlebars.register_helper("actor_emoji", Box::new(actor_emoji_helper));
    handlebars.register_helper("actor_link", Box::new(actor_link_helper));
    handlebars.register_helper("mermaid_safe", Box::new(mermaid_safe_helper));
}

/// Helper to display an actor name
/// Usage: {{actor_link actor_name}}
/// Returns: actor_name (emoji should be in the data passed to template)
/// Note: Use actor_with_emoji helper if you need to add emoji programmatically
fn actor_link_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let actor_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(actor_name)?;
    Ok(())
}

/// Helper to make text safe for mermaid diagrams
/// Usage: {{mermaid_safe text}}
/// Returns: text with quotes replaced by single quotes to avoid HTML entity issues
fn mermaid_safe_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    // Replace double quotes with single quotes to avoid HTML entity conversion issues in mermaid
    let safe_text = text.replace('"', "'");
    out.write(&safe_text)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_unique_actors_helper() {
        let mut handlebars = Handlebars::new();
        register_helpers(&mut handlebars);

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
        register_helpers(&mut handlebars);

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
        register_helpers(&mut handlebars);

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
        register_helpers(&mut handlebars);

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
}
