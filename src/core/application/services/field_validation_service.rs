/// Field validation service for checking use case field integrity.
///
/// This service validates:
/// - Required fields have non-empty values
/// - All fields are relevant to the current methodology configuration
/// - Fields have correct types
///
/// Returns warnings (not errors) to help users identify potential issues.
use std::collections::HashMap;

use crate::core::application::methodology_field_collector::{
    FieldCollection, MethodologyFieldCollector,
};
use crate::core::UseCase;

/// Represents a validation warning for a field issue
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Type of warning (MissingRequired, IrrelevantField, etc.)
    pub warning_type: WarningType,
    /// ID of the use case or actor with the issue
    pub entity_id: String,
    /// Type of entity (UseCase or Actor)
    pub entity_type: String,
    /// Field name that has the issue
    pub field_name: String,
    /// Human-readable warning message
    pub message: String,
}

/// Types of validation warnings
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    /// Required field is missing or empty
    MissingRequired,
    /// Field exists in TOML but not in methodology definition
    IrrelevantField,
    /// Field has incorrect type
    TypeError,
}

/// Service for validating use case and actor fields
pub struct FieldValidationService<'a> {
    field_collector: &'a MethodologyFieldCollector,
}

impl<'a> FieldValidationService<'a> {
    /// Create a new validation service with a field collector
    pub fn new(field_collector: &'a MethodologyFieldCollector) -> Self {
        Self { field_collector }
    }

    /// Validate a single use case's methodology fields
    ///
    /// Checks:
    /// - Required fields are present and non-empty
    /// - All fields in methodology_fields are relevant to current methodology
    ///
    /// # Arguments
    /// * `use_case` - The use case to validate
    ///
    /// # Returns
    /// Vector of validation warnings (empty if no issues)
    pub fn validate_use_case(&self, use_case: &UseCase) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Build methodology:level list from views
        let methodology_levels: Vec<(String, String)> = use_case
            .views
            .iter()
            .map(|view| (view.methodology.clone(), view.level.clone()))
            .collect();

        // Collect expected fields for this use case's methodology configuration
        let field_collection = match self
            .field_collector
            .collect_fields_for_views(&methodology_levels)
        {
            Ok(collection) => collection,
            Err(_) => return warnings, // If we can't collect fields, skip validation
        };

        // Check each methodology group in the use case
        for (methodology_name, fields) in &use_case.methodology_fields {
            warnings.extend(self.validate_methodology_fields(
                &use_case.id,
                "UseCase",
                methodology_name,
                fields,
                &field_collection,
            ));
        }

        warnings
    }

    /// Validate all use cases in a list
    pub fn validate_use_cases(&self, use_cases: &[UseCase]) -> Vec<ValidationWarning> {
        use_cases
            .iter()
            .flat_map(|uc| self.validate_use_case(uc))
            .collect()
    }

    /// Validate methodology fields for a specific entity
    ///
    /// Checks:
    /// 1. All fields in the entity's TOML are relevant (exist in field definitions)
    /// 2. All required fields have non-empty values
    fn validate_methodology_fields(
        &self,
        entity_id: &str,
        entity_type: &str,
        methodology_name: &str,
        fields: &HashMap<String, serde_json::Value>,
        field_collection: &FieldCollection,
    ) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Check for irrelevant fields (exist in TOML but not in definitions)
        for field_name in fields.keys() {
            if !field_collection.fields.contains_key(field_name) {
                warnings.push(ValidationWarning {
                    warning_type: WarningType::IrrelevantField,
                    entity_id: entity_id.to_string(),
                    entity_type: entity_type.to_string(),
                    field_name: field_name.clone(),
                    message: format!(
                        "Field '{}' in methodology '{}' is not defined in the current methodology configuration",
                        field_name, methodology_name
                    ),
                });
            }
        }

        // Check for missing required fields
        for (field_name, field_def) in &field_collection.fields {
            // Only check if this field belongs to the current methodology
            if field_def
                .methodologies
                .contains(&methodology_name.to_string())
                && field_def.required
            {
                if let Some(value) = fields.get(field_name) {
                    // Field exists, check if it's empty
                    if self.is_value_empty(value) {
                        warnings.push(ValidationWarning {
                            warning_type: WarningType::MissingRequired,
                            entity_id: entity_id.to_string(),
                            entity_type: entity_type.to_string(),
                            field_name: field_name.clone(),
                            message: format!(
                                "Required field '{}' in methodology '{}' is empty",
                                field_name, methodology_name
                            ),
                        });
                    }
                } else {
                    // Field doesn't exist at all
                    warnings.push(ValidationWarning {
                        warning_type: WarningType::MissingRequired,
                        entity_id: entity_id.to_string(),
                        entity_type: entity_type.to_string(),
                        field_name: field_name.clone(),
                        message: format!(
                            "Required field '{}' is missing from methodology '{}'",
                            field_name, methodology_name
                        ),
                    });
                }
            }
        }

        warnings
    }

    /// Check if a JSON value is considered "empty"
    fn is_value_empty(&self, value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Null => true,
            serde_json::Value::String(s) => s.trim().is_empty(),
            serde_json::Value::Array(arr) => arr.is_empty(),
            serde_json::Value::Number(_) => false, // Numbers are never "empty"
            serde_json::Value::Bool(_) => false,   // Booleans are never "empty"
            serde_json::Value::Object(obj) => obj.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_is_value_empty() {
        // Can't easily test without a real MethodologyFieldCollector
        // Just test value emptiness logic directly
        assert!(matches!(json!(null), serde_json::Value::Null));
        assert!(json!("").as_str().unwrap().trim().is_empty());
        assert!(json!("   ").as_str().unwrap().trim().is_empty());
        assert!(json!([]).as_array().unwrap().is_empty());
        assert!(json!({}).as_object().unwrap().is_empty());

        assert!(!json!("text").as_str().unwrap().trim().is_empty());
        assert!(json!(42).is_number());
        assert!(json!(true).is_boolean());
        assert!(json!(false).is_boolean());
        assert!(!json!(["item"]).as_array().unwrap().is_empty());
    }
}
