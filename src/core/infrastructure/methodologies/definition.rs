//! Methodology definition implementation.
//!
//! Provides concrete implementation of the Methodology trait loaded from TOML files.

use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::r#trait::{DocumentationLevel, Methodology};
use super::types::CustomFieldConfig;

/// A methodology definition loaded from external TOML configuration.
///
/// This struct represents a methodology that has been loaded from
/// `source-templates/methodologies/{name}/info.toml` and `config.toml` files.
/// It contains all the metadata and configuration needed to support that methodology
/// in the system and help users choose the appropriate methodology.
///
/// Methodology definitions are created by reading TOML configuration files
/// from the filesystem.
#[derive(Debug, Clone)]
pub struct MethodologyDefinition {
    /// The primary name of the methodology
    name: String,
    /// Display title of the methodology
    title: String,
    /// Description of the methodology
    description: String,
    /// When to use this methodology
    when_to_use: Vec<String>,
    /// Key features of this methodology
    key_features: Vec<String>,
    /// Available documentation levels
    levels: Vec<DocumentationLevel>,
    /// Preferred documentation style
    preferred_style: String,
    /// Custom fields specific to this methodology (flattened from all levels for backward compatibility)
    custom_fields: HashMap<String, CustomFieldConfig>,
    /// Per-level configuration (for field resolution with inheritance)
    pub(crate) level_configs: HashMap<String, LevelConfig>,
}

/// Configuration for a specific documentation level
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LevelConfig {
    /// Custom fields specific to this level
    #[serde(default)]
    pub(crate) custom_fields: HashMap<String, CustomFieldConfig>,
    /// Optional path to scenario template (e.g., "scenarios/scenario.hbs", "business-scenario.hbs")
    /// Paths use forward slashes, converted to OS-specific at runtime
    pub(crate) scenario_template: Option<String>,
}

impl MethodologyDefinition {
    /// Creates a methodology definition by loading from a single TOML configuration file.
    ///
    /// This method reads the `methodology.toml` file from the specified methodology directory
    /// and deserializes it into a MethodologyDefinition. The file contains all methodology
    /// information including metadata, template settings, levels, and usage guidance.
    ///
    /// # Arguments
    /// * `methodology_dir` - Path to the methodology directory containing methodology.toml
    ///
    /// # Returns
    /// A `Result` containing the loaded `MethodologyDefinition` or an error
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The TOML file cannot be read or parsed
    /// - Required fields are missing from the TOML
    /// - The `methodology_dir` is not a valid path
    pub fn from_toml<P: AsRef<Path>>(methodology_dir: P) -> anyhow::Result<Self> {
        let methodology_dir = methodology_dir.as_ref();

        // Single unified structure for methodology.toml
        #[derive(serde::Deserialize)]
        struct MethodologyData {
            methodology: MethodologyMeta,
            template: TemplateConfig,
            usage: UsageConfig,
            #[serde(default)]
            levels: HashMap<String, LevelWithCustomFields>,
        }

        #[derive(serde::Deserialize)]
        struct MethodologyMeta {
            name: String,
            description: String,
        }

        #[derive(serde::Deserialize)]
        struct TemplateConfig {
            preferred_style: String,
        }

        #[derive(serde::Deserialize)]
        struct UsageConfig {
            when_to_use: Vec<String>,
            key_features: Vec<String>,
        }

        #[derive(serde::Deserialize)]
        struct LevelWithCustomFields {
            name: String,
            abbreviation: String,
            filename: String,
            description: String,
            #[serde(default)]
            inherits: Vec<String>,
            #[serde(default)]
            custom_fields: HashMap<String, CustomFieldConfig>,
            /// Optional path to scenario template (e.g., "scenarios/scenario.hbs")
            /// Paths use forward slashes, converted to OS-specific at runtime
            scenario_template: Option<String>,
        }

        let methodology_path = methodology_dir.join("methodology.toml");
        let content = fs::read_to_string(&methodology_path)
            .context("Failed to read methodology.toml file")?;
        let data: MethodologyData =
            toml::from_str(&content).context("Failed to parse methodology.toml")?;

        // Convert levels to the expected format
        let levels: Vec<DocumentationLevel> = data
            .levels
            .values()
            .map(|level_data| DocumentationLevel {
                name: level_data.name.clone(),
                abbreviation: level_data.abbreviation.clone(),
                filename: level_data.filename.clone(),
                description: level_data.description.clone(),
                inherits: level_data.inherits.clone(),
            })
            .collect();

        // Convert levels to LevelConfig format for level_configs field
        let level_configs: HashMap<String, LevelConfig> = data
            .levels
            .iter()
            .map(|(level_name, level_data)| {
                (
                    level_name.clone(),
                    LevelConfig {
                        custom_fields: level_data.custom_fields.clone(),
                        scenario_template: level_data.scenario_template.clone(),
                    },
                )
            })
            .collect();

        // Flatten all custom fields from all levels for backward compatibility
        let mut all_custom_fields = HashMap::new();
        for level_data in data.levels.values() {
            all_custom_fields.extend(level_data.custom_fields.clone());
        }

        Ok(Self {
            name: data.methodology.name.clone(),
            title: format!("{} Methodology", data.methodology.name),
            description: data.methodology.description,
            when_to_use: data.usage.when_to_use,
            key_features: data.usage.key_features,
            levels,
            preferred_style: data.template.preferred_style,
            custom_fields: all_custom_fields,
            level_configs,
        })
    }
}

