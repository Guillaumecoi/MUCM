//! Common test utilities shared across integration tests

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper for creating isolated test template environments
///
/// Creates a temporary directory with test templates and sets MUCM_TEST_TEMPLATES_DIR
/// to bypass user config caching. This ensures complete test isolation without
/// cache conflicts or race conditions in parallel test execution.
///
/// # Example
/// ```no_run
/// let _template_mgr = TestTemplateManager::new()?;
/// // Now all template lookups will use isolated test templates
/// ```
pub struct TestTemplateManager {
    #[allow(dead_code)] // Kept alive to prevent temp dir cleanup
    temp_dir: TempDir,
    templates_dir: PathBuf,
}

impl TestTemplateManager {
    /// Create isolated test template environment
    ///
    /// Sets up a temporary directory with minimal test templates and configures
    /// the environment to use them instead of cached user config templates.
    #[allow(dead_code)] // Used in integration test files
    pub fn new() -> std::io::Result<Self> {
        let temp_dir = TempDir::new()?;

        // Create source-templates structure in temp dir
        create_minimal_source_templates(temp_dir.path())?;

        let templates_dir = temp_dir.path().join("source-templates");

        // Set env var to bypass user config caching
        unsafe {
            std::env::set_var("MUCM_TEST_TEMPLATES_DIR", &templates_dir);
        }

        Ok(Self {
            temp_dir,
            templates_dir,
        })
    }

    /// Get path to templates directory
    #[allow(dead_code)] // May be used by tests for verification
    pub fn templates_dir(&self) -> &Path {
        &self.templates_dir
    }
}

impl Drop for TestTemplateManager {
    fn drop(&mut self) {
        // Clean up env var when test completes
        unsafe {
            std::env::remove_var("MUCM_TEST_TEMPLATES_DIR");
        }
    }
}

/// Create minimal source-templates structure for testing in Docker/CI environments
/// This allows tests to work when the actual source-templates directory is not available
pub fn create_minimal_source_templates(base_path: &Path) -> std::io::Result<()> {
    let templates_dir = base_path.join("source-templates");
    fs::create_dir_all(&templates_dir)?;

    // Create config.toml
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
aliases = []
"#,
                lang,
                if *lang == "python" {
                    "py"
                } else if *lang == "javascript" {
                    "js"
                } else {
                    "rs"
                }
            ),
        )?;
        fs::write(lang_dir.join("test.hbs"), "# Test template\n")?;
    }

    // Create minimal methodology structure
    let methodologies_dir = templates_dir.join("methodologies");
    for methodology in &["business", "developer", "feature", "tester"] {
        let method_dir = methodologies_dir.join(methodology);
        fs::create_dir_all(&method_dir)?;

        // Build methodology.toml content based on methodology type
        let content = match *methodology {
            "developer" => {
                r#"[methodology]
name = "developer"
abbreviation = "dev"
description = "Test developer"

[template]
preferred_style = "Normal"

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Normal level"
inherits = []

[levels.normal.custom_fields]
api_endpoints = { label = "API Endpoints", type = "array", required = false }
database_tables = { label = "Database Tables", type = "array", required = false }

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

[levels.advanced.custom_fields]
performance_requirements = { label = "Performance Requirements", type = "string", required = false }
security_considerations = { label = "Security Considerations", type = "array", required = false }
technical_dependencies = { label = "Technical Dependencies", type = "array", required = false }
error_scenarios = { label = "Error Scenarios", type = "array", required = false }

[usage]
when_to_use = []
key_features = []
"#
            }
            "feature" => {
                r#"[methodology]
name = "feature"
abbreviation = "fea"
description = "Test feature"

[template]
preferred_style = "Normal"

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Normal level"
inherits = []

[levels.normal.custom_fields]
user_segment = { label = "Target User Segment", type = "string", required = false }
success_metrics = { label = "Success Metrics", type = "array", required = false }
hypothesis = { label = "Product Hypothesis", type = "text", required = false }

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

[usage]
when_to_use = []
key_features = []
"#
            }
            _ => &format!(
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
business_goal = {{ label = "Business Goal", type = "string", required = false }}
success_criteria = {{ label = "Success Criteria", type = "array", required = false }}

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

[levels.advanced.custom_fields]
stakeholder_impact = {{ label = "Stakeholder Impact", type = "text", required = false }}
business_rules = {{ label = "Business Rules", type = "array", required = false }}

[usage]
when_to_use = []
key_features = []
"#,
                methodology,
                &methodology[..3],
                methodology
            ),
        };

        fs::write(method_dir.join("methodology.toml"), content)?;

        // Create minimal but functional templates with key placeholders
        let template_content = r#"# {{id}}: {{title}}

**Category:** {{category}}
**Status:** {{status}}
**Priority:** {{priority}}

## Description

{{#if description}}
{{description}}
{{else}}
_No description provided._
{{/if}}

## Summary

{{#if summary}}
{{summary}}
{{else}}
_No summary provided._
{{/if}}
"#;
        fs::write(method_dir.join("uc_normal.hbs"), template_content)?;
        fs::write(method_dir.join("uc_simple.hbs"), template_content)?;
        fs::write(method_dir.join("uc_detailed.hbs"), template_content)?;
        fs::write(method_dir.join("uc_advanced.hbs"), template_content)?;
    }

    // Create scenario templates directory
    let scenarios_dir = templates_dir.join("scenarios");
    fs::create_dir_all(&scenarios_dir)?;
    fs::write(
        scenarios_dir.join("scenario.hbs"),
        r#"# {{scenario_id}}: {{title}}

{{#if description}}
{{description}}
{{/if}}
"#,
    )?;
    fs::write(
        scenarios_dir.join("scenario_mermaid.hbs"),
        r#"```mermaid
sequenceDiagram
{{#each steps}}
    {{sender}} ->> {{receiver}}: {{action}}
{{/each}}
```
"#,
    )?;

    // Create actor template
    fs::write(
        templates_dir.join("actor.hbs"),
        r#"# {{name}}

{{#if description}}
{{description}}
{{/if}}
"#,
    )?;

    // Create overview template
    fs::write(
        templates_dir.join("overview.hbs"),
        r#"# Use Cases Overview

{{#each use_cases}}
- {{id}}: {{title}}
{{/each}}
"#,
    )?;

    Ok(())
}
