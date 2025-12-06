//! # Generators Module
//!
//! This module contains specialized generators for different types of content:
//!
//! - **MarkdownGenerator**: Generates use case markdown documentation
//! - **TestGenerator**: Generates test documentation for use cases
//! - **OverviewGenerator**: Generates project overview documentation
//! - **CategoryOverviewGenerator**: Generates category-level overview documentation
//! - **OutputManager**: Manages output filenames for single/multi-view use cases
//!
//! These generators encapsulate the logic for creating various types of
//! documentation, separating concerns from the main application service.

pub mod category_overview_generator;
pub mod markdown_generator;
pub mod output_manager;
pub mod overview_generator;
pub mod test_generator;

pub use category_overview_generator::CategoryOverviewGenerator;
pub use markdown_generator::MarkdownGenerator;
pub use output_manager::OutputManager;
pub use overview_generator::OverviewGenerator;
pub use test_generator::TestGenerator;