impl Methodology for MethodologyDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn when_to_use(&self) -> &[String] {
        &self.when_to_use
    }

    fn key_features(&self) -> &[String] {
        &self.key_features
    }

    fn levels(&self) -> &[DocumentationLevel] {
        &self.levels
    }

    fn preferred_style(&self) -> &str {
        &self.preferred_style
    }

    fn custom_fields(&self) -> &HashMap<String, CustomFieldConfig> {
        &self.custom_fields
    }
}

#[cfg(test)]
mod tests {
    use super::super::r#trait::Methodology;
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a temporary methodology directory with methodology.toml
    fn create_test_methodology(
        dir: &std::path::Path,
        name: &str,
        _title: &str,
        description: &str,
        preferred_style: &str,
    ) -> std::path::PathBuf {
        let methodology_dir = dir.join(name);
        fs::create_dir(&methodology_dir).unwrap();

        let methodology_content = format!(
            r#"[methodology]
name = "{}"
abbreviation = "test"
description = "{}"

[template]
preferred_style = "{}"

[generation]
auto_generate_tests = false
overwrite_test_documentation = false

[usage]
when_to_use = [
    "Use case 1",
    "Use case 2"
]
key_features = [
    "Feature 1",
    "Feature 2"
]

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Standard level"
inherits = []

[levels.normal.custom_fields]

[levels.advanced]
name = "Advanced"
abbreviation = "a"
filename = "uc_advanced.hbs"
description = "Advanced level"
inherits = ["Normal"]

[levels.advanced.custom_fields]
"#,
            name, description, preferred_style
        );
        fs::write(
            methodology_dir.join("methodology.toml"),
            methodology_content,
        )
        .unwrap();

        methodology_dir
    }

    #[test]
    fn test_methodology_definition_from_toml() {
        let temp_dir = TempDir::new().unwrap();
        let methodology_dir = create_test_methodology(
            temp_dir.path(),
            "testmethod",
            "Test Methodology",
            "Test description",
            "normal",
        );

        let result = MethodologyDefinition::from_toml(&methodology_dir);
        assert!(result.is_ok());

        let methodology = result.unwrap();
        assert_eq!(methodology.name(), "testmethod");
        assert_eq!(methodology.title(), "testmethod Methodology");
        assert_eq!(methodology.description(), "Test description");
        assert_eq!(methodology.when_to_use(), &["Use case 1", "Use case 2"]);
        assert_eq!(methodology.key_features(), &["Feature 1", "Feature 2"]);
        assert_eq!(methodology.levels().len(), 2);

        // Find levels by name (order not guaranteed from HashMap)
        let normal_level = methodology
            .levels()
            .iter()
            .find(|l| l.name == "Normal")
            .expect("normal level");
        let advanced_level = methodology
            .levels()
            .iter()
            .find(|l| l.name == "Advanced")
            .expect("advanced level");

        assert_eq!(normal_level.name, "Normal");
        assert_eq!(normal_level.abbreviation, "n");
        assert_eq!(normal_level.filename, "uc_normal.hbs");
        assert_eq!(normal_level.inherits, Vec::<String>::new());

        assert_eq!(advanced_level.name, "Advanced");
        assert_eq!(advanced_level.abbreviation, "a");
        assert_eq!(advanced_level.filename, "uc_advanced.hbs");
        assert_eq!(advanced_level.inherits, vec!["Normal"]);

        assert_eq!(methodology.preferred_style(), "normal");
    }

