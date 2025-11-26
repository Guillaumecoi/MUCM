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
}

/// Helper to return an emoji for an actor name
/// Usage: {{actor_emoji actor_name}}
/// Returns an emoji prefix for common actor types, or a generic emoji
fn actor_emoji_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let actor_name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    let actor_lower = actor_name.to_lowercase();
    let emoji = if actor_lower.contains("user")
        || actor_lower.contains("customer")
        || actor_lower.contains("admin")
    {
        "👤"
    } else if actor_lower.contains("system")
        || actor_lower.contains("server")
        || actor_lower.contains("application")
    {
        "⚙️"
    } else if actor_lower.contains("database")
        || actor_lower.contains("db")
        || actor_lower.contains("storage")
    {
        "💾"
    } else if actor_lower.contains("api")
        || actor_lower.contains("service")
        || actor_lower.contains("gateway")
    {
        "🔌"
    } else if actor_lower.contains("email")
        || actor_lower.contains("mail")
        || actor_lower.contains("notification")
    {
        "📧"
    } else if actor_lower.contains("payment")
        || actor_lower.contains("transaction")
        || actor_lower.contains("billing")
    {
        "💳"
    } else if actor_lower.contains("auth")
        || actor_lower.contains("security")
        || actor_lower.contains("login")
    {
        "🔐"
    } else if actor_lower.contains("cache")
        || actor_lower.contains("redis")
        || actor_lower.contains("memcache")
    {
        "🗃️"
    } else if actor_lower.contains("queue")
        || actor_lower.contains("message")
        || actor_lower.contains("broker")
    {
        "📮"
    } else if actor_lower.contains("file")
        || actor_lower.contains("document")
        || actor_lower.contains("upload")
    {
        "📄"
    } else {
        "🔷" // Generic component
    };

    out.write(emoji)?;
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

    // Extract actors from each scenario's steps
    for scenario in scenarios_array {
        if let Some(steps) = scenario.get("steps").and_then(|v| v.as_array()) {
            for step in steps {
                if let Some(actor) = step.get("actor") {
                    // Actor can be a string (enum variant) or an object with a variant field
                    let actor_str = match actor {
                        Value::String(s) => s.clone(),
                        Value::Object(map) => {
                            // Handle Actor enum serialization - look for variant name
                            if let Some(Value::String(s)) = map.values().next() {
                                s.clone()
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    };
                    actors.insert(actor_str);
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
                    "steps": [
                        {"actor": "User", "action": "clicks"},
                        {"actor": "System", "action": "validates"}
                    ]
                },
                {
                    "steps": [
                        {"actor": "User", "action": "enters"},
                        {"actor": "Database", "action": "stores"}
                    ]
                }
            ]
        });

        let result = handlebars.render("test", &data).unwrap();
        // The helper returns a JSON array
        assert_eq!(result, r#"["Database","System","User"]"#);
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
