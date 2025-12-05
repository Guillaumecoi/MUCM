use super::{Condition, Metadata, MethodologyView, Scenario, Status, UseCaseReference};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "LOW"),
            Priority::Medium => write!(f, "MEDIUM"),
            Priority::High => write!(f, "HIGH"),
            Priority::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCase {
    pub id: String,
    pub title: String,
    pub category: String,
    // NEW: Category abbreviation used in ID generation (e.g., "AUT" for "Authentication")
    pub category_abbreviation: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: Metadata,

    // NEW: Multi-view support - defines which methodology/level combinations are active
    // Each view generates a separate markdown file (e.g., UC-001-feat-s.md, UC-001-bus-n.md)
    // Empty vec means single-view mode (backward compatible)
    #[serde(default)]
    pub views: Vec<MethodologyView>,

    // NEW: Preconditions - what must be true before executing (can optionally reference use cases/scenarios)
    #[serde(default)]
    pub preconditions: Vec<Condition>,

    // NEW: Postconditions - what will be true after executing (can optionally reference use cases/scenarios)
    #[serde(default)]
    pub postconditions: Vec<Condition>,

    // NEW: References to other use cases
    #[serde(default)]
    pub use_case_references: Vec<UseCaseReference>,

    // NEW: Scenarios for this use case
    #[serde(default)]
    pub scenarios: Vec<Scenario>,

    // Methodology-specific custom fields, grouped by methodology name
    // Example: { "business": { "business_value": "...", "stakeholders": [...] } }
    // This allows clean separation and easy add/remove of methodology fields
    pub methodology_fields:
        std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,

    // Catch-all for standard extra fields (author, reviewer, etc.)
    // Note: methodology-specific fields go in methodology_fields above
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl UseCase {
    /// Create a use case with explicit category abbreviation
    pub fn new(
        id: String,
        title: String,
        category: String,
        category_abbreviation: String,
        description: String,
        priority: String,
    ) -> Result<Self, String> {
        let priority = Priority::from_str(&priority)?;
        Ok(Self {
            id,
            title,
            category,
            category_abbreviation,
            description,
            priority,
            metadata: Metadata::new(),
            views: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            use_case_references: Vec::new(),
            scenarios: Vec::new(),
            methodology_fields: std::collections::HashMap::new(),
            extra: std::collections::HashMap::new(),
        })
    }

    pub fn status(&self) -> Status {
        if self.scenarios.is_empty() {
            return Status::Planned;
        }

        self.scenarios
            .iter()
            .map(|s| s.status)
            .min() // Status implements Ord
            .unwrap_or(Status::Planned)
    }

    /// Add a precondition to this use case
    pub fn add_precondition(&mut self, condition: Condition) {
        // Check for duplicates based on text and target
        if !self.preconditions.iter().any(|c| {
            c.text == condition.text
                && c.target_id == condition.target_id
                && c.target_type == condition.target_type
        }) {
            self.preconditions.push(condition);
            self.metadata.touch();
        }
    }

    /// Add a postcondition to this use case
    pub fn add_postcondition(&mut self, condition: Condition) {
        // Check for duplicates based on text and target
        if !self.postconditions.iter().any(|c| {
            c.text == condition.text
                && c.target_id == condition.target_id
                && c.target_type == condition.target_type
        }) {
            self.postconditions.push(condition);
            self.metadata.touch();
        }
    }

    /// Remove a precondition by text
    pub fn remove_precondition(&mut self, text: &str) {
        self.preconditions.retain(|c| c.text != text);
        self.metadata.touch();
    }

    /// Remove a postcondition by text
    pub fn remove_postcondition(&mut self, text: &str) {
        self.postconditions.retain(|c| c.text != text);
        self.metadata.touch();
    }

    /// Add a reference to another use case
    pub fn add_reference(&mut self, reference: UseCaseReference) {
        // Prevent duplicate references
        if !self
            .use_case_references
            .iter()
            .any(|r| r.target_id == reference.target_id && r.relationship == reference.relationship)
        {
            self.use_case_references.push(reference);
            self.metadata.touch();
        }
    }

    /// Get next scenario ID for this use case
    pub fn next_scenario_id(&self) -> String {
        let next_num = self.scenarios.len() + 1;
        format!("{}-S{:02}", self.id, next_num)
    }

    /// Add a scenario
    pub fn add_scenario(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
        self.metadata.touch();
    }

    /// Add a step to a specific scenario
    pub fn add_step_to_scenario(
        &mut self,
        scenario_id: &str,
        step: super::ScenarioStep,
    ) -> anyhow::Result<()> {
        if let Some(scenario) = self.scenarios.iter_mut().find(|s| s.id == scenario_id) {
            scenario.add_step(step);
            self.metadata.touch();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Scenario with ID '{}' not found",
                scenario_id
            ))
        }
    }

    /// Update the status of a specific scenario
    pub fn update_scenario_status(
        &mut self,
        scenario_id: &str,
        new_status: Status,
    ) -> anyhow::Result<()> {
        if let Some(scenario) = self.scenarios.iter_mut().find(|s| s.id == scenario_id) {
            scenario.set_status(new_status);
            self.metadata.touch();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Scenario with ID '{}' not found",
                scenario_id
            ))
        }
    }

    /// Remove a step from a specific scenario
    pub fn remove_step_from_scenario(
        &mut self,
        scenario_id: &str,
        step_order: &str,
    ) -> anyhow::Result<()> {
        if let Some(scenario) = self.scenarios.iter_mut().find(|s| s.id == scenario_id) {
            scenario.remove_step(step_order);
            self.metadata.touch();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Scenario with ID '{}' not found",
                scenario_id
            ))
        }
    }

    /// Add a methodology view (methodology/level combination)
    pub fn add_view(&mut self, view: MethodologyView) {
        // Prevent duplicate views
        if !self.views.iter().any(|v| v.key() == view.key()) {
            self.views.push(view);
            self.metadata.touch();
        }
    }

    /// Remove a methodology view by key (methodology-level)
    pub fn remove_view(&mut self, methodology: &str, level: &str) -> bool {
        let initial_len = self.views.len();
        self.views
            .retain(|v| v.methodology != methodology || v.level != level);
        let removed = self.views.len() != initial_len;
        if removed {
            self.metadata.touch();
        }
        removed
    }

    /// Enable or disable a specific view
    pub fn set_view_enabled(&mut self, methodology: &str, level: &str, enabled: bool) -> bool {
        if let Some(view) = self
            .views
            .iter_mut()
            .find(|v| v.methodology == methodology && v.level == level)
        {
            if view.enabled != enabled {
                view.enabled = enabled;
                self.metadata.touch();
            }
            true
        } else {
            false
        }
    }

    /// Get all enabled views
    ///
    /// Note: Every use case must have at least one enabled view.
    pub fn enabled_views(&self) -> impl Iterator<Item = &MethodologyView> {
        self.views.iter().filter(|v| v.enabled)
    }

    /// Get all views (enabled and disabled)
    pub fn views(&self) -> &[MethodologyView] {
        &self.views
    }

    /// Get the primary (first enabled) view for this use case
    ///
    /// Returns the first enabled view, or None if all views are disabled
    /// (which should not happen in normal operation).
    pub fn primary_view(&self) -> Option<&MethodologyView> {
        self.enabled_views().next()
    }
}

