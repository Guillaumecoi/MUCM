//! Language definition implementation.
//!
//! Provides concrete implementation of the Language trait loaded from TOML files.

use anyhow::Context;
use std::fs;
use std::path::Path;

use super::r#trait::Language;

/// A language definition loaded from external TOML configuration.
///
/// This struct represents a programming language that has been loaded from
/// a `source-templates/languages/{name}/info.toml` file. It contains all
/// the metadata and templates needed to support that language in the system.
///
/// Language definitions are created by reading TOML configuration files
/// and their associated template files from the filesystem.
#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    /// The primary name of the language
    name: String,
    /// Alternative names/aliases for the language
    aliases: Vec<String>,
    /// File extension for this language
    file_extension: String,
    /// The test template content loaded from the template file
    test_template: String,
}

impl LanguageDefinition {
    /// Creates a new language definition by loading from a TOML configuration file.
    ///
    /// This method reads the `info.toml` file at the specified path and loads
    /// the associated template file. The `info_path` should point to an
    /// `info.toml` file in a language directory.
    ///
    /// # Arguments
    /// * `info_path` - Path to the `info.toml` file for the language
    ///
    /// # Returns
    /// A `Result` containing the loaded `LanguageDefinition` or an error
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The TOML file cannot be read or parsed
    /// - The template file specified in the TOML cannot be found or read
    /// - The `info_path` is not a valid path
    pub fn from_toml<P: AsRef<Path>>(info_path: P) -> anyhow::Result<Self> {
        // Deserialize directly into a temporary struct for TOML data
        #[derive(serde::Deserialize)]
        struct TomlData {
            name: String,
            aliases: Vec<String>,
            file_extension: String,
            template_file: String,
        }

        let content = fs::read_to_string(&info_path)?;
        let data: TomlData = toml::from_str(&content)?;

        // Read the template file relative to the info.toml location
        let template_path = info_path
            .as_ref()
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid info.toml path: no parent directory"))?
            .join(&data.template_file);
        let test_template = fs::read_to_string(&template_path).with_context(|| {
            format!("Failed to read template file: {}", template_path.display())
        })?;

        Ok(Self {
            name: data.name,
            aliases: data.aliases,
            file_extension: data.file_extension,
            test_template,
        })
    }
}

/// Implements the Language trait for LanguageDefinition.
impl Language for LanguageDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }

    fn file_extension(&self) -> &str {
        &self.file_extension
    }

    fn test_template(&self) -> &str {
        &self.test_template
    }
}

#[cfg(test)]
mod tests {
    use super::super::r#trait::Language;
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a temporary language directory with info.toml and template
    fn create_test_language(
        dir: &std::path::Path,
        name: &str,
        aliases: &[&str],
        extension: &str,
        template_content: &str,
    ) -> std::path::PathBuf {
        let lang_dir = dir.join(name);
        fs::create_dir(&lang_dir).unwrap();

        // Create info.toml with proper TOML array syntax
        let aliases_str = if aliases.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                aliases
                    .iter()
                    .map(|a| format!("\"{}\"", a))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let info_content = format!(
            r#"name = "{}"
aliases = {}
file_extension = "{}"
template_file = "test.hbs""#,
            name, aliases_str, extension
        );
        fs::write(lang_dir.join("info.toml"), info_content).unwrap();

        // Create template file
        fs::write(lang_dir.join("test.hbs"), template_content).unwrap();

        lang_dir
    }

    #[test]
    fn test_language_definition_from_toml() {
        let temp_dir = TempDir::new().unwrap();
        let lang_dir = create_test_language(
            temp_dir.path(),
            "testlang",
            &["tl"],
            "tl",
            "template content",
        );

        let result = LanguageDefinition::from_toml(lang_dir.join("info.toml"));
        assert!(result.is_ok());

        let lang = result.unwrap();
        assert_eq!(lang.name(), "testlang");
        assert_eq!(lang.aliases(), vec!["tl"]);
        assert_eq!(lang.file_extension(), "tl");
        assert_eq!(lang.test_template(), "template content");
    }

    #[test]
    fn test_language_definition_from_toml_missing_template() {
        let temp_dir = TempDir::new().unwrap();
        let lang_dir = temp_dir.path().join("testlang");
        fs::create_dir(&lang_dir).unwrap();

        // Create info.toml but no template file
        let info_content = r#"name = "testlang"
aliases = ["tl"]
file_extension = "tl"
template_file = "test.hbs""#;
        fs::write(lang_dir.join("info.toml"), info_content).unwrap();

        let result = LanguageDefinition::from_toml(lang_dir.join("info.toml"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read template file"));
    }

    #[test]
    fn test_language_definition_from_toml_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let lang_dir = temp_dir.path().join("testlang");
        fs::create_dir(&lang_dir).unwrap();

        // Create invalid TOML
        fs::write(lang_dir.join("info.toml"), "invalid toml content").unwrap();

        let result = LanguageDefinition::from_toml(lang_dir.join("info.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_language_definition_from_toml_missing_info_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = LanguageDefinition::from_toml(temp_dir.path().join("nonexistent.toml"));
        assert!(result.is_err());
    }
}
