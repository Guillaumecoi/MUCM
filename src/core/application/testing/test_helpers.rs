// Test helper functions for application service tests

use crate::config::Config;
use crate::core::LanguageRegistry;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Helper to initialize a project for tests.
///
/// This function sets up a test project environment by:
/// - Creating the .config/.mucm directory
/// - Configuring the default Config with optional language settings
/// - Copying templates from source to the config directory
///
/// # Arguments
///
/// * `language` - Optional language to configure for test generation
///
/// # Returns
///
/// Returns the configured `Config` instance or an error
///
/// # Example
///
/// ```no_run
/// use tempfile::TempDir;
/// use std::env;
///
/// let temp_dir = TempDir::new()?;
/// env::set_current_dir(&temp_dir)?;
///
/// let config = init_test_project(Some("rust".to_string()))?;
/// ```
pub fn init_test_project(language: Option<String>) -> Result<Config> {
    let config_dir = Path::new(".config/.mucm");
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    let mut config = Config::default();

    if let Some(ref lang) = language {
        // Try to find source templates directory, but don't fail if not found
        match crate::config::TemplateManager::find_source_templates_dir() {
            Ok(templates_dir) => {
                let language_registry = LanguageRegistry::new_dynamic(&templates_dir)?;
                if let Some(lang_def) = language_registry.get(lang) {
                    let primary_name = lang_def.name().to_string();
                    config.generation.test_language = primary_name.clone();
                } else {
                    config.generation.test_language = lang.clone();
                }
            }
            Err(_) => {
                // Source templates not available, just set language directly
                config.generation.test_language = lang.clone();
            }
        }
    }

    config.save_in_dir(".")?;

    // Only try to copy templates if source templates directory exists
    if language.is_some() {
        if crate::config::TemplateManager::find_source_templates_dir().is_ok() {
            Config::copy_templates_to_config_with_language(language)?;
        }
    } else if crate::config::TemplateManager::find_source_templates_dir().is_ok() {
        Config::copy_templates_to_config_with_language(None)?;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// RAII guard that restores the original working directory on drop
    struct DirGuard {
        original: PathBuf,
    }

    impl DirGuard {
        fn new() -> Result<Self> {
            Ok(Self {
                original: env::current_dir()?,
            })
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

    #[test]
    #[serial]
    fn test_init_test_project_creates_config_dir() -> Result<()> {
        let _guard = DirGuard::new()?;
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        env::set_current_dir(temp_path)?;

        init_test_project(None)?;

        let config_dir = Path::new(".config/.mucm");
        assert!(config_dir.exists(), "Config directory should be created");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_init_test_project_with_language() -> Result<()> {
        let _guard = DirGuard::new()?;
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        env::set_current_dir(temp_path)?;

        let config = init_test_project(Some("python".to_string()))?;

        assert_eq!(config.generation.test_language, "python");
        assert_eq!(config.generation.test_language, "python");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_init_test_project_without_language() -> Result<()> {
        let _guard = DirGuard::new()?;
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        env::set_current_dir(temp_path)?;

        let config = init_test_project(None)?;

        // Should have default values
        assert!(!config.generation.test_language.is_empty());

        Ok(())
    }
}