#[cfg(test)]
mod priority_tests {
    use super::*;

    /// Test Priority enum variants exist and have correct debug representation
    #[test]
    fn test_priority_enum_variants() {
        let priority = Priority::Low;
        assert_eq!(format!("{:?}", priority), "Low");

        let priority = Priority::Medium;
        assert_eq!(format!("{:?}", priority), "Medium");

        let priority = Priority::High;
        assert_eq!(format!("{:?}", priority), "High");

        let priority = Priority::Critical;
        assert_eq!(format!("{:?}", priority), "Critical");
    }

    /// Test Priority Display trait implementation
    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::Low.to_string(), "LOW");
        assert_eq!(Priority::Medium.to_string(), "MEDIUM");
        assert_eq!(Priority::High.to_string(), "HIGH");
        assert_eq!(Priority::Critical.to_string(), "CRITICAL");
    }

    /// Test Priority FromStr trait implementation with valid inputs
    #[test]
    fn test_priority_from_str_valid() {
        assert_eq!(Priority::from_str("low").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("LOW").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("Low").unwrap(), Priority::Low);

        assert_eq!(Priority::from_str("medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("MEDIUM").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("Medium").unwrap(), Priority::Medium);

        assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
        assert_eq!(Priority::from_str("HIGH").unwrap(), Priority::High);
        assert_eq!(Priority::from_str("High").unwrap(), Priority::High);

        assert_eq!(Priority::from_str("critical").unwrap(), Priority::Critical);
        assert_eq!(Priority::from_str("CRITICAL").unwrap(), Priority::Critical);
        assert_eq!(Priority::from_str("Critical").unwrap(), Priority::Critical);
    }

    /// Test Priority FromStr trait implementation with invalid inputs
    #[test]
    fn test_priority_from_str_invalid() {
        assert!(Priority::from_str("invalid").is_err());
        assert!(Priority::from_str("urgent").is_err());
        assert!(Priority::from_str("").is_err());
        assert!(Priority::from_str("123").is_err());

        let error = Priority::from_str("invalid").unwrap_err();
        assert!(error.contains("Invalid priority: invalid"));
    }

    /// Test Priority equality and PartialEq
    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::Low, Priority::Low);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Critical, Priority::Critical);

        assert_ne!(Priority::Low, Priority::Medium);
        assert_ne!(Priority::Medium, Priority::High);
        assert_ne!(Priority::High, Priority::Critical);
    }

    /// Test Priority clone functionality
    #[test]
    fn test_priority_clone() {
        let priority = Priority::High;
        let cloned = priority.clone();
        assert_eq!(priority, cloned);
    }

    /// Test Priority serialization and deserialization
    #[test]
    fn test_priority_serialization() {
        let priority = Priority::Critical;
        let serialized = serde_json::to_string(&priority).expect("Failed to serialize");
        let deserialized: Priority =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(priority, deserialized);

        // Test all variants
        let priorities = vec![
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Critical,
        ];
        for original in priorities {
            let serialized = serde_json::to_string(&original).expect("Failed to serialize");
            let deserialized: Priority =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
            assert_eq!(original, deserialized);
        }
    }

    /// Test Priority in collections (Hash trait)
    #[test]
    fn test_priority_in_collections() {
        use std::collections::HashMap;

        let mut priority_map = HashMap::new();
        priority_map.insert(Priority::Low, "Low priority tasks");
        priority_map.insert(Priority::Medium, "Medium priority tasks");
        priority_map.insert(Priority::High, "High priority tasks");
        priority_map.insert(Priority::Critical, "Critical priority tasks");

        assert_eq!(
            priority_map.get(&Priority::Low),
            Some(&"Low priority tasks")
        );
        assert_eq!(
            priority_map.get(&Priority::Critical),
            Some(&"Critical priority tasks")
        );
        assert_eq!(priority_map.len(), 4);
    }

    /// Test Priority in vector operations
    #[test]
    fn test_priority_in_vectors() {
        let priorities = [
            Priority::Medium,
            Priority::High,
            Priority::Low,
            Priority::Critical,
        ];

        assert!(priorities.contains(&Priority::Medium));
        assert!(priorities.contains(&Priority::Critical));
        assert!(priorities.contains(&Priority::Low));

        let high_priority_count = priorities.iter().filter(|&p| *p == Priority::High).count();
        assert_eq!(high_priority_count, 1);
    }

    /// Test Priority case insensitive parsing edge cases
    #[test]
    fn test_priority_case_insensitive_edge_cases() {
        assert_eq!(Priority::from_str("lOw").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("mEdIuM").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("HiGh").unwrap(), Priority::High);
        assert_eq!(Priority::from_str("cRiTiCaL").unwrap(), Priority::Critical);
    }
}

