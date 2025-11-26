//! Persistence layer for use case storage.
//!
//! This module provides different storage backends (TOML, SQLite)
//! with a unified interface through the UseCaseRepository trait.

pub mod file_operations;
pub mod repository_factory;
pub mod sqlite;
pub mod toml;
pub mod traits;

// Re-export for convenience
pub use repository_factory::RepositoryFactory;
pub use sqlite::{SqliteActorRepository, SqliteUseCaseRepository};
pub use toml::{TomlActorRepository, TomlCategoryRepository, TomlUseCaseRepository};
pub use traits::UseCaseRepository;
