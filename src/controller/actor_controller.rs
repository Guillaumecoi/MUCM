//! # Actor Controller
//!
//! This module provides the controller for actor management operations.
//! It handles the coordination between CLI commands and actor application
//! services, providing a unified interface for managing both personas and
//! system actors.
//!
//! ## Responsibilities
//!
//! - Persona creation with Sommerville-aligned custom fields
//! - System actor creation with emojis (Database, API, etc.)
//! - Standard actor initialization
//! - Actor updating (name, emoji, and custom fields)
//! - Actor listing and retrieval
//! - Actor deletion
//! - Data retrieval for interactive selection prompts

use crate::config::Config;
use crate::controller::dto::DisplayResult;
use crate::core::{
    ActorEntity, ActorMarkdownGenerator, ActorRepository, ActorType, Persona, PersonaRepository,
    SqliteActorRepository, TomlActorRepository,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::str::FromStr;

/// Controller for actor operations and management.
///
/// Manages both persona and system actor operations including creation,
/// updating, listing, and deletion. Acts as the coordination layer between
/// CLI commands and actor persistence.
pub struct ActorController {
    /// Repository for actor persistence
    actor_repository: Box<dyn ActorRepository>,
    /// Repository for persona backward compatibility
    persona_repository: Box<dyn PersonaRepository>,
    /// Project configuration
    config: Config,
}

/// Legacy controller name for backward compatibility
pub type PersonaController = ActorController;

impl ActorController {
    /// Create a new actor controller instance.
    ///
    /// Initializes the controller with the appropriate repository backend
    /// based on the project configuration (TOML or SQLite).
    ///
    /// # Returns
    /// A new ActorController instance ready for use
    ///
    /// # Errors
    /// Returns error if the configuration cannot be loaded or repository creation fails
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        let (actor_repository, persona_repository): (
            Box<dyn ActorRepository>,
            Box<dyn PersonaRepository>,
        ) = match config.storage.backend {
            crate::config::StorageBackend::Sqlite => {
                use rusqlite::Connection;
                use std::sync::{Arc, Mutex};

                let db_path = format!("{}/mucm.db", config.directories.data_dir);
                let conn = Arc::new(Mutex::new(Connection::open(&db_path)?));
                SqliteActorRepository::initialize(&conn.lock().unwrap())?;

                // Create separate instances sharing the same connection
                let actor_repo = SqliteActorRepository::new(Arc::clone(&conn));
                let persona_repo = SqliteActorRepository::new(conn);

                (Box::new(actor_repo), Box::new(persona_repo))
            }
            crate::config::StorageBackend::Toml => {
                // For TOML, create two separate instances with the same config
                let actor_repo = TomlActorRepository::new(config.clone());
                let persona_repo = TomlActorRepository::new(config.clone());
                (Box::new(actor_repo), Box::new(persona_repo))
            }
        };

        Ok(Self {
            actor_repository,
            persona_repository,
            config,
        })
    }

    /// Create a new persona with basic information.
    ///
    /// Creates a persona with an ID and name, initializing custom fields
    /// based on the project's persona configuration.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the persona (e.g., "customer", "admin")
    /// * `name` - Display name for the persona
    ///
    /// # Returns
    /// DisplayResult with success message and persona information
    ///
    /// # Errors
    /// Returns error if persona creation fails or ID already exists
    pub fn create_persona(
        &self,
        id: String,
        name: String,
        function: String,
    ) -> Result<DisplayResult> {
        // Validate ID format
        if let Err(e) = ActorEntity::validate_id(&id) {
            return Ok(DisplayResult::error(e));
        }

        // Check if actor already exists
        if self.actor_repository.actor_exists(&id)? {
            return Ok(DisplayResult::error(format!(
                "Actor with ID '{}' already exists",
                id
            )));
        }

        // Create persona with config fields
        let persona = Persona::from_config_fields(
            id.clone(),
            name,
            function,
            &self.config.actor.persona_fields,
        );

        // Save the persona
        self.persona_repository.save(&persona)?;

        // Generate markdown documentation
        if let Err(e) = self.generate_actor_markdown(&id) {
            eprintln!("Warning: Failed to generate actor markdown: {}", e);
        }

        Ok(DisplayResult::success(format!(
            "✅ Created persona: {} {} ({})",
            persona
                .extra
                .get("emoji")
                .and_then(|v| v.as_str())
                .unwrap_or("🙂"),
            persona.name,
            persona.id
        )))
    }

    /// Create a new system actor.
    ///
    /// Creates a system actor (Database, API, etc.) with an emoji for visual identification.
    ///
    /// # Arguments
    /// * `id` - Unique identifier (e.g., "payment-api", "auth-database")
    /// * `name` - Display name
    /// * `actor_type` - Type of system actor
    /// * `emoji` - Optional emoji (uses defaults if not specified)
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if actor creation fails or ID already exists
    pub fn create_system_actor(
        &self,
        id: String,
        name: String,
        actor_type: String,
        emoji: Option<String>,
    ) -> Result<DisplayResult> {
        // Validate ID format
        if let Err(e) = ActorEntity::validate_id(&id) {
            return Ok(DisplayResult::error(e));
        }

        // Check if actor already exists
        if self.actor_repository.actor_exists(&id)? {
            return Ok(DisplayResult::error(format!(
                "Actor with ID '{}' already exists",
                id
            )));
        }

        // Parse actor type
        let parsed_type = ActorType::from_str(&actor_type).map_err(|e| anyhow::anyhow!(e))?;

        // Use default emoji if not specified
        let final_emoji = emoji.unwrap_or_else(|| match parsed_type {
            ActorType::Database => "💾".to_string(),
            ActorType::System => "🖥️".to_string(),
            ActorType::ExternalService => "🌐".to_string(),
            _ => "⚙️".to_string(),
        });

        // Create actor entity
        let actor = ActorEntity::new(id.clone(), name.clone(), parsed_type, final_emoji.clone());

        // Save the actor
        self.actor_repository.save_actor(&actor)?;

        // Generate markdown documentation
        if let Err(e) = self.generate_actor_markdown(&id) {
            eprintln!("Warning: Failed to generate actor markdown: {}", e);
        }

        Ok(DisplayResult::success(format!(
            "✅ Created system actor: {} {} ({})",
            final_emoji, name, id
        )))
    }

    /// Initialize standard system actors.
    ///
    /// Creates a set of commonly used system actors with default emojis:
    /// - Database 💾
    /// - Web Server 🖥️
    /// - API 🌐
    /// - Payment Gateway 💳
    /// - Email Service 📧
    /// - Cache ⚡
    ///
    /// # Returns
    /// DisplayResult with count of created actors
    ///
    /// # Errors
    /// Returns error if creation fails
    pub fn init_standard_actors(&self) -> Result<DisplayResult> {
        let standard_actors = ActorEntity::standard_actors();
        let mut created_count = 0;
        let mut skipped = Vec::new();

        for actor in standard_actors {
            if self.actor_repository.actor_exists(&actor.id)? {
                skipped.push(actor.display());
            } else {
                self.actor_repository.save_actor(&actor)?;
                created_count += 1;
            }
        }

        let mut message = format!("✅ Initialized {} standard system actors", created_count);
        if !skipped.is_empty() {
            message.push_str(&format!(
                "\n⏭️  Skipped {} existing: {}",
                skipped.len(),
                skipped.join(", ")
            ));
        }

        Ok(DisplayResult::success(message))
    }

    /// Update an actor's emoji.
    ///
    /// Changes the emoji used for visual identification of an actor.
    ///
    /// # Arguments
    /// * `id` - The actor ID to update
    /// * `emoji` - New emoji
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if actor not found or update fails
    pub fn update_emoji(&self, id: String, emoji: String) -> Result<DisplayResult> {
        // Load existing actor
        let mut actor = self
            .actor_repository
            .load_actor_by_id(&id)?
            .context(format!("Actor '{}' not found", id))?;

        actor.emoji = emoji.clone();

        // Save updated actor
        self.actor_repository.save_actor(&actor)?;

        Ok(DisplayResult::success(format!(
            "✅ Updated emoji for {}: {} {}",
            id, emoji, actor.name
        )))
    }

    /// Update an actor's name (works for all actor types).
    ///
    /// Updates the actor's display name. This works for any actor type
    /// (personas, systems, databases, etc.).
    ///
    /// # Arguments
    /// * `id` - The actor ID to update
    /// * `name` - New name for the actor
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if actor not found or update fails
    pub fn update_actor_name(&self, id: String, name: String) -> Result<DisplayResult> {
        // Load existing actor
        let mut actor = self
            .actor_repository
            .load_actor_by_id(&id)?
            .context(format!("Actor '{}' not found", id))?;

        actor.name = name;

        // Save updated actor
        self.actor_repository.save_actor(&actor)?;

        Ok(DisplayResult::success(format!(
            "✅ Updated name for actor: {}",
            id
        )))
    }

    /// Update a persona's basic information.
    ///
    /// Updates the persona's name. Custom fields are updated separately
    /// via update_persona_fields().
    ///
    /// # Arguments
    /// * `id` - The persona ID to update
    /// * `name` - Optional new name
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if persona not found or update fails
    pub fn update_persona(&self, id: String, name: Option<String>) -> Result<DisplayResult> {
        // Load existing persona
        let mut persona = self
            .persona_repository
            .load_by_id(&id)?
            .context(format!("Persona '{}' not found", id))?;

        // Update fields
        if let Some(new_name) = name {
            persona.name = new_name;
        }

        // Save updated persona
        self.persona_repository.save(&persona)?;

        Ok(DisplayResult::success(format!(
            "Updated persona: {}",
            persona.id
        )))
    }

    /// Update persona custom fields.
    ///
    /// Updates or adds custom fields to a persona. Fields are merged with
    /// existing values - only specified fields are updated.
    ///
    /// # Arguments
    /// * `id` - The persona ID to update
    /// * `fields` - Map of field names to new values (as strings)
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if persona not found or update fails
    pub fn update_persona_fields(
        &self,
        id: String,
        fields: HashMap<String, String>,
    ) -> Result<DisplayResult> {
        // Load existing persona
        let mut persona = self
            .persona_repository
            .load_by_id(&id)?
            .context(format!("Persona '{}' not found", id))?;

        // Update custom fields
        for (field_name, field_value) in fields {
            // Parse field value as JSON to support different types
            let json_value: serde_json::Value = if field_value.contains('\n') {
                // Array format (newline-separated)
                let items: Vec<String> = field_value
                    .split('\n')
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                serde_json::json!(items)
            } else if let Ok(num) = field_value.parse::<f64>() {
                // Number
                serde_json::json!(num)
            } else if field_value == "true" || field_value == "false" {
                // Boolean
                serde_json::json!(field_value.parse::<bool>().unwrap())
            } else {
                // String
                serde_json::json!(field_value)
            };

            persona.extra.insert(field_name, json_value);
        }

        // Save updated persona
        self.persona_repository.save(&persona)?;

        Ok(DisplayResult::success(format!(
            "Updated custom fields for persona: {}",
            persona.id
        )))
    }

    /// Delete an actor (persona or system actor).
    ///
    /// Removes an actor from the project, including its data file and
    /// any generated markdown documentation.
    ///
    /// # Arguments
    /// * `id` - The actor ID to delete
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if actor not found or deletion fails
    pub fn delete_actor(&self, id: String) -> Result<DisplayResult> {
        // Check if actor exists
        if !self.actor_repository.actor_exists(&id)? {
            return Ok(DisplayResult::error(format!("Actor '{}' not found", id)));
        }

        // Delete the actor
        self.actor_repository.delete_actor(&id)?;

        Ok(DisplayResult::success(format!("🗑️  Deleted actor: {}", id)))
    }

    /// Delete a persona (legacy method for backward compatibility).
    ///
    /// Removes a persona from the project, including its data file and
    /// any generated markdown documentation.
    ///
    /// # Arguments
    /// * `id` - The persona ID to delete
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if persona not found or deletion fails
    pub fn delete_persona(&self, id: String) -> Result<DisplayResult> {
        // Check if persona exists
        if !self.persona_repository.exists(&id)? {
            return Ok(DisplayResult::error(format!("Persona '{}' not found", id)));
        }

        // Delete the persona
        self.persona_repository.delete(&id)?;

        Ok(DisplayResult::success(format!("Deleted persona: {}", id)))
    }

    /// Get a single persona by ID.
    ///
    /// Retrieves a persona for viewing or editing.
    ///
    /// # Arguments
    /// * `id` - The persona ID to retrieve
    ///
    /// # Returns
    /// Reference to the persona
    ///
    /// # Errors
    /// Returns error if persona not found
    pub fn get_persona(&self, id: &str) -> Result<Persona> {
        self.persona_repository
            .load_by_id(id)?
            .context(format!("Persona '{}' not found", id))
    }

    /// Get a single actor by ID.
    ///
    /// Retrieves an actor for viewing or editing.
    ///
    /// # Arguments
    /// * `id` - The actor ID to retrieve
    ///
    /// # Returns
    /// The actor entity
    ///
    /// # Errors
    /// Returns error if actor not found
    pub fn get_actor(&self, id: &str) -> Result<ActorEntity> {
        self.actor_repository
            .load_actor_by_id(id)?
            .context(format!("Actor '{}' not found", id))
    }

    /// List all personas in the project.
    ///
    /// Retrieves all personas for display or selection.
    ///
    /// # Returns
    /// Vector of all personas
    ///
    /// # Errors
    /// Returns error if persona retrieval fails
    pub fn list_personas(&self) -> Result<Vec<Persona>> {
        self.persona_repository.load_all()
    }

    /// List all actors in the project.
    ///
    /// Retrieves all actors for display or selection, optionally filtered by type.
    ///
    /// # Arguments
    /// * `actor_type_filter` - Optional actor type to filter by
    ///
    /// # Returns
    /// Vector of all actors
    ///
    /// # Errors
    /// Returns error if actor retrieval fails
    pub fn list_actors(&self, actor_type_filter: Option<ActorType>) -> Result<Vec<ActorEntity>> {
        let all_actors = self.actor_repository.load_all_actors()?;

        if let Some(filter) = actor_type_filter {
            Ok(all_actors
                .into_iter()
                .filter(|a| a.actor_type == filter)
                .collect())
        } else {
            Ok(all_actors)
        }
    }

    /// Get list of persona IDs for selection prompts.
    ///
    /// Returns a simple list of IDs suitable for dropdown menus
    /// and selection prompts.
    ///
    /// # Returns
    /// Vector of persona IDs
    ///
    /// # Errors
    /// Returns error if persona retrieval fails
    pub fn get_persona_ids(&self) -> Result<Vec<String>> {
        let personas = self.persona_repository.load_all()?;
        Ok(personas.into_iter().map(|p| p.id).collect())
    }

    /// Get list of actor IDs for selection prompts.
    ///
    /// Returns a simple list of IDs suitable for dropdown menus.
    ///
    /// # Returns
    /// Vector of actor IDs
    ///
    /// # Errors
    /// Returns error if actor retrieval fails
    pub fn get_actor_ids(&self) -> Result<Vec<String>> {
        let actors = self.actor_repository.load_all_actors()?;
        Ok(actors.into_iter().map(|a| a.id).collect())
    }

    /// Get persona field configuration.
    ///
    /// Returns the custom field definitions configured for personas
    /// in the project, useful for dynamic form generation.
    ///
    /// # Returns
    /// Map of field name to field configuration
    pub fn get_persona_field_config(&self) -> HashMap<String, crate::core::CustomFieldConfig> {
        self.config.actor.persona_fields.clone()
    }

    /// Get current custom field values for a persona.
    ///
    /// Retrieves the current values of custom fields for editing.
    ///
    /// # Arguments
    /// * `id` - The persona ID
    ///
    /// # Returns
    /// Map of field names to JSON values
    ///
    /// # Errors
    /// Returns error if persona not found
    pub fn get_persona_field_values(&self, id: &str) -> Result<HashMap<String, serde_json::Value>> {
        let persona = self.get_persona(id)?;
        Ok(persona.extra.clone())
    }

    /// Check if the controller is using SQLite backend.
    ///
    /// Returns true if the storage backend is SQLite, false if TOML.
    /// Used to show SQLite WIP disclaimers.
    ///
    /// # Returns
    /// true if using SQLite, false if using TOML
    pub fn is_using_sqlite(&self) -> bool {
        matches!(
            self.config.storage.backend,
            crate::config::StorageBackend::Sqlite
        )
    }

    // ========== Markdown Generation Methods ==========

    /// Generate and save markdown documentation for a single actor.
    ///
    /// Creates markdown documentation using the actor.hbs template and saves it
    /// to the appropriate location in the docs directory.
    ///
    /// # Arguments
    /// * `actor_id` - The ID of the actor to generate documentation for
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// Returns error if actor not found or markdown generation fails
    pub fn generate_actor_markdown(&self, actor_id: &str) -> Result<()> {
        // Load the actor
        let actor = self
            .actor_repository
            .load_actor_by_id(actor_id)?
            .with_context(|| format!("Actor '{}' not found", actor_id))?;

        // Generate markdown
        let generator = ActorMarkdownGenerator::new()?;
        let markdown = generator.generate(&actor)?;

        // Save markdown
        self.actor_repository
            .save_actor_markdown(actor_id, &markdown)?;

        Ok(())
    }

    /// Regenerate markdown documentation for all actors.
    ///
    /// Generates fresh markdown documentation for all actors (personas and system actors)
    /// in the project. Useful after template changes or bulk actor updates.
    ///
    /// # Returns
    /// DisplayResult with count of regenerated actor docs
    ///
    /// # Errors
    /// Returns error if markdown generation fails
    pub fn regenerate_all_actor_markdown(&self) -> Result<DisplayResult> {
        let actors = self.actor_repository.load_all_actors()?;
        let generator = ActorMarkdownGenerator::new()?;

        for actor in &actors {
            let markdown = generator.generate(actor)?;
            self.actor_repository
                .save_actor_markdown(&actor.id, &markdown)?;
        }

        Ok(DisplayResult::success(format!(
            "✅ Regenerated documentation for {} actor(s)",
            actors.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Result<()> {
        let config_content = r#"
[project]
name = "Test Project"
description = "Test project"

[directories]
use_case_dir = "docs/use-cases"
test_dir = "tests"
actor_dir = "docs/actors"
data_dir = "data"

[templates]
methodologies = ["feature"]
default_methodology = "feature"

[generation]
test_language = "rust"
auto_generate_tests = false
overwrite_test_documentation = false

[storage]
backend = "toml"

[metadata]
created = true
last_updated = true

[actor.persona_fields]
department = { type = "string", required = false }
experience_level = { type = "string", required = false }
"#;

        // Create .config/.mucm directory structure (where config is expected)
        fs::create_dir_all(temp_dir.path().join(".config/.mucm"))?;
        fs::create_dir_all(temp_dir.path().join("data"))?;
        fs::write(
            temp_dir.path().join(".config/.mucm/mucm.toml"),
            config_content,
        )?;
        std::env::set_current_dir(temp_dir.path())?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_create_persona() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        let result = controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        assert!(result.success);
        assert!(result.message.contains("Created persona"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_duplicate_persona() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        let result = controller.create_persona(
            "test_user".to_string(),
            "Another User".to_string(),
            "Test Role".to_string(),
        )?;

        assert!(!result.success);
        assert!(result.message.contains("already exists"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_update_persona() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        let result =
            controller.update_persona("test_user".to_string(), Some("Updated Name".to_string()))?;

        assert!(result.success);

        let persona = controller.get_persona("test_user")?;
        assert_eq!(persona.name, "Updated Name");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_update_persona_fields() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        // Update custom fields
        let mut fields = HashMap::new();
        fields.insert("department".to_string(), "Engineering".to_string());
        fields.insert("experience_level".to_string(), "Senior".to_string());

        let result = controller.update_persona_fields("test_user".to_string(), fields)?;

        assert!(result.success);
        assert!(result.message.contains("Updated"));

        // Verify the fields were updated
        let persona = controller.get_persona("test_user")?;
        assert_eq!(
            persona.extra.get("department"),
            Some(&serde_json::Value::String("Engineering".to_string()))
        );
        assert_eq!(
            persona.extra.get("experience_level"),
            Some(&serde_json::Value::String("Senior".to_string()))
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_update_persona_fields_with_different_types() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        // Update with different data types (as strings - the controller converts them)
        let mut fields = HashMap::new();
        fields.insert("department".to_string(), "Sales".to_string());
        fields.insert("years_experience".to_string(), "5".to_string());
        fields.insert("is_manager".to_string(), "true".to_string());
        fields.insert(
            "skills".to_string(),
            "communication\nleadership\nproblem-solving".to_string(),
        );

        let result = controller.update_persona_fields("test_user".to_string(), fields)?;

        assert!(result.success);

        // Verify all field types were stored correctly
        let persona = controller.get_persona("test_user")?;
        assert_eq!(
            persona.extra.get("department"),
            Some(&serde_json::Value::String("Sales".to_string()))
        );
        assert_eq!(
            persona.extra.get("years_experience"),
            Some(&serde_json::json!(5.0))
        );
        assert_eq!(
            persona.extra.get("is_manager"),
            Some(&serde_json::Value::Bool(true))
        );
        assert!(persona.extra.contains_key("skills"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_delete_persona() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        // Verify it exists
        assert!(controller.get_persona("test_user").is_ok());

        // Delete it
        let result = controller.delete_persona("test_user".to_string())?;
        assert!(result.success);
        assert!(result.message.contains("Deleted persona"));

        // Verify it's gone
        assert!(controller.get_persona("test_user").is_err());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_list_personas() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;

        // Create multiple personas
        controller.create_persona(
            "user1".to_string(),
            "User One".to_string(),
            "Test Role".to_string(),
        )?;
        controller.create_persona(
            "user2".to_string(),
            "User Two".to_string(),
            "Test Role".to_string(),
        )?;
        controller.create_persona(
            "user3".to_string(),
            "User Three".to_string(),
            "Test Role".to_string(),
        )?;

        // List all personas
        let personas = controller.list_personas()?;
        assert_eq!(personas.len(), 3);

        let ids: Vec<String> = personas.iter().map(|p| p.id.clone()).collect();
        assert!(ids.contains(&"user1".to_string()));
        assert!(ids.contains(&"user2".to_string()));
        assert!(ids.contains(&"user3".to_string()));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_get_persona_ids() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;

        // Create personas
        controller.create_persona(
            "admin".to_string(),
            "Admin User".to_string(),
            "Test Role".to_string(),
        )?;
        controller.create_persona(
            "developer".to_string(),
            "Dev User".to_string(),
            "Test Role".to_string(),
        )?;

        // Get IDs
        let ids = controller.get_persona_ids()?;
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"admin".to_string()));
        assert!(ids.contains(&"developer".to_string()));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_get_persona_field_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;

        // Get field config
        let config = controller.get_persona_field_config();

        // Verify our test config fields are present
        assert!(config.contains_key("department"));
        assert!(config.contains_key("experience_level"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_get_persona_field_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        let controller = PersonaController::new()?;
        controller.create_persona(
            "test_user".to_string(),
            "Test User".to_string(),
            "Test Role".to_string(),
        )?;

        // Add some field values
        let mut fields = HashMap::new();
        fields.insert("department".to_string(), "Marketing".to_string());
        controller.update_persona_fields("test_user".to_string(), fields)?;

        // Get field values
        let values = controller.get_persona_field_values("test_user")?;
        assert!(values.contains_key("department"));
        assert_eq!(
            values.get("department"),
            Some(&serde_json::Value::String("Marketing".to_string()))
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_persona_markdown_generation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        // Create necessary directories
        fs::create_dir_all(temp_dir.path().join("docs/actors"))?;

        let controller = PersonaController::new()?;

        // Create a persona
        let result = controller.create_persona(
            "test-user".to_string(),
            "Test User".to_string(),
            "Software Engineer".to_string(),
        )?;

        assert!(result.success);

        // Check that markdown file was created
        let markdown_path = temp_dir.path().join("docs/actors/test-user.md");
        assert!(
            markdown_path.exists(),
            "Markdown file should be created at {:?}",
            markdown_path
        );

        // Read and verify markdown content
        let markdown_content = fs::read_to_string(&markdown_path)?;
        assert!(markdown_content.contains("# 🙂 Test User"));
        assert!(markdown_content.contains("**ID:** `test-user`"));
        assert!(markdown_content.contains("**Type:** Persona"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_system_actor_markdown_generation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        // Create necessary directories
        fs::create_dir_all(temp_dir.path().join("docs/actors"))?;

        let controller = ActorController::new()?;

        // Create a system actor
        let result = controller.create_system_actor(
            "test-database".to_string(),
            "Test Database".to_string(),
            "database".to_string(),
            Some("💾".to_string()),
        )?;

        assert!(result.success);

        // Check that markdown file was created
        let markdown_path = temp_dir.path().join("docs/actors/test-database.md");
        assert!(
            markdown_path.exists(),
            "Markdown file should be created at {:?}",
            markdown_path
        );

        // Read and verify markdown content
        let markdown_content = fs::read_to_string(&markdown_path)?;
        assert!(markdown_content.contains("# 💾 Test Database"));
        assert!(markdown_content.contains("**ID:** `test-database`"));
        assert!(markdown_content.contains("**Type:** Database"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_regenerate_all_actor_markdown() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(&temp_dir)?;

        // Create necessary directories
        fs::create_dir_all(temp_dir.path().join("docs/actors"))?;

        let controller = ActorController::new()?;

        // Create multiple actors
        controller.create_persona(
            "persona1".to_string(),
            "Persona One".to_string(),
            "Role 1".to_string(),
        )?;

        controller.create_system_actor(
            "system1".to_string(),
            "System One".to_string(),
            "system".to_string(),
            None,
        )?;

        // Delete markdown files to test regeneration
        let _ = fs::remove_file(temp_dir.path().join("docs/actors/persona1.md"));
        let _ = fs::remove_file(temp_dir.path().join("docs/actors/system1.md"));

        // Regenerate all markdown
        let result = controller.regenerate_all_actor_markdown()?;
        assert!(result.success);
        assert!(result.message.contains("2 actor(s)"));

        // Verify both markdown files were regenerated
        assert!(temp_dir.path().join("docs/actors/persona1.md").exists());
        assert!(temp_dir.path().join("docs/actors/system1.md").exists());

        Ok(())
    }
}
