//! Text formatting Handlebars helpers.
//!
//! Provides helpers for:
//! - Case conversions (snake_case, PascalCase)
//! - Mermaid-safe text escaping
//! - Date formatting

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};

/// Register all formatting-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("snake_case_id", Box::new(snake_case_id_helper));
    handlebars.register_helper("pascal_case_id", Box::new(pascal_case_id_helper));
    handlebars.register_helper("title_pascal_case", Box::new(title_pascal_case_helper));
    handlebars.register_helper("mermaid_safe", Box::new(mermaid_safe_helper));
    handlebars.register_helper("date_format", Box::new(date_format_helper));
}

/// Helper to convert an `id` field to snake_case
/// Usage: {{snake_case_id id}}
/// Takes the id as a parameter and converts it to lowercase snake_case
/// Returns: lowercase snake_case version of the id (e.g., "UC-AUTH-001-S01" -> "uc_auth_001_s01")
fn snake_case_id_helper(
    h: &Helper,
    _: &Handlebars,
    _ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Get the first parameter (the id to convert)
    let id_value = h
        .param(0)
        .ok_or_else(|| {
            RenderErrorReason::Other("snake_case_id helper requires an id parameter".to_string())
        })?
        .value()
        .as_str()
        .ok_or_else(|| {
            RenderErrorReason::Other("snake_case_id parameter must be a string".to_string())
        })?;

    // Convert to snake_case
    let snake_case = crate::core::to_snake_case(id_value);
    out.write(&snake_case)?;

    Ok(())
}

/// Helper to convert an id string to PascalCase
/// Usage: {{pascal_case_id id}}
/// Example: "user-login" -> "UserLogin", "my_test_id" -> "MyTestId"
fn pascal_case_id_helper(
    h: &Helper,
    _: &Handlebars,
    _ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let id_value = h
        .param(0)
        .ok_or_else(|| {
            RenderErrorReason::Other("pascal_case_id helper requires an id parameter".to_string())
        })?
        .value()
        .as_str()
        .ok_or_else(|| {
            RenderErrorReason::Other("pascal_case_id parameter must be a string".to_string())
        })?;

    let pascal_case = crate::core::to_pascal_case(id_value);
    out.write(&pascal_case)?;

    Ok(())
}

/// Helper to convert a title string to PascalCase
/// Usage: {{title_pascal_case}}
/// Takes title from the current context
fn title_pascal_case_helper(
    _h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let title = ctx
        .data()
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            RenderErrorReason::Other(
                "title_pascal_case helper requires title in context".to_string(),
            )
        })?;

    let pascal_case = crate::core::to_pascal_case(title);
    out.write(&pascal_case)?;

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

/// Helper to format dates according to the configured date format
/// Usage: {{date_format date_string}}
/// Returns: The date formatted according to the config's date_format setting
fn date_format_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let date_str = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    // Load config for date formatting
    let date_format = crate::config::Config::load()
        .map(|c| c.metadata.date_format)
        .unwrap_or_else(|_| "%d/%m/%Y".to_string());

    // Parse the date string and format it
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        let formatted = dt.format(&date_format).to_string();
        out.write(&formatted)?;
    } else {
        // If parsing fails, return the original string
        out.write(date_str)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mermaid_safe_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = r#"{{{mermaid_safe text}}}"#;
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "text": r#"User says "hello world" to System"#
        });

        let result = handlebars.render("test", &data).unwrap();
        // Double quotes should be replaced with single quotes
        assert_eq!(result, "User says 'hello world' to System");
    }

    #[test]
    fn test_mermaid_safe_helper_with_mixed_quotes() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = r#"{{{mermaid_safe text}}}"#;
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "text": r#"User's action: "submit" form"#
        });

        let result = handlebars.render("test", &data).unwrap();
        // Double quotes replaced, single quotes preserved
        assert_eq!(result, "User's action: 'submit' form");
    }

    #[test]
    fn test_snake_case_id_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{snake_case_id id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "id": "UC-AUTH-001-S01"
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "uc_auth_001_s01");
    }

    #[test]
    fn test_pascal_case_id_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{pascal_case_id id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "id": "user-login"
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "UserLogin");
    }

    #[test]
    fn test_pascal_case_id_with_underscores() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{pascal_case_id id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "id": "my_test_id"
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "MyTestId");
    }

    #[test]
    fn test_title_pascal_case_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{title_pascal_case}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "title": "User Login Flow"
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "UserLoginFlow");
    }

    #[test]
    fn test_date_format_helper_with_rfc3339() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{date_format date}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "date": "2025-01-15T10:30:00+00:00"
        });

        // The result depends on config, but it should not panic
        let result = handlebars.render("test", &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_date_format_helper_with_invalid_date() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{date_format date}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "date": "not-a-date"
        });

        let result = handlebars.render("test", &data).unwrap();
        // Should fall back to original string
        assert_eq!(result, "not-a-date");
    }

    #[test]
    fn test_mermaid_safe_empty_string() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = r#"{{{mermaid_safe text}}}"#;
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({
            "text": ""
        });

        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "");
    }
}
