//! Mermaid diagram syntax Handlebars helpers.
//!
//! Provides helpers for generating Mermaid diagram syntax in templates,
//! supporting both sequence diagrams and flowcharts.
//!
//! ## Sequence Diagram Helpers
//! - `mermaid_loop_start` - Start a loop block
//! - `mermaid_loop_end` - End a loop block
//! - `mermaid_alt` - Start an alt (alternative) block
//! - `mermaid_else` - Add an else branch
//! - `mermaid_opt` - Start an opt (optional) block
//! - `mermaid_break` - Add a break statement
//! - `mermaid_end` - End any block (loop, alt, opt)
//!
//! ## Flowchart Helpers
//! - `mermaid_subgraph_start` - Start a subgraph
//! - `mermaid_subgraph_end` - End a subgraph
//! - `mermaid_node` - Create a node with shape
//! - `mermaid_link` - Create a link between nodes

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

/// Register all mermaid-related helpers
pub fn register(handlebars: &mut Handlebars) {
    // Sequence Diagram helpers
    handlebars.register_helper("mermaid_loop_start", Box::new(loop_start_helper));
    handlebars.register_helper("mermaid_loop_end", Box::new(loop_end_helper));
    handlebars.register_helper("mermaid_alt", Box::new(alt_helper));
    handlebars.register_helper("mermaid_else", Box::new(else_helper));
    handlebars.register_helper("mermaid_opt", Box::new(opt_helper));
    handlebars.register_helper("mermaid_break", Box::new(break_helper));
    handlebars.register_helper("mermaid_end", Box::new(end_helper));

    // Flowchart helpers
    handlebars.register_helper("mermaid_subgraph_start", Box::new(subgraph_start_helper));
    handlebars.register_helper("mermaid_subgraph_end", Box::new(subgraph_end_helper));
    handlebars.register_helper("mermaid_node", Box::new(node_helper));
    handlebars.register_helper("mermaid_link", Box::new(link_helper));
}

// === Sequence Diagram Helpers ===

/// Usage: {{mermaid_loop_start condition}}
/// or:    {{mermaid_loop_start from_step to_step condition}}
/// Output: loop condition
fn loop_start_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Support both {{mermaid_loop_start condition}} and {{mermaid_loop_start from to condition}}
    let condition = if h.params().len() >= 3 {
        // Three params: from_step, to_step, condition
        h.param(2).and_then(|v| v.value().as_str()).unwrap_or("")
    } else {
        // One param: just condition
        h.param(0).and_then(|v| v.value().as_str()).unwrap_or("")
    };

    out.write(&format!("loop {}", condition))?;
    Ok(())
}

/// Usage: {{mermaid_loop_end}}
/// Output: end
fn loop_end_helper(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write("end")?;
    Ok(())
}

/// Usage: {{mermaid_alt label}}
/// Output: alt label
fn alt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let label = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&format!("alt {}", label))?;
    Ok(())
}

/// Usage: {{mermaid_else label}}
/// Output: else label
fn else_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let label = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    if label.is_empty() {
        out.write("else")?;
    } else {
        out.write(&format!("else {}", label))?;
    }
    Ok(())
}

/// Usage: {{mermaid_opt label}}
/// Output: opt label
fn opt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let label = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&format!("opt {}", label))?;
    Ok(())
}

/// Usage: {{mermaid_break condition}}
/// Output: break condition
fn break_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let condition = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&format!("break {}", condition))?;
    Ok(())
}

/// Usage: {{mermaid_end}}
/// Output: end
fn end_helper(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write("end")?;
    Ok(())
}

// === Flowchart Helpers ===

/// Usage: {{mermaid_subgraph_start id label}}
/// Output: subgraph id[label]
fn subgraph_start_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let id = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let label = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");

    if label.is_empty() {
        out.write(&format!("subgraph {}", id))?;
    } else {
        out.write(&format!("subgraph {}[{}]", id, label))?;
    }
    Ok(())
}

/// Usage: {{mermaid_subgraph_end}}
/// Output: end
fn subgraph_end_helper(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write("end")?;
    Ok(())
}

/// Usage: {{mermaid_node id text shape}}
/// Output: Mermaid node syntax based on shape
///
/// Supported shapes:
/// - `rect` (default): `id[text]`
/// - `rounded`: `id(text)`
/// - `circle`: `id((text))`
/// - `diamond`: `id{text}`
/// - `stadium`: `id([text])`
/// - `parallelogram`: `id[/text/]`
/// - `parallelogram_alt`: `id[\text\]`
/// - `trapezoid`: `id[/text\]`
/// - `trapezoid_alt`: `id[\text/]`
/// - `hexagon`: `id{{text}}`
/// - `cylinder`: `id[(text)]`
/// - `asymmetric`: `id>text]`
fn node_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let id = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let text = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
    let shape = h
        .param(2)
        .and_then(|v| v.value().as_str())
        .unwrap_or("rect");

    let output = match shape {
        "rect" => format!("{}[{}]", id, text),
        "rounded" => format!("{}({})", id, text),
        "circle" => format!("{}(({}))", id, text),
        "diamond" => format!("{}{{{}}}", id, text),
        "stadium" => format!("{}([{}])", id, text),
        "parallelogram" => format!("{}[/{}\\]", id, text),
        "parallelogram_alt" => format!("{}[\\{}/]", id, text),
        "trapezoid" => format!("{}[/{}\\]", id, text),
        "trapezoid_alt" => format!("{}[\\{}/]", id, text),
        "hexagon" => format!("{}{{{{{}}}}}", id, text),
        "cylinder" => format!("{}[({})]", id, text),
        "asymmetric" => format!("{}>{}]", id, text),
        _ => format!("{}[{}]", id, text), // Default to rectangle
    };
    out.write(&output)?;
    Ok(())
}

