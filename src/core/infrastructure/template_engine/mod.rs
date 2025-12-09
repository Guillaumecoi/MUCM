//! Template rendering infrastructure.
//!
//! This module provides the template engine and helpers for generating
//! documentation from Handlebars templates.

mod actor_markdown_generator;
mod engine;
mod helpers;

// Public exports
pub use actor_markdown_generator::ActorMarkdownGenerator;
pub use engine::TemplateEngine;
