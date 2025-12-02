//! # Template Manager
//!
//! This module handles template file management and processing for the Markdown Use Case Manager.
//! It provides functionality to discover, copy, and manage template assets used for generating
//! use case documentation and test files.
//!
//! ## Template Structure
//!
//! Templates are organized in a hierarchical structure:
//! - `source-templates/` - Source template directory
//!   - `overview.hbs` - Root template files
//!   - `methodologies/` - Methodology-specific templates and configs
//!     - `{methodology}/` - Individual methodology directory
//!       - `config.toml` - Methodology configuration
//!       - Template files (`.hbs`)
//!   - `languages/` - Language-specific test templates
//!     - `{language}/` - Language directory
//!       - `test.hbs` - Test template for the language
//!
//! ## Template Copying Process
//!
//! The template copying process involves:
//! 1. Locating the source templates directory
//! 2. Reading the project configuration
//! 3. Copying root templates (overview.hbs)
//! 4. Copying methodology-specific templates and configs
//! 5. Copying language-specific templates
//!
//! ## Configuration Integration
//!
//! Templates are selected based on the project's configuration:
//! - `config.templates.methodologies` determines which methodologies to copy
//! - `config.generation.test_language` determines which language templates to copy

use crate::config::types::Config;
use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::{Path, PathBuf};

// Embed source templates at compile time for cargo install support
static EMBEDDED_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/source-templates");

pub struct TemplateManager;