/// Usage: {{mermaid_link from to text style}}
/// Output: Arrow syntax with optional text and style
///
/// Supported styles:
/// - `solid` (default): `-->`
/// - `dotted`: `-.->`
/// - `thick`: `==>`
/// - `solid_open`: `---`
/// - `dotted_open`: `-.-`
/// - `thick_open`: `===`
fn link_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let from = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let to = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
    let text = h.param(2).and_then(|v| v.value().as_str());
    let style = h
        .param(3)
        .and_then(|v| v.value().as_str())
        .unwrap_or("solid");

    let arrow = match style {
        "dotted" => "-.->",
        "thick" => "==>",
        "solid_open" => "---",
        "dotted_open" => "-.-",
        "thick_open" => "===",
        _ => "-->", // solid (default)
    };

    let output = if let Some(label) = text {
        if !label.is_empty() {
            format!("{} {}|{}| {}", from, arrow, label, to)
        } else {
            format!("{} {} {}", from, arrow, to)
        }
    } else {
        format!("{} {} {}", from, arrow, to)
    };
    out.write(&output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // === Sequence Diagram Tests ===

    #[test]
    fn test_loop_start_with_condition() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_loop_start condition}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "condition": "until authenticated" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "loop until authenticated");
    }

    #[test]
    fn test_loop_start_with_from_to_condition() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_loop_start from to condition}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "from": "2", "to": "5", "condition": "retry up to 3 times" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "loop retry up to 3 times");
    }

    #[test]
    fn test_loop_end() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_loop_end}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert_eq!(result, "end");
    }

    #[test]
    fn test_alt_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_alt label}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "label": "valid credentials" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "alt valid credentials");
    }

    #[test]
    fn test_else_helper_with_label() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_else label}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "label": "invalid credentials" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "else invalid credentials");
    }

    #[test]
    fn test_else_helper_without_label() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_else}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert_eq!(result, "else");
    }

    #[test]
    fn test_opt_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_opt label}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "label": "remember me selected" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "opt remember me selected");
    }

    #[test]
    fn test_break_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_break condition}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "condition": "max attempts reached" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "break max attempts reached");
    }

    #[test]
    fn test_end_helper() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_end}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert_eq!(result, "end");
    }

    // === Flowchart Tests ===

    #[test]
    fn test_subgraph_start_with_label() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_subgraph_start id label}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "auth", "label": "Authentication Flow" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "subgraph auth[Authentication Flow]");
    }

    #[test]
    fn test_subgraph_start_without_label() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_subgraph_start id}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "auth" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "subgraph auth");
    }

    #[test]
    fn test_subgraph_end() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_subgraph_end}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert_eq!(result, "end");
    }

    #[test]
    fn test_node_rect() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"rect\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "A", "text": "Start" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A[Start]");
    }

    #[test]
    fn test_node_default_rect() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "A", "text": "Start" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A[Start]");
    }

    #[test]
    fn test_node_rounded() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"rounded\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "B", "text": "Process" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "B(Process)");
    }

    #[test]
    fn test_node_circle() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"circle\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "C", "text": "End" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "C((End))");
    }

    #[test]
    fn test_node_diamond() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"diamond\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "D", "text": "Decision" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "D{Decision}");
    }

    #[test]
    fn test_node_stadium() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"stadium\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "E", "text": "Terminal" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "E([Terminal])");
    }

    #[test]
    fn test_node_hexagon() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"hexagon\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "F", "text": "Prepare" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "F{{Prepare}}");
    }

    #[test]
    fn test_node_cylinder() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_node id text \"cylinder\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "id": "G", "text": "Database" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "G[(Database)]");
    }

    #[test]
    fn test_link_solid() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_link from to}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "from": "A", "to": "B" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A --> B");
    }

    #[test]
    fn test_link_with_text() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_link from to text}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "from": "A", "to": "B", "text": "click" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A -->|click| B");
    }

    #[test]
    fn test_link_dotted() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_link from to text \"dotted\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "from": "A", "to": "B", "text": "optional" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A -.->|optional| B");
    }

    #[test]
    fn test_link_thick() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = "{{mermaid_link from to \"\" \"thick\"}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let data = json!({ "from": "A", "to": "B" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "A ==> B");
    }

    #[test]
    fn test_complete_sequence_diagram() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = r#"sequenceDiagram
{{mermaid_alt "valid"}}
User->>System: Login
{{mermaid_else "invalid"}}
System->>User: Error
{{mermaid_end}}
{{mermaid_loop_start "retry"}}
User->>System: Retry
{{mermaid_loop_end}}"#;
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert!(result.contains("alt valid"));
        assert!(result.contains("else invalid"));
        assert!(result.contains("loop retry"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_complete_flowchart() {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);

        let template = r#"flowchart TD
{{mermaid_node "A" "Start" "circle"}}
{{mermaid_node "B" "Process" "rect"}}
{{mermaid_node "C" "Decision" "diamond"}}
{{mermaid_link "A" "B"}}
{{mermaid_link "B" "C" "check"}}"#;
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let result = handlebars.render("test", &json!({})).unwrap();
        assert!(result.contains("A((Start))"));
        assert!(result.contains("B[Process]"));
        assert!(result.contains("C{Decision}"));
        assert!(result.contains("A --> B"));
        assert!(result.contains("B -->|check| C"));
    }
}
