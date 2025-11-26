// Core layer - Business logic and domain

// Private with explicit exports:
mod application;
mod domain;
mod infrastructure;
pub mod utils; // Public utilities

// Explicit public exports from private modules
// Public exports - Explicit API surface
pub use application::{
    methodology_field_collector::{CollectedField, FieldCollection, MethodologyFieldCollector},
    UseCaseCoordinator,
};

// Re-export domain types (from domain's public interface)
pub use domain::{
    Actor, ActorEntity, ActorRepository, ActorType, Category, Condition, ExtensionPointUpdater,
    MethodologyView, Persona, PersonaRepository, ReferenceType, RepeatBlock, ScenarioFlowValidator,
    ScenarioReference, ScenarioReferenceValidator, ScenarioType, Status, StepOrder, UseCase,
};

// Exported for integration tests (appear unused to lib but required by tests/)
pub use domain::Scenario;

// Re-export infrastructure types (from infrastructure's public interface)
pub use infrastructure::{
    file_operations, CustomFieldConfig, DocumentationLevel, FieldResolver, LanguageRegistry,
    Methodology, MethodologyDefinition, MethodologyRegistry, RepositoryFactory, TemplateEngine,
    UseCaseRepository,
};

// Exported for integration tests (appear unused to lib but required by tests/)
pub use infrastructure::{
    SqliteActorRepository, SqliteUseCaseRepository, TomlActorRepository, TomlCategoryRepository,
    TomlUseCaseRepository,
};

// Re-export utility functions
pub use utils::to_snake_case;
