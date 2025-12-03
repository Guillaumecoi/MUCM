/// Collects and merges methodology-specific custom fields from multiple methodologies
///
/// This module handles:
/// - Field collection from multiple methodology:level pairs
/// - Field inheritance (simple → normal → detailed)
/// - Collision detection between methodologies
/// - Conflict warnings for standard field overlaps
/// - Validation of duplicate fields within same methodology inheritance chain
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::config::Config;
use crate::core::{CustomFieldConfig, FieldResolver, MethodologyDefinition};

/// Represents a field collected from one or more methodologies
#[derive(Debug, Clone)]
pub struct CollectedField {
    /// Field name
    pub name: String,
    /// Field type (string, array, number, boolean)
    pub field_type: String,
    /// Display label for prompts
    pub label: String,
    /// Whether field is required
    pub required: bool,
    /// Default value if any
    pub default: Option<String>,
    /// Description/help text
    pub description: Option<String>,
    /// Example value for guidance
    pub example: Option<String>,
    /// Which methodologies define this field
    pub methodologies: Vec<String>,
    /// Which level this field comes from (simple, normal, detailed)
    pub level: String,
}

/// Collection of fields from multiple methodologies
#[derive(Debug, Default)]
pub struct FieldCollection {
    /// All collected fields keyed by field name
    pub fields: HashMap<String, CollectedField>,
    /// Warnings generated during collection (e.g., standard field conflicts)
    pub warnings: Vec<String>,
}

/// Collects and merges methodology-specific custom fields
pub struct MethodologyFieldCollector {
    templates_dir: String,
}

impl MethodologyFieldCollector {
    /// Create a new field collector
    pub fn new() -> Result<Self> {
        let templates_dir = format!(".config/.mucm/{}/methodologies", Config::TEMPLATES_DIR);
        Ok(Self { templates_dir })
    }

    /// Collect all custom fields for given methodology views
    ///
    /// # Arguments
    /// * `views` - List of (methodology, level) pairs (e.g., [("business", "detailed"), ("feature", "simple")])
    ///
    /// # Returns
    /// FieldCollection with merged fields and any warnings
    ///
    /// # Errors
    /// - If methodology not found
    /// - If field collision detected between different methodologies
    /// - If duplicate field within same methodology inheritance chain
    pub fn collect_fields_for_views(&self, views: &[(String, String)]) -> Result<FieldCollection> {
        let mut collection = FieldCollection::default();

        // Standard extra field names (these have priority over methodology fields)
        let standard_fields = ["author", "reviewer", "description"];

        // Collect fields from each view
        for (methodology, level) in views {
            let methodology_fields = self.collect_fields_for_methodology(methodology, level)?;

            for (field_name, field_config) in methodology_fields {
                // Check if this conflicts with a standard field
                if standard_fields.contains(&field_name.as_str()) {
                    collection.warnings.push(format!(
                        "⚠️  Methodology '{}' defines field '{}' which conflicts with standard field. Using standard field.",
                        methodology, field_name
                    ));
                    continue; // Skip methodology field, use standard
                }

                // Check if field already exists from another methodology (collision)
                if let Some(existing_field) = collection.fields.get(&field_name) {
                    if !existing_field.methodologies.contains(methodology) {
                        // This is a collision between different methodologies
                        return Err(anyhow::anyhow!(
                            "Field name collision detected!\n\n\
                             Field '{}' is defined by multiple methodologies:\n\
                             - {} ({}): \"{}\"\n\
                             - {} ({}): \"{}\"\n\n\
                             Please either:\n\
                             1. Remove one of these methodologies from your use case views\n\
                             2. Contact the methodology maintainer to rename conflicting fields",
                            field_name,
                            existing_field.methodologies.join(", "),
                            existing_field.field_type,
                            existing_field.label,
                            methodology,
                            field_config.field_type,
                            field_config.label.as_deref().unwrap_or(&field_name)
                        ));
                    }
                } else {
                    // New field - add it
                    collection.fields.insert(
                        field_name.clone(),
                        CollectedField {
                            name: field_name.clone(),
                            field_type: field_config.field_type.clone(),
                            label: field_config
                                .label
                                .clone()
                                .unwrap_or_else(|| field_name.clone()),
                            example: field_config.example.clone(),
                            required: field_config.required,
                            default: field_config.default.clone(),
                            description: field_config.description.clone(),
                            methodologies: vec![methodology.clone()],
                            level: level.clone(),
                        },
                    );
                }
            }
        }

        Ok(collection)
    }

