//! # Actor Entity
//!
//! Represents a managed actor in the system - either a human persona or a system actor.
//! Actors are participants that can be referenced in scenarios and use cases.
//!
//! Based on Sommerville's software engineering principles, human actors (personas)
//! represent archetypal users with specific characteristics, backgrounds, and motivations.
//! System actors represent technical entities like databases, APIs, and external services.

use super::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of actor in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// Human persona (user archetype)
    Persona,
    /// System component
    System,
    /// External service or API
    ExternalService,
    /// Database system
    Database,
    /// Custom actor type
    Custom,
}

impl ActorType {
    /// Check if this is a human actor type
    pub fn is_human(&self) -> bool {
        matches!(self, ActorType::Persona)
    }

    /// Check if this is a system/technical actor type
    pub fn is_system(&self) -> bool {
        matches!(
            self,
            ActorType::System | ActorType::ExternalService | ActorType::Database
        )
    }
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ActorType::Persona => write!(f, "Persona"),
            ActorType::System => write!(f, "System"),
            ActorType::ExternalService => write!(f, "ExternalService"),
            ActorType::Database => write!(f, "Database"),
            ActorType::Custom => write!(f, "Custom"),
        }
    }
}

impl std::str::FromStr for ActorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Persona" | "persona" => Ok(ActorType::Persona),
            "System" | "system" => Ok(ActorType::System),
            "ExternalService" | "external_service" => Ok(ActorType::ExternalService),
            "Database" | "database" => Ok(ActorType::Database),
            "Custom" | "custom" => Ok(ActorType::Custom),
            _ => Err(format!("Invalid actor type: {}", s)),
        }
    }
}

