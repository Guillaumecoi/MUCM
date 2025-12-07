//! Markdown generator for use case documentation.
//!
//! Handles generation of markdown documentation from use cases using templates.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use crate::config::Config;
use crate::core::{MethodologyView, TemplateEngine, UseCase};

/// Generator for use case markdown documentation.
pub struct MarkdownGenerator {
    config: Config,
    template_engine: TemplateEngine,
}

impl MarkdownGenerator {
    /// Creates a new markdown generator with the given configuration.
    pub fn new(config: Config) -> Self {
        let template_engine = TemplateEngine::with_config(Some(&config));
        Self {
            config,
            template_engine,
        }
    }

    /// Generates markdown for a use case with flexible rendering options.
    ///
    /// Converts UseCase to JSON and passes it to the template engine.
    /// This allows templates to access ANY field from the TOML file without hardcoding.
    ///
    /// # Arguments
    /// * `use_case` - The use case to generate markdown for
    /// * `methodology` - Optional specific methodology (uses default if None)
    /// * `view` - Optional methodology view (methodology + level combination)
    ///
    /// # Returns
    /// The generated markdown content
    ///
    /// # Note
    /// If both `methodology` and `view` are provided, `view` takes precedence.
    pub fn generate(
        &self,
        use_case: &UseCase,
        methodology: Option<&str>,
        view: Option<&MethodologyView>,
    ) -> Result<String> {
        // Convert UseCase directly to JSON - templates can access any field from TOML
        let use_case_json = serde_json::to_value(use_case)?;

        // Convert to HashMap for template engine compatibility
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json)?;

        // Format dates according to config
        crate::core::utils::format_dates_from_metadata(
            &mut data,
            &self.config.metadata.date_format,
        );

        // Merge extra fields into top-level HashMap so templates can access them directly
        if let Some(Value::Object(extra_map)) = data.remove("extra") {
            for (key, value) in extra_map {
                data.insert(key, value);
            }
        }

        // Process preconditions and postconditions to convert link syntax
        Self::process_condition_links(&mut data);

        // Determine which methodology to use for field flattening
        let methodology_name = if let Some(v) = view {
            &v.methodology
        } else if let Some(m) = methodology {
            m
        } else {
            &self.config.templates.default_methodology
        };

        // Merge methodology_fields for the SPECIFIC methodology into top-level HashMap
        // This flattens methodology_fields.{current_methodology}.{field} -> {field}
        if let Some(Value::Object(methodology_fields_map)) = data.remove("methodology_fields") {
            if let Some(Value::Object(field_map)) = methodology_fields_map.get(methodology_name) {
                for (field_name, field_value) in field_map {
                    // Only insert if not already present (standard fields take priority)
                    data.entry(field_name.clone())
                        .or_insert(field_value.clone());
                }
            }
        }