    /// Collect fields for a single methodology:level pair with inheritance
    ///
    /// # Arguments
    /// * `methodology` - Methodology name (e.g., "business")
    /// * `level` - Documentation level (e.g., "detailed")
    ///
    /// # Returns
    /// HashMap of field_name -> CustomFieldConfig
    ///
    /// Includes inherited fields using FieldResolver for proper inheritance chain resolution
    fn collect_fields_for_methodology(
        &self,
        methodology: &str,
        level: &str,
    ) -> Result<HashMap<String, CustomFieldConfig>> {
        let methodology_dir = Path::new(&self.templates_dir).join(methodology);

        if !methodology_dir.exists() || !methodology_dir.join("methodology.toml").exists() {
            return Err(anyhow::anyhow!(
                "Methodology '{}' not found in {}",
                methodology,
                self.templates_dir
            ));
        }

        let methodology_def = MethodologyDefinition::from_toml(&methodology_dir)
            .context(format!("Failed to load methodology '{}'", methodology))?;

        // Map backward compatibility aliases to actual level names
        let level_name = match level.to_lowercase().as_str() {
            "simple" | "s" => "Normal",     // Backward compatibility
            "detailed" | "d" => "Advanced", // Backward compatibility
            "normal" | "n" => "Normal",
            "advanced" | "a" => "Advanced",
            _ => level, // Use as-is for custom levels
        };

        // Use FieldResolver for proper inheritance handling
        let resolver = FieldResolver::new(&methodology_def);
        let all_fields = resolver
            .resolve_fields_for_level(level_name)
            .context(format!(
                "Failed to resolve fields for level '{}' in methodology '{}'",
                level_name, methodology
            ))?;

        Ok(all_fields)
    }

    /// Apply user-provided values to field collection, converting types appropriately
    ///
    /// # Arguments
    /// * `field_collection` - The collected fields with metadata
    /// * `user_values` - User-provided values as strings (from CLI prompts)
    ///
    /// # Returns
    /// HashMap ready to be inserted into UseCase.methodology_fields
    pub fn apply_user_values(
        &self,
        field_collection: &FieldCollection,
        user_values: HashMap<String, String>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();

        // First, add all fields from the collection with appropriate defaults
        // This ensures ALL fields are present in the TOML, even if empty
        for (field_name, field) in &field_collection.fields {
            // Check if user provided a value
            if let Some(user_value) = user_values.get(field_name) {
                // User provided value - convert and use it
                let json_value = self.convert_to_json_type(user_value, &field.field_type);
                result.insert(field_name.clone(), json_value);
            } else if let Some(default) = &field.default {
                // No user value but has default - use default
                let json_value = self.convert_to_json_type(default, &field.field_type);
                result.insert(field_name.clone(), json_value);
            } else {
                // No user value and no default - use empty value based on type
                let json_value = match field.field_type.as_str() {
                    "array" => serde_json::Value::Array(vec![]),
                    "number" => serde_json::Value::Number(serde_json::Number::from(0)),
                    "boolean" => serde_json::Value::Bool(false),
                    _ => serde_json::Value::String(String::new()),
                };
                result.insert(field_name.clone(), json_value);
            }
        }

        result
    }

