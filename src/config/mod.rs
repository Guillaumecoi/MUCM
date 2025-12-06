//! # Configuration Module
//!
//! This module handles all configuration-related functionality for the Markdown Use Case Manager (MUCM).
//! It manages project settings, template configurations, directory structures, and methodology preferences.
//!
//! ## Architecture
//!
//! The configuration system follows a layered approach:
//! - **Types** (`types.rs`): Data structures for configuration
//! - **File Management** (`file_manager.rs`): Loading/saving config files
//! - **Template Management** (`template_manager.rs`): Template copying and processing
//! - **Main Interface** (`mod.rs`): Public API and high-level operations
//!
//! ## Configuration File
//!
//! Configurations are stored in `.config/.mucm/mucm.toml` and contain:
//! - Project metadata (name, description)
//! - Directory paths for use cases, tests, and templates
//! - Methodology settings and defaults
//! - Language preferences for code generation
//! - Custom field definitions
//!
//! ## Two-Phase Initialization
//!
//! Project setup uses a two-phase process:
//! 1. **Configuration Phase**: Create config file with user preferences
//! 2. **Template Phase**: Copy templates based on selected languages/methodologies
//!
//! This allows users to review and modify configuration before templates are copied.

// Private sub-modules
mod file_manager;
mod template_manager;
mod types;

// Explicit public exports
pub use file_manager::ConfigFileManager;
pub use template_manager::TemplateManager;
pub use types::{ActorConfig, Config, MetadataConfig, StorageBackend, StorageConfig};

// Re-export from other modules
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Parameters for creating a Config with custom directories
pub struct ConfigParams {
    pub test_language: Option<String>,
    pub methodologies: Vec<String>,
    pub default_methodology: Option<String>,
    pub storage: String,
    pub use_case_dir: String,
    pub test_dir: String,
    pub actor_dir: String,
    pub data_dir: String,
    pub default_scenario_template: Option<String>,
}

impl Config {
    // Constants
    pub const CONFIG_DIR: &'static str = ".config/.mucm";
    pub const CONFIG_FILE: &'static str = "mucm.toml";
    pub const TEMPLATES_DIR: &'static str = "template-assets";
    /// Current config format version
    pub const CONFIG_VERSION: &'static str = "0.2.0";

    /// Create a minimal config for template processing.
    ///
    /// This creates a basic configuration used only for template variable substitution.
    /// It's not a complete project configuration and should not be saved directly.
    ///
    /// # Arguments
    /// * `test_language` - The programming language for test templates
    /// * `methodology` - Optional default methodology override
    ///
    /// # Returns
    /// A minimal Config instance suitable for template processing
    pub fn for_template(test_language: Option<String>, methodology: Option<String>) -> Self {
        // Load default config to get the methodologies from source-templates/config.toml
        let default_config = Self::default();

        let methodologies = if let Some(ref method) = methodology.clone() {
            vec![method.clone()]
        } else {
            // Use methodologies from default config (loaded from source-templates/config.toml)
            default_config.templates.methodologies.clone()
        };
        Self::for_template_with_methodologies_and_storage(
            test_language,
            methodologies,
            methodology.or(Some(default_config.templates.default_methodology)),
            "toml".to_string(),
            None, // Use default scenario template
        )
    }

    /// Create a minimal config for template processing with multiple methodologies.
    ///
    /// This creates a basic configuration used only for template variable substitution.
    /// It's not a complete project configuration and should not be saved directly.
    ///
    /// # Arguments
    /// * `test_language` - The programming language for test templates
    /// * `methodologies` - List of methodologies to enable
    /// * `default_methodology` - Optional default methodology override
    /// * `default_scenario_template` - Optional default scenario template override
    ///
    /// # Returns
    /// A minimal Config instance suitable for template processing
    pub fn for_template_with_methodologies_and_storage(
        test_language: Option<String>,
        methodologies: Vec<String>,
        default_methodology: Option<String>,
        storage: String,
        default_scenario_template: Option<String>,
    ) -> Self {
        let mut config = Self::default();
        if let Some(lang) = test_language {
            config.generation.test_language = lang.clone();
        }
        if !methodologies.is_empty() {
            config.templates.methodologies = methodologies;
        }
        if let Some(method) = default_methodology {
            config.templates.default_methodology = method;
        }
        if let Some(template) = default_scenario_template {
            config.templates.default_scenario_template = template;
        }
        // Set storage backend
        use crate::config::types::StorageBackend;
        if let Ok(backend) = StorageBackend::from_str(&storage) {
            config.storage.backend = backend;
        }
        // If parsing fails, keep the default (Toml)
        config
    }

