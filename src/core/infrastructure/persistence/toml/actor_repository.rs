// TOML-based implementation of ActorRepository
use crate::config::Config;
use crate::core::domain::{ActorEntity, ActorRepository, Persona, PersonaRepository};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Repository implementation that persists actors (personas and system actors) to TOML files
///
/// Architecture:
/// - TOML files (.toml) are the source of truth in .mucm/actors/
/// - Markdown files (.md) are generated documentation in docs/actors/
/// - Supports both ActorEntity (new unified system) and Persona (backward compatibility)
pub struct TomlActorRepository {
    config: Config,
}

impl TomlActorRepository {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get the directory for actor data files (TOML)
    /// Stores in data_dir/actors alongside use case data
    fn get_data_dir(&self) -> String {
        format!("{}/actors", &self.config.directories.data_dir)
    }

    /// Get the directory for actor markdown files
    /// Stores in docs/actors (configured via actor_dir)
    fn get_markdown_dir(&self) -> String {
        self.config.directories.actor_dir.clone()
    }
}

// === ActorRepository implementation (new unified actor system) ===

impl ActorRepository for TomlActorRepository {
    fn save_actor(&self, actor: &ActorEntity) -> Result<()> {
        // Create data directory structure
        let data_dir_str = self.get_data_dir();
        let data_dir = Path::new(&data_dir_str);
        fs::create_dir_all(data_dir)?;

        // Filter out Null values from extra fields before serialization
        // TOML doesn't support null values like JSON does
        let mut actor_for_toml = actor.clone();
        actor_for_toml.extra.retain(|_, v| !v.is_null());

        // Save TOML file (source of truth in data directory)
        let toml_path = data_dir.join(format!("{}.toml", actor.id));
        let toml_content = toml::to_string_pretty(&actor_for_toml)?;
        fs::write(&toml_path, toml_content)?;

        Ok(())
    }

