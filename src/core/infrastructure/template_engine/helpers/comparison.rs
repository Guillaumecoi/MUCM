//! Comparison Handlebars helpers.
//!
//! Provides helpers for comparing values in templates:
//! - gt (greater than)
//! - lt (less than)
//! - eq (equal)
//! - ne (not equal)
//! - gte (greater than or equal)
//! - lte (less than or equal)

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

/// Register all comparison-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("gt", Box::new(gt_helper));
    handlebars.register_helper("lt", Box::new(lt_helper));
    handlebars.register_helper("eq", Box::new(eq_helper));
    handlebars.register_helper("ne", Box::new(ne_helper));
    handlebars.register_helper("gte", Box::new(gte_helper));
    handlebars.register_helper("lte", Box::new(lte_helper));
}

/// Helper to check if first parameter is greater than second
/// Usage: {{#if (gt a b)}} ... {{/if}}
/// Returns: true if a > b, false otherwise
fn gt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if a > b {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if first parameter is less than second
/// Usage: {{#if (lt a b)}} ... {{/if}}
/// Returns: true if a < b, false otherwise
fn lt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if a < b {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if two parameters are equal
/// Usage: {{#if (eq a b)}} ... {{/if}}
/// Returns: true if a == b, false otherwise
/// Works with numbers (compared as f64) and strings
fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param_a = h.param(0).map(|v| v.value());
    let param_b = h.param(1).map(|v| v.value());

    let is_equal = match (param_a, param_b) {
        (Some(a), Some(b)) => {
            // Try numeric comparison first
            if let (Some(num_a), Some(num_b)) = (a.as_f64(), b.as_f64()) {
                (num_a - num_b).abs() < f64::EPSILON
            } else {
                // Fall back to JSON value comparison
                a == b
            }
        }
        (None, None) => true,
        _ => false,
    };

    if is_equal {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if two parameters are not equal
/// Usage: {{#if (ne a b)}} ... {{/if}}
/// Returns: true if a != b, false otherwise
fn ne_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param_a = h.param(0).map(|v| v.value());
    let param_b = h.param(1).map(|v| v.value());

    let is_not_equal = match (param_a, param_b) {
        (Some(a), Some(b)) => {
            // Try numeric comparison first
            if let (Some(num_a), Some(num_b)) = (a.as_f64(), b.as_f64()) {
                (num_a - num_b).abs() >= f64::EPSILON
            } else {
                // Fall back to JSON value comparison
                a != b
            }
        }
        (None, None) => false,
        _ => true,
    };

    if is_not_equal {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if first parameter is greater than or equal to second
/// Usage: {{#if (gte a b)}} ... {{/if}}
/// Returns: true if a >= b, false otherwise
fn gte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if a >= b {
        out.write("true")?;
    }

    Ok(())
}

/// Helper to check if first parameter is less than or equal to second
/// Usage: {{#if (lte a b)}} ... {{/if}}
/// Returns: true if a <= b, false otherwise
fn lte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if a <= b {
        out.write("true")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_gt_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 10, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_gt_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 10 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_gt_helper_equal() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_lt_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (lt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 10 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_lt_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (lt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 10, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_eq_helper_numbers_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (eq a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_eq_helper_numbers_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (eq a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 10 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_eq_helper_strings_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (eq a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": "hello", "b": "hello" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_eq_helper_strings_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (eq a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": "hello", "b": "world" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_ne_helper_true() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (ne a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 10 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_ne_helper_false() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (ne a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_gte_helper_greater() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 10, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_gte_helper_equal() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_gte_helper_less() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 3, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_lte_helper_less() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (lte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 3, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_lte_helper_equal() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (lte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_lte_helper_greater() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (lte a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 10, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_comparison_with_floats() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gt a b)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "a": 3.14, "b": 2.71 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_comparison_with_missing_params() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{#if (gt a)}}yes{{else}}no{{/if}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        // Missing second param defaults to 0
        let data = json!({ "a": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "yes"); // 5 > 0
    }
}
