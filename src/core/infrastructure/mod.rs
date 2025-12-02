// Infrastructure layer - Implementation details

mod languages;
mod methodologies;
mod persistence;
mod template_engine;

// Re-exports
pub use languages::LanguageRegistry;
pub use methodologies::{
    CustomFieldConfig, DocumentationLevel, FieldResolver, Methodology, MethodologyDefinition,
    MethodologyRegistry,
};
pub use persistence::{
    file_operations, RepositoryFactory, SqliteActorRepository, SqliteUseCaseRepository,
    TomlActorRepository, TomlCategoryRepository, TomlMigrator, TomlUseCaseRepository,
    UseCaseRepository,
};
pub use template_engine::{ActorMarkdownGenerator, TemplateEngine};
