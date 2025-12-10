//! Handlebars helpers for template rendering.
//!
//! This module provides custom Handlebars helpers organized by domain:
//! - **actor**: Actor and persona resolution helpers
//! - **scenario**: Scenario linking and merging helpers
//! - **formatting**: Text transformation helpers
//! - **comparison**: Value comparison helpers
//! - **mermaid**: Mermaid diagram syntax helpers

mod actor;
mod comparison;
mod formatting;
mod mermaid;
mod scenario;
mod status;

use handlebars::Handlebars;

/// Register all custom Handlebars helpers for template rendering
pub fn register_helpers(handlebars: &mut Handlebars) {
    actor::register(handlebars);
    status::register(handlebars);
    scenario::register(handlebars);
    formatting::register(handlebars);
    comparison::register(handlebars);
    mermaid::register(handlebars);
}