    #[test]
    fn test_methodology_definition_from_toml_missing_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = MethodologyDefinition::from_toml(temp_dir.path().join("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_methodology_definition_from_toml_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let methodology_dir = temp_dir.path().join("testmethod");
        fs::create_dir(&methodology_dir).unwrap();

        // Create invalid methodology.toml
        fs::write(
            methodology_dir.join("methodology.toml"),
            "invalid toml content",
        )
        .unwrap();

        let result = MethodologyDefinition::from_toml(&methodology_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_methodology_with_custom_fields() {
        let temp_dir = TempDir::new().unwrap();
        let methodology_dir = temp_dir.path().join("feature");
        fs::create_dir(&methodology_dir).unwrap();

        // Create methodology.toml with custom fields
        fs::write(
            methodology_dir.join("methodology.toml"),
            r#"
[methodology]
name = "feature"
abbreviation = "feat"
description = "Feature-focused development methodology"

[template]
preferred_style = "normal"

[generation]
auto_generate_tests = true
overwrite_test_documentation = false

[usage]
when_to_use = ["Feature development", "User story tracking"]
key_features = ["User stories", "Acceptance criteria", "Product metrics"]

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Simple feature specification"
inherits = []

[levels.normal.custom_fields]
user_segment = { label = "Target User Segment", type = "string", required = true }
success_metrics = { label = "Success Metrics", type = "array", required = true }
hypothesis = { label = "Product Hypothesis", type = "text", required = false, default = "To be defined" }
"#,
        )
        .unwrap();

        let result = MethodologyDefinition::from_toml(&methodology_dir);
        if let Err(e) = &result {
            panic!("Failed to load methodology: {:?}", e);
        }

        let methodology = result.unwrap();
        assert_eq!(methodology.name(), "feature");

        // Test custom fields
        let custom_fields = methodology.custom_fields();
        assert_eq!(custom_fields.len(), 3);

        // Check user_segment field
        let user_segment = custom_fields.get("user_segment").unwrap();
        assert_eq!(user_segment.label, Some("Target User Segment".to_string()));
        assert_eq!(user_segment.field_type, "string");
        assert!(user_segment.required);
        assert_eq!(user_segment.default, None);

        // Check success_metrics field
        let success_metrics = custom_fields.get("success_metrics").unwrap();
        assert_eq!(success_metrics.label, Some("Success Metrics".to_string()));
        assert_eq!(success_metrics.field_type, "array");
        assert!(success_metrics.required);

        // Check hypothesis field (with default)
        let hypothesis = custom_fields.get("hypothesis").unwrap();
        assert_eq!(hypothesis.label, Some("Product Hypothesis".to_string()));
        assert_eq!(hypothesis.field_type, "text");
        assert!(!hypothesis.required);
        assert_eq!(hypothesis.default, Some("To be defined".to_string()));
    }

    #[test]
    fn test_methodology_without_custom_fields() {
        let temp_dir = TempDir::new().unwrap();
        let methodology_dir = temp_dir.path().join("simple");
        fs::create_dir(&methodology_dir).unwrap();

        // Create methodology.toml without custom fields
        fs::write(
            methodology_dir.join("methodology.toml"),
            r#"
[methodology]
name = "simple"
abbreviation = "simp"
description = "Simple methodology without custom fields"

[template]
preferred_style = "simple"

[generation]
auto_generate_tests = false
overwrite_test_documentation = false

[usage]
when_to_use = ["Simple use cases"]
key_features = ["Basic documentation"]

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Simple use case"
inherits = []

[levels.normal.custom_fields]
"#,
        )
        .unwrap();

        // Create config.toml without custom_fields section
        fs::write(
            methodology_dir.join("config.toml"),
            r#"
[template]
name = "simple"
preferred_style = "simple"

[generation]
auto_generate_tests = false
overwrite_test_documentation = false
"#,
        )
        .unwrap();

        let result = MethodologyDefinition::from_toml(&methodology_dir);
        assert!(result.is_ok());

        let methodology = result.unwrap();
        assert_eq!(methodology.name(), "simple");

        // Custom fields should be empty (thanks to #[serde(default)])
        let custom_fields = methodology.custom_fields();
        assert_eq!(custom_fields.len(), 0);
    }
}
