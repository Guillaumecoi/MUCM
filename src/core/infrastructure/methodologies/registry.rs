//! Methodology registry implementation.
//!
//! Provides lookup and discovery of methodologies loaded from the filesystem.

use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::definition::MethodologyDefinition;
use super::r#trait::Methodology;

/// Registry for managing methodology definitions.
///
/// Provides lookup and discovery of methodologies loaded from the filesystem.
/// Acts as a central repository for methodology metadata and configuration.
#[derive(Debug)]
pub struct MethodologyRegistry {
    /// Map of methodology name to methodology definition
    methodologies: HashMap<String, MethodologyDefinition>,
}

impl MethodologyRegistry {
    /// Creates a new methodology registry by discovering and loading all available methodologies.
    ///
    /// This method scans the `templates_dir/methodologies/` directory for methodology
    /// configuration files and loads them into the registry.
    ///
    /// # Arguments
    /// * `templates_dir` - Base directory containing the methodologies subdirectory
    ///
    /// # Returns
    /// A `Result` containing the loaded `MethodologyRegistry` or an error
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The methodologies directory cannot be found or read
    /// - Methodology loading fails for any methodology
    pub fn new_dynamic<P: AsRef<Path>>(templates_dir: P) -> anyhow::Result<Self> {
        let methodologies_dir = templates_dir.as_ref().join("methodologies");

        if !methodologies_dir.exists() {
            return Ok(Self {
                methodologies: HashMap::new(),
            });
        }

        let mut methodologies = HashMap::new();

        for entry in
            fs::read_dir(&methodologies_dir).context("Failed to read methodologies directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(methodology_name) = path.file_name().and_then(|n| n.to_str()) {
                    let methodology_path = path.join("methodology.toml");

                    if methodology_path.exists() {
                        match MethodologyDefinition::from_toml(&path) {
                            Ok(methodology) => {
                                methodologies.insert(methodology_name.to_string(), methodology);
                            }
                            Err(e) => {
                                // Log the error but continue loading other methodologies
                                eprintln!(
                                    "Warning: Failed to load methodology '{}': {}",
                                    methodology_name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { methodologies })
    }

    /// Gets a methodology by name.
    ///
    /// Performs case-insensitive lookup of methodologies by name.
    ///
    /// # Arguments
    /// * `name` - The name of the methodology to retrieve
    ///
    /// # Returns
    /// An `Option` containing the methodology if found, or `None` if not found
    pub fn get(&self, name: &str) -> Option<&MethodologyDefinition> {
        // Try exact match first
        if let Some(methodology) = self.methodologies.get(name) {
            return Some(methodology);
        }

        // Try case-insensitive match
        self.methodologies
            .values()
            .find(|m| m.name().eq_ignore_ascii_case(name))
    }

    /// Returns a list of all available methodology names.
    ///
    /// Returns the names of all successfully loaded methodologies,
    /// sorted alphabetically.
    ///
    /// # Returns
    /// A vector of methodology names
    pub fn available_methodologies(&self) -> Vec<String> {
        let mut names: Vec<String> = self.methodologies.keys().cloned().collect();
        names.sort();
        names
    }

    /// Discovers available methodologies from the templates directory.
    ///
    /// This is a convenience method that creates a registry and returns
    /// the list of available methodology names.
    ///
    /// # Arguments
    /// * `templates_dir` - Base directory containing the methodologies subdirectory
    ///
    /// # Returns
    /// A `Result` containing a vector of methodology names or an error
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The templates directory cannot be found or read
    /// - Methodology loading fails
    pub fn discover_available<P: AsRef<Path>>(templates_dir: P) -> anyhow::Result<Vec<String>> {
        let registry = Self::new_dynamic(templates_dir)?;
        Ok(registry.available_methodologies())
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
abbreviation = "{}"
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
description = "Basic level"
inherits = []

[levels.normal.custom_fields]

[levels.advanced]
name = "Advanced"
abbreviation = "a"
filename = "uc_advanced.hbs"
description = "Detailed level"
inherits = ["Normal"]

[levels.advanced.custom_fields]
"#,
            name,
            &name[..3.min(name.len())], // abbreviation = first 3 chars
            description,
            preferred_style
        );
        fs::write(
            methodology_dir.join("methodology.toml"),
            methodology_content,
        )
        .unwrap();

        methodology_dir
    }

    #[test]
    fn test_methodology_registry_new_dynamic() {
        let temp_dir = TempDir::new().unwrap();
        let methodologies_dir = temp_dir.path().join("methodologies");
        fs::create_dir(&methodologies_dir).unwrap();

        // Create test methodologies
        create_test_methodology(
            &methodologies_dir,
            "method1",
            "Method 1",
            "Description 1",
            "simple",
        );
        create_test_methodology(
            &methodologies_dir,
            "method2",
            "Method 2",
            "Description 2",
            "detailed",
        );

        let result = MethodologyRegistry::new_dynamic(temp_dir.path());
        assert!(result.is_ok());

        let registry = result.unwrap();

        // Check that methodologies were loaded
        assert!(registry.get("method1").is_some());
        assert!(registry.get("method2").is_some());

        // Check that they have correct data
        let method1 = registry.get("method1").unwrap();
        assert_eq!(method1.name(), "method1");
        assert_eq!(method1.title(), "method1 Methodology");
        assert_eq!(method1.description(), "Description 1");
        assert_eq!(method1.preferred_style(), "simple");
    }

    #[test]
    fn test_methodology_registry_new_dynamic_no_methodologies_dir() {
        let temp_dir = TempDir::new().unwrap();

        let result = MethodologyRegistry::new_dynamic(temp_dir.path());
        assert!(result.is_ok());

        let registry = result.unwrap();
        assert!(registry.available_methodologies().is_empty());
    }

    #[test]
    fn test_methodology_registry_get() {
        let temp_dir = TempDir::new().unwrap();
        let methodologies_dir = temp_dir.path().join("methodologies");
        fs::create_dir(&methodologies_dir).unwrap();

        create_test_methodology(
            &methodologies_dir,
            "business",
            "Business Methodology",
            "Business-focused",
            "detailed",
        );

        let registry = MethodologyRegistry::new_dynamic(temp_dir.path()).unwrap();

        // Test primary name
        let methodology = registry.get("business").unwrap();
        assert_eq!(methodology.name(), "business");

        // Test case insensitive
        let methodology_lower = registry.get("BUSINESS").unwrap();
        assert_eq!(methodology_lower.name(), "business");

        // Test nonexistent
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_methodology_registry_available_methodologies() {
        let temp_dir = TempDir::new().unwrap();
        let methodologies_dir = temp_dir.path().join("methodologies");
        fs::create_dir(&methodologies_dir).unwrap();

        create_test_methodology(
            &methodologies_dir,
            "method1",
            "Method 1",
            "Desc 1",
            "simple",
        );
        create_test_methodology(
            &methodologies_dir,
            "method2",
            "Method 2",
            "Desc 2",
            "detailed",
        );

        let registry = MethodologyRegistry::new_dynamic(temp_dir.path()).unwrap();
        let available = registry.available_methodologies();

        assert!(available.contains(&"method1".to_string()));
        assert!(available.contains(&"method2".to_string()));
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn test_methodology_registry_with_malformed_methodology() {
        let temp_dir = TempDir::new().unwrap();
        let methodologies_dir = temp_dir.path().join("methodologies");
        fs::create_dir(&methodologies_dir).unwrap();

        // Create a valid methodology
        create_test_methodology(
            &methodologies_dir,
            "valid",
            "Valid Methodology",
            "Valid description",
            "simple",
        );

        // Create a malformed methodology directory (invalid TOML)
        let bad_methodology_dir = methodologies_dir.join("bad");
        fs::create_dir(&bad_methodology_dir).unwrap();
        fs::write(bad_methodology_dir.join("methodology.toml"), "invalid toml").unwrap();

        let result = MethodologyRegistry::new_dynamic(temp_dir.path());
        assert!(result.is_ok()); // Should succeed despite one bad methodology

        let registry = result.unwrap();

        // Valid methodology should still be loaded
        assert!(registry.get("valid").is_some());

        // Bad methodology should not be loaded
        assert!(registry.get("bad").is_none());
    }

    #[test]
    fn test_methodology_registry_discover_available() {
        let temp_dir = TempDir::new().unwrap();
        let methodologies_dir = temp_dir.path().join("methodologies");
        fs::create_dir(&methodologies_dir).unwrap();

        create_test_methodology(
            &methodologies_dir,
            "method1",
            "Method 1",
            "Desc 1",
            "simple",
        );
        create_test_methodology(
            &methodologies_dir,
            "method2",
            "Method 2",
            "Desc 2",
            "detailed",
        );

        let result = MethodologyRegistry::discover_available(temp_dir.path());
        assert!(result.is_ok());

        let methodologies = result.unwrap();
        assert!(methodologies.contains(&"method1".to_string()));
        assert!(methodologies.contains(&"method2".to_string()));
        assert_eq!(methodologies.len(), 2);
    }
}
