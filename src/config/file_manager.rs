use crate::config::types::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration file management utilities.
///
/// This module provides low-level operations for loading, saving, and checking
/// the existence of configuration files. It handles the serialization/deserialization
/// of TOML configuration files and manages the configuration directory structure.
///
/// The configuration file is stored at `.config/.mucm/mucm.toml` and contains
/// all project settings, template configurations, and metadata.
pub struct ConfigFileManager;

impl ConfigFileManager {
    /// Load configuration from the standard config file.
    ///
    /// Reads the configuration from `.config/.mucm/mucm.toml` and deserializes
    /// it into a `Config` struct. This method expects the configuration file
    /// to already exist and be valid TOML.
    ///
    /// # Errors (This function will return an error if)
    /// * The configuration file does not exist (project not initialized)
    /// * The file cannot be read (permission issues)
    /// * The TOML content is malformed or invalid
    pub fn load() -> Result<Config> {
        let config_path = Config::config_path();

        if !config_path.exists() {
            anyhow::bail!("No markdown use case manager project found. Run 'mucm init' first.");
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file in specified directory.
    ///
    /// This method preserves comments and formatting by reading the existing file
    /// and updating only the changed values. If the file doesn't exist, it will
    /// serialize the full config.
    ///
    /// # Arguments
    /// * `config` - The configuration to save
    /// * `base_dir` - Base directory where the config should be saved
    ///
    /// # Errors (This function will return an error if)
    /// * The directory structure cannot be created
    /// * The configuration cannot be serialized to TOML
    /// * The file cannot be written (permission issues)
    pub fn save_in_dir(config: &Config, base_dir: &str) -> Result<()> {
        let base_path = Path::new(base_dir);
        let config_dir = base_path.join(Config::CONFIG_DIR);
        let config_path = config_dir.join("mucm.toml");

        // Create the config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        // If file exists, preserve comments by doing selective updates
        let content = if config_path.exists() {
            Self::update_config_preserving_comments(&config_path, config)?
        } else {
            // New file, just serialize
            toml::to_string_pretty(config).context("Failed to serialize config")?
        };

        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Update config file while preserving comments and formatting.
    ///
    /// This reads the existing config file and updates only the values that have changed,
    /// preserving all comments, blank lines, and formatting.
    fn update_config_preserving_comments(
        config_path: &Path,
        new_config: &Config,
    ) -> Result<String> {
        let mut content =
            fs::read_to_string(config_path).context("Failed to read existing config")?;

        // Update project values
        content = Self::update_toml_value(
            &content,
            "project",
            "name",
            &format!(r#""{}""#, new_config.project.name),
        );
        content = Self::update_toml_value(
            &content,
            "project",
            "description",
            &format!(r#""{}""#, new_config.project.description),
        );

        // Update directory settings
        content = Self::update_toml_value(
            &content,
            "directories",
            "use_case_dir",
            &format!(r#""{}""#, new_config.directories.use_case_dir),
        );
        content = Self::update_toml_value(
            &content,
            "directories",
            "test_dir",
            &format!(r#""{}""#, new_config.directories.test_dir),
        );
        content = Self::update_toml_value(
            &content,
            "directories",
            "data_dir",
            &format!(r#""{}""#, new_config.directories.data_dir),
        );

        // Update template settings
        let methodologies_str = new_config
            .templates
            .methodologies
            .iter()
            .map(|m| format!(r#""{}""#, m))
            .collect::<Vec<_>>()
            .join(", ");
        content = Self::update_toml_value(
            &content,
            "templates",
            "methodologies",
            &format!("[{}]", methodologies_str),
        );
        content = Self::update_toml_value(
            &content,
            "templates",
            "default_methodology",
            &format!(r#""{}""#, new_config.templates.default_methodology),
        );
        content = Self::update_toml_value(
            &content,
            "templates",
            "default_scenario_template",
            &format!(r#""{}""#, new_config.templates.default_scenario_template),
        );

        // Update generation settings
        content = Self::update_toml_value(
            &content,
            "generation",
            "test_language",
            &format!(r#""{}""#, new_config.generation.test_language),
        );
        content = Self::update_toml_value(
            &content,
            "generation",
            "auto_generate_tests",
            &new_config.generation.auto_generate_tests.to_string(),
        );
        content = Self::update_toml_value(
            &content,
            "generation",
            "overwrite_test_documentation",
            &new_config
                .generation
                .overwrite_test_documentation
                .to_string(),
        );

        // Update metadata settings
        content = Self::update_toml_value(
            &content,
            "metadata",
            "created",
            &new_config.metadata.created.to_string(),
        );
        content = Self::update_toml_value(
            &content,
            "metadata",
            "last_updated",
            &new_config.metadata.last_updated.to_string(),
        );

        // Update storage backend
        let backend_str = match new_config.storage.backend {
            crate::config::StorageBackend::Toml => "toml",
            crate::config::StorageBackend::Sqlite => "sqlite",
        };
        content = Self::update_toml_value(
            &content,
            "storage",
            "backend",
            &format!(r#""{}""#, backend_str),
        );

        Ok(content)
    }

    /// Update a single TOML value while preserving everything else.
    ///
    /// Finds lines like `key = old_value` within the specified section and replaces
    /// with `key = new_value`, preserving any inline comments. Only updates the first
    /// occurrence within the target section.
    ///
    /// # Arguments
    /// * `content` - The full TOML content
    /// * `section` - The TOML section name (e.g., "project", "templates")
    /// * `key` - The key to update within that section
    /// * `new_value` - The new value to set
    fn update_toml_value(content: &str, section: &str, key: &str, new_value: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        let mut i = 0;
        let section_header = format!("[{}]", section);
        let mut in_target_section = false;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim_start();

            // Track which section we're in
            if trimmed.starts_with('[') && trimmed.contains(']') {
                // Entering a new section
                in_target_section =
                    trimmed == section_header || trimmed.starts_with(&format!("[{}.", section));
            }

            // Check if this line starts our key
            if in_target_section
                && trimmed.starts_with(key)
                && (trimmed.chars().nth(key.len()) == Some(' ')
                    || trimmed.chars().nth(key.len()) == Some('='))
            {
                // Find the = sign
                if let Some(eq_pos) = line.find('=') {
                    let after_eq = &line[eq_pos + 1..].trim_start();

                    // Check if this is a multi-line array (starts with [)
                    if after_eq.starts_with('[') && !after_eq.contains(']') {
                        // Multi-line array - find the closing bracket
                        let mut array_end = i;
                        let mut bracket_count = 0;

                        for j in i..lines.len() {
                            let array_line = lines[j].trim();
                            bracket_count += array_line.chars().filter(|&c| c == '[').count();
                            bracket_count -= array_line.chars().filter(|&c| c == ']').count();

                            if bracket_count == 0 && array_line.contains(']') {
                                array_end = j;
                                break;
                            }
                        }

                        // Replace the entire array block
                        let indent = line.len() - trimmed.len();
                        let indent_str = " ".repeat(indent);
                        result.push_str(&format!("{}{} = {}\n", indent_str, key, new_value));

                        // Skip the old array lines
                        i = array_end + 1;
                        continue;
                    } else {
                        // Single-line value - handle normally
                        let comment_pos = after_eq.find('#');

                        let indent = line.len() - trimmed.len();
                        let indent_str = " ".repeat(indent);

                        if let Some(comment_start) = comment_pos {
                            let comment = &after_eq[comment_start..];
                            result.push_str(&format!(
                                "{}{} = {} {}\n",
                                indent_str, key, new_value, comment
                            ));
                        } else {
                            result.push_str(&format!("{}{} = {}\n", indent_str, key, new_value));
                        }
                        i += 1;
                        continue;
                    }
                }
            }

            // Keep line as-is
            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        result
    }

    /// Check if templates have already been copied to .config/.mucm/handlebars/
    ///
    /// This method checks whether the template assets directory exists and is a directory.
    /// It's used to determine if the project has completed the template setup phase
    /// of initialization.
    ///
    /// # Returns
    /// Returns `true` if the templates directory exists and is a directory, `false` otherwise.
    pub fn check_templates_exist() -> bool {
        let base_path = Path::new(".");
        let templates_dir = base_path
            .join(Config::CONFIG_DIR)
            .join(Config::TEMPLATES_DIR);
        templates_dir.exists() && templates_dir.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_load_nonexistent_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = ConfigFileManager::load();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No markdown use case manager project found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_load_invalid_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory
        fs::create_dir_all(Config::CONFIG_DIR)?;

        // Write invalid TOML
        let invalid_toml = r#"
            [project
            name = "Test Project"
            # Missing closing bracket
        "#;
        fs::write(Config::config_path(), invalid_toml)?;

        let result = ConfigFileManager::load();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse config file"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_load_valid_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory
        fs::create_dir_all(Config::CONFIG_DIR)?;

        // Create a valid config
        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".")?;

        // Load it back
        let loaded_config = ConfigFileManager::load()?;
        assert_eq!(loaded_config.project.name, config.project.name);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_save_in_dir_creates_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let config = Config::default();
        let custom_base_dir = "custom/project";

        // Save to custom directory (should create directories automatically)
        ConfigFileManager::save_in_dir(&config, custom_base_dir)?;

        // Check that directories were created and file exists
        let config_path = Path::new(custom_base_dir)
            .join(Config::CONFIG_DIR)
            .join("mucm.toml");
        assert!(config_path.exists());

        // Verify content
        let content = fs::read_to_string(config_path)?;
        let loaded_config: Config = toml::from_str(&content)?;
        assert_eq!(loaded_config.project.name, config.project.name);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_save_in_dir_overwrites_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory
        fs::create_dir_all(Config::CONFIG_DIR)?;

        // Save initial config
        let mut config = Config::default();
        config.project.name = "Original Name".to_string();
        ConfigFileManager::save_in_dir(&config, ".")?;

        // Modify and save again
        config.project.name = "Updated Name".to_string();
        ConfigFileManager::save_in_dir(&config, ".")?;

        // Load and verify
        let loaded_config = ConfigFileManager::load()?;
        assert_eq!(loaded_config.project.name, "Updated Name");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_check_templates_exist_when_missing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Templates directory doesn't exist
        assert!(!ConfigFileManager::check_templates_exist());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_check_templates_exist_when_file_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create the templates directory path but as a file instead
        let templates_path = Path::new(Config::CONFIG_DIR).join(Config::TEMPLATES_DIR);
        fs::create_dir_all(templates_path.parent().unwrap())?;
        fs::write(&templates_path, "not a directory")?;

        // Should return false because it's a file, not a directory
        assert!(!ConfigFileManager::check_templates_exist());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_check_templates_exist_when_directory_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create the templates directory
        let templates_path = Path::new(Config::CONFIG_DIR).join(Config::TEMPLATES_DIR);
        fs::create_dir_all(&templates_path)?;

        // Should return true
        assert!(ConfigFileManager::check_templates_exist());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_save_in_dir_with_invalid_path() -> Result<()> {
        let config = Config::default();

        // Try to save to a path that can't be created (should fail)
        let result = ConfigFileManager::save_in_dir(&config, "/root/forbidden/path");

        // This might succeed or fail depending on permissions, but either way
        // the function should handle it gracefully
        // We mainly want to ensure it doesn't panic
        match result {
            Ok(_) => {
                // If it succeeded, clean up
                let _ = fs::remove_file("/root/forbidden/path/.config/.mucm/mucm.toml");
            }
            Err(_) => {
                // Expected - permissions denied
            }
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_load_with_read_permission_denied() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory and file
        fs::create_dir_all(Config::CONFIG_DIR)?;
        let config_path = Config::config_path();
        let config = Config::default();
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;

        // Remove read permissions (if possible on this system)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o000); // No permissions
            fs::set_permissions(&config_path, perms)?;

            let result = ConfigFileManager::load();

            // Restore permissions for cleanup first
            let mut restore_perms = fs::metadata(&config_path)?.permissions();
            restore_perms.set_mode(0o644);
            let _ = fs::set_permissions(&config_path, restore_perms);

            // In Docker/CI (running as root), permission restrictions don't work
            // Skip assertion if we can still read the file (indicates root)
            if result.is_ok() {
                // Running as root or permission test doesn't work on this system
                // Just verify the file can be read
                let _ = ConfigFileManager::load()?;
            } else {
                // Permission restriction worked, verify error
                assert!(result.is_err());
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just verify normal operation
            let loaded_config = ConfigFileManager::load()?;
            assert_eq!(loaded_config.project.name, config.project.name);
        }

        Ok(())
    }
}
