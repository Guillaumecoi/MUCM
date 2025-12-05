//! # Controller Layer
//!
//! This module provides the controller layer for the Markdown Use Case Manager.
//! Controllers handle business logic coordination between the presentation layer
//! and the application services, providing a clean interface for CLI commands
//! and user interactions.
//!
//! ## Architecture
//!
//! The controller layer follows a clean architecture pattern:
//! - **Controllers**: Coordinate operations and handle user requests
//! - **DTOs**: Data transfer objects for clean data exchange
//! - **Application Services**: Core business logic (in the core module)
//! - **Infrastructure**: External dependencies and persistence
//!
//! ## Controllers
//!
//! - `ProjectController`: Handles project initialization and configuration
//! - `UseCaseController`: Manages use case creation, listing, and regeneration
//!
//! ## Data Transfer Objects
//!
//! - `DisplayResult`: Standardized operation results for user feedback
//! - `SelectionOptions`: Available options for user selection prompts
//! - `MethodologyInfo`: Methodology metadata for display and selection

mod actor_controller;
mod category_controller;
pub mod dto;
mod project_controller;
mod result_ext;
mod scenario_controller;
mod use_case_controller;

#[cfg(test)]
mod tests;

// Re-export commonly used controllers
pub use actor_controller::{ActorController, PersonaController};
pub use category_controller::CategoryController;
pub use project_controller::{InitProjectParams, ProjectController};
pub use result_ext::{DisplayMessage, ResultExt};
pub use scenario_controller::{
    CreateExtensionParams, CreateScenarioParams, InsertStepParams, ScenarioController,
};
pub use use_case_controller::{CreateUseCaseParams, UseCaseController};

// Re-export DTOs for use in CLI layer
pub use dto::{DisplayResult, MethodologyInfo};
