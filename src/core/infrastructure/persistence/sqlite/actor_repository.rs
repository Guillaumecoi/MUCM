//! SQLite implementation of ActorRepository.
//!
//! Stores actors (personas and system actors) in a SQLite database with full CRUD operations.

use crate::core::domain::{
    ActorEntity, ActorRepository, ActorType, Metadata, Persona, PersonaRepository,
};
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// SQLite-backed actor repository.
pub struct SqliteActorRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteActorRepository {
    /// Create a new SQLite actor repository.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Initialize the actors table in the database.
    /// Uses a flexible schema with JSON for extra fields.
    /// Also creates the old personas table for backward compatibility.
    pub fn initialize(conn: &Connection) -> Result<()> {
        // Create actors table (new unified system)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS actors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                actor_type TEXT NOT NULL,
                emoji TEXT NOT NULL DEFAULT '🙂',
                extra_fields TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Index for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_actors_name 
             ON actors(name COLLATE NOCASE)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_actors_type 
             ON actors(actor_type)",
            [],
        )?;

        // Create personas table for backward compatibility
        // This will be migrated to actors table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS personas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                extra_fields TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_personas_name 
             ON personas(name COLLATE NOCASE)",
            [],
        )?;

        Ok(())
    }
}

// === ActorRepository implementation (new unified actor system) ===

impl ActorRepository for SqliteActorRepository {
    fn save_actor(&self, actor: &ActorEntity) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        // Serialize extra fields to JSON
        let extra_json = serde_json::to_string(&actor.extra)?;

        // Convert actor_type to string
        let actor_type_str = match actor.actor_type {
            ActorType::Persona => "Persona",
            ActorType::System => "System",
            ActorType::ExternalService => "ExternalService",
            ActorType::Database => "Database",
            ActorType::Custom => "Custom",
        };

        conn.execute(
            "INSERT OR REPLACE INTO actors 
             (id, name, actor_type, emoji, extra_fields, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 
                     COALESCE((SELECT created_at FROM actors WHERE id = ?1), ?6), 
                     ?6)",
            params![
                actor.id,
                actor.name,
                actor_type_str,
                actor.emoji,
                extra_json,
                now
            ],
        )?;