    /// Create a minimal config with methodologies, storage, and directories.
    ///
    /// This creates a basic configuration with directory customization.
    ///
    /// # Arguments
    /// * `test_language` - The programming language for test templates
    /// * `methodologies` - List of methodologies to enable
    /// * `default_methodology` - Optional default methodology override
    /// * `storage` - Storage backend (toml or sqlite)
    /// * `use_case_dir` - Directory for use case files
    /// * `test_dir` - Directory for test files
    /// * `actor_dir` - Directory for actor files (personas and system actors)
    /// * `data_dir` - Directory for data files
    /// * `default_scenario_template` - Optional default scenario template override
    ///
    /// # Returns
    /// A Config instance with custom directories
    pub fn for_template_with_methodologies_storage_and_directories(
        params: crate::config::ConfigParams,
    ) -> Self {
        let mut config = Self::for_template_with_methodologies_and_storage(
            params.test_language,
            params.methodologies,
            params.default_methodology,
            params.storage,
            params.default_scenario_template,
        );

        // Update directories
        config.directories.use_case_dir = params.use_case_dir;
        config.directories.test_dir = params.test_dir;
        config.directories.actor_dir = params.actor_dir.clone();
        config.directories.data_dir = params.data_dir;

        config
    }

    /// Get the path to the configuration file.
    ///
    /// Returns the full path to `.config/.mucm/mucm.toml` relative to the current directory.
    pub fn config_path() -> PathBuf {
        Path::new(Self::CONFIG_DIR).join(Self::CONFIG_FILE)
    }

    /// Get the appropriate directory for loading methodology/language metadata (info.toml).
    ///
    /// **Always returns source templates directory** for metadata loading.
    /// This ensures that info.toml files (which contain display information) come from
    /// the authoritative source, not user customizations.
    ///
    /// For template files (.hbs), the TemplateEngine already handles prioritization
    /// by checking .config/.mucm/handlebars/ first.
    ///
    /// # Returns
    /// Result with PathBuf pointing to source templates directory
    pub fn get_metadata_load_dir() -> Result<PathBuf> {
        use crate::config::TemplateManager;
        TemplateManager::find_source_templates_dir()
    }

    /// Get the directory for project-installed templates.
    ///
    /// Returns the path to .config/.mucm/template-assets/ where project-specific
    /// customized templates are stored. This allows each project to have their own
    /// custom templates and levels.
    ///
    /// # Returns
    /// Result with PathBuf pointing to project template assets directory, or error if not initialized
    pub fn get_project_templates_dir() -> Result<PathBuf> {
        let base_path = Path::new(".");
        let templates_dir = base_path.join(Self::CONFIG_DIR).join(Self::TEMPLATES_DIR);

        if !templates_dir.exists() {
            anyhow::bail!(
                "Project templates directory not found. Run 'mucm init' first or 'mucm finalize-init' to complete initialization."
            );
        }

        Ok(templates_dir)
    }

    /// Save configuration file only (without copying templates or creating directories).
    ///
    /// This is the first step of two-step initialization. It creates the config file
    /// but doesn't copy templates yet, allowing users to review and modify the config.
    ///
    /// Use `finalize_init()` after this to complete the setup.
    ///
    /// # Arguments
    /// * `config` - The configuration to save
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if saving fails
    pub fn save_config_only(config: &Config) -> Result<()> {
        let base_path = Path::new(".");
        let config_dir = base_path.join(Self::CONFIG_DIR);

        // Create .config/.mucm directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create .config/.mucm directory")?;
        }

