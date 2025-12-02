// Test helper functions for application service tests

use crate::config::Config;
use crate::core::LanguageRegistry;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Create minimal source-templates structure for testing
fn create_minimal_source_templates(base_path: &Path) -> std::io::Result<()> {
    let templates_dir = base_path.join("source-templates");
    fs::create_dir_all(&templates_dir)?;

    // Create config.toml with all required sections
    fs::write(
        templates_dir.join("config.toml"),
        r#"[project]
name = "Test Project"
description = "Test"

[directories]
use_case_dir = "docs/use-cases"
test_dir = "tests/use-cases"
actor_dir = "docs/actors"
data_dir = "use-cases-data"

[templates]
methodologies = ["business", "developer", "feature", "tester"]
default_methodology = "feature"
default_scenario_template = "scenarios/scenario.hbs"

[generation]
test_language = "none"
auto_generate_tests = false
overwrite_test_documentation = false

[storage]
backend = "toml"

[metadata]
created = true
last_updated = true
"#,
    )?;

    // Create minimal language structure
    let languages_dir = templates_dir.join("languages");
    for lang in &["rust", "python", "javascript"] {
        let lang_dir = languages_dir.join(lang);
        fs::create_dir_all(&lang_dir)?;
        fs::write(
            lang_dir.join("info.toml"),
            format!(
                r#"name = "{}"
file_extension = "{}"
template_file = "test.hbs"
"#,
                lang,
                if *lang == "python" { "py" } else if *lang == "javascript" { "js" } else { "rs" }
            ),
        )?;
        fs::write(lang_dir.join("test.hbs"), "# Test template\n")?;
    }

    // Create minimal methodology structure
    let methodologies_dir = templates_dir.join("methodologies");
    for methodology in &["business", "developer", "feature", "tester"] {
        let method_dir = methodologies_dir.join(methodology);
        fs::create_dir_all(&method_dir)?;
        fs::write(
            method_dir.join("methodology.toml"),
            format!(
                r#"[methodology]
name = "{}"
abbreviation = "{}"
description = "Test {}"

[template]
preferred_style = "Normal"

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Normal level"
inherits = []

[levels.simple]
name = "Simple"
abbreviation = "s"
filename = "uc_simple.hbs"
description = "Simple level"
inherits = []

[levels.detailed]
name = "Detailed"
abbreviation = "d"
filename = "uc_detailed.hbs"
description = "Detailed level"
inherits = []

[levels.advanced]
name = "Advanced"
abbreviation = "a"
filename = "uc_advanced.hbs"
description = "Advanced level"
inherits = ["Normal"]
"#,
                methodology,
                &methodology[..3],
                methodology
            ),
        )?;
        fs::write(method_dir.join("uc_normal.hbs"), "# Template\n")?;
        fs::write(method_dir.join("uc_simple.hbs"), "# Template\n")?;
        fs::write(method_dir.join("uc_detailed.hbs"), "# Template\n")?;
        fs::write(method_dir.join("uc_advanced.hbs"), "# Template\n")?;
    }

    Ok(())
}

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

    // Create minimal source-templates for testing
    create_minimal_source_templates(Path::new("."))?;
    
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