    fn load_all_actors(&self) -> Result<Vec<ActorEntity>> {
        let data_dir_str = self.get_data_dir();
        let data_dir = Path::new(&data_dir_str);
        let mut actors = Vec::new();

        if !data_dir.exists() {
            return Ok(actors); // No actors yet
        }

        for entry in fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .toml files
            if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                let content = fs::read_to_string(&path)?;
                // Parse TOML to intermediate value, then convert to JSON value to ensure
                // extra fields are serde_json::Value instead of toml::Value
                let toml_value: toml::Value = toml::from_str(&content)?;
                let json_str = serde_json::to_string(&toml_value)?;
                let actor: ActorEntity = serde_json::from_str(&json_str)?;
                actors.push(actor);
            }
        }

        Ok(actors)
    }

    fn load_actor_by_id(&self, id: &str) -> Result<Option<ActorEntity>> {
        let toml_path = Path::new(&self.get_data_dir()).join(format!("{}.toml", id));

        if !toml_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&toml_path)?;
        let toml_value: toml::Value = toml::from_str(&content)?;
        let json_str = serde_json::to_string(&toml_value)?;
        let actor: ActorEntity = serde_json::from_str(&json_str)?;

        Ok(Some(actor))
    }

    fn delete_actor(&self, id: &str) -> Result<()> {
        // Delete TOML file from data directory
        let toml_path = Path::new(&self.get_data_dir()).join(format!("{}.toml", id));
        if toml_path.exists() {
            fs::remove_file(&toml_path)?;
        }

        // Delete markdown file
        let md_path = Path::new(&self.get_markdown_dir()).join(format!("{}.md", id));
        if md_path.exists() {
            fs::remove_file(&md_path)?;
        }

        Ok(())
    }

    fn actor_exists(&self, id: &str) -> Result<bool> {
        let toml_path = Path::new(&self.get_data_dir()).join(format!("{}.toml", id));
        Ok(toml_path.exists())
    }

    fn save_actor_markdown(&self, actor_id: &str, markdown_content: &str) -> Result<()> {
        // Create markdown directory structure
        let md_dir_str = self.get_markdown_dir();
        let md_dir = Path::new(&md_dir_str);
        fs::create_dir_all(md_dir)?;

        // Save markdown file (generated output)
        let md_path = md_dir.join(format!("{}.md", actor_id));
        fs::write(&md_path, markdown_content)?;

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

impl PersonaRepository for TomlActorRepository {
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
    use crate::core::domain::ActorType;
    use tempfile::TempDir;

    fn create_test_repo() -> (TomlActorRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create a test config pointing to temp directory
        let mut config = Config::default();
        config.directories.use_case_dir = format!("{}/docs/use-cases", temp_path);
        config.directories.data_dir = format!("{}/.mucm", temp_path);
        config.directories.actor_dir = format!("{}/docs/actors", temp_path);

        let repo = TomlActorRepository::new(config);
        (repo, temp_dir)
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
            "test-actor".to_string(),
            "Test Database".to_string(),
            ActorType::Database,
            "💾".to_string(),
        )
    }

    #[test]
    fn test_save_and_load_persona() {
        let (repo, _temp_dir) = create_test_repo();

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
        let (repo, _temp_dir) = create_test_repo();

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
        let (repo, _temp_dir) = create_test_repo();

        let result = repo.load_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_persona() {
        let (repo, _temp_dir) = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        assert!(repo.exists("test-persona").unwrap());

        repo.delete("test-persona").unwrap();

        assert!(!repo.exists("test-persona").unwrap());
        assert!(repo.load_by_id("test-persona").unwrap().is_none());
    }

    #[test]
    fn test_exists() {
        let (repo, _temp_dir) = create_test_repo();

        assert!(!repo.exists("test-persona").unwrap());

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        assert!(repo.exists("test-persona").unwrap());
    }

    #[test]
    fn test_save_markdown() {
        let (repo, _temp_dir) = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        let markdown_content = "# Test User\n\nA test persona documentation.";
        repo.save_markdown("test-persona", markdown_content)
            .unwrap();

        let md_dir_str = repo.get_markdown_dir();
        let md_path = Path::new(&md_dir_str).join("test-persona.md");
        assert!(md_path.exists());

        let content = fs::read_to_string(&md_path).unwrap();
        assert_eq!(content, markdown_content);
    }

    #[test]
    fn test_update_persona() {
        let (repo, _temp_dir) = create_test_repo();

        let mut persona = create_test_persona();
        repo.save(&persona).unwrap();

        persona.name = "Updated Test User".to_string();
        repo.save(&persona).unwrap();

        let loaded = repo.load_by_id("test-persona").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Test User");
    }

    // === Actor-specific tests ===

    #[test]
    fn test_save_and_load_actor() {
        let (repo, _temp_dir) = create_test_repo();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let loaded = repo.load_actor_by_id("test-actor").unwrap();
        assert!(loaded.is_some());

        let loaded_actor = loaded.unwrap();
        assert_eq!(loaded_actor.id, "test-actor");
        assert_eq!(loaded_actor.name, "Test Database");
        assert_eq!(loaded_actor.emoji, "💾");
        assert!(matches!(loaded_actor.actor_type, ActorType::Database));
    }

    #[test]
    fn test_load_all_actors() {
        let (repo, _temp_dir) = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let actors = repo.load_all_actors().unwrap();
        assert_eq!(actors.len(), 2);
        assert!(actors.iter().any(|a| a.id == "test-persona"));
        assert!(actors.iter().any(|a| a.id == "test-actor"));
    }

    #[test]
    fn test_load_all_personas_filters_actors() {
        let (repo, _temp_dir) = create_test_repo();

        let persona = create_test_persona();
        repo.save(&persona).unwrap();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let personas = repo.load_all_personas().unwrap();
        assert_eq!(personas.len(), 1);
        assert!(personas.iter().any(|p| p.id == "test-persona"));
        assert!(!personas.iter().any(|p| p.id == "test-actor"));
    }

    #[test]
    fn test_delete_actor() {
        let (repo, _temp_dir) = create_test_repo();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        assert!(repo.actor_exists("test-actor").unwrap());

        repo.delete_actor("test-actor").unwrap();

        assert!(!repo.actor_exists("test-actor").unwrap());
        assert!(repo.load_actor_by_id("test-actor").unwrap().is_none());
    }

    #[test]
    fn test_save_actor_markdown() {
        let (repo, _temp_dir) = create_test_repo();

        let actor = create_test_actor();
        repo.save_actor(&actor).unwrap();

        let markdown_content = "# Test Database 💾\n\nA test database actor.";
        repo.save_actor_markdown("test-actor", markdown_content)
            .unwrap();

        let md_dir_str = repo.get_markdown_dir();
        let md_path = Path::new(&md_dir_str).join("test-actor.md");
        assert!(md_path.exists());

        let content = fs::read_to_string(&md_path).unwrap();
        assert_eq!(content, markdown_content);
    }
}
