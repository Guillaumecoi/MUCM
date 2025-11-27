//! # Interactive CLI Module
//!
//! Interactive command-line interface for the Markdown Use Case Manager.
//! Provides guided, menu-driven interaction for users who prefer wizard-style workflows.
//!
//! ## Architecture
//!
//! This module contains the interactive CLI implementation with a layered architecture:
//! - `runner.rs`: Business logic coordinator for interactive workflows
//! - `selectors.rs`: Pure data selection functions for UI presentation
//! - `ui.rs`: Presentation layer for interactive prompts and displays
//! - `menus/`: Menu navigation and selection systems
//!   - `menu.rs`: Main menu navigation and selection logic
//!   - `settings.rs`: Settings configuration submenu
//! - `workflows/`: Specialized workflow handlers
//!   - `initialization.rs`: Project initialization wizard
//!   - `config.rs`: General configuration workflow
//!   - `methodology.rs`: Methodology management workflow
//!   - `use_case.rs`: Use case operations workflow
//!
//! ## Interactive Workflows
//!
//! - **Project Initialization**: Guided setup wizard for new projects
//! - **Use Case Creation**: Interactive prompts for creating use cases
//! - **Settings Configuration**: Menu-driven settings management
//! - **Status Display**: Interactive project status viewing
//! - **Main Menu**: Central navigation hub for all interactive features

// Private modules - used only within CLI interactive
mod field_helpers;
mod menus;
pub mod prompts;
mod runner;
mod selectors;
mod ui;
mod workflows;

#[cfg(test)]
mod tests;

// Public exports
pub use menus::menu::run_interactive_session;
#[cfg(test)]
pub use runner::InteractiveRunner;
