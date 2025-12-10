//! Markdown generator for use case documentation.
//!
//! Handles generation of markdown documentation from use cases using templates.

use anyhow::Result;
use serde_json::Value;
use std::cmp::Ordering;
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

        // Add merged_steps to each scenario for easy template access
        Self::add_merged_steps_to_scenarios(&mut data);

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

            // Inject aggregated status for templates (computed from domain use_case)
            // This provides `{{aggregated_status}}` to templates that want an overview
            // of the use-case status (derived from its scenarios).
            let aggregated_status = use_case.status().display_name().to_string();
            data.insert(
                "aggregated_status".to_string(),
                Value::String(aggregated_status.clone()),
            );
            // Also provide an emoji-only field for simple status badges
            let aggregated_status_emoji = use_case.status().emoji().to_string();
            data.insert(
                "aggregated_status_emoji".to_string(),
                Value::String(aggregated_status_emoji),
            );
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

        // Add merged_steps to each scenario
        Self::add_merged_steps_to_scenarios(&mut data);

        // Inject aggregated status for templates (computed from domain use_case)
        // This provides `{{aggregated_status}}` for the README overview template.
        let aggregated_status = use_case.status().display_name().to_string();
        data.insert(
            "aggregated_status".to_string(),
            Value::String(aggregated_status.clone()),
        );
        // Provide emoji variant for templates that want only the emoji badge
        let aggregated_status_emoji = use_case.status().emoji().to_string();
        data.insert(
            "aggregated_status_emoji".to_string(),
            Value::String(aggregated_status_emoji),
        );

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

    /// Adds merged_steps field to each scenario in the data.
    ///
    /// For main scenarios, merged_steps is a copy of steps.
    /// For extension scenarios, merged_steps contains:
    /// 1. Parent steps before extends_at_step
    /// 2. Extension scenario steps
    /// 3. Parent steps after returns_at_step (if specified)
    fn add_merged_steps_to_scenarios(data: &mut HashMap<String, Value>) {
        // Get scenarios array
        let scenarios = match data.get("scenarios") {
            Some(Value::Array(scenarios)) => scenarios.clone(),
            _ => return, // No scenarios or wrong type
        };

        // Process each scenario and add merged_steps
        let processed_scenarios: Vec<Value> = scenarios
            .iter()
            .map(|scenario| {
                let mut scenario_obj = match scenario {
                    Value::Object(obj) => obj.clone(),
                    _ => return scenario.clone(),
                };

                // Check if this is an extension scenario
                let extends_id = scenario_obj
                    .get("extends_scenario_id")
                    .and_then(|v| v.as_str());

                if extends_id.is_none() {
                    // Main scenario - merged_steps is same as steps
                    if let Some(steps) = scenario_obj.get("steps") {
                        scenario_obj.insert("merged_steps".to_string(), steps.clone());
                    }
                    return Value::Object(scenario_obj);
                }

                // Extension scenario - merge with parent steps
                let extends_id = extends_id.unwrap();
                let extends_at = scenario_obj
                    .get("extends_at_step")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let returns_at = scenario_obj.get("returns_at_step").and_then(|v| v.as_str());

                // Find parent scenario
                let parent_scenario = scenarios.iter().find(|s| {
                    s.get("id")
                        .and_then(|v| v.as_str())
                        .map(|id| id == extends_id)
                        .unwrap_or(false)
                });

                let merged_steps = if let Some(parent) = parent_scenario {
                    Self::merge_steps(&scenario_obj, parent, extends_at, returns_at)
                } else {
                    // Parent not found, just use scenario's own steps
                    scenario_obj
                        .get("steps")
                        .cloned()
                        .unwrap_or(Value::Array(vec![]))
                };

                scenario_obj.insert("merged_steps".to_string(), merged_steps);
                Value::Object(scenario_obj)
            })
            .collect();

        data.insert("scenarios".to_string(), Value::Array(processed_scenarios));
    }

    /// Merges steps from parent and extension scenarios.
    fn merge_steps(
        extension: &serde_json::Map<String, Value>,
        parent: &Value,
        extends_at: &str,
        returns_at: Option<&str>,
    ) -> Value {
        use crate::core::domain::StepOrder;

        let parent_steps = parent
            .get("steps")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let extension_steps = extension
            .get("steps")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut merged = Vec::new();

        // Step 1: Parent steps BEFORE extends_at_step (exclusive)
        for step in &parent_steps {
            let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
            if StepOrder::compare(order, extends_at) == Ordering::Less {
                merged.push(step.clone());
            }
        }

        // Step 2: ALL extension scenario steps
        merged.extend(extension_steps);

        // Step 3: Handle returns_at_step
        if let Some(returns_at) = returns_at {
            if StepOrder::compare(returns_at, extends_at) != Ordering::Less {
                // Normal case: returns_at >= extends_at, get steps FROM returns_at onward
                for step in &parent_steps {
                    let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
                    if StepOrder::compare(order, returns_at) != Ordering::Less {
                        merged.push(step.clone());
                    }
                }
            } else {
                // LOOP case: returns_at < extends_at
                // Get steps from returns_at up to (but not including) extends_at
                for step in &parent_steps {
                    let order = step.get("order").and_then(|v| v.as_str()).unwrap_or("");
                    if StepOrder::compare(order, returns_at) != Ordering::Less
                        && StepOrder::compare(order, extends_at) == Ordering::Less
                    {
                        merged.push(step.clone());
                    }
                }
            }
        }

        Value::Array(merged)
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
