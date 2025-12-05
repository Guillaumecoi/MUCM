use super::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Business persona - stakeholder experiencing scenarios
///
/// Based on persona modeling in software engineering (Sommerville et al.),
/// personas represent archetypal users with specific characteristics, goals,
/// and pain points that help drive requirements and design decisions.
///
/// Personas are created with minimal required fields (id and name).
/// Additional fields are determined by the persona configuration in the
/// project's config file and can be filled in by editing the TOML/SQL directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier (e.g., "customer", "admin", "guest")
    pub id: String,

    /// Display name (required)
    pub name: String,

    /// Short call name for use in scenarios (e.g., "guest", "admin")
    /// Defaults to full name if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_name: Option<String>,

    /// Function/role of the persona (e.g., "System Administrator", "End User")
    pub function: String,

    pub metadata: Metadata,

    /// All other persona fields are stored as flexible extra fields
    /// These are determined by the persona configuration in the project config
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Persona {
    /// Create a new persona with minimal required fields
    pub fn new(id: String, name: String, function: String) -> Self {
        Self {
            id,
            name,
            call_name: None,
            function,
            metadata: Metadata::new(),
            extra: HashMap::new(),
        }
    }

    /// Create a new persona with call_name
    pub fn with_call_name(
        id: String,
        name: String,
        call_name: Option<String>,
        function: String,
    ) -> Self {
        Self {
            id,
            name,
            call_name,
            function,
            metadata: Metadata::new(),
            extra: HashMap::new(),
        }
    }

    /// Get the call name, defaulting to full name if not set
    pub fn get_call_name(&self) -> &str {
        self.call_name.as_deref().unwrap_or(&self.name)
    }

    /// Create a persona from config fields
    /// This initializes extra fields based on the persona configuration
    pub fn from_config_fields(
        id: String,
        name: String,
        function: String,
        config_fields: &HashMap<String, crate::core::CustomFieldConfig>,
    ) -> Self {
        let mut persona = Self::new(id, name, function);

        // Initialize extra fields with empty/default values based on config
        for (field_name, field_config) in config_fields {
            let default_value = if let Some(default) = &field_config.default {
                // Use the default value from config if provided
                serde_json::Value::String(default.clone())
            } else {
                // Otherwise initialize with empty/zero values based on type
                match field_config.field_type.as_str() {
                    "string" => serde_json::Value::String(String::new()),
                    "array" => serde_json::Value::Array(vec![]),
                    "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                    "boolean" => serde_json::Value::Bool(false),
                    _ => serde_json::Value::String(String::new()),
                }
            };
            persona.extra.insert(field_name.clone(), default_value);
        }

        persona
    }

    /// Get emoji representation based on persona characteristics
    pub fn emoji(&self) -> &str {
        // Based on persona ID or role
        match self.id.to_lowercase().as_str() {
            id if id.contains("admin") || id.contains("manager") => "👨‍💼",
            id if id.contains("customer") || id.contains("buyer") => "👤",
            id if id.contains("guest") || id.contains("visitor") => "🚶",
            id if id.contains("developer") || id.contains("engineer") => "👨‍💻",
            id if id.contains("support") || id.contains("agent") => "🎧",
            id if id.contains("analyst") || id.contains("data") => "📊",
            _ => "🙂",
        }
    }

    /// Convert Persona to ActorEntity (for unified actor system)
    pub fn to_actor(&self) -> super::ActorEntity {
        let mut extra = self.extra.clone();
        // Store function in extra fields for ActorEntity compatibility
        extra.insert("function".to_string(), serde_json::json!(self.function));

        super::ActorEntity {
            id: self.id.clone(),
            name: self.name.clone(),
            call_name: self.call_name.clone(),
            actor_type: super::ActorType::Persona,
            emoji: self.emoji().to_string(),
            metadata: self.metadata.clone(),
            extra,
        }
    }

    /// Convert ActorEntity to Persona (for backward compatibility)
    /// Returns None if the actor is not a Persona type
    pub fn from_actor(actor: &super::ActorEntity) -> Option<Self> {
        if matches!(actor.actor_type, super::ActorType::Persona) {
            let function = actor
                .extra
                .get("function")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Some(Self {
                id: actor.id.clone(),
                name: actor.name.clone(),
                call_name: actor.call_name.clone(),
                function,
                metadata: actor.metadata.clone(),
                extra: actor.extra.clone(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_creation() {
        let persona = Persona::new(
            "customer".to_string(),
            "Regular Customer".to_string(),
            "End User".to_string(),
        );

        assert_eq!(persona.id, "customer");
        assert_eq!(persona.name, "Regular Customer");
        assert_eq!(persona.function, "End User");
        assert!(persona.extra.is_empty());
    }

    #[test]
    fn test_persona_emoji() {
        assert_eq!(
            Persona::new(
                "admin".to_string(),
                "Admin".to_string(),
                "System Admin".to_string()
            )
            .emoji(),
            "👨‍💼"
        );
        assert_eq!(
            Persona::new(
                "customer".to_string(),
                "Customer".to_string(),
                "End User".to_string()
            )
            .emoji(),
            "👤"
        );
        assert_eq!(
            Persona::new(
                "guest".to_string(),
                "Guest".to_string(),
                "Visitor".to_string()
            )
            .emoji(),
            "🚶"
        );
        assert_eq!(
            Persona::new(
                "developer".to_string(),
                "Developer".to_string(),
                "Software Engineer".to_string()
            )
            .emoji(),
            "👨‍💻"
        );
    }

    #[test]
    fn test_persona_with_extra_fields() {
        let mut persona = Persona::new(
            "test".to_string(),
            "Test User".to_string(),
            "Tester".to_string(),
        );

        persona
            .extra
            .insert("department".to_string(), serde_json::json!("Engineering"));
        persona
            .extra
            .insert("experience_level".to_string(), serde_json::json!("Expert"));

        assert_eq!(persona.extra.len(), 2);
        assert_eq!(
            persona.extra.get("department"),
            Some(&serde_json::json!("Engineering"))
        );
    }

    #[test]
    fn test_persona_serialization() {
        let mut persona = Persona::new(
            "test".to_string(),
            "Test".to_string(),
            "Test Role".to_string(),
        );
        persona
            .extra
            .insert("role".to_string(), serde_json::json!("Developer"));

        let json = serde_json::to_string(&persona).unwrap();
        let deserialized: Persona = serde_json::from_str(&json).unwrap();

        assert_eq!(persona.id, deserialized.id);
        assert_eq!(persona.name, deserialized.name);
        assert_eq!(persona.extra, deserialized.extra);
    }
}