#[cfg(test)]
mod use_case_tests {
    use super::*;
    use crate::core::domain::entities::{Scenario, ScenarioType};
    use serde_json::json;

    /// Test UseCase::new with valid priority strings
    #[test]
    fn test_use_case_new_valid_priorities() {
        let use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "low".to_string(),
        )
        .unwrap();

        assert_eq!(use_case.id, "UC-TEST-001");
        assert_eq!(use_case.title, "Test Use Case");
        assert_eq!(use_case.category, "Test");
        assert_eq!(use_case.description, "A test use case");
        assert_eq!(use_case.priority, Priority::Low);
        assert!(use_case.extra.is_empty());

        // Test all valid priorities
        let priorities = vec![
            ("medium", Priority::Medium),
            ("HIGH", Priority::High),
            ("Critical", Priority::Critical),
        ];

        for (priority_str, expected_priority) in priorities {
            let use_case = UseCase::new(
                "UC-TEST-002".to_string(),
                "Test Use Case".to_string(),
                "Test".to_string(),
                "TES".to_string(),
                "A test use case".to_string(),
                priority_str.to_string(),
            )
            .unwrap();
            assert_eq!(use_case.priority, expected_priority);
            assert!(use_case.extra.is_empty());
        }
    }

    /// Test UseCase::new with invalid priority strings
    #[test]
    fn test_use_case_new_invalid_priorities() {
        let invalid_priorities = vec!["invalid", "urgent", "", "123", "normal", "extreme"];

        for invalid_priority in invalid_priorities {
            let result = UseCase::new(
                "UC-TEST-001".to_string(),
                "Test Use Case".to_string(),
                "Test".to_string(),
                "TES".to_string(),
                "A test use case".to_string(),
                invalid_priority.to_string(),
            );

            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.contains("Invalid priority"));
            assert!(error.contains(invalid_priority));
        }
    }

    /// Test UseCase extra field initialization
    #[test]
    fn test_use_case_extra_field_initialization() {
        let use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        assert!(use_case.extra.is_empty());
    }

    /// Test UseCase serialization with extra fields
    #[test]
    fn test_use_case_serialization_with_extra_fields() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "high".to_string(),
        )
        .unwrap();

        // Add extra fields
        use_case
            .extra
            .insert("business_value".to_string(), json!("High impact"));
        use_case.extra.insert(
            "acceptance_criteria".to_string(),
            json!(["Must work", "Must be fast"]),
        );
        use_case
            .extra
            .insert("prerequisites".to_string(), json!("None"));
        use_case
            .extra
            .insert("estimated_effort".to_string(), json!(5));

        // Serialize to JSON
        let serialized = serde_json::to_string(&use_case).expect("Failed to serialize");

        // Deserialize back
        let deserialized: UseCase =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify all fields
        assert_eq!(deserialized.id, "UC-TEST-001");
        assert_eq!(deserialized.title, "Test Use Case");
        assert_eq!(deserialized.category, "Test");
        assert_eq!(deserialized.description, "A test use case");
        assert_eq!(deserialized.priority, Priority::High);

        // Verify extra fields
        assert_eq!(deserialized.extra.len(), 4);
        assert_eq!(deserialized.extra["business_value"], json!("High impact"));
        assert_eq!(
            deserialized.extra["acceptance_criteria"],
            json!(["Must work", "Must be fast"])
        );
        assert_eq!(deserialized.extra["prerequisites"], json!("None"));
        assert_eq!(deserialized.extra["estimated_effort"], json!(5));
    }

    /// Test UseCase deserialization from TOML-like structure with extra fields
    #[test]
    fn test_use_case_deserialization_from_toml_like() {
        // Simulate TOML deserialization with extra fields
        let toml_like_json = json!({
            "id": "UC-TEST-001",
            "title": "Test Use Case",
            "category": "Test",
            "category_abbreviation": "TES",
            "description": "A test use case",
            "priority": "Critical",
            "metadata": {
                "created_at": "2023-01-01T00:00:00Z",
                "updated_at": "2023-01-01T00:00:00Z",
                "version": 1
            },
            "methodology_fields": {},
            "business_value": "High impact",
            "acceptance_criteria": ["Must work", "Must be fast", "Must be secure"],
            "prerequisites": "User authentication",
            "estimated_hours": 8,
            "stakeholders": ["Product Manager", "Developer", "QA"]
        });

        let use_case: UseCase =
            serde_json::from_value(toml_like_json).expect("Failed to deserialize");

        // Verify standard fields
        assert_eq!(use_case.id, "UC-TEST-001");
        assert_eq!(use_case.title, "Test Use Case");
        assert_eq!(use_case.category, "Test");
        assert_eq!(use_case.description, "A test use case");
        assert_eq!(use_case.priority, Priority::Critical);

        // Verify extra fields are captured
        assert_eq!(use_case.extra.len(), 5);
        assert_eq!(use_case.extra["business_value"], json!("High impact"));
        assert_eq!(
            use_case.extra["acceptance_criteria"],
            json!(["Must work", "Must be fast", "Must be secure"])
        );
        assert_eq!(
            use_case.extra["prerequisites"],
            json!("User authentication")
        );
        assert_eq!(use_case.extra["estimated_hours"], json!(8));
        assert_eq!(
            use_case.extra["stakeholders"],
            json!(["Product Manager", "Developer", "QA"])
        );
    }

    /// Test UseCase status method
    #[test]
    fn test_use_case_status() {
        let use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "low".to_string(),
        )
        .unwrap();

        assert_eq!(use_case.status(), Status::Planned);
    }

    /// Test UseCase new initializes empty arrays for new fields
    #[test]
    fn test_use_case_new_initializes_empty_arrays() {
        let use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        assert!(use_case.preconditions.is_empty());
        assert!(use_case.postconditions.is_empty());
        assert!(use_case.use_case_references.is_empty());
        assert!(use_case.scenarios.is_empty());
    }

    /// Test add_precondition method
    #[test]
    fn test_add_precondition() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_precondition(Condition::new("User is logged in".to_string()));
        assert_eq!(use_case.preconditions.len(), 1);
        assert_eq!(use_case.preconditions[0].text, "User is logged in");

        // Duplicate should not be added
        use_case.add_precondition(Condition::new("User is logged in".to_string()));
        assert_eq!(use_case.preconditions.len(), 1);
    }

    /// Test add_postcondition method
    #[test]
    fn test_add_postcondition() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_postcondition(Condition::new("Session is created".to_string()));
        assert_eq!(use_case.postconditions.len(), 1);
        assert_eq!(use_case.postconditions[0].text, "Session is created");

        // Duplicate should not be added
        use_case.add_postcondition(Condition::new("Session is created".to_string()));
        assert_eq!(use_case.postconditions.len(), 1);
    }

    /// Test add_reference method
    #[test]
    fn test_add_reference() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        let reference = UseCaseReference::new("UC-AUTH-001".to_string(), "depends_on".to_string());

        use_case.add_reference(reference);
        assert_eq!(use_case.use_case_references.len(), 1);
        assert_eq!(use_case.use_case_references[0].target_id, "UC-AUTH-001");
        assert_eq!(use_case.use_case_references[0].relationship, "depends_on");

        // Duplicate reference should not be added
        let duplicate_ref =
            UseCaseReference::new("UC-AUTH-001".to_string(), "depends_on".to_string());
        use_case.add_reference(duplicate_ref);
        assert_eq!(use_case.use_case_references.len(), 1);
    }

    // Removed: test_backward_compatibility - backward compatibility not required

    /// Test serialization with new fields
    #[test]
    fn test_serialization_with_new_fields() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "high".to_string(),
        )
        .unwrap();

        // Add new fields
        use_case.add_precondition(Condition::new("User authenticated".to_string()));
        use_case.add_precondition(Condition::new("Cart not empty".to_string()));
        use_case.add_postcondition(Condition::new("Order created".to_string()));
        use_case.add_reference(
            UseCaseReference::new("UC-AUTH-001".to_string(), "depends_on".to_string())
                .with_description("Authentication required".to_string()),
        );

        // Serialize
        let serialized = serde_json::to_string(&use_case).unwrap();

        // Deserialize
        let deserialized: UseCase = serde_json::from_str(&serialized).unwrap();

        // Verify all fields
        assert_eq!(deserialized.preconditions.len(), 2);
        assert_eq!(deserialized.preconditions[0].text, "User authenticated");
        assert_eq!(deserialized.preconditions[1].text, "Cart not empty");

        assert_eq!(deserialized.postconditions.len(), 1);
        assert_eq!(deserialized.postconditions[0].text, "Order created");

        assert_eq!(deserialized.use_case_references.len(), 1);
        assert_eq!(deserialized.use_case_references[0].target_id, "UC-AUTH-001");
        assert_eq!(
            deserialized.use_case_references[0].relationship,
            "depends_on"
        );
        assert_eq!(
            deserialized.use_case_references[0].description,
            Some("Authentication required".to_string())
        );
    }

    /// Test UseCase TOML serialization and deserialization with custom fields
    #[test]
    fn test_use_case_toml_serialization_with_custom_fields() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "high".to_string(),
        )
        .unwrap();

        // Add custom fields (as would be populated from methodology)
        use_case.extra.insert(
            "user_story".to_string(),
            json!("As a user, I want to test this feature"),
        );
        use_case.extra.insert(
            "acceptance_criteria".to_string(),
            json!("The feature works correctly"),
        );
        use_case.extra.insert("story_points".to_string(), json!(5));
        use_case
            .extra
            .insert("is_critical".to_string(), json!(true));

        // Serialize to TOML
        let toml_content = toml::to_string_pretty(&use_case).expect("Failed to serialize to TOML");

        // Verify TOML contains custom fields
        assert!(toml_content.contains("user_story"));
        assert!(toml_content.contains("acceptance_criteria"));
        assert!(toml_content.contains("story_points"));
        assert!(toml_content.contains("is_critical"));

        // Deserialize from TOML through intermediate value conversion
        // (This is how the repository loads use cases)
        let toml_value: toml::Value = toml::from_str(&toml_content).expect("Failed to parse TOML");
        let json_str = serde_json::to_string(&toml_value).expect("Failed to convert to JSON");
        let deserialized: UseCase = serde_json::from_str(&json_str).expect("Failed to deserialize");

        // Verify all fields
        assert_eq!(deserialized.id, "UC-TEST-001");
        assert_eq!(deserialized.title, "Test Use Case");
        assert_eq!(deserialized.category, "Test");
        assert_eq!(deserialized.description, "A test use case");
        assert_eq!(deserialized.priority, Priority::High);

        // Verify custom fields survived the round trip
        assert_eq!(deserialized.extra.len(), 4);
        assert_eq!(
            deserialized.extra["user_story"],
            json!("As a user, I want to test this feature")
        );
        assert_eq!(
            deserialized.extra["acceptance_criteria"],
            json!("The feature works correctly")
        );
        assert_eq!(deserialized.extra["story_points"], json!(5));
        assert_eq!(deserialized.extra["is_critical"], json!(true));
    }

    /// Test UseCase status calculation from scenarios
    #[test]
    fn test_use_case_status_from_scenarios() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        // No scenarios - should be Planned
        assert_eq!(use_case.status(), Status::Planned);

        // Add scenarios with different statuses
        let scenario1 = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Happy Path".to_string(),
            "Main success scenario".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );
        // scenario1 status is Planned by default

        let mut scenario2 = Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Error Case".to_string(),
            "Error handling scenario".to_string(),
            ScenarioType::ExceptionFlow,
            "user".to_string(),
        );
        scenario2.set_status(Status::Implemented);

        use_case.add_scenario(scenario1);
        use_case.add_scenario(scenario2);

        // Status should be the minimum (earliest) status: Planned
        assert_eq!(use_case.status(), Status::Planned);

        // Update all scenarios to Implemented
        for scenario in &mut use_case.scenarios {
            scenario.set_status(Status::Implemented);
        }
        assert_eq!(use_case.status(), Status::Implemented);
    }

    /// Test next_scenario_id method
    #[test]
    fn test_next_scenario_id() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        assert_eq!(use_case.next_scenario_id(), "UC-TEST-001-S01");

        use_case.add_scenario(Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "First Scenario".to_string(),
            "Description".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        ));

        assert_eq!(use_case.next_scenario_id(), "UC-TEST-001-S02");

        use_case.add_scenario(Scenario::new(
            "UC-TEST-001-S02".to_string(),
            "Second Scenario".to_string(),
            "Description".to_string(),
            ScenarioType::AlternativeFlow,
            "user".to_string(),
        ));

        assert_eq!(use_case.next_scenario_id(), "UC-TEST-001-S03");
    }

    /// Test add_scenario method
    #[test]
    fn test_add_scenario() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        let scenario = Scenario::new(
            "UC-TEST-001-S01".to_string(),
            "Test Scenario".to_string(),
            "A test scenario".to_string(),
            ScenarioType::HappyPath,
            "user".to_string(),
        );

        use_case.add_scenario(scenario);

        assert_eq!(use_case.scenarios.len(), 1);
        assert_eq!(use_case.scenarios[0].id, "UC-TEST-001-S01");
        assert_eq!(use_case.scenarios[0].title, "Test Scenario");
    }

    /// Test add_view method
    #[test]
    fn test_add_view() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        let view = MethodologyView::new("feature", "simple");
        use_case.add_view(view);

        assert_eq!(use_case.views.len(), 1);
        assert_eq!(use_case.views[0].methodology, "feature");
        assert_eq!(use_case.views[0].level, "simple");
        assert!(use_case.views[0].enabled);
    }

    /// Test add_view prevents duplicates
    #[test]
    fn test_add_view_no_duplicates() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        let view1 = MethodologyView::new("feature", "simple");
        let view2 = MethodologyView::new("feature", "simple");

        use_case.add_view(view1);
        use_case.add_view(view2); // Should be ignored

        assert_eq!(use_case.views.len(), 1);
    }

    /// Test remove_view method
    #[test]
    fn test_remove_view() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new("feature", "simple"));
        use_case.add_view(MethodologyView::new("business", "normal"));

        let removed = use_case.remove_view("feature", "simple");
        assert!(removed);
        assert_eq!(use_case.views.len(), 1);
        assert_eq!(use_case.views[0].methodology, "business");
    }

    /// Test remove_view returns false for non-existent view
    #[test]
    fn test_remove_view_nonexistent() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new("feature", "simple"));

        let removed = use_case.remove_view("business", "normal");
        assert!(!removed);
        assert_eq!(use_case.views.len(), 1);
    }

    /// Test set_view_enabled method
    #[test]
    fn test_set_view_enabled() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new("feature", "simple"));

        let success = use_case.set_view_enabled("feature", "simple", false);
        assert!(success);
        assert!(!use_case.views[0].enabled);

        let success = use_case.set_view_enabled("feature", "simple", true);
        assert!(success);
        assert!(use_case.views[0].enabled);
    }

    /// Test set_view_enabled returns false for non-existent view
    #[test]
    fn test_set_view_enabled_nonexistent() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        let success = use_case.set_view_enabled("feature", "simple", false);
        assert!(!success);
    }

    /// Test enabled_views method
    #[test]
    fn test_enabled_views() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new("feature", "simple"));
        use_case.add_view(MethodologyView::new_disabled("business", "normal"));
        use_case.add_view(MethodologyView::new("tester", "detailed"));

        let enabled: Vec<_> = use_case.enabled_views().collect();
        assert_eq!(enabled.len(), 2);
        assert_eq!(enabled[0].methodology, "feature");
        assert_eq!(enabled[1].methodology, "tester");
    }

    /// Test views() and primary_view() methods
    #[test]
    fn test_views_and_primary_view() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        // Initially no views
        assert_eq!(use_case.views().len(), 0);
        assert!(use_case.primary_view().is_none());

        // Add first view
        use_case.add_view(MethodologyView::new("feature", "simple"));
        assert_eq!(use_case.views().len(), 1);
        assert_eq!(use_case.primary_view().unwrap().methodology, "feature");

        // Add second view
        use_case.add_view(MethodologyView::new("business", "normal"));
        assert_eq!(use_case.views().len(), 2);
        // Primary view should still be the first one
        assert_eq!(use_case.primary_view().unwrap().methodology, "feature");
    }

    /// Test views field serialization
    #[test]
    fn test_views_serialization() {
        let mut use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new("feature", "simple"));
        use_case.add_view(MethodologyView::new("business", "normal"));

        let toml = toml::to_string(&use_case).unwrap();
        assert!(toml.contains("[[views]]"));
        assert!(toml.contains("methodology = \"feature\""));
        assert!(toml.contains("level = \"simple\""));
    }
}