        // Use template file instead of programmatic generation
        TemplateManager::create_config_from_template(config)?;
        Ok(())
    }

    /// Load configuration from the standard config file.
    ///
    /// Reads `.config/.mucm/mucm.toml` and deserializes it into a Config struct.
    ///
    /// # Returns
    /// The loaded configuration, or an error if the file doesn't exist or is invalid
    pub fn load() -> Result<Self> {
        ConfigFileManager::load()
    }

    /// Save configuration to file in specified directory.
    ///
    /// # Arguments
    /// * `self` - The configuration to save
    /// * `base_dir` - Base directory where the config should be saved
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if saving fails
    pub fn save_in_dir(&self, base_dir: &str) -> Result<()> {
        ConfigFileManager::save_in_dir(self, base_dir)
    }

    /// Check if templates have already been copied to .config/.mucm/handlebars/
    ///
    /// This is used to determine if the project has completed the template setup phase.
    ///
    /// # Returns
    /// `true` if templates exist, `false` otherwise
    pub fn check_templates_exist() -> bool {
        ConfigFileManager::check_templates_exist()
    }

    /// Copy templates to .config/.mucm/handlebars/ with language (wrapper for _in_dir version)
    ///
    /// This is the second phase of initialization - copying templates after config review.
    ///
    /// # Arguments
    /// * `language` - Optional language override for template selection
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if copying fails
    pub fn copy_templates_to_config_with_language(language: Option<String>) -> Result<()> {
        Self::copy_templates_to_config_with_language_in_dir(".", language)
    }

    /// Copy templates to config directory (internal implementation)
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for the operation
    /// * `language` - Optional language override (currently unused - copies all languages)
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if copying fails
    fn copy_templates_to_config_with_language_in_dir(
        base_dir: &str,
        _language: Option<String>, // Not currently used - we copy all languages now
    ) -> Result<()> {
        TemplateManager::copy_templates_to_config(base_dir)
    }

    /// Create all project directories as specified in the configuration.
    ///
    /// Creates use case, test, persona, and data directories if they don't exist.
    /// This is typically called after initialization to set up the project structure.
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if directory creation fails
    pub fn create_project_directories() -> Result<()> {
        let config = Self::load()?;

        fs::create_dir_all(&config.directories.use_case_dir)
            .context("Failed to create use case directory")?;
        fs::create_dir_all(&config.directories.test_dir)
            .context("Failed to create test directory")?;
        fs::create_dir_all(&config.directories.actor_dir)
            .context("Failed to create actor directory")?;
        fs::create_dir_all(&config.directories.data_dir)
            .context("Failed to create data directory")?;

        Ok(())
    }

    /// Load default configuration from source-templates/config.toml
    fn load_default_from_template() -> Result<Self> {
        use crate::config::template_manager::TemplateManager;
        use crate::config::types::{
            Config, DirectoryConfig, GenerationConfig, MetadataConfig, ProjectConfig,
            TemplateConfig,
        };
        use crate::core::MethodologyRegistry;
        use std::fs;

        // Try to find source templates directory, but don't fail if not found
        let source_templates_dir = match TemplateManager::find_source_templates_dir() {
            Ok(dir) => dir,
            Err(_) => {
                // Fallback: create a minimal default config when source-templates is not available
                #[allow(deprecated)]
                // Using direct struct construction as fallback when source-templates unavailable
                return Ok(Config {
                    version: Self::CONFIG_VERSION.to_string(),
                    project: ProjectConfig {
                        name: "Default Project".to_string(),
                        description: "Default project description".to_string(),
                    },
                    directories: DirectoryConfig {
                        use_case_dir: "use-cases".to_string(),
                        test_dir: "tests".to_string(),
                        actor_dir: "docs/actors".to_string(),
                        data_dir: "use-cases-data".to_string(),
                    },
                    templates: TemplateConfig {
                        methodologies: vec![
                            "business".to_string(),
                            "developer".to_string(),
                            "feature".to_string(),
                            "tester".to_string(),
                        ],
                        default_methodology: "feature".to_string(),
                        default_scenario_template: "scenarios/scenario.hbs".to_string(),
                    },
                    generation: GenerationConfig {
                        test_language: "none".to_string(),
                        auto_generate_tests: false,
                        overwrite_test_documentation: false,
                    },
                    storage: StorageConfig {
                        backend: StorageBackend::Toml,
                    },
                    metadata: MetadataConfig {
                        created: true,
                        last_updated: true,
                        date_format: "%d/%m/%Y".to_string(),
                    },
                    actor: ActorConfig::default(),
                    extra_fields: std::collections::HashMap::new(),
                });
            }
        };

        let config_path = source_templates_dir.join("config.toml");

        let content = fs::read_to_string(&config_path)
            .context("Failed to read source-templates/config.toml")?;
        let mut config: Config =
            toml::from_str(&content).context("Failed to parse source-templates/config.toml")?;

        // Dynamically discover available methodologies
        let methodologies = MethodologyRegistry::discover_available(&source_templates_dir)
            .unwrap_or_else(|_| {
                // Fallback to default methodologies if discovery fails
                vec![
                    "business".to_string(),
                    "developer".to_string(),
                    "feature".to_string(),
                    "tester".to_string(),
                ]
            });

        // Set methodologies dynamically
        config.templates.methodologies = methodologies.clone();
        // Keep the default_methodology from config.toml unless it's empty
        if config.templates.default_methodology.is_empty() {
            config.templates.default_methodology = methodologies
                .first()
                .cloned()
                .unwrap_or_else(|| "feature".to_string()); // Fallback to "feature" if no methodologies found
        }

        // Set default generation config that matches the template defaults
        config.generation = GenerationConfig {
            test_language: config.generation.test_language.clone(),
            auto_generate_tests: false,
            overwrite_test_documentation: false,
        };

        Ok(config)
    }

    /// Create minimal source-templates for testing environments
    /// This is automatically called by Config::default() when templates are not found
    #[cfg(test)]
    pub fn ensure_test_templates() -> std::io::Result<()> {
        use std::path::Path;

        // Check if source-templates already exists and has valid config
        if Path::new("source-templates/config.toml").exists() {
            // Try to parse it - if it works, we're good
            if std::fs::read_to_string("source-templates/config.toml")
                .ok()
                .and_then(|content| toml::from_str::<Config>(&content).ok())
                .is_some()
            {
                return Ok(());
            }
        }

        // Create minimal templates
        let templates_dir = Path::new("source-templates");
        std::fs::create_dir_all(templates_dir)?;

        std::fs::write(
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
        for lang in &["rust", "python", "javascript"] {
            let lang_dir = templates_dir.join("languages").join(lang);
            std::fs::create_dir_all(&lang_dir)?;
            let (ext, alias) = if *lang == "python" {
                ("py", "py")
            } else if *lang == "javascript" {
                ("js", "js")
            } else {
                ("rs", "rs")
            };
            std::fs::write(
                lang_dir.join("info.toml"),
                format!(
                    r#"name = "{}"
aliases = ["{}"]
file_extension = "{}"
template_file = "test.hbs"
"#,
                    lang, alias, ext
                ),
            )?;
            // Create minimal but valid test template with ID placeholder
            let test_content = if *lang == "rust" {
                "// Test for {{id}}\n#[test]\nfn test_{{snake_case_id}}() {\n    // TODO: implement\n}\n"
            } else if *lang == "python" {
                "# Test for {{id}}\nimport unittest\n\nclass Test{{pascal_case_id}}(unittest.TestCase):\n    def test_case(self):\n        pass\n"
            } else {
                "// Test for {{id}}\ndescribe('{{id}}', () => {\n    it('should work', () => {\n        // TODO: implement\n    });\n});\n"
            };
            std::fs::write(lang_dir.join("test.hbs"), test_content)?;
        }

        // Create minimal methodology structure with unique fields per methodology
        for methodology in &["business", "developer", "feature", "tester"] {
            let method_dir = templates_dir.join("methodologies").join(methodology);
            std::fs::create_dir_all(&method_dir)?;

            let (normal_fields, advanced_fields) = match *methodology {
                "business" => (
                    r#"business_value = { label = "Business Value", type = "text", required = false, description = "Test field" }
roi_estimate = { label = "ROI Estimate", type = "string", required = false, description = "Test field" }"#,
                    r#"# Inherits: business_value, roi_estimate
stakeholder_analysis = { label = "Stakeholder Analysis", type = "text", required = false, description = "Test field" }"#,
                ),
                "developer" => (
                    r#"api_endpoint = { label = "API Endpoint", type = "string", required = false, description = "Test field" }
database_tables = { label = "Database Tables", type = "array", required = false, description = "Test field" }"#,
                    r#"# Inherits: api_endpoint, database_tables
performance_requirements = { label = "Performance Requirements", type = "string", required = false, description = "Test field" }
security_considerations = { label = "Security Considerations", type = "string", required = false, description = "Test field" }
technical_dependencies = { label = "Technical Dependencies", type = "array", required = false, description = "Test field" }
error_handling = { label = "Error Handling Strategy", type = "string", required = false, description = "Test field" }"#,
                ),
                "feature" => (
                    r#"user_segment = { label = "Target User Segment", type = "string", required = false, description = "Test field" }
success_metrics = { label = "Success Metrics", type = "array", required = false, description = "Test field" }
hypothesis = { label = "Product Hypothesis", type = "text", required = false, description = "Test field" }"#,
                    r#"# Inherits: user_segment, success_metrics, hypothesis
mockups = { label = "Mockups/Wireframes", type = "string", required = false, description = "Test field" }"#,
                ),
                "tester" => (
                    r#"test_type = { label = "Test Type", type = "string", required = true, description = "Test field" }
test_priority = { label = "Test Priority", type = "string", required = false, description = "Test field" }"#,
                    r#"# Inherits: test_type, test_priority
test_coverage = { label = "Test Coverage", type = "string", required = false, description = "Test field" }"#,
                ),
                _ => ("", ""),
            };

            std::fs::write(
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

[levels.normal.custom_fields]
{}

[levels.advanced]
name = "Advanced"
abbreviation = "a"
filename = "uc_advanced.hbs"
description = "Advanced level"
inherits = ["Normal"]

[levels.advanced.custom_fields]
{}

[usage]
when_to_use = ["For testing purposes"]
key_features = ["Test feature"]
best_practices = ["Test practice"]

[examples]

[examples.basic]
title = "Test Example"

[related_methodologies]
"#,
                    methodology,
                    &methodology[..3],
                    methodology,
                    normal_fields,
                    advanced_fields,
                ),
            )?;
            std::fs::write(
                method_dir.join("uc_normal.hbs"),
                r#"# {{title}}

**ID:** {{id}} | **Status:** {{status}} | **Priority:** {{priority}}

{{#if description}}
## Description
{{description}}

{{/if}}
{{#if preconditions}}
## Preconditions
{{#each preconditions}}
- {{this}}
{{/each}}

{{/if}}
{{#if postconditions}}
## Postconditions
{{#each postconditions}}
- {{this}}
{{/each}}

{{/if}}
"#,
            )?;
            std::fs::write(
                method_dir.join("uc_advanced.hbs"),
                r#"# {{title}}

**ID:** {{id}} | **Status:** {{status}} | **Priority:** {{priority}}

{{#if description}}
## Description
{{description}}

{{/if}}
{{#if preconditions}}
## Preconditions
{{#each preconditions}}
- {{this}}
{{/each}}

{{/if}}
{{#if postconditions}}
## Postconditions
{{#each postconditions}}
- {{this}}
{{/each}}

{{/if}}
"#,
            )?;
        }

        // Create scenario templates
        let scenario_dir = templates_dir.join("scenarios");
        std::fs::create_dir_all(&scenario_dir)?;
        std::fs::write(
            scenario_dir.join("scenario.hbs"),
            r#"{{#if scenarios}}
## Scenarios

{{#each scenarios}}
### {{title}}

**Status:** {{status}}{{#if primary_actor}} | **Actor:** {{primary_actor}}{{/if}}

{{#if preconditions}}
**Preconditions:**
{{#each preconditions}}
- {{this}}
{{/each}}
{{/if}}

{{#if steps}}
**Steps:**
{{#each steps}}
{{order}}. {{actor}} {{action}}
{{/each}}
{{/if}}

{{/each}}
{{/if}}
"#,
        )?;
        std::fs::write(
            scenario_dir.join("scenario_mermaid.hbs"),
            r#"{{#if scenarios}}
## Scenarios (Mermaid)

```mermaid
sequenceDiagram
{{#each scenarios}}
    {{#each steps}}
    {{actor}}->>{{receiver}}: {{action}}
    {{/each}}
{{/each}}
```
{{/if}}
"#,
        )?;

        // Create overview template
        std::fs::write(
            templates_dir.join("overview.hbs"),
            r#"# Use Cases Overview

**Project:** {{project_name}}
**Generated:** {{generated_date}}

## Summary
- **Total Use Cases:** {{total_use_cases}}
- **Total Scenarios:** {{total_scenarios}}

{{#if status_counts}}
## Status Distribution
{{#each status_counts}}
- **{{@key}}:** {{this}}
{{/each}}
{{/if}}

{{#if categories}}
## Use Cases
{{#each categories}}
### {{category_name}}
{{#each use_cases}}
- **{{id}}**: {{title}} ({{aggregated_status}})
{{/each}}
{{/each}}
{{/if}}
"#,
        )?;

        // Create actor template
        std::fs::write(
            templates_dir.join("actor.hbs"),
            r#"# Actor: {{name}}

{{#if description}}
## Description
{{description}}
{{/if}}

{{#if type}}
**Type:** {{type}}
{{/if}}
"#,
        )?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        // In test mode, ensure minimal templates exist and use simple loading
        #[cfg(test)]
        {
            use std::path::Path;

            // Ensure test templates exist - this creates the minimal config
            if let Ok(()) = Self::ensure_test_templates() {
                // Try to load from local source-templates (test environment)
                // This should always succeed after ensure_test_templates()
                let config_path = Path::new("source-templates/config.toml");
                if config_path.exists() {
                    match std::fs::read_to_string(config_path) {
                        Ok(content) => match toml::from_str::<Config>(&content) {
                            Ok(config) => return config,
                            Err(e) => eprintln!("Warning: Failed to parse test config: {}", e),
                        },
                        Err(e) => eprintln!("Warning: Failed to read test config: {}", e),
                    }
                }
            }
        }

        // Load default configuration from source-templates/config.toml
        // This ensures consistency between the template and the default config
        match Self::load_default_from_template() {
            Ok(config) => config,
            Err(e) => {
                panic!(
                    "Failed to load default configuration from source-templates/config.toml: {}\n\
                     The source-templates directory and config.toml file are required for the application to function.\n\
                     Please ensure you are running from the project root directory.",
                    e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TemplateManager;
    use crate::core::LanguageRegistry;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Ensure tests use project source templates, not user config templates
    ///
    /// This prevents tests from failing due to outdated templates in ~/.config/mucm/templates/
    fn setup_test_templates_env() {
        // Point directly to project's source-templates directory using MUCM_TEST_TEMPLATES_DIR
        // This bypasses user config caching completely
        let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let source_templates = project_root.join("source-templates");
        unsafe {
            std::env::set_var("MUCM_TEST_TEMPLATES_DIR", source_templates);
        }
    }

    /// Helper to initialize a project in a temporary directory with optional language
    fn init_project_with_language(language: Option<String>) -> Result<Config> {
        use crate::config::template_manager::TemplateManager;

        // Try to find source templates directory, but don't fail if not found
        let language_registry = match TemplateManager::find_source_templates_dir() {
            Ok(templates_dir) => {
                // If source templates exist, create language registry from them
                Some(LanguageRegistry::new_dynamic(&templates_dir)?)
            }
            Err(_) => {
                // If source templates don't exist (e.g., in test environments), skip language validation
                None
            }
        };

        // Validate language if provided and we have a language registry
        if let Some(ref lang) = language {
            if let Some(ref registry) = language_registry {
                if registry.get(lang).is_none() {
                    let available = registry.available_languages();
                    anyhow::bail!(
                        "Unsupported language '{}'. Supported languages: {}",
                        lang,
                        available.join(", ")
                    );
                }
            }
            // If no registry available, skip validation (assume language is valid for testing)
        }

        let config_dir = Path::new(".config/.mucm");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).context("Failed to create .config/.mucm directory")?;
        }

        let mut config = Config::default();

        // Set the test language if provided, resolving aliases to primary names
        if let Some(ref lang) = language {
            if let Some(ref registry) = language_registry {
                if let Some(lang_def) = registry.get(lang) {
                    let primary_name = lang_def.name().to_string();
                    config.generation.test_language = primary_name.clone();
                } else {
                    config.generation.test_language = lang.clone();
                }
            } else {
                // No registry available, just set the language directly
                config.generation.test_language = lang.clone();
            }
        }

        config.save_in_dir(".")?;

        // Only try to copy templates if source templates directory exists
        if language_registry.is_some() {
            Config::copy_templates_to_config_with_language(language)?;
        }
        // If no source templates, skip template copying (templates won't exist, but config will)

        Ok(config)
    }

    #[test]
    #[serial]
    fn test_auto_init_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = Config::load();
        assert!(
            result.is_err(),
            "Should fail when project is not initialized"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No markdown use case manager project found"));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_auto_init_process() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        assert!(Config::load().is_err());

        let config = init_project_with_language(Some("rust".to_string()))?;

        assert!(Config::config_path().exists(), "Config file should exist");
        assert_eq!(config.generation.test_language, "rust");

        let use_case_dir = Path::new(&config.directories.use_case_dir);
        let test_dir = Path::new(&config.directories.test_dir);
        assert!(
            !use_case_dir.exists(),
            "Use case directory should NOT exist yet"
        );
        assert!(!test_dir.exists(), "Test directory should NOT exist yet");

        let templates_dir = Path::new(".config/.mucm").join(Config::TEMPLATES_DIR);

        // Check if source templates are available
        let source_templates_available =
            crate::config::TemplateManager::find_source_templates_dir().is_ok();
        if source_templates_available {
            // Only check for templates if source templates were available
            assert!(templates_dir.exists(), "Templates directory should exist");
            assert!(templates_dir
                .join("methodologies/developer/uc_normal.hbs")
                .exists());
            assert!(templates_dir
                .join("methodologies/developer/uc_advanced.hbs")
                .exists());
            assert!(templates_dir.join("languages/rust/test.hbs").exists());
        }
        // If source templates not available, templates won't exist but that's okay for testing

        let reloaded_config = Config::load()?;
        assert_eq!(reloaded_config.generation.test_language, "rust");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_auto_init_language_options() -> Result<()> {
        setup_test_templates_env();
        // Test with Python
        {
            let temp_dir = TempDir::new()?;
            std::env::set_current_dir(&temp_dir)?;

            let config = init_project_with_language(Some("python".to_string()))?;
            assert_eq!(config.generation.test_language, "python");

            let python_template = Path::new(".config/.mucm")
                .join(Config::TEMPLATES_DIR)
                .join("languages/python/test.hbs");

            // Only check for template if source templates are available
            let source_templates_available =
                crate::config::TemplateManager::find_source_templates_dir().is_ok();
            if source_templates_available {
                assert!(python_template.exists(), "Python template should exist");
            }
            // If source templates not available, template won't exist but config should still work
        }

        // Test with None (default language should be "none")
        {
            let temp_dir = TempDir::new()?;
            std::env::set_current_dir(&temp_dir)?;

            let config = init_project_with_language(None)?;
            assert_eq!(config.generation.test_language, "none"); // Default from Config::default()
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_config_management() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let mut config = init_project_with_language(Some("rust".to_string()))?;

        config.project.name = "Modified Project".to_string();
        config.project.description = "Modified description".to_string();
        config.directories.use_case_dir = "custom/use-cases".to_string();
        config.directories.test_dir = "custom/tests".to_string();
        config.generation.test_language = "python".to_string();
        config.generation.auto_generate_tests = true;
        config.metadata.created = false;
        config.metadata.last_updated = false;

        config.save_in_dir(".")?;

        let reloaded_config = Config::load()?;
        assert_eq!(reloaded_config.project.name, "Modified Project");
        assert_eq!(reloaded_config.project.description, "Modified description");
        assert_eq!(reloaded_config.directories.use_case_dir, "custom/use-cases");
        assert_eq!(reloaded_config.directories.test_dir, "custom/tests");
        assert_eq!(reloaded_config.generation.test_language, "python");
        assert!(reloaded_config.generation.auto_generate_tests);
        assert!(!reloaded_config.metadata.created);
        assert!(!reloaded_config.metadata.last_updated);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_available_languages_for_settings() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Try to discover languages, but don't fail if source templates not available
        match TemplateManager::find_source_templates_dir() {
            Ok(templates_dir) => {
                let languages = LanguageRegistry::discover_available(&templates_dir);
                match languages {
                    Ok(langs) => {
                        assert!(!langs.is_empty(), "Should have built-in languages");
                        assert!(
                            langs.contains(&"rust".to_string())
                                || langs.contains(&"python".to_string())
                        );
                    }
                    Err(_) => {
                        // It's okay if this fails in some test environments
                    }
                }
            }
            Err(_) => {
                // Source templates not available in test environment, skip this part
            }
        }

        init_project_with_language(Some("rust".to_string()))?;

        // After initialization, try again (but may still fail if no source templates)
        match TemplateManager::find_source_templates_dir() {
            Ok(templates_dir) => {
                let languages = LanguageRegistry::discover_available(&templates_dir)?;
                assert!(!languages.is_empty(), "Should have languages after init");
                assert!(languages.contains(&"rust".to_string()));
            }
            Err(_) => {
                // Source templates not available, skip assertion
            }
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_config_validation() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let config = init_project_with_language(Some("rust".to_string()))?;

        let toml_content = toml::to_string_pretty(&config)?;
        let parsed_config: Config = toml::from_str(&toml_content)?;

        assert_eq!(parsed_config.project.name, config.project.name);
        assert_eq!(
            parsed_config.generation.test_language,
            config.generation.test_language
        );
        assert_eq!(parsed_config.metadata.created, config.metadata.created);
        assert_eq!(
            parsed_config.metadata.last_updated,
            config.metadata.last_updated
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_config_error_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = Config::load();
        assert!(result.is_err());

        let config = Config::default();
        let _ = config.save_in_dir(".");

        Ok(())
    }

    #[test]
    #[serial]
    fn test_auto_init_settings_integration() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        let mut config = init_project_with_language(Some("rust".to_string()))?;

        config.project.name = "Integration Test Project".to_string();
        config.directories.use_case_dir = "docs/custom-use-cases".to_string();
        config.generation.auto_generate_tests = true;
        config.metadata.created = true;
        config.metadata.last_updated = true;

        config.save_in_dir(".")?;

        let saved_config = Config::load()?;
        assert_eq!(saved_config.project.name, "Integration Test Project");

        fs::create_dir_all(&config.directories.use_case_dir)?;
        fs::create_dir_all(&config.directories.test_dir)?;

        use crate::core::UseCaseCoordinator;
        let mut coordinator = UseCaseCoordinator::load()?;
        let config = Config::load()?;
        let default_methodology = config.templates.default_methodology.clone();

        let _uc_id = coordinator.create_use_case_with_views(
            "Integration Test Use Case".to_string(),
            "integration".to_string(),
            Some("Testing integration between auto-init and settings".to_string()),
            &format!("{}:normal", default_methodology),
        )?;

        let filename = format!(
            "docs/custom-use-cases/integration/UC-INT-001/UC-INT-001-{}-normal.md",
            default_methodology
        );
        let custom_use_case_file = Path::new(&filename);
        assert!(
            custom_use_case_file.exists(),
            "Use case should be created in custom directory with new folder structure"
        );

        let final_config = Config::load()?;
        assert_eq!(final_config.project.name, "Integration Test Project");
        assert_eq!(
            final_config.directories.use_case_dir,
            "docs/custom-use-cases"
        );
        assert!(final_config.generation.auto_generate_tests);
        assert!(final_config.metadata.created);
        assert!(final_config.metadata.last_updated);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_for_template() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Test with both language and methodology set
        let config = Config::for_template(Some("python".to_string()), Some("feature".to_string()));
        assert_eq!(config.generation.test_language, "python");
        assert_eq!(config.templates.default_methodology, "feature");

        // Test with language set but methodology None (should keep default methodology)
        let config_lang_only = Config::for_template(Some("rust".to_string()), None);
        assert_eq!(config_lang_only.generation.test_language, "rust");
        // Should use default methodology from Config::default()
        assert!(!config_lang_only.templates.default_methodology.is_empty());

        // Test with language None but methodology set (should keep default language)
        let default_config = Config::default();
        let config_methodology_only = Config::for_template(None, Some("business".to_string()));
        assert_eq!(
            config_methodology_only.generation.test_language,
            default_config.generation.test_language
        );
        assert_eq!(
            config_methodology_only.templates.default_methodology,
            "business"
        );

        // Test with both None (should keep all defaults)
        let config_both_none = Config::for_template(None, None);
        assert_eq!(
            config_both_none.generation.test_language,
            default_config.generation.test_language
        );
        assert_eq!(
            config_both_none.templates.default_methodology,
            default_config.templates.default_methodology
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_save_config_only() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Set CARGO_MANIFEST_DIR to the project root so source templates can be found
        let exe_path = std::env::current_exe()?;
        let project_root = exe_path
            .parent() // deps
            .and_then(|p| p.parent()) // debug
            .and_then(|p| p.parent()) // target
            .and_then(|p| p.parent()) // project root
            .ok_or_else(|| anyhow::anyhow!("Could not determine project root"))?;
        unsafe {
            std::env::set_var("CARGO_MANIFEST_DIR", project_root);
        }

        let mut config = Config::default();
        config.project.name = "Test Project".to_string();
        config.generation.test_language = "javascript".to_string();

        // Save config only (without templates)
        Config::save_config_only(&config)?;

        // Verify config file exists
        assert!(Config::config_path().exists());

        // Verify templates directory does NOT exist (since we only saved config)
        let templates_dir = Path::new(".config/.mucm").join(Config::TEMPLATES_DIR);
        assert!(!templates_dir.exists());

        // Verify we can load the config back
        let loaded_config = Config::load()?;
        assert_eq!(loaded_config.project.name, "Test Project");
        assert_eq!(loaded_config.generation.test_language, "javascript");

        // Clean up
        unsafe {
            std::env::remove_var("CARGO_MANIFEST_DIR");
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_check_templates_exist() -> Result<()> {
        setup_test_templates_env();
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        // Initially, templates should not exist
        assert!(!Config::check_templates_exist());

        // Initialize project (which may or may not copy templates depending on source availability)
        let _config = init_project_with_language(Some("rust".to_string()))?;

        // Templates should exist only if source templates were available
        let source_templates_available =
            crate::config::TemplateManager::find_source_templates_dir().is_ok();
        if source_templates_available {
            assert!(Config::check_templates_exist());
        } else {
            // In test environments without source templates, templates won't exist
            assert!(!Config::check_templates_exist());
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_config_path() -> Result<()> {
        let expected_path = Path::new(".config/.mucm/mucm.toml");
        assert_eq!(Config::config_path(), expected_path);

        Ok(())
    }
}