    /// Convert string value to appropriate JSON type
    ///
    /// # Array Parsing Support
    /// Arrays can be specified in three formats (tried in order):
    /// 1. **JSON array** (preferred): `["item1", "item2", "item with, comma"]`
    ///    - Eliminates all parsing ambiguity
    ///    - Allows commas, newlines, and special characters within items
    /// 2. **Newline-separated**: `"item1\nitem2\nitem3"`
    ///    - Each line is one item
    ///    - Allows commas within items
    /// 3. **Comma-separated** (legacy): `"item1, item2, item3"`
    ///    - For backward compatibility
    ///    - Cannot handle commas within items
    fn convert_to_json_type(&self, value: &str, field_type: &str) -> serde_json::Value {
        match field_type {
            "array" => {
                // Try JSON array first (e.g., ["item1", "item2", "item with, comma"])
                // This format allows any characters within items including commas and newlines
                if let Ok(json_value) = serde_json::from_str::<Vec<String>>(value) {
                    return serde_json::Value::Array(
                        json_value.into_iter().map(serde_json::Value::String).collect()
                    );
                }
                
                // Fall back to newline-separated (allows commas within items)
                // or comma-separated (legacy, for backward compatibility)
                let items: Vec<String> = if value.contains('\n') {
                    value.lines().map(|s| s.trim().to_string()).collect()
                } else {
                    value.split(',').map(|s| s.trim().to_string()).collect()
                };
                let filtered: Vec<String> = items.into_iter().filter(|s| !s.is_empty()).collect();
                serde_json::Value::Array(filtered.into_iter().map(serde_json::Value::String).collect())
            }
            "number" => {
                // Try to parse as number
                if let Ok(num) = value.parse::<i64>() {
                    serde_json::Value::Number(serde_json::Number::from(num))
                } else if let Ok(num) = value.parse::<f64>() {
                    serde_json::json!(num)
                } else {
                    serde_json::Value::String(value.to_string())
                }
            }
            "boolean" => {
                let val = matches!(value.to_lowercase().as_str(), "true" | "yes" | "1");
                serde_json::Value::Bool(val)
            }
            _ => serde_json::Value::String(value.to_string()),
        }
    }
}

