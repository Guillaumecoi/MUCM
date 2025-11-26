//! TOML-based persistence implementation.
//!
//! This module provides TOML file-based storage for use cases and actors (personas and system actors).
//! Each entity is stored as a separate TOML file, making it
//! git-friendly and human-readable.

mod actor_repository;
mod category_repository;
mod repository;

pub use actor_repository::TomlActorRepository;
pub use category_repository::TomlCategoryRepository;
pub use repository::TomlUseCaseRepository;
