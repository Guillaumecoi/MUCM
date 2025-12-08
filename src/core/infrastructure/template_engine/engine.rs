use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TemplateEngine {
    handlebars: RefCell<Handlebars<'static>>,
    /// Map of language name to template name for test generation
    /// TODO: Use this when implementing test file generation feature
    test_templates: HashMap<String, String>,
    methodologies: Vec<String>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();

        // Register custom helpers for actor and persona support
        super::helpers::register_helpers(&mut handlebars);

        // First try to load templates from user's config directory
        // Then fall back to source-templates if not found
        let user_templates_path = Path::new(".config/.mucm")
            .join(crate::config::Config::TEMPLATES_DIR)
            .join("methodologies");
        let source_templates_path = Path::new("source-templates/methodologies").to_path_buf();

        let methodologies_path = if user_templates_path.exists() {
            &user_templates_path
        } else {
            &source_templates_path
        };

        let mut methodologies = Vec::new();

        if methodologies_path.exists() {
            for entry in fs::read_dir(methodologies_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(methodology_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Register all uc_*.hbs templates for this methodology
                        // This allows users to add custom levels beyond simple/normal/detailed
                        for template_entry in fs::read_dir(&path)? {
                            let template_entry = template_entry?;
                            let template_path = template_entry.path();

                            if template_path.is_file() {
                                if let Some(filename) =
                                    template_path.file_name().and_then(|n| n.to_str())
                                {
                                    // Only register uc_*.hbs files
                                    if filename.starts_with("uc_") && filename.ends_with(".hbs") {
                                        // Extract level name from filename (e.g., "uc_simple.hbs" -> "simple")
                                        let level_name = filename
                                            .strip_prefix("uc_")
                                            .and_then(|s| s.strip_suffix(".hbs"))
                                            .unwrap_or(filename);

                                        let template = fs::read_to_string(&template_path)?;
                                        handlebars.register_template_string(
                                            &format!("{}-{}", methodology_name, level_name),
                                            template,
                                        )?;
                                    }
                                }
                            }
                        }

                        methodologies.push(methodology_name.to_string());
                    }
                }
            }
        }

        // Load test templates from languages directory
        let user_languages_path = Path::new(".config/.mucm")
            .join(crate::config::Config::TEMPLATES_DIR)
            .join("languages");
        let source_languages_path = Path::new("source-templates/languages").to_path_buf();

        let languages_path = if user_languages_path.exists() {
            &user_languages_path
        } else {
            &source_languages_path
        };

        let mut test_templates = HashMap::new();

        if languages_path.exists() {
            for entry in fs::read_dir(languages_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(language_name) = path.file_name().and_then(|n| n.to_str()) {
                        let test_hbs_path = path.join("test.hbs");
                        if test_hbs_path.exists() {
                            let template = fs::read_to_string(&test_hbs_path)?;
                            let template_name = format!("test-{}", language_name);
                            handlebars.register_template_string(&template_name, template)?;
                            test_templates.insert(language_name.to_string(), template_name);
                        }
                    }
                }
            }
        }

        // Register general overview template (not methodology-specific)
        // Overview.hbs is at the root of template-assets, not in methodologies subdirectory
        let overview_path = if user_templates_path.parent().is_some()
            && user_templates_path.parent().unwrap().exists()
        {
            user_templates_path.parent().unwrap().join("overview.hbs") // .config/.mucm/{TEMPLATES_DIR}/overview.hbs
        } else {
            Path::new("source-templates/overview.hbs").to_path_buf()
        };
        if overview_path.exists() {
            let template = fs::read_to_string(overview_path)?;
            handlebars.register_template_string("overview", template)?;
        } else {
            // If no overview template found, register a default one matching the new category structure
            let default_overview_template = r#"# Use Cases Overview

**Project:** {{project_name}}  
**Generated:** {{generated_date}}

## Summary
- **Total Use Cases:** {{total_use_cases}}
- **Total Categories:** {{total_categories}}

## Categories

{{#each categories}}
### [{{category_name}}](./{{category_path}}/README.md)

{{#if use_case_count}}**Use Cases:** {{use_case_count}}{{/if}}

---

{{/each}}
"#;
            handlebars.register_template_string("overview", default_overview_template)?;
        }

        // Register category overview template
        let category_overview_path = if user_templates_path.parent().is_some()
            && user_templates_path.parent().unwrap().exists()
        {
            user_templates_path
                .parent()
                .unwrap()
                .join("category_overview.hbs")
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("source-templates/category_overview.hbs")
        };
        if category_overview_path.exists() {
            let template = fs::read_to_string(category_overview_path)?;
            handlebars.register_template_string("category_overview", template)?;
        } else {
            // If no category overview template found, register a default one
            let default_category_overview_template = r#"# {{category_name}} Use Cases

## Use Cases

{{#each use_cases}}
- [{{id}}]({{id}}/README.md) - {{title}}
  - Status: {{aggregated_status}}
  - Priority: {{priority}}
{{/each}}
"#;
            handlebars.register_template_string(
                "category_overview",
                default_category_overview_template,
            )?;
        }

        // Register use case overview template (multi-view README)
        let use_case_overview_path = if user_templates_path.parent().is_some()
            && user_templates_path.parent().unwrap().exists()
        {
            user_templates_path
                .parent()
                .unwrap()
                .join("use_case_overview.hbs")
        } else {
            Path::new("source-templates/use_case_overview.hbs").to_path_buf()
        };
        if use_case_overview_path.exists() {
            let template = fs::read_to_string(use_case_overview_path)?;
            handlebars.register_template_string("use_case_overview", template)?;
        }

        // Register language test templates using LanguageRegistry
        // let mut test_templates = HashMap::new();  // Remove redeclaration

        use super::super::languages::LanguageRegistry;
        use crate::config::TemplateManager;

        // Try to load language templates, but don't fail if source templates not available
        match TemplateManager::find_source_templates_dir() {
            Ok(templates_dir) => {
                match LanguageRegistry::new_dynamic(&templates_dir) {
                    Ok(language_registry) => {
                        for language_name in language_registry.available_languages() {
                            if !test_templates.contains_key(&language_name) {
                                if let Some(language) = language_registry.get(&language_name) {
                                    let template_name = format!("{}_test", language.name());
                                    handlebars.register_template_string(
                                        &template_name,
                                        language.test_template(),
                                    )?;
                                    test_templates.insert(language.name().to_string(), template_name);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // If language registry fails, continue without language templates
                        // This allows the template engine to work in test environments
                    }
                }
            }
            Err(_) => {
                // If source templates not available, continue without language templates
                // This allows the template engine to work in test environments
            }
        }

        // If no test templates were found (e.g., in test environments), provide defaults
        if test_templates.is_empty() {
            let default_languages = vec!["rust", "python", "javascript"];
            let default_test_template = r#"# Test for {{title}}

This is a generated test file for the use case: {{title}}

Use case ID: {{id}}
Category: {{category}}
Priority: {{priority}}

Description: {{description}}

Status: {{status}}

Generated at: {{generated_at}}
"#;

            for lang in default_languages {
                let template_name = format!("{}_test", lang);
                handlebars.register_template_string(&template_name, default_test_template)?;
                test_templates.insert(lang.to_string(), template_name);
            }
        }

        // If no methodologies were found (e.g., in test environments), provide defaults
        if methodologies.is_empty() {
            methodologies = vec![
                "business".to_string(),
                "developer".to_string(),
                "feature".to_string(),
                "tester".to_string(),
            ];

            // Register default templates for these methodologies
            // These are simple fallback templates that just output the use case data
            let default_template = r#"# {{title}}

**ID:** {{id}}
**Category:** {{category}}
**Priority:** {{priority}}

## Description
{{description}}

## Status
{{status}}

## Metadata
- **Created:** {{created}}
- **Last Updated:** {{last_updated}}
"#;

            for methodology in &methodologies {
                handlebars.register_template_string(
                    &format!("{}-simple", methodology),
                    default_template,
                )?;
                handlebars.register_template_string(
                    &format!("{}-normal", methodology),
                    default_template,
                )?;
                handlebars.register_template_string(
                    &format!("{}-detailed", methodology),
                    default_template,
                )?;
            }
        }

        Ok(TemplateEngine {
            handlebars: RefCell::new(handlebars),
            test_templates,
            methodologies,
        })
    }

    pub fn with_config(_config: Option<&crate::config::Config>) -> Self {
        Self::new().unwrap()
    }

    pub fn render_overview(&self, data: &HashMap<String, Value>) -> Result<String> {
        self.handlebars
            .borrow()
            .render("overview", data)
            .context("Failed to render overview template")
    }

    pub fn render_category_overview(&self, data: &HashMap<String, Value>) -> Result<String> {
        self.handlebars
            .borrow()
            .render("category_overview", data)
            .context("Failed to render category overview template")
    }

    pub fn render_use_case_overview(&self, data: &HashMap<String, Value>) -> Result<String> {
        self.handlebars
            .borrow()
            .render("use_case_overview", data)
            .context("Failed to render use case overview template")
    }

    /// Render use case with specific template
    pub fn render_use_case_with_template(
        &self,
        template_name: &str,
        data: &HashMap<String, Value>,
    ) -> Result<String> {
        self.handlebars
            .borrow()
            .render(template_name, data)
            .with_context(|| format!("Failed to render use case with template: {}", template_name))
    }

    /// Render test file for a specific language
    pub fn render_test(&self, language: &str, data: &HashMap<String, Value>) -> Result<String> {
        let language_lower = language.to_lowercase();
        let template_key = self
            .test_templates
            .get(&language_lower)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

        self.handlebars
            .borrow()
            .render(template_key, data)
            .with_context(|| format!("Failed to render {} test template", language))
    }

    /// Render use case with methodology-specific template
    /// Simple version: just renders the template with the data - no processing
    /// Defaults to "normal" level
    pub fn render_use_case_with_methodology(
        &self,
        data: &HashMap<String, Value>,
        methodology: &str,
    ) -> Result<String> {
        // Delegate to render_use_case_with_methodology_and_level with "normal" level
        self.render_use_case_with_methodology_and_level(data, methodology, "normal")
    }

    /// Render a use case with specific methodology and level
    pub fn render_use_case_with_methodology_and_level(
        &self,
        data: &HashMap<String, Value>,
        methodology: &str,
        level: &str,
    ) -> Result<String> {
        let template_name = format!("{}-{}", methodology, level);
        if self
            .handlebars
            .borrow()
            .get_template(&template_name)
            .is_none()
        {
            anyhow::bail!(
                "Invalid source-templates: Methodology '{}' does not have a valid uc_{}.hbs template. \
                Check source-templates/methodologies/{}/uc_{}.hbs exists and is valid.",
                methodology, level, methodology, level
            );
        }

        // Load scenario template for this level and register as partial
        self.register_scenario_partial_for_level(methodology, level)?;

        self.render_use_case_with_template(&template_name, data)
    }

    /// Resolve the path to a scenario template based on the template path specification
    ///
    /// # Path Resolution Rules
    /// - If path contains `/`: relative to source-templates root (e.g., "scenarios/scenario.hbs")
    /// - If path is filename only: relative to methodology directory (e.g., "custom-scenario.hbs")
    /// - Paths use forward slashes in TOML, converted to OS-specific separators by PathBuf
    fn resolve_scenario_template_path(
        templates_dir: &Path,
        methodology: &str,
        scenario_template_path: &str,
    ) -> PathBuf {
        if scenario_template_path.contains('/') {
            // Absolute path from source-templates root
            // e.g., "scenarios/scenario.hbs" -> source-templates/scenarios/scenario.hbs
            templates_dir.join(scenario_template_path)
        } else {
            // Relative to methodology directory
            // e.g., "business-scenario.hbs" -> source-templates/methodologies/business/business-scenario.hbs
            templates_dir
                .join("methodologies")
                .join(methodology)
                .join(scenario_template_path)
        }
    }

    /// Register the scenario partial for a specific methodology and level
    /// This loads the scenario template configured for the level and registers it as "scenario"
    /// If the scenario template cannot be loaded, this returns Ok(()) to allow rendering to continue
    fn register_scenario_partial_for_level(&self, methodology: &str, level: &str) -> Result<()> {
        use super::super::methodologies::MethodologyDefinition;

        // Determine templates directory
        let user_templates_path =
            Path::new(".config/.mucm").join(crate::config::Config::TEMPLATES_DIR);
        let source_templates_path = Path::new("source-templates").to_path_buf();

        let templates_dir = if user_templates_path.exists() {
            &user_templates_path
        } else {
            &source_templates_path
        };

        // Load methodology definition to get scenario_template config
        let methodology_dir = templates_dir.join("methodologies").join(methodology);
        let methodology_def = match MethodologyDefinition::from_toml(&methodology_dir) {
            Ok(def) => def,
            Err(e) => {
                // If we can't load the methodology definition, just skip scenario partial registration
                // This allows templates without scenario_template config to continue working
                eprintln!(
                    "Warning: Could not load methodology definition for '{}': {}",
                    methodology, e
                );
                return Ok(());
            }
        };

        // Get level config
        let level_config = match methodology_def.level_configs.get(level) {
            Some(config) => config,
            None => {
                eprintln!(
                    "Warning: Level '{}' not found in methodology '{}'",
                    level, methodology
                );
                return Ok(());
            }
        };

        // Determine scenario template path with fallback hierarchy:
        // 1. Level-specific override from methodology.toml
        // 2. Global default from mucm.toml config
        // 3. Hard-coded default "scenarios/scenario.hbs"
        let default_from_config = crate::config::Config::load()
            .ok()
            .map(|c| c.templates.default_scenario_template);

        let scenario_template_path = level_config
            .scenario_template
            .as_deref()
            .or(default_from_config.as_deref())
            .unwrap_or("scenarios/scenario.hbs");

        // Resolve full path
        let full_path = Self::resolve_scenario_template_path(
            templates_dir,
            methodology,
            scenario_template_path,
        );

        // Load and register as "scenario" partial
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)
                .with_context(|| format!("Failed to read scenario template: {:?}", full_path))?;

            self.handlebars
                .borrow_mut()
                .register_partial("scenario", content)
                .with_context(|| {
                    format!("Failed to register scenario partial from {:?}", full_path)
                })?;
        } else {
            // Scenario template not found - this is optional, so just warn
            eprintln!("Warning: Scenario template not found at {:?}", full_path);
        }

        // Register all other .hbs files in the scenarios directory as partials
        let scenarios_dir = templates_dir.join("scenarios");
        if scenarios_dir.exists() && scenarios_dir.is_dir() {
            for entry in
                fs::read_dir(&scenarios_dir).context("Failed to read scenarios directory")?
            {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Register all .hbs files except scenario.hbs (already registered)
                        if filename.ends_with(".hbs") && filename != "scenario.hbs" {
                            let partial_name = filename.strip_suffix(".hbs").unwrap_or(filename);
                            let content = fs::read_to_string(&path)
                                .with_context(|| format!("Failed to read partial: {:?}", path))?;

                            self.handlebars
                                .borrow_mut()
                                .register_partial(partial_name, content)
                                .with_context(|| {
                                    format!(
                                        "Failed to register partial '{}' from {:?}",
                                        partial_name, path
                                    )
                                })?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get available methodologies
    pub fn available_methodologies(&self) -> Vec<String> {
        self.methodologies.clone()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