impl Default for MethodologyFieldCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create MethodologyFieldCollector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require actual methodology config files to exist
    // They will be integration tests rather than pure unit tests

    #[test]
    fn test_convert_to_json_type_string() {
        let collector = MethodologyFieldCollector::default();
        let value = collector.convert_to_json_type("hello", "string");
        assert_eq!(value, serde_json::Value::String("hello".to_string()));
    }

    #[test]
    fn test_convert_to_json_type_array() {
        let collector = MethodologyFieldCollector::default();
        let value = collector.convert_to_json_type("item1, item2, item3", "array");
        assert_eq!(value, serde_json::json!(["item1", "item2", "item3"]));
    }

    #[test]
    fn test_convert_to_json_type_number() {
        let collector = MethodologyFieldCollector::default();
        let value = collector.convert_to_json_type("42", "number");
        assert_eq!(
            value,
            serde_json::Value::Number(serde_json::Number::from(42))
        );
    }

    #[test]
    fn test_convert_to_json_type_boolean() {
        let collector = MethodologyFieldCollector::default();
        let value = collector.convert_to_json_type("true", "boolean");
        assert_eq!(value, serde_json::Value::Bool(true));

        let value = collector.convert_to_json_type("false", "boolean");
        assert_eq!(value, serde_json::Value::Bool(false));
    }

    #[test]
    fn test_apply_user_values_all_fields_present_when_empty() {
        let collector = MethodologyFieldCollector::default();

        // Create a field collection with multiple field types
        let mut field_collection = FieldCollection::default();

        // Required string field
        field_collection.fields.insert(
            "required_field".to_string(),
            CollectedField {
                name: "required_field".to_string(),
                field_type: "string".to_string(),
                label: "Required Field".to_string(),
                required: true,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Optional string field
        field_collection.fields.insert(
            "optional_string".to_string(),
            CollectedField {
                name: "optional_string".to_string(),
                field_type: "string".to_string(),
                label: "Optional String".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Optional array field
        field_collection.fields.insert(
            "optional_array".to_string(),
            CollectedField {
                name: "optional_array".to_string(),
                field_type: "array".to_string(),
                label: "Optional Array".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Optional number field
        field_collection.fields.insert(
            "optional_number".to_string(),
            CollectedField {
                name: "optional_number".to_string(),
                field_type: "number".to_string(),
                label: "Optional Number".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Optional boolean field
        field_collection.fields.insert(
            "optional_boolean".to_string(),
            CollectedField {
                name: "optional_boolean".to_string(),
                field_type: "boolean".to_string(),
                label: "Optional Boolean".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Apply with empty user_values (simulating user skipping all optional fields)
        let user_values = HashMap::new();
        let result = collector.apply_user_values(&field_collection, user_values);

        // ALL fields should be present, even though no values were provided
        assert_eq!(
            result.len(),
            5,
            "All 5 fields should be present in the result"
        );

        // Verify each field has appropriate empty/default value
        assert_eq!(
            result.get("required_field"),
            Some(&serde_json::Value::String(String::new())),
            "Required field should have empty string"
        );
        assert_eq!(
            result.get("optional_string"),
            Some(&serde_json::Value::String(String::new())),
            "Optional string should have empty string"
        );
        assert_eq!(
            result.get("optional_array"),
            Some(&serde_json::Value::Array(vec![])),
            "Optional array should have empty array"
        );
        assert_eq!(
            result.get("optional_number"),
            Some(&serde_json::Value::Number(serde_json::Number::from(0))),
            "Optional number should have 0"
        );
        assert_eq!(
            result.get("optional_boolean"),
            Some(&serde_json::Value::Bool(false)),
            "Optional boolean should have false"
        );
    }

    #[test]
    fn test_apply_user_values_with_defaults() {
        let collector = MethodologyFieldCollector::default();

        let mut field_collection = FieldCollection::default();

        // Field with default value
        field_collection.fields.insert(
            "field_with_default".to_string(),
            CollectedField {
                name: "field_with_default".to_string(),
                field_type: "string".to_string(),
                label: "Field With Default".to_string(),
                required: false,
                default: Some("default value".to_string()),
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Apply without user value - should use default
        let user_values = HashMap::new();
        let result = collector.apply_user_values(&field_collection, user_values);

        assert_eq!(
            result.get("field_with_default"),
            Some(&serde_json::Value::String("default value".to_string())),
            "Field should use default value when no user value provided"
        );
    }

    #[test]
    fn test_apply_user_values_user_overrides_default() {
        let collector = MethodologyFieldCollector::default();

        let mut field_collection = FieldCollection::default();

        // Field with default value
        field_collection.fields.insert(
            "field_with_default".to_string(),
            CollectedField {
                name: "field_with_default".to_string(),
                field_type: "string".to_string(),
                label: "Field With Default".to_string(),
                required: false,
                default: Some("default value".to_string()),
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Apply with user value - should override default
        let mut user_values = HashMap::new();
        user_values.insert("field_with_default".to_string(), "user value".to_string());
        let result = collector.apply_user_values(&field_collection, user_values);

        assert_eq!(
            result.get("field_with_default"),
            Some(&serde_json::Value::String("user value".to_string())),
            "User value should override default"
        );
    }

    #[test]
    fn test_apply_user_values_mixed_filled_and_empty() {
        let collector = MethodologyFieldCollector::default();

        let mut field_collection = FieldCollection::default();

        // Add three fields
        field_collection.fields.insert(
            "filled_field".to_string(),
            CollectedField {
                name: "filled_field".to_string(),
                field_type: "string".to_string(),
                label: "Filled Field".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        field_collection.fields.insert(
            "empty_field".to_string(),
            CollectedField {
                name: "empty_field".to_string(),
                field_type: "string".to_string(),
                label: "Empty Field".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        field_collection.fields.insert(
            "array_field".to_string(),
            CollectedField {
                name: "array_field".to_string(),
                field_type: "array".to_string(),
                label: "Array Field".to_string(),
                required: false,
                default: None,
                description: None,
                example: None,
                methodologies: vec!["test".to_string()],
                level: "normal".to_string(),
            },
        );

        // Only fill one field
        let mut user_values = HashMap::new();
        user_values.insert("filled_field".to_string(), "some value".to_string());

        let result = collector.apply_user_values(&field_collection, user_values);

        // All three fields should be present
        assert_eq!(result.len(), 3, "All fields should be present");

        // Check values
        assert_eq!(
            result.get("filled_field"),
            Some(&serde_json::Value::String("some value".to_string())),
            "Filled field should have user value"
        );
        assert_eq!(
            result.get("empty_field"),
            Some(&serde_json::Value::String(String::new())),
            "Empty field should have empty string"
        );
        assert_eq!(
            result.get("array_field"),
            Some(&serde_json::Value::Array(vec![])),
            "Empty array field should have empty array"
        );
    }
}