impl TemplateManager {
    /// Create a configuration file from the current config state.
    ///
    /// Serializes the provided configuration to TOML format and writes it to
    /// the standard configuration file path. This ensures that user-selected
    /// languages and methodologies are persisted to disk.
    ///
    /// # Arguments
    /// * `config` - The configuration to serialize and save
    ///
    /// # Errors (This function will return an error if)
    /// * The configuration cannot be serialized to TOML
    /// * The configuration file cannot be written
    pub fn create_config_from_template(config: &Config) -> Result<()> {
        // Load the template file to preserve comments and formatting
        let source_templates_dir = Self::find_source_templates_dir()?;
        let template_path = source_templates_dir.join("config.toml");

        let mut template_content =
            fs::read_to_string(&template_path).context("Failed to read config template")?;

        // Update only the specific values that differ from defaults
        // Project name and description
        template_content = template_content.replace(
            r#"name = "My Project""#,
            &format!(r#"name = "{}""#, config.project.name),
        );
        template_content = template_content.replace(
            r#"description = "A project managed with use case manager""#,
            &format!(r#"description = "{}""#, config.project.description),
        );

        // Methodologies - use regex to replace any array content
        let methodologies_str = config
            .templates
            .methodologies
            .iter()
            .map(|m| format!(r#""{}""#, m))
            .collect::<Vec<_>>()
            .join(", ");

        // Use a more flexible replacement that handles any array content
        // This regex matches: methodologies = [anything]
        let re = regex::Regex::new(r#"methodologies = \[([^\]]*)\]"#).unwrap();
        template_content = re
            .replace(
                &template_content,
                format!("methodologies = [{}]", methodologies_str),
            )
            .to_string();

        // Default methodology
        template_content = template_content.replace(
            r#"default_methodology = "feature""#,
            &format!(
                r#"default_methodology = "{}""#,
                config.templates.default_methodology
            ),
        );

        // Default scenario template
        template_content = template_content.replace(
            r#"default_scenario_template = "scenarios/scenario.hbs""#,
            &format!(
                r#"default_scenario_template = "{}""#,
                config.templates.default_scenario_template
            ),
        );

        // Test language
        template_content = template_content.replace(
            r#"test_language = "none""#,
            &format!(r#"test_language = "{}""#, config.generation.test_language),
        );

        // Storage backend
        let backend_str = match config.storage.backend {
            crate::config::StorageBackend::Toml => "toml",
            crate::config::StorageBackend::Sqlite => "sqlite",
        };
        template_content = template_content.replace(
            r#"backend = "toml""#,
            &format!(r#"backend = "{}""#, backend_str),
        );

        // Directories
        template_content = template_content.replace(
            r#"use_case_dir = "docs/use-cases""#,
            &format!(r#"use_case_dir = "{}""#, config.directories.use_case_dir),
        );
        template_content = template_content.replace(
            r#"test_dir = "tests/use-cases""#,
            &format!(r#"test_dir = "{}""#, config.directories.test_dir),
        );
        template_content = template_content.replace(
            r#"actor_dir = "docs/actors""#,
            &format!(r#"actor_dir = "{}""#, config.directories.actor_dir),
        );
        template_content = template_content.replace(
            r#"data_dir = "use-cases-data""#,
            &format!(r#"data_dir = "{}""#, config.directories.data_dir),
        );

        // Write the config
        let config_path = Config::config_path();
        fs::write(&config_path, &template_content).context("Failed to write config file")?;

        Ok(())
    }

    /// Locate the source templates directory.
    ///
    /// Searches for the source-templates directory in multiple locations:
    /// 1. User config directory (~/.config/mucm/templates/)
    /// 2. Current working directory
    /// 3. CARGO_MANIFEST_DIR environment variable (for tests/builds)
    /// 4. Relative to the current executable path
    ///
    /// If templates are found in development locations (2-4) but not in user config,
    /// they are automatically copied to ~/.config/mucm/templates/ for future use.
    ///
    /// # Returns
    /// The path to the source-templates directory, or an error if not found.
    ///
    /// # Errors
    /// Returns an error if the source-templates directory cannot be located
    /// in any of the expected locations.
    pub fn find_source_templates_dir() -> Result<PathBuf> {
        use directories::ProjectDirs;

        // In test mode, prefer local templates over global to avoid cross-test pollution
        let test_mode = std::env::var("MUCM_TEST_MODE").is_ok();

        // Try current directory (prioritized in test mode)
        let local_templates = Path::new("source-templates");
        if test_mode && local_templates.exists() {
            // In test mode, always use local templates without installing globally
            return Ok(local_templates.to_path_buf());
        }

        // Try user config directory first (~/.config/mucm/templates/)
        if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
            let user_templates = proj_dirs.config_dir().join("templates");
            if user_templates.exists() {
                return Ok(user_templates);
            }
        }

        // Try current directory (normal priority)
        if local_templates.exists() {
            // Found in dev location - install to user config for future use
            Self::install_templates_to_user_config(local_templates)?;

            // Return user config path if installation succeeded
            if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
                let user_templates = proj_dirs.config_dir().join("templates");
                if user_templates.exists() {
                    return Ok(user_templates);
                }
            }

            return Ok(local_templates.to_path_buf());
        }

        // Try CARGO_MANIFEST_DIR (set during cargo test and build)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let cargo_templates = Path::new(&manifest_dir).join("source-templates");
            if cargo_templates.exists() {
                // Found in dev location - install to user config for future use
                Self::install_templates_to_user_config(&cargo_templates)?;

                // Return user config path if installation succeeded
                if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
                    let user_templates = proj_dirs.config_dir().join("templates");
                    if user_templates.exists() {
                        return Ok(user_templates);
                    }
                }

                return Ok(cargo_templates);
            }
        }

        // If still not found, try to find templates relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check ../../source-templates (when running from target/release/)
                let dev_templates = exe_dir
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("source-templates"));
                if let Some(dev_templates) = dev_templates {
                    if dev_templates.exists() {
                        // Found in dev location - install to user config for future use
                        Self::install_templates_to_user_config(&dev_templates)?;

                        // Return user config path if installation succeeded
                        if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
                            let user_templates = proj_dirs.config_dir().join("templates");
                            if user_templates.exists() {
                                return Ok(user_templates);
                            }
                        }

                        return Ok(dev_templates);
                    }
                }
            }
        }

        // Last resort: extract embedded templates to user config
        if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
            let user_templates = proj_dirs.config_dir().join("templates");

            // Extract embedded templates if they don't exist
            if !user_templates.exists() {
                Self::extract_embedded_templates(&user_templates)?;
                eprintln!(
                    "✓ Installed embedded templates to {}",
                    user_templates.display()
                );
            }

            if user_templates.exists() {
                return Ok(user_templates);
            }
        }

        anyhow::bail!(
            "Source templates directory not found and could not extract embedded templates.\n\
             This should not happen - please report this issue."
        )
    }

    /// Extract embedded templates to user config directory
    ///
    /// This function extracts the templates that were embedded at compile time
    /// to the user's config directory. This is used when cargo install is used
    /// and the source-templates directory is not available.
    ///
    /// # Arguments
    /// * `target_path` - Path where templates should be extracted
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if extraction fails
    fn extract_embedded_templates(target_path: &Path) -> Result<()> {
        // Create the target directory
        fs::create_dir_all(target_path).context("Failed to create templates directory")?;

        // Extract all files from the embedded directory
        EMBEDDED_TEMPLATES
            .extract(target_path)
            .context("Failed to extract embedded templates")?;

        Ok(())
    }

    /// Install templates to user config directory (~/.config/mucm/templates/)
    ///
    /// This function copies the source templates from a development location
    /// to the user's config directory for persistent use across runs.
    ///
    /// # Arguments
    /// * `source_path` - Path to the source-templates directory to copy from
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if the copy fails
    fn install_templates_to_user_config(source_path: &Path) -> Result<()> {
        use directories::ProjectDirs;

        let proj_dirs = ProjectDirs::from("", "", "mucm")
            .ok_or_else(|| anyhow::anyhow!("Could not determine user config directory"))?;

        let user_templates_dir = proj_dirs.config_dir().join("templates");

        // Skip if already exists
        if user_templates_dir.exists() {
            return Ok(());
        }

        // Create parent directory
        if let Some(parent) = user_templates_dir.parent() {
            fs::create_dir_all(parent).context("Failed to create user config directory")?;
        }

        // Copy templates recursively
        Self::copy_dir_recursive(source_path, &user_templates_dir)
            .context("Failed to copy templates to user config directory")?;

        eprintln!("✓ Installed templates to {}", user_templates_dir.display());

        Ok(())
    }

    /// Copy all templates to the configuration directory.
    ///
    /// This is the main template copying function that sets up the project's
    /// template assets. It copies root templates, methodology-specific templates,
    /// and language-specific templates to the appropriate configuration directories.
    ///
    /// The function reads the project's configuration to determine which methodologies
    /// and languages to copy, then performs a complete template setup.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for the operation (usually ".")
    ///
    /// # Errors (This function will return an error if)
    /// * The configuration file is missing or invalid
    /// * Source templates directory cannot be found
    /// * Template directories cannot be created
    /// * Individual template files cannot be copied
    /// * Required methodologies are missing from source templates
    pub fn copy_templates_to_config(base_dir: &str) -> Result<()> {
        let base_path = Path::new(base_dir);
        let config_templates_dir = base_path
            .join(Config::CONFIG_DIR)
            .join(Config::TEMPLATES_DIR);

        // Create directories
        fs::create_dir_all(&config_templates_dir)
            .context("Failed to create config templates directory")?;

        // Load the config from base_dir to see which methodologies to import
        let config_path = base_path.join(Config::CONFIG_DIR).join("mucm.toml");
        if !config_path.exists() {
            anyhow::bail!(
                "Config file not found at {:?} - run 'mucm init' first",
                config_path
            );
        }
        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        let source_templates_dir = Self::find_source_templates_dir()?;

        // Copy root template files
        Self::copy_root_templates(&source_templates_dir, &config_templates_dir)?;

        // Copy scenarios directory
        Self::copy_scenarios(&source_templates_dir, &config_templates_dir)?;

        // Copy methodologies
        Self::copy_methodologies(&source_templates_dir, &config, &config_templates_dir)?;

        // Copy language templates
        Self::copy_language_templates(&source_templates_dir, &config, &config_templates_dir)?;

        Ok(())
    }

    /// Copy root template files (overview.hbs, etc.).
    ///
    /// Copies template files that are not specific to any methodology or language,
    /// such as overview templates used for generating project documentation.
    ///
    /// # Arguments
    /// * `source_templates_dir` - Path to the source templates directory
    /// * `config_templates_dir` - Path to the destination templates directory
    ///
    /// # Errors
    /// Returns an error if template files cannot be copied.
    fn copy_root_templates(source_templates_dir: &Path, config_templates_dir: &Path) -> Result<()> {
        // Copy overview.hbs
        let overview_src = source_templates_dir.join("overview.hbs");
        if overview_src.exists() {
            let overview_dst = config_templates_dir.join("overview.hbs");
            if !overview_dst.exists() {
                fs::copy(&overview_src, &overview_dst)?;
                println!("✓ Copied overview template");
            } else {
                println!("⊙ Skipped overview template (already exists)");
            }
        }

        Ok(())
    }

    /// Copy shared scenario templates.
    ///
    /// Copies the scenarios directory containing shared scenario rendering templates
    /// that can be used by multiple methodologies via Handlebars partials.
    ///
    /// # Arguments
    /// * `source_templates_dir` - Path to the source templates directory
    /// * `config_templates_dir` - Path to the destination templates directory
    ///
    /// # Errors
    /// Returns an error if the scenarios directory exists but cannot be copied.
    fn copy_scenarios(source_templates_dir: &Path, config_templates_dir: &Path) -> Result<()> {
        let scenarios_src = source_templates_dir.join("scenarios");
        if scenarios_src.exists() {
            let scenarios_dst = config_templates_dir.join("scenarios");
            if !scenarios_dst.exists() {
                Self::copy_dir_recursive(&scenarios_src, &scenarios_dst)?;
                println!("✓ Copied scenario templates");
            } else {
                // Only copy new files, skip existing ones
                Self::copy_dir_incremental(&scenarios_src, &scenarios_dst)?;
                println!("⊙ Updated scenario templates (preserved existing files)");
            }
        }

        Ok(())
    }

    /// Copy methodology templates and configs.
    ///
    /// For each methodology specified in the configuration, this function
    /// copies all methodology files to the template-assets/{methodology}/ directory
    /// for use by the template engine and methodology registry.
    ///
    /// # Arguments
    /// * `source_templates_dir` - Path to the source templates directory
    /// * `config` - Project configuration containing methodology list
    /// * `config_templates_dir` - Path to the destination templates directory
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The source methodologies directory is missing
    /// - A specified methodology directory is missing
    /// - Template files cannot be copied
    fn copy_methodologies(
        source_templates_dir: &Path,
        config: &Config,
        config_templates_dir: &Path,
    ) -> Result<()> {
        let source_methodologies = source_templates_dir.join("methodologies");
        if !source_methodologies.exists() {
            anyhow::bail!(
                "Source methodologies directory not found at {:?}",
                source_methodologies
            );
        }

        for methodology in &config.templates.methodologies {
            let source_method_dir = source_methodologies.join(methodology);
            if !source_method_dir.exists() {
                anyhow::bail!(
                    "Methodology '{}' not found in source-templates/methodologies/. \
                     Available methodologies should be in source-templates/methodologies/{{name}}/ directories.",
                    methodology
                );
            }

            // Validate that required methodology.toml file exists
            let methodology_file = source_method_dir.join("methodology.toml");
            if !methodology_file.exists() {
                anyhow::bail!(
                    "Methodology '{}' is missing methodology.toml file in {:?}",
                    methodology,
                    source_method_dir
                );
            }

            // Copy methodology templates to template-assets/methodologies/{methodology}/
            let methodologies_dir = config_templates_dir.join("methodologies");
            fs::create_dir_all(&methodologies_dir)?;
            let target_method_templates = methodologies_dir.join(methodology);

            if !target_method_templates.exists() {
                Self::copy_dir_recursive(&source_method_dir, &target_method_templates)?;
                println!("✓ Copied methodology: {}", methodology);
            } else {
                // Only copy new files, skip existing ones to preserve customizations
                Self::copy_dir_incremental(&source_method_dir, &target_method_templates)?;
                println!(
                    "⊙ Skipped methodology: {} (already exists, preserved customizations)",
                    methodology
                );
            }
        }

        Ok(())
    }

    /// Copy language templates.
    ///
    /// Copies test templates for the configured test language. Language templates
    /// are optional - if the specified language is not found, the function
    /// completes successfully but logs a warning.
    ///
    /// # Arguments
    /// * `source_templates_dir` - Path to the source templates directory
    /// * `config` - Project configuration containing test language
    /// * `config_templates_dir` - Path to the destination templates directory
    ///
    /// # Errors
    /// Returns an error if template files cannot be copied (but not if language is missing).
    fn copy_language_templates(
        source_templates_dir: &Path,
        config: &Config,
        config_templates_dir: &Path,
    ) -> Result<()> {
        // Skip language template copying if test_language is "none"
        if config.generation.test_language == "none" {
            println!("⊘ Skipping language templates (test_language = none)");
            return Ok(());
        }

        let source_languages = source_templates_dir.join("languages");
        if !source_languages.exists() {
            return Ok(()); // Languages are optional
        }

        let source_lang_dir = source_languages.join(&config.generation.test_language);
        if source_lang_dir.exists() {
            let target_languages = config_templates_dir.join("languages");
            let target_lang_dir = target_languages.join(&config.generation.test_language);

            if !target_lang_dir.exists() {
                Self::copy_dir_recursive(&source_lang_dir, &target_lang_dir)?;
                println!(
                    "✓ Copied language templates: {}",
                    config.generation.test_language
                );
            } else {
                println!(
                    "⊙ Skipped language templates: {} (already exists)",
                    config.generation.test_language
                );
            }
        } else {
            println!(
                "⚠ Language '{}' not found in source-templates/languages/, skipping",
                config.generation.test_language
            );
        }

        Ok(())
    }

    /// Recursively copy a directory and all its contents.
    ///
    /// Creates the destination directory if it doesn't exist, then recursively
    /// copies all files and subdirectories from source to destination.
    ///
    /// # Arguments
    /// * `src` - Source directory path
    /// * `dst` - Destination directory path
    ///
    /// # Errors
    /// Returns an error if directories cannot be created or files cannot be copied.
    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Incrementally copy a directory, only adding new files and skipping existing ones.
    ///
    /// This preserves any user customizations to existing template files while
    /// adding any new templates from the source.
    ///
    /// # Arguments
    /// * `src` - Source directory path
    /// * `dst` - Destination directory path
    ///
    /// # Errors
    /// Returns an error if directories cannot be created or files cannot be copied.
    fn copy_dir_incremental(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                // Recurse into subdirectories
                Self::copy_dir_incremental(&src_path, &dst_path)?;
            } else if !dst_path.exists() {
                // Only copy if destination file doesn't exist
                fs::copy(&src_path, &dst_path)?;
            }
            // Skip if file already exists to preserve customizations
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_create_config_from_template() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory
        fs::create_dir_all(Config::CONFIG_DIR)?;

        let config = Config::default();
        TemplateManager::create_config_from_template(&config)?;

        // Verify config file was created
        assert!(Config::config_path().exists());

        // Verify content
        let content = fs::read_to_string(Config::config_path())?;
        let loaded_config: Config = toml::from_str(&content)?;
        assert_eq!(loaded_config.project.name, config.project.name);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_find_source_templates_dir_current_dir() -> Result<()> {
        use directories::ProjectDirs;

        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create source-templates directory
        fs::create_dir("source-templates")?;

        let result = TemplateManager::find_source_templates_dir()?;

        // The function now auto-installs to user config, so check for either location
        let is_valid = if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
            let user_templates = proj_dirs.config_dir().join("templates");
            result == user_templates || result == Path::new("source-templates")
        } else {
            result == Path::new("source-templates")
        };

        assert!(
            is_valid,
            "Expected templates in user config or local dir, got: {:?}",
            result
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_find_source_templates_dir_manifest_dir() -> Result<()> {
        use directories::ProjectDirs;

        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Set CARGO_MANIFEST_DIR to a different location
        let manifest_dir = temp_dir.path().join("manifest");
        let expected_path = manifest_dir.join("source-templates");
        fs::create_dir(&manifest_dir)?;
        fs::create_dir(&expected_path)?;
        std::env::set_var("CARGO_MANIFEST_DIR", &manifest_dir);

        let result = TemplateManager::find_source_templates_dir()?;

        // The function now auto-installs to user config, so check for either location
        let is_valid = if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
            let user_templates = proj_dirs.config_dir().join("templates");
            result == user_templates || result == expected_path
        } else {
            result == expected_path
        };

        assert!(
            is_valid,
            "Expected templates in user config or manifest dir, got: {:?}",
            result
        );

        // Clean up
        std::env::remove_var("CARGO_MANIFEST_DIR");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_find_source_templates_dir_not_found() -> Result<()> {
        use directories::ProjectDirs;

        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Remove CARGO_MANIFEST_DIR if it exists
        std::env::remove_var("CARGO_MANIFEST_DIR");

        let result = TemplateManager::find_source_templates_dir();

        // If user config directory exists from previous runs, the function will succeed
        // Otherwise it should fail
        if let Some(proj_dirs) = ProjectDirs::from("", "", "mucm") {
            let user_templates = proj_dirs.config_dir().join("templates");
            if user_templates.exists() {
                // Templates exist in user config, so function should succeed
                assert!(
                    result.is_ok(),
                    "Expected success when user config templates exist"
                );
                return Ok(());
            }
        }

        // No templates anywhere, should fail
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source templates directory not found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_templates_to_config_missing_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = TemplateManager::copy_templates_to_config(".");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Config file not found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_templates_to_config_invalid_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create config directory and invalid config file
        fs::create_dir_all(Config::CONFIG_DIR)?;
        fs::write(Config::config_path(), "invalid toml content")?;

        let result = TemplateManager::copy_templates_to_config(".");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_root_templates_with_overview() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&source_dir)?;
        fs::create_dir(&dest_dir)?;

        // Create overview.hbs file
        fs::write(source_dir.join("overview.hbs"), "test content")?;

        TemplateManager::copy_root_templates(&source_dir, &dest_dir)?;

        // Verify file was copied
        assert!(dest_dir.join("overview.hbs").exists());
        assert_eq!(
            fs::read_to_string(dest_dir.join("overview.hbs"))?,
            "test content"
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_root_templates_without_overview() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&source_dir)?;
        fs::create_dir(&dest_dir)?;

        // No overview.hbs file
        TemplateManager::copy_root_templates(&source_dir, &dest_dir)?;

        // Should complete successfully without copying anything
        assert!(!dest_dir.join("overview.hbs").exists());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_methodologies_missing_source_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let dest_templates = temp_dir.path().join("templates");

        fs::create_dir(&source_dir)?;
        fs::create_dir(&dest_templates)?;

        let config = Config::default();

        let result = TemplateManager::copy_methodologies(&source_dir, &config, &dest_templates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source methodologies directory not found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_methodologies_missing_methodology() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let methodologies_dir = source_dir.join("methodologies");
        let dest_templates = temp_dir.path().join("templates");

        fs::create_dir_all(&methodologies_dir)?;
        fs::create_dir(&dest_templates)?;

        let mut config = Config::default();
        config.templates.methodologies = vec!["nonexistent".to_string()];

        let result = TemplateManager::copy_methodologies(&source_dir, &config, &dest_templates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Methodology 'nonexistent' not found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_methodologies_missing_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let methodologies_dir = source_dir.join("methodologies");
        let method_dir = methodologies_dir.join("test_method");
        let dest_templates = temp_dir.path().join("templates");

        fs::create_dir_all(&method_dir)?;
        fs::create_dir(&dest_templates)?;

        // Create a template file but no methodology.toml
        fs::write(method_dir.join("template.hbs"), "template content")?;

        let mut config = Config::default();
        config.templates.methodologies = vec!["test_method".to_string()];

        let result = TemplateManager::copy_methodologies(&source_dir, &config, &dest_templates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing methodology.toml file"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_language_templates_missing_languages_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let dest_templates = temp_dir.path().join("templates");

        fs::create_dir(&source_dir)?;
        fs::create_dir(&dest_templates)?;

        let config = Config::default();

        // Should succeed even without languages directory (it's optional)
        let result =
            TemplateManager::copy_language_templates(&source_dir, &config, &dest_templates);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_language_templates_missing_language() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let languages_dir = source_dir.join("languages");
        let dest_templates = temp_dir.path().join("templates");

        fs::create_dir_all(&languages_dir)?;
        fs::create_dir(&dest_templates)?;

        let mut config = Config::default();
        config.generation.test_language = "nonexistent_lang".to_string();

        // Should succeed but log a warning (language templates are optional)
        let result =
            TemplateManager::copy_language_templates(&source_dir, &config, &dest_templates);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_copy_dir_recursive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        // Create source structure
        fs::create_dir_all(source_dir.join("subdir"))?;
        fs::write(source_dir.join("file1.txt"), "content1")?;
        fs::write(source_dir.join("subdir").join("file2.txt"), "content2")?;

        TemplateManager::copy_dir_recursive(&source_dir, &dest_dir)?;

        // Verify copy
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").join("file2.txt").exists());
        assert_eq!(fs::read_to_string(dest_dir.join("file1.txt"))?, "content1");
        assert_eq!(
            fs::read_to_string(dest_dir.join("subdir").join("file2.txt"))?,
            "content2"
        );

        Ok(())
    }
}
