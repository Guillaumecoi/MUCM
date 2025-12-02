//! Common test utilities shared across integration tests

use std::fs;
use std::path::Path;

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
