//! # Markdown Use Case Manager (MUCM)
//!
//! A command-line tool for managing use case documentation using markdown templates.
//! MUCM provides a structured approach to creating, organizing, and maintaining
//! use case documentation with support for multiple methodologies and programming languages.
//!
//! ## Architecture
//!
//! The application follows Clean Architecture principles with clear separation of concerns:
//!
//! - **CLI Layer** (`cli/`): Command-line interface and argument parsing
//! - **Controller Layer** (`controller/`): Business logic coordination and user interaction
//! - **Core Layer** (`core/`): Domain entities, business rules, and application services
//! - **Infrastructure Layer** (`core/infrastructure/`): External dependencies and persistence
//! - **Presentation Layer** (`presentation/`): Output formatting and user interface
//! - **Configuration Layer** (`config/`): Project configuration and template management
//!
//! ## Key Features
//!
//! - **Modular Methodology Support**: Extensible methodology system for different documentation approaches
//! - **Template-Driven Generation**: Handlebars templates for consistent documentation
//! - **Language Integration**: Support for multiple programming languages through modular templates
//! - **TOML-Based Storage**: Human-readable use case data storage
//! - **Markdown Output**: Generated documentation in standard Markdown format
//! - **Project Management**: Initialize, configure, and manage use case projects
//!
//! ## Usage Workflow
//!
//! 1. **Initialize Project**: `mucm init` to set up project configuration
//! 2. **Create Use Cases**: `mucm create --category <cat> "<title>"` to add use cases
//! 3. **Manage Documentation**: Use various commands to list, regenerate, and organize
//! 4. **Customize Templates**: Modify templates in `.config/.mucm/handlebars/`
//!
//! ## Configuration
//!
//! Project configuration is stored in `.config/.mucm/mucm.toml` and includes:
//! - Project metadata (name, description)
//! - Directory paths for use cases, tests, and templates
//! - Methodology and language preferences
//! - Generation settings and metadata options

use anyhow::Result;
use mucm::cli;

/// Application entry point.
///
/// Initializes and runs the Markdown Use Case Manager CLI application.
/// This function serves as the main entry point for the binary and delegates
/// all functionality to the CLI module for command processing and execution.
fn main() -> Result<()> {
    cli::run()
}
