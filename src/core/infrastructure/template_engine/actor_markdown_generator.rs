use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::core::domain::ActorEntity;

/// Generator for actor markdown documentation
pub struct ActorMarkdownGenerator {
    handlebars: Handlebars<'static>,
}

impl ActorMarkdownGenerator {
    /// Create a new actor markdown generator
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register custom helpers
        super::helpers::register_helpers(&mut handlebars);

        // Load actor template
        let user_templates_path =
            Path::new(".config/.mucm").join(crate::config::Config::TEMPLATES_DIR);
        let source_templates_path = Path::new("source-templates");

        let templates_dir = if user_templates_path.exists() {
            &user_templates_path
        } else {
            source_templates_path
        };

        let actor_template_path = templates_dir.join("actor.hbs");

        if actor_template_path.exists() {
            let template_content = fs::read_to_string(&actor_template_path)
                .context("Failed to read actor.hbs template")?;
            handlebars
                .register_template_string("actor", template_content)
                .context("Failed to register actor template")?;
        } else {
            // Register a default template if actor.hbs doesn't exist
            let default_template = r#"# {{emoji}} {{name}}

**ID:** `{{id}}`  
**Type:** {{actor_type}}

{{#if (eq actor_type "Persona")}}
## Persona Details

{{#if background}}
**Background:** {{background}}
{{/if}}

{{#if job_role}}
**Role:** {{job_role}}
{{/if}}

{{#if education}}
**Education:** {{education}}
{{/if}}

{{#if technical_experience}}
**Technical Experience:** {{technical_experience}}
{{/if}}

{{#if motivation_for_product}}
**Motivation:** {{motivation_for_product}}
{{/if}}
{{else}}
## System Actor

{{#if description}}
{{description}}
{{else}}
System actor: {{name}}
{{/if}}
{{/if}}

---
*Created: {{metadata.created_at}}*  
{{#if metadata.updated_at}}*Last Updated: {{metadata.updated_at}}*{{/if}}
"#;
            handlebars
                .register_template_string("actor", default_template)
                .context("Failed to register default actor template")?;
        }

        Ok(Self { handlebars })
    }

    /// Generate markdown for an actor
    pub fn generate(&self, actor: &ActorEntity) -> Result<String> {
        let data = self.actor_to_template_data(actor);
        self.handlebars
            .render("actor", &data)
            .context("Failed to render actor markdown")
    }

    /// Convert actor entity to template data
    fn actor_to_template_data(&self, actor: &ActorEntity) -> Value {
        // Load config for date formatting
        let date_format = crate::config::Config::load()
            .map(|c| c.metadata.date_format)
            .unwrap_or_else(|_| "%d/%m/%Y".to_string());

        // Format dates according to config
        let (created, last_updated) = crate::core::utils::format_datetime_pair(
            &actor.metadata.created_at,
            &actor.metadata.updated_at,
            &date_format,
        );

        let mut data = json!({
            "id": actor.id,
            "name": actor.name,
            "actor_type": actor.actor_type.to_string(),
            "emoji": actor.emoji,
            "created": created,
            "created_date": created,
            "last_updated": last_updated,
            "metadata": {
                "created_at": actor.metadata.created_at.to_rfc3339(),
                "updated_at": actor.metadata.updated_at.to_rfc3339(),
            }
        });

        // Add persona-specific fields if they exist
        if let Some(obj) = data.as_object_mut() {
            for (key, value) in &actor.extra {
                // Skip emoji since it's already at top level
                if key != "emoji" {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }

        data
    }
}

impl Default for ActorMarkdownGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create ActorMarkdownGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::ActorType;

    #[test]
    fn test_generate_persona_markdown() {
        let generator = ActorMarkdownGenerator::new().unwrap();

        let mut actor = ActorEntity::new(
            "sarah-chen".to_string(),
            "Sarah Chen".to_string(),
            ActorType::Persona,
            "👤".to_string(),
        );

        actor.extra.insert(
            "background".to_string(),
            json!("32-year-old marketing professional"),
        );
        actor
            .extra
            .insert("job_role".to_string(), json!("Digital Marketing Manager"));

        let markdown = generator.generate(&actor).unwrap();

        assert!(markdown.contains("# 👤 Sarah Chen"));
        assert!(markdown.contains("**ID:** `sarah-chen`"));
        assert!(markdown.contains("**Type:** Persona"));
        assert!(markdown.contains("32-year-old marketing professional"));
    }

    #[test]
    fn test_generate_system_actor_markdown() {
        let generator = ActorMarkdownGenerator::new().unwrap();

        let mut actor = ActorEntity::new(
            "database".to_string(),
            "Database".to_string(),
            ActorType::Database,
            "💾".to_string(),
        );

        actor.extra.insert(
            "description".to_string(),
            json!("PostgreSQL database for persistent storage"),
        );

        let markdown = generator.generate(&actor).unwrap();

        assert!(markdown.contains("# 💾 Database"));
        assert!(markdown.contains("**ID:** `database`"));
        assert!(markdown.contains("**Type:** Database"));
    }
}