        // Render based on what parameters were provided
        if let Some(v) = view {
            self.template_engine
                .render_use_case_with_methodology_and_level(&data, &v.methodology, &v.level)
        } else {
            self.template_engine
                .render_use_case_with_methodology(&data, methodology_name)
        }
    }

    /// Generates the README.md overview for a multi-view use case.
    ///
    /// Creates a summary document that lists all available methodology views
    /// and provides quick reference information about the use case.
    ///
    /// # Arguments
    /// * `use_case` - The use case to generate the README for
    ///
    /// # Returns
    /// The generated README markdown content
    pub fn generate_use_case_readme(&self, use_case: &UseCase) -> Result<String> {
        // Convert UseCase to JSON for template rendering
        let use_case_json = serde_json::to_value(use_case)?;
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json)?;

        // Format dates according to config
        crate::core::utils::format_dates_from_metadata(
            &mut data,
            &self.config.metadata.date_format,
        );

        // Merge extra fields into top-level HashMap
        if let Some(Value::Object(extra_map)) = data.remove("extra") {
            for (key, value) in extra_map {
                data.insert(key, value);
            }
        }

        // Render using the use_case_overview template
        self.template_engine.render_use_case_overview(&data)
    }

    /// Processes preconditions and postconditions to convert link syntax to markdown links.
    ///
    /// Parses text in the format: "condition text||UC:target_id:relationship"
    /// and converts it to: "condition text ([UC-XXX-NNN](../../category/UC-XXX-NNN/README.md))"
    fn process_condition_links(data: &mut HashMap<String, Value>) {
        // Process preconditions
        if let Some(Value::Array(preconditions)) = data.get("preconditions") {
            let processed: Vec<Value> = preconditions
                .iter()
                .map(|cond| {
                    if let Value::Object(cond_obj) = cond {
                        let mut new_cond = cond_obj.clone();
                        if let Some(Value::String(text)) = cond_obj.get("text") {
                            new_cond.insert(
                                "text".to_string(),
                                Value::String(Self::convert_condition_link(text)),
                            );
                        }
                        Value::Object(new_cond)
                    } else if let Value::String(text) = cond {
                        // Handle simple string preconditions (legacy format)
                        Value::String(Self::convert_condition_link(text))
                    } else {
                        cond.clone()
                    }
                })
                .collect();
            data.insert("preconditions".to_string(), Value::Array(processed));
        }

        // Process postconditions
        if let Some(Value::Array(postconditions)) = data.get("postconditions") {
            let processed: Vec<Value> = postconditions
                .iter()
                .map(|cond| {
                    if let Value::Object(cond_obj) = cond {
                        let mut new_cond = cond_obj.clone();
                        if let Some(Value::String(text)) = cond_obj.get("text") {
                            new_cond.insert(
                                "text".to_string(),
                                Value::String(Self::convert_condition_link(text)),
                            );
                        }
                        Value::Object(new_cond)
                    } else if let Value::String(text) = cond {
                        // Handle simple string postconditions (legacy format)
                        Value::String(Self::convert_condition_link(text))
                    } else {
                        cond.clone()
                    }
                })
                .collect();
            data.insert("postconditions".to_string(), Value::Array(processed));
        }
    }

    /// Converts a condition text with link syntax to markdown.
    ///
    /// Input: "User must have a registered account||UC:UC-AUTH-001:depend"
    /// Output: "User must have a registered account ([UC-AUTH-001](../../authentication/UC-AUTH-001/README.md))"
    fn convert_condition_link(text: &str) -> String {
        if let Some((condition_text, link_part)) = text.split_once("||UC:") {
            // Parse the link part: target_id:relationship
            let parts: Vec<&str> = link_part.split(':').collect();
            if parts.len() >= 2 {
                let target_id = parts[0];
                // We need to resolve the category of the target use case
                // For now, we'll use the resolve_use_case_link logic from helpers.rs
                use crate::core::to_snake_case;

                if let Ok(_config) = Config::load() {
                    if let Ok(coordinator) = crate::core::UseCaseCoordinator::load() {
                        if let Some(target_uc) = coordinator
                            .get_all_use_cases()
                            .iter()
                            .find(|uc| uc.id == target_id)
                        {
                            let category_snake = to_snake_case(&target_uc.category);
                            return format!(
                                "{} ([{}](../../{}/{}/README.md))",
                                condition_text.trim(),
                                target_id,
                                category_snake,
                                target_id
                            );
                        }
                    }
                }

                // Fallback: assume same category
                return format!(
                    "{} ([{}](../{}/README.md))",
                    condition_text.trim(),
                    target_id,
                    target_id
                );
            }
        }

        // No link syntax found, return as-is
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::UseCase;
    use serde_json::json;

    #[test]
    fn test_methodology_fields_flattening() {
        // Create a use case with methodology_fields
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(), // category_abbreviation
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        // Add methodology fields
        let mut business_fields = std::collections::HashMap::new();
        business_fields.insert("business_value".to_string(), json!("High impact"));
        business_fields.insert(
            "stakeholders".to_string(),
            json!(["Product Manager", "Developer"]),
        );

        let mut feature_fields = std::collections::HashMap::new();
        feature_fields.insert("acceptance_criteria".to_string(), json!(["Must work"]));

        use_case
            .methodology_fields
            .insert("business".to_string(), business_fields);
        use_case
            .methodology_fields
            .insert("feature".to_string(), feature_fields);

        // Convert to JSON and flatten
        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply flattening logic (same as in generate_with_methodology)
        if let Some(Value::Object(methodology_fields_map)) = data.remove("methodology_fields") {
            for (_methodology_name, fields) in methodology_fields_map {
                if let Value::Object(field_map) = fields {
                    for (field_name, field_value) in field_map {
                        data.entry(field_name).or_insert(field_value);
                    }
                }
            }
        }

        // Verify fields are accessible at top level
        assert!(data.contains_key("business_value"));
        assert_eq!(data["business_value"], json!("High impact"));
        assert!(data.contains_key("stakeholders"));
        assert_eq!(
            data["stakeholders"],
            json!(["Product Manager", "Developer"])
        );
        assert!(data.contains_key("acceptance_criteria"));
        assert_eq!(data["acceptance_criteria"], json!(["Must work"]));
    }

    #[test]
    fn test_standard_fields_take_priority_over_methodology_fields() {
        // Create a use case with both extra and methodology_fields
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(), // category_abbreviation
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        // Add standard extra field
        use_case
            .extra
            .insert("author".to_string(), json!("Standard Author"));

        // Add methodology field with same name
        let mut business_fields = std::collections::HashMap::new();
        business_fields.insert("author".to_string(), json!("Methodology Author"));

        use_case
            .methodology_fields
            .insert("business".to_string(), business_fields);

        // Convert to JSON and flatten
        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply flattening logic (extra first, then methodology_fields)
        if let Some(Value::Object(extra_map)) = data.remove("extra") {
            for (key, value) in extra_map {
                data.insert(key, value);
            }
        }

        if let Some(Value::Object(methodology_fields_map)) = data.remove("methodology_fields") {
            for (_methodology_name, fields) in methodology_fields_map {
                if let Value::Object(field_map) = fields {
                    for (field_name, field_value) in field_map {
                        data.entry(field_name).or_insert(field_value);
                    }
                }
            }
        }

        // Verify standard field takes priority
        assert_eq!(data["author"], json!("Standard Author"));
    }

    #[test]
    fn test_date_formatting_default() {
        use chrono::{TimeZone, Utc};

        // Create a use case with known timestamps
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        // Set specific timestamps
        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 0).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 0).unwrap();

        // Convert to JSON (this triggers RFC3339 serialization)
        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply date formatting logic with default format
        let date_format = "%d/%m/%Y";
        if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("created_date".to_string(), Value::String(formatted.clone()));
                    data.insert("created".to_string(), Value::String(formatted));
                }
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("last_updated".to_string(), Value::String(formatted));
                }
            }
        }

        // Verify formatted dates are present
        assert_eq!(data["created_date"], json!("03/12/2025"));
        assert_eq!(data["created"], json!("03/12/2025"));
        assert_eq!(data["last_updated"], json!("03/12/2025"));
    }

    #[test]
    fn test_date_formatting_us_format() {
        use chrono::{TimeZone, Utc};

        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 0).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 0).unwrap();

        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply date formatting logic with US format
        let date_format = "%m/%d/%Y";
        if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("created_date".to_string(), Value::String(formatted.clone()));
                    data.insert("created".to_string(), Value::String(formatted));
                }
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("last_updated".to_string(), Value::String(formatted));
                }
            }
        }

        // Verify US format dates
        assert_eq!(data["created_date"], json!("12/03/2025"));
        assert_eq!(data["created"], json!("12/03/2025"));
        assert_eq!(data["last_updated"], json!("12/03/2025"));
    }

    #[test]
    fn test_date_formatting_iso_format() {
        use chrono::{TimeZone, Utc};

        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 0).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 0).unwrap();

        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply date formatting logic with ISO format
        let date_format = "%Y-%m-%d";
        if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("created_date".to_string(), Value::String(formatted.clone()));
                    data.insert("created".to_string(), Value::String(formatted));
                }
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("last_updated".to_string(), Value::String(formatted));
                }
            }
        }

        // Verify ISO format dates
        assert_eq!(data["created_date"], json!("2025-12-03"));
        assert_eq!(data["created"], json!("2025-12-03"));
        assert_eq!(data["last_updated"], json!("2025-12-03"));
    }

    #[test]
    fn test_date_formatting_long_format() {
        use chrono::{TimeZone, Utc};

        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 0).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 0).unwrap();

        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply date formatting logic with long format
        let date_format = "%B %d, %Y";
        if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("created_date".to_string(), Value::String(formatted.clone()));
                    data.insert("created".to_string(), Value::String(formatted));
                }
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("last_updated".to_string(), Value::String(formatted));
                }
            }
        }

        // Verify long format dates
        assert_eq!(data["created_date"], json!("December 03, 2025"));
        assert_eq!(data["created"], json!("December 03, 2025"));
        assert_eq!(data["last_updated"], json!("December 03, 2025"));
    }

    #[test]
    fn test_date_formatting_with_time() {
        use chrono::{TimeZone, Utc};

        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 45).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 30).unwrap();

        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Apply date formatting logic with date and time
        let date_format = "%Y-%m-%d %H:%M:%S";
        if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("created_date".to_string(), Value::String(formatted.clone()));
                    data.insert("created".to_string(), Value::String(formatted));
                }
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                    let formatted = dt.format(date_format).to_string();
                    data.insert("last_updated".to_string(), Value::String(formatted));
                }
            }
        }

        // Verify format with time
        assert_eq!(data["created_date"], json!("2025-12-03 10:30:45"));
        assert_eq!(data["created"], json!("2025-12-03 10:30:45"));
        assert_eq!(data["last_updated"], json!("2025-12-03 15:45:30"));
    }

    #[test]
    fn test_date_formatting_handles_serialization() {
        use chrono::{TimeZone, Utc};

        // Test that our date formatting works correctly with the serialization process
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Test description".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        use_case.metadata.created_at = Utc.with_ymd_and_hms(2025, 12, 3, 10, 30, 0).unwrap();
        use_case.metadata.updated_at = Utc.with_ymd_and_hms(2025, 12, 3, 15, 45, 0).unwrap();

        // Serialize to JSON (this produces RFC3339 strings)
        let use_case_json = serde_json::to_value(&use_case).unwrap();
        let data: HashMap<String, Value> = serde_json::from_value(use_case_json).unwrap();

        // Verify that metadata contains RFC3339 formatted strings
        if let Some(Value::Object(metadata)) = data.get("metadata") {
            if let Some(Value::String(created_at)) = metadata.get("created_at") {
                // Should be able to parse as RFC3339
                assert!(chrono::DateTime::parse_from_rfc3339(created_at).is_ok());
            }
            if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
                // Should be able to parse as RFC3339
                assert!(chrono::DateTime::parse_from_rfc3339(updated_at).is_ok());
            }
        }
    }
}