        Ok(())
    }

    fn load_all_actors(&self) -> Result<Vec<ActorEntity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, actor_type, emoji, extra_fields, created_at, updated_at
             FROM actors
             ORDER BY name",
        )?;

        let actors = stmt
            .query_map([], |row| {
                let extra_json: String = row.get(4).unwrap_or_else(|_| "{}".to_string());
                let extra: HashMap<String, serde_json::Value> =
                    serde_json::from_str(&extra_json).unwrap_or_default();

                let created_str: String = row.get(5)?;
                let updated_str: String = row.get(6)?;

                let created_at = created_str
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_else(|_| chrono::Utc::now());
                let updated_at = updated_str
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_else(|_| chrono::Utc::now());

                let actor_type_str: String = row.get(2)?;
                let actor_type = ActorType::from_str(&actor_type_str).unwrap_or(ActorType::Custom);

                Ok(ActorEntity {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    actor_type,
                    emoji: row.get(3)?,
                    metadata: Metadata {
                        created_at,
                        updated_at,
                    },
                    extra,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(actors)
    }

    fn load_actor_by_id(&self, id: &str) -> Result<Option<ActorEntity>> {
        let conn = self.conn.lock().unwrap();
        let actor = conn
            .query_row(
                "SELECT id, name, actor_type, emoji, extra_fields, created_at, updated_at
                 FROM actors
                 WHERE id = ?1",
                params![id],
                |row| {
                    let extra_json: String = row.get(4).unwrap_or_else(|_| "{}".to_string());
                    let extra: HashMap<String, serde_json::Value> =
                        serde_json::from_str(&extra_json).unwrap_or_default();

                    let created_str: String = row.get(5)?;
                    let updated_str: String = row.get(6)?;

                    let created_at = created_str
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_else(|_| chrono::Utc::now());
                    let updated_at = updated_str
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap_or_else(|_| chrono::Utc::now());

                    let actor_type_str: String = row.get(2)?;
                    let actor_type =
                        ActorType::from_str(&actor_type_str).unwrap_or(ActorType::Custom);

                    Ok(ActorEntity {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        actor_type,
                        emoji: row.get(3)?,
                        metadata: Metadata {
                            created_at,
                            updated_at,
                        },
                        extra,
                    })
                },
            )
            .optional()?;

        Ok(actor)
    }

    fn delete_actor(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM actors WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn actor_exists(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM actors WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn save_actor_markdown(&self, _actor_id: &str, _markdown_content: &str) -> Result<()> {
        // SQLite backend doesn't store markdown files separately
        // Markdown generation is handled by the application layer
        Ok(())
    }

    // === Persona compatibility methods (backward compatibility) ===

    fn save_persona(&self, persona: &Persona) -> Result<()> {
        let actor = persona.to_actor();
        self.save_actor(&actor)
    }

    fn load_all_personas(&self) -> Result<Vec<Persona>> {
        let actors = self.load_all_actors()?;
        let personas = actors.iter().filter_map(Persona::from_actor).collect();
        Ok(personas)
    }

    fn load_persona_by_id(&self, id: &str) -> Result<Option<Persona>> {
        let actor = self.load_actor_by_id(id)?;
        Ok(actor.and_then(|a| Persona::from_actor(&a)))
    }

    fn delete_persona(&self, id: &str) -> Result<()> {
        self.delete_actor(id)
    }

    fn persona_exists(&self, id: &str) -> Result<bool> {
        self.actor_exists(id)
    }

    fn save_persona_markdown(&self, persona_id: &str, markdown_content: &str) -> Result<()> {
        self.save_actor_markdown(persona_id, markdown_content)
    }
}

// === PersonaRepository implementation (for backward compatibility with existing code) ===

impl PersonaRepository for SqliteActorRepository {
    fn save(&self, persona: &Persona) -> Result<()> {
        self.save_persona(persona)
    }

    fn load_all(&self) -> Result<Vec<Persona>> {
        self.load_all_personas()
    }

    fn load_by_id(&self, id: &str) -> Result<Option<Persona>> {
        self.load_persona_by_id(id)
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.delete_persona(id)
    }

    fn exists(&self, id: &str) -> Result<bool> {
        self.persona_exists(id)
    }

    fn save_markdown(&self, persona_id: &str, markdown_content: &str) -> Result<()> {
        self.save_persona_markdown(persona_id, markdown_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo() -> SqliteActorRepository {
        let conn = Connection::open_in_memory().unwrap();
        SqliteActorRepository::initialize(&conn).unwrap();
        SqliteActorRepository::new(Arc::new(Mutex::new(conn)))
    }

    fn create_test_persona() -> Persona {
        Persona::new(
            "test-persona".to_string(),
            "Test User".to_string(),
            "Test Function".to_string(),
        )
    }

    fn create_test_actor() -> ActorEntity {
        ActorEntity::new(
            "test-database".to_string(),
            "Test Database".to_string(),
            ActorType::Database,
            "💾".to_string(),
        )
    }

    #[test]
    fn test_initialize_creates_table() {
        let conn = Connection::open_in_memory().unwrap();
        SqliteActorRepository::initialize(&conn).unwrap();

        // Verify actors table exists
        let actors_table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='actors'",
                [],
                |row| {
                    let count: i32 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap();
        assert!(actors_table_exists);

        // Verify personas table exists (backward compatibility)
        let personas_table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='personas'",
                [],
                |row| {
                    let count: i32 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap();

        assert!(personas_table_exists);
    }

    #[test]
    fn test_save_and_load_persona() {
        let repo = create_test_repo();
        let persona = create_test_persona();

        repo.save(&persona).unwrap();

        let loaded = repo.load_by_id("test-persona").unwrap();
        assert!(loaded.is_some());

        let loaded_persona = loaded.unwrap();
        assert_eq!(loaded_persona.id, "test-persona");
        assert_eq!(loaded_persona.name, "Test User");
    }

    #[test]
    fn test_load_all_personas() {
        let repo = create_test_repo();

        let persona1 = create_test_persona();
        repo.save(&persona1).unwrap();

        let persona2 = Persona::new(
            "admin-persona".to_string(),
            "Admin User".to_string(),
            "Test Function".to_string(),
        );
        repo.save(&persona2).unwrap();

        let personas = repo.load_all().unwrap();
        assert_eq!(personas.len(), 2);
        assert!(personas.iter().any(|p| p.id == "test-persona"));
        assert!(personas.iter().any(|p| p.id == "admin-persona"));
    }

    #[test]
    fn test_load_nonexistent_persona() {
        let repo = create_test_repo();
        let result = repo.load_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_persona() {
        let repo = create_test_repo();
        let persona = create_test_persona();

        repo.save(&persona).unwrap();
        assert!(repo.exists("test-persona").unwrap());

        repo.delete("test-persona").unwrap();
        assert!(!repo.exists("test-persona").unwrap());
        assert!(repo.load_by_id("test-persona").unwrap().is_none());
    }

    #[test]
    fn test_exists() {
        let repo = create_test_repo();
        assert!(!repo.exists("test-persona").unwrap());

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        assert!(repo.exists("test-persona").unwrap());
    }

    #[test]
    fn test_update_persona() {
        let repo = create_test_repo();

        let mut persona = create_test_persona();
        repo.save(&persona).unwrap();

        persona.name = "Updated Test User".to_string();
        repo.save(&persona).unwrap();

        let loaded = repo.load_by_id("test-persona").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Test User");
    }

    #[test]
    fn test_delete_nonexistent_persona() {
        let repo = create_test_repo();
        // Should not error when deleting nonexistent persona
        let result = repo.delete("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_overwrite() {
        let repo = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        // Save again with same ID should overwrite
        let mut updated = persona.clone();
        updated.name = "Updated Name".to_string();
        repo.save(&updated).unwrap();

        let loaded = repo.load_by_id("test-persona").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Name");
    }

    #[test]
    fn test_multiple_personas() {
        let repo = create_test_repo();

        for i in 1..=5 {
            let persona = Persona::new(
                format!("persona-{}", i),
                format!("User {}", i),
                "Test Function".to_string(),
            );
            repo.save(&persona).unwrap();
        }

        let personas = repo.load_all().unwrap();
        assert_eq!(personas.len(), 5);
    }

    #[test]
    fn test_empty_repository() {
        let repo = create_test_repo();
        let personas = repo.load_all().unwrap();
        assert_eq!(personas.len(), 0);
    }

    #[test]
    fn test_save_markdown_noop() {
        let repo = create_test_repo();
        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        // save_markdown should not error (it's a no-op for SQLite)
        let result = repo.save_markdown("test-persona", "# Test");
        assert!(result.is_ok());
    }

    // === Actor-specific tests ===

    #[test]
    fn test_save_and_load_actor() {
        let repo = create_test_repo();
        let actor = create_test_actor();

        repo.save_actor(&actor).unwrap();

        let loaded = repo.load_actor_by_id("test-database").unwrap();
        assert!(loaded.is_some());

        let loaded_actor = loaded.unwrap();
        assert_eq!(loaded_actor.id, "test-database");
        assert_eq!(loaded_actor.name, "Test Database");
        assert_eq!(loaded_actor.emoji, "💾");
        assert!(matches!(loaded_actor.actor_type, ActorType::Database));
    }

    #[test]
    fn test_load_all_actors() {
        let repo = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let actors = repo.load_all_actors().unwrap();
        assert_eq!(actors.len(), 2);
        assert!(actors.iter().any(|a| a.id == "test-persona"));
        assert!(actors.iter().any(|a| a.id == "test-database"));
    }

    #[test]
    fn test_load_all_personas_filters_actors() {
        let repo = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let personas = repo.load_all_personas().unwrap();
        assert_eq!(personas.len(), 1);
        assert!(personas.iter().any(|p| p.id == "test-persona"));
        assert!(!personas.iter().any(|p| p.id == "test-database"));
    }

    #[test]
    fn test_delete_actor() {
        let repo = create_test_repo();
        let actor = create_test_actor();

        repo.save_actor(&actor).unwrap();
        assert!(repo.actor_exists("test-database").unwrap());

        repo.delete_actor("test-database").unwrap();
        assert!(!repo.actor_exists("test-database").unwrap());
        assert!(repo.load_actor_by_id("test-database").unwrap().is_none());
    }

    #[test]
    fn test_actor_types() {
        let repo = create_test_repo();

        // Test different actor types
        for (id, name, actor_type, emoji) in [
            ("database", "Database", ActorType::Database, "💾"),
            ("webserver", "WebServer", ActorType::System, "🖥️"),
            ("api", "API", ActorType::ExternalService, "🌐"),
            ("custom", "Custom", ActorType::Custom, "🔧"),
        ] {
            let actor = ActorEntity::new(
                id.to_string(),
                name.to_string(),
                actor_type,
                emoji.to_string(),
            );
            repo.save_actor(&actor).unwrap();
        }

        let actors = repo.load_all_actors().unwrap();
        assert_eq!(actors.len(), 4);

        // Verify each actor type was preserved
        let db = actors.iter().find(|a| a.id == "database").unwrap();
        assert!(matches!(db.actor_type, ActorType::Database));

        let ws = actors.iter().find(|a| a.id == "webserver").unwrap();
        assert!(matches!(ws.actor_type, ActorType::System));

        let api = actors.iter().find(|a| a.id == "api").unwrap();
        assert!(matches!(api.actor_type, ActorType::ExternalService));

        let custom = actors.iter().find(|a| a.id == "custom").unwrap();
        assert!(matches!(custom.actor_type, ActorType::Custom));
    }
}