/// Managed actor entity - represents a participant in scenarios
///
/// Actors can be:
/// - **Personas**: Human user archetypes with detailed backgrounds (Sommerville)
/// - **System Actors**: Technical components (Database, API, WebServer, etc.)
///
/// Actors are created once and can be referenced in multiple scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorEntity {
    /// Unique identifier (kebab-case recommended, e.g., "primary-teacher", "payment-api")
    /// Used in filenames and scenario references - immutable after creation
    pub id: String,

    /// Display name (e.g., "Jack", "Payment Gateway")
    pub name: String,

    /// Short call name for use in scenarios (e.g., "guest", "database")
    /// Defaults to full name if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_name: Option<String>,

    /// Type of actor
    pub actor_type: ActorType,

    /// Emoji representation for visual identification
    /// Default: 🙂 for personas, specific emojis for system actors
    pub emoji: String,

    /// Metadata (created_at, updated_at)
    pub metadata: Metadata,

    /// Type-specific fields stored as flexible JSON
    /// For personas: background, job_role, education, technical_experience, motivation_for_product
    /// For system actors: hostname, port, connection_string, etc.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ActorEntity {
    /// Create a new actor with minimal required fields
    pub fn new(id: String, name: String, actor_type: ActorType, emoji: String) -> Self {
        Self {
            id,
            name,
            call_name: None,
            actor_type,
            emoji,
            metadata: Metadata::new(),
            extra: HashMap::new(),
        }
    }

    /// Create a new actor with call_name
    pub fn with_call_name(
        id: String,
        name: String,
        call_name: Option<String>,
        actor_type: ActorType,
        emoji: String,
    ) -> Self {
        Self {
            id,
            name,
            call_name,
            actor_type,
            emoji,
            metadata: Metadata::new(),
            extra: HashMap::new(),
        }
    }

    /// Get the call name, defaulting to full name if not set
    pub fn get_call_name(&self) -> &str {
        self.call_name.as_deref().unwrap_or(&self.name)
    }

    /// Create a persona actor with default emoji
    pub fn persona(id: String, name: String) -> Self {
        Self::new(id, name, ActorType::Persona, "🙂".to_string())
    }

    /// Create a system actor with specific emoji
    pub fn system(id: String, name: String, emoji: String) -> Self {
        Self::new(id, name, ActorType::System, emoji)
    }

    /// Validate actor ID format (kebab-case recommended)
    ///
    /// # Arguments
    /// * `id` - The ID to validate
    ///
    /// # Returns
    /// * `Ok(())` if valid
    /// * `Err(String)` with helpful error message
    pub fn validate_id(id: &str) -> Result<(), String> {
        if id.is_empty() {
            return Err("Actor ID cannot be empty".to_string());
        }

        // Check for valid characters (alphanumeric, hyphen, underscore)
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!(
                "Actor ID '{}' contains invalid characters. Use only lowercase letters, numbers, hyphens, and underscores.",
                id
            ));
        }

        // Check for lowercase (kebab-case convention)
        if id.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(format!(
                "Actor ID '{}' should use lowercase letters. Recommended format: kebab-case (e.g., 'primary-teacher', 'payment-gateway').",
                id
            ));
        }

        // Check for leading/trailing hyphens or underscores
        if id.starts_with('-') || id.starts_with('_') || id.ends_with('-') || id.ends_with('_') {
            return Err(format!(
                "Actor ID '{}' cannot start or end with hyphens or underscores.",
                id
            ));
        }

        Ok(())
    }

    /// Get standard system actors with default emojis
    ///
    /// Returns a list of commonly used system actors that can be initialized
    /// in a project for quick reference in scenarios.
    pub fn standard_actors() -> Vec<ActorEntity> {
        vec![
            ActorEntity::new(
                "database".to_string(),
                "Database".to_string(),
                ActorType::Database,
                "💾".to_string(),
            ),
            ActorEntity::system(
                "webserver".to_string(),
                "Web Server".to_string(),
                "🖥️".to_string(),
            ),
            ActorEntity::system("api".to_string(), "API".to_string(), "🌐".to_string()),
            ActorEntity::new(
                "payment-gateway".to_string(),
                "Payment Gateway".to_string(),
                ActorType::ExternalService,
                "💳".to_string(),
            ),
            ActorEntity::new(
                "email-service".to_string(),
                "Email Service".to_string(),
                ActorType::ExternalService,
                "📧".to_string(),
            ),
            ActorEntity::system("cache".to_string(), "Cache".to_string(), "⚡".to_string()),
            ActorEntity::system(
                "message-queue".to_string(),
                "Message Queue".to_string(),
                "📬".to_string(),
            ),
            ActorEntity::new(
                "auth-service".to_string(),
                "Auth Service".to_string(),
                ActorType::ExternalService,
                "🔐".to_string(),
            ),
            ActorEntity::system(
                "storage".to_string(),
                "Storage".to_string(),
                "📦".to_string(),
            ),
            ActorEntity::system(
                "load-balancer".to_string(),
                "Load Balancer".to_string(),
                "⚖️".to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_entity_creation() {
        let actor = ActorEntity::persona("admin-user".to_string(), "Admin User".to_string());

        assert_eq!(actor.id, "admin-user");
        assert_eq!(actor.name, "Admin User");
        assert_eq!(actor.actor_type, ActorType::Persona);
        assert_eq!(actor.emoji, "🙂");
        assert!(actor.extra.is_empty());
    }

    #[test]
    fn test_system_actor_creation() {
        let actor = ActorEntity::system(
            "api".to_string(),
            "API Gateway".to_string(),
            "🌐".to_string(),
        );

        assert_eq!(actor.id, "api");
        assert_eq!(actor.name, "API Gateway");
        assert_eq!(actor.actor_type, ActorType::System);
        assert_eq!(actor.emoji, "🌐");
    }

    #[test]
    fn test_actor_type_is_human() {
        assert!(ActorType::Persona.is_human());
        assert!(!ActorType::System.is_human());
        assert!(!ActorType::Database.is_human());
        assert!(!ActorType::ExternalService.is_human());
    }

    #[test]
    fn test_actor_type_is_system() {
        assert!(ActorType::System.is_system());
        assert!(ActorType::Database.is_system());
        assert!(ActorType::ExternalService.is_system());
        assert!(!ActorType::Persona.is_system());
    }

    #[test]
    fn test_validate_id_valid() {
        assert!(ActorEntity::validate_id("admin-user").is_ok());
        assert!(ActorEntity::validate_id("payment-gateway").is_ok());
        assert!(ActorEntity::validate_id("api_service").is_ok());
        assert!(ActorEntity::validate_id("user123").is_ok());
    }

    #[test]
    fn test_validate_id_invalid() {
        assert!(ActorEntity::validate_id("").is_err());
        assert!(ActorEntity::validate_id("Admin-User").is_err()); // uppercase
        assert!(ActorEntity::validate_id("admin user").is_err()); // space
        assert!(ActorEntity::validate_id("-admin").is_err()); // leading hyphen
        assert!(ActorEntity::validate_id("admin-").is_err()); // trailing hyphen
        assert!(ActorEntity::validate_id("admin@user").is_err()); // invalid char
    }

    #[test]
    fn test_standard_actors() {
        let actors = ActorEntity::standard_actors();

        assert_eq!(actors.len(), 10);
        assert!(actors.iter().any(|a| a.id == "database"));
        assert!(actors.iter().any(|a| a.id == "api"));
        assert!(actors.iter().any(|a| a.id == "payment-gateway"));

        // Check emojis
        let db = actors.iter().find(|a| a.id == "database").unwrap();
        assert_eq!(db.emoji, "💾");
    }

    #[test]
    fn test_actor_serialization() {
        let actor = ActorEntity::persona("test-user".to_string(), "Test User".to_string());

        let json = serde_json::to_string(&actor).unwrap();
        let deserialized: ActorEntity = serde_json::from_str(&json).unwrap();

        assert_eq!(actor.id, deserialized.id);
        assert_eq!(actor.name, deserialized.name);
        assert_eq!(actor.actor_type, deserialized.actor_type);
        assert_eq!(actor.emoji, deserialized.emoji);
    }
}
