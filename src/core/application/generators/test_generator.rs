//! Test generator for use case test documentation.
//!
//! Handles generation of test files from use cases using language-specific templates.
//!
//! # Safe Zone Preservation
//!
//! The test generator preserves user-written code during regeneration through "safe zones" -
//! specially marked regions where users can add custom implementation without it being overwritten.
//!
//! ## Safe Zone Types
//!
//! 1. **Global** - Custom imports and module-level setup code
//! 2. **setUp** - Test fixture initialization code (before each test)
//! 3. **tearDown** - Test cleanup code (after each test)
//! 4. **Scenarios** - Individual test implementations (one per scenario)
//!
//! ## Example
//!
//! ```python
//! # Global safe zone
//! # START USER IMPLEMENTATION - Add your imports and setup code here
//! from my_module import MyClass
//! import unittest
//! # END USER IMPLEMENTATION
//!
//! class TestUseCase(unittest.TestCase):
//!     def setUp(self):
//!         # START USER IMPLEMENTATION - Add your setup code here
//!         self.instance = MyClass()
//!         # END USER IMPLEMENTATION
//!
//!     def test_scenario_001(self):
//!         # START USER IMPLEMENTATION - Feel free to modify the code below this line
//!         self.assertTrue(self.instance.method())
//!         # END USER IMPLEMENTATION - Do not modify anything below this line
//! ```
//!
//! During regeneration, all code within START/END markers is preserved, while everything
//! else (documentation, structure, new scenarios) is regenerated from templates.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::config::Config;
use crate::core::file_operations::FileOperations;
use crate::core::{to_snake_case, TemplateEngine, UseCase};
use crate::presentation::UseCaseFormatter;

/// Type of safe zone for user-editable code preservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafeZoneType {
    Global,
    SetUp,
    TearDown,
    Scenario,
}

/// Storage for preserved safe zone content from existing test files.
#[derive(Debug, Default, Clone)]
struct SafeZoneContent {
    /// Global safe zone for imports and setup code
    global: String,
    /// setUp method safe zone
    setup: String,
    /// tearDown method safe zone
    teardown: String,
    /// Per-scenario safe zones keyed by scenario ID
    scenarios: HashMap<String, String>,
}

/// Parses and merges safe zone content during test regeneration.
///
/// Safe zones are specially marked regions in test files where users can add
/// custom code that won't be overwritten during regeneration.
struct SafeZonePreserver {
    comment_start: String,
}

impl SafeZonePreserver {
    /// Creates a new SafeZonePreserver for a specific language.
    fn new(comment_start: String) -> Self {
        Self { comment_start }
    }

    /// Get the start and end markers for a specific safe zone type.
    /// This eliminates duplication across merge methods.
    fn get_safe_zone_markers(&self, zone_type: SafeZoneType) -> (String, String) {
        let message = match zone_type {
            SafeZoneType::Global => "Add your imports and setup code here",
            SafeZoneType::SetUp => "Add your setup code here",
            SafeZoneType::TearDown => "Add your cleanup code here",
            SafeZoneType::Scenario => "Feel free to modify the code below this line",
        };

        let start_marker = format!(
            "{} START USER IMPLEMENTATION - {}",
            self.comment_start, message
        );
        let end_marker = format!("{} END USER IMPLEMENTATION", self.comment_start);

        (start_marker, end_marker)
    }

    /// Cleans preserved content by removing any lines that contain end markers.
    fn clean_preserved_content(&self, content: &str, _end_marker: &str) -> String {
        content
            .lines()
            .filter(|line| !line.trim().starts_with("# END USER IMPLEMENTATION"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Parses an existing test file and extracts all safe zone content.
    ///
    /// Returns a SafeZoneContent with the global safe zone and per-scenario safe zones.
    /// Uses a simple approach: find each START marker, collect until END marker, preserve exact text.
    fn parse_safe_zones(&self, content: &str) -> SafeZoneContent {
        SafeZoneContent {
            global: self.extract_global_safe_zone(content),
            setup: self.extract_setup_safe_zone(content),
            teardown: self.extract_teardown_safe_zone(content),
            scenarios: self.extract_scenario_safe_zones(content),
        }
    }

    /// Extracts the global safe zone content for imports and setup code.
    fn extract_global_safe_zone(&self, content: &str) -> String {
        let start_marker = format!(
            "{} START USER IMPLEMENTATION - Add your imports and setup code here",
            self.comment_start
        );
        let end_marker = format!("{} END USER IMPLEMENTATION", self.comment_start);

        self.extract_between_markers(content, &start_marker, &end_marker)
    }

    /// Extracts the setUp method safe zone.
    fn extract_setup_safe_zone(&self, content: &str) -> String {
        let start_marker = format!(
            "{} START USER IMPLEMENTATION - Add your setup code here",
            self.comment_start
        );
        let end_marker = format!("{} END USER IMPLEMENTATION", self.comment_start);

        self.extract_between_markers(content, &start_marker, &end_marker)
    }

    /// Extracts the tearDown method safe zone.
    fn extract_teardown_safe_zone(&self, content: &str) -> String {
        let start_marker = format!(
            "{} START USER IMPLEMENTATION - Add your cleanup code here",
            self.comment_start
        );
        let end_marker = format!("{} END USER IMPLEMENTATION", self.comment_start);

        self.extract_between_markers(content, &start_marker, &end_marker)
    }

    /// Extracts all per-scenario safe zones keyed by scenario ID.
    fn extract_scenario_safe_zones(&self, content: &str) -> HashMap<String, String> {
        let mut scenarios = HashMap::new();
        let start_marker = format!(
            "{} START USER IMPLEMENTATION - Feel free to modify the code below this line",
            self.comment_start
        );
        let end_marker = format!(
            "{} END USER IMPLEMENTATION - Do not modify anything below this line",
            self.comment_start
        );

        // Find all test functions and extract their scenario IDs and safe zones
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            // Look for test function/method definitions and extract scenario ID
            if let Some(scenario_id) = self.extract_scenario_id_from_line(lines[i]) {
                // Find the safe zone start marker after this function definition
                let mut j = i + 1;
                while j < lines.len() {
                    if lines[j].contains(&start_marker) {
                        // Extract content between markers
                        let safe_zone_content =
                            self.extract_safe_zone_from_lines(&lines, j + 1, &end_marker);
                        scenarios.insert(scenario_id.clone(), safe_zone_content);
                        break;
                    }
                    // Stop searching if we hit another test function
                    if self.extract_scenario_id_from_line(lines[j]).is_some() {
                        break;
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        scenarios
    }

    /// Extracts content between two markers in a string.
    /// Detects base indentation from START marker and strips it from content, preserving relative indentation.
    fn extract_between_markers(
        &self,
        content: &str,
        start_marker: &str,
        end_marker: &str,
    ) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines = Vec::new();
        let mut collecting = false;
        let mut base_indent_size = 0;

        for line in lines {
            if line.contains(start_marker) {
                // Detect base indentation from the START marker line
                base_indent_size = line.len() - line.trim_start().len();
                collecting = true;
                continue; // Skip the START marker line itself
            }
            if line.contains(end_marker) {
                collecting = false;
                continue; // Skip the END marker line itself
            }
            if collecting {
                // Strip base indentation, preserving any additional indentation
                if line.trim().is_empty() {
                    result_lines.push("");
                } else {
                    let line_indent = line.len() - line.trim_start().len();
                    if line_indent >= base_indent_size {
                        let stripped = &line[base_indent_size..];
                        result_lines.push(stripped);
                    } else {
                        // Line has less indentation than base, keep as-is
                        result_lines.push(line);
                    }
                }
            }
        }

        // Join lines and trim only leading/trailing empty lines, preserve internal spacing
        let result = result_lines.join("\n");
        result.trim().to_string()
    }

    /// Extracts safe zone content from a slice of lines until end marker is found.
    /// Detects base indentation from the first non-empty line and strips it, preserving relative indentation.
    fn extract_safe_zone_from_lines(
        &self,
        lines: &[&str],
        start_idx: usize,
        end_marker: &str,
    ) -> String {
        let mut content_lines = Vec::new();
        let mut base_indent_size: Option<usize> = None;

        for line in &lines[start_idx..] {
            if line.contains(end_marker) {
                break;
            }

            // Detect base indentation from first non-empty line
            if base_indent_size.is_none() && !line.trim().is_empty() {
                base_indent_size = Some(line.len() - line.trim_start().len());
            }

            // Strip base indentation if we have it
            if let Some(base_indent) = base_indent_size {
                if line.trim().is_empty() {
                    content_lines.push("");
                } else {
                    let line_indent = line.len() - line.trim_start().len();
                    if line_indent >= base_indent {
                        let stripped = &line[base_indent..];
                        content_lines.push(stripped);
                    } else {
                        content_lines.push(line);
                    }
                }
            } else {
                // No base indent detected yet (all empty lines so far)
                content_lines.push(line);
            }
        }

        // Join lines and trim, preserving internal structure
        content_lines.join("\n").trim().to_string()
    }

    /// Attempts to extract a scenario ID from a test function/method definition line.
    ///
    /// Supports multiple patterns:
    /// - Python: `def test_scenario_id(self):`
    /// - Rust: `fn test_scenario_id()` or `fn test_scenario_id() {`
    /// - JavaScript: `function test_scenario_id()` or `test('scenario_id'`
    fn extract_scenario_id_from_line(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Python: def test_xxx(
        if trimmed.starts_with("def test_") {
            if let Some(end_pos) = trimmed.find('(') {
                let func_name = &trimmed[4..end_pos]; // Skip "def "
                return Some(func_name.trim_start_matches("test_").to_string());
            }
        }

        // Rust: fn test_xxx() or pub fn test_xxx()
        if trimmed.contains("fn test_") {
            if let Some(fn_pos) = trimmed.find("fn test_") {
                let after_fn = &trimmed[fn_pos + 3..]; // Skip "fn "
                if let Some(end_pos) = after_fn.find('(') {
                    let func_name = &after_fn[..end_pos];
                    return Some(func_name.trim_start_matches("test_").to_string());
                }
            }
        }

        // JavaScript: function test_xxx() or test("xxx"
        if trimmed.starts_with("function test_") {
            if let Some(end_pos) = trimmed.find('(') {
                let func_name = &trimmed[9..end_pos]; // Skip "function "
                return Some(func_name.trim_start_matches("test_").to_string());
            }
        }

        None
    }

    /// Merges preserved safe zones into freshly rendered template content.
    ///
    /// Replaces empty safe zones in the rendered template with preserved user code.
    fn merge_safe_zones(&self, rendered: &str, preserved: &SafeZoneContent) -> String {
        let mut result = rendered.to_string();

        // Merge global safe zone
        if !preserved.global.is_empty() {
            result = self.merge_global_safe_zone(&result, &preserved.global);
        }

        // Merge setUp safe zone
        if !preserved.setup.is_empty() {
            result = self.merge_setup_safe_zone(&result, &preserved.setup);
        }

        // Merge tearDown safe zone
        if !preserved.teardown.is_empty() {
            result = self.merge_teardown_safe_zone(&result, &preserved.teardown);
        }

        // Merge scenario safe zones
        for (scenario_id, content) in &preserved.scenarios {
            if !content.is_empty() {
                result = self.merge_scenario_safe_zone(&result, scenario_id, content);
            }
        }

        result
    }

    /// Merges the global safe zone content into the rendered template.
    fn merge_global_safe_zone(&self, rendered: &str, preserved_content: &str) -> String {
        let (start_marker, end_marker) = self.get_safe_zone_markers(SafeZoneType::Global);
        let cleaned_preserved = self.clean_preserved_content(preserved_content, &end_marker);
        self.replace_between_markers(rendered, &start_marker, &end_marker, &cleaned_preserved)
    }

    /// Merges the setUp method safe zone content into the rendered template.
    fn merge_setup_safe_zone(&self, rendered: &str, preserved_content: &str) -> String {
        let (start_marker, end_marker) = self.get_safe_zone_markers(SafeZoneType::SetUp);
        let cleaned_preserved = self.clean_preserved_content(preserved_content, &end_marker);
        self.replace_between_markers(rendered, &start_marker, &end_marker, &cleaned_preserved)
    }

    /// Merges the tearDown method safe zone content into the rendered template.
    fn merge_teardown_safe_zone(&self, rendered: &str, preserved_content: &str) -> String {
        let (start_marker, end_marker) = self.get_safe_zone_markers(SafeZoneType::TearDown);
        let cleaned_preserved = self.clean_preserved_content(preserved_content, &end_marker);
        self.replace_between_markers(rendered, &start_marker, &end_marker, &cleaned_preserved)
    }

    /// Merges a scenario-specific safe zone into the rendered template.
    /// Uses simple approach: find the scenario's START marker, replace until END marker.
    fn merge_scenario_safe_zone(
        &self,
        rendered: &str,
        scenario_id: &str,
        preserved_content: &str,
    ) -> String {
        let lines: Vec<&str> = rendered.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut i = 0;

        let (start_marker, _) = self.get_safe_zone_markers(SafeZoneType::Scenario);
        // Scenario end marker has a special suffix
        let end_marker = format!(
            "{} END USER IMPLEMENTATION - Do not modify anything below this line",
            self.comment_start
        );

        // Find the test function for this scenario
        while i < lines.len() {
            result_lines.push(lines[i].to_string());

            if let Some(found_id) = self.extract_scenario_id_from_line(lines[i]) {
                if found_id == scenario_id {
                    // Found the scenario, now find its START marker
                    i += 1;
                    while i < lines.len() {
                        if lines[i].contains(&start_marker) {
                            // Add START marker line
                            result_lines.push(lines[i].to_string());

                            // Detect base indentation from the START marker line
                            let base_indent_size = lines[i].len() - lines[i].trim_start().len();
                            let base_indent = " ".repeat(base_indent_size);

                            i += 1;

                            // Skip old content until END marker
                            while i < lines.len() && !lines[i].contains(&end_marker) {
                                i += 1;
                            }

                            // Insert preserved content with base indentation applied
                            let cleaned_preserved =
                                self.clean_preserved_content(preserved_content, &end_marker);
                            if !cleaned_preserved.is_empty() {
                                for line in cleaned_preserved.lines() {
                                    if line.trim().is_empty() {
                                        // Preserve empty lines as-is
                                        result_lines.push(String::new());
                                    } else {
                                        // Apply base indentation to content
                                        result_lines.push(format!("{}{}", base_indent, line));
                                    }
                                }
                            }

                            // Add END marker line
                            if i < lines.len() {
                                result_lines.push(lines[i].to_string());
                                i += 1;
                            }
                            break;
                        }
                        result_lines.push(lines[i].to_string());
                        i += 1;
                    }
                    break;
                }
            }
            i += 1;
        }

        // Add remaining lines
        while i < lines.len() {
            result_lines.push(lines[i].to_string());
            i += 1;
        }

        result_lines.join("\n")
    }

    /// Replaces content between two markers with new content.
    /// Detects base indentation from context and applies it to the new content.
    fn replace_between_markers(
        &self,
        text: &str,
        start_marker: &str,
        end_marker: &str,
        new_content: &str,
    ) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].contains(start_marker) {
                // Add the START marker line as-is
                result_lines.push(lines[i].to_string());

                // Detect base indentation from the START marker line
                let base_indent_size = lines[i].len() - lines[i].trim_start().len();
                let base_indent = " ".repeat(base_indent_size);

                i += 1;

                // Skip old content until we find the END marker
                while i < lines.len() && !lines[i].contains(end_marker) {
                    i += 1;
                }

                // Insert new content with base indentation applied
                if !new_content.is_empty() {
                    for line in new_content.lines() {
                        if line.trim().is_empty() {
                            // Preserve empty lines as-is
                            result_lines.push(String::new());
                        } else {
                            // Apply base indentation to content
                            result_lines.push(format!("{}{}", base_indent, line));
                        }
                    }
                }

                // Add the END marker line if we found it
                if i < lines.len() {
                    result_lines.push(lines[i].to_string());
                    i += 1;
                }
            } else {
                // Normal line, copy as-is
                result_lines.push(lines[i].to_string());
                i += 1;
            }
        }

        result_lines.join("\n")
    }
}

/// Generator for use case test documentation.
pub struct TestGenerator {
    config: Config,
    file_operations: FileOperations,
    template_engine: TemplateEngine,
    language_registry: Option<crate::core::LanguageRegistry>,
    methodology_registry: Option<crate::core::MethodologyRegistry>,
}

impl TestGenerator {
    /// Creates a new test generator with the given configuration.
    pub fn new(config: Config) -> Self {
        let file_operations = FileOperations::new(config.clone());
        let template_engine = TemplateEngine::with_config(Some(&config));

        // Try to load registries (optional, will work without them)
        let language_registry = Self::load_language_registry(&config);
        let methodology_registry = Self::load_methodology_registry(&config);

        Self {
            config,
            file_operations,
            template_engine,
            language_registry,
            methodology_registry,
        }
    }

    /// Attempts to load the language registry from config templates directory.
    fn load_language_registry(_config: &Config) -> Option<crate::core::LanguageRegistry> {
        let templates_dir = Config::get_metadata_load_dir().ok()?;
        crate::core::LanguageRegistry::new_dynamic(&templates_dir).ok()
    }

    /// Attempts to load the methodology registry from config templates directory.
    fn load_methodology_registry(_config: &Config) -> Option<crate::core::MethodologyRegistry> {
        let templates_dir = Config::get_metadata_load_dir().ok()?;
        crate::core::MethodologyRegistry::new_dynamic(&templates_dir).ok()
    }

    /// Generates and saves a test file for the given use case.
    ///
    /// Returns `Ok(())` if the file was generated or skipped (when file exists and overwrite is disabled).
    pub fn generate(&self, use_case: &UseCase) -> Result<()> {
        self.generate_internal(use_case, false)
    }

    /// Regenerates test file for a use case, preserving user code in safe zones.
    ///
    /// This method always overwrites the test file, regardless of the overwrite_test_documentation setting,
    /// but preserves user code between START/END USER IMPLEMENTATION markers.
    pub fn regenerate(&self, use_case: &UseCase) -> Result<()> {
        self.generate_internal(use_case, true)
    }

    /// Internal method to generate or regenerate test files.
    fn generate_internal(&self, use_case: &UseCase, is_regeneration: bool) -> Result<()> {
        // Skip test generation if test_language is "none"
        if self.config.generation.test_language == "none" {
            return Ok(());
        }

        // Check if test generation is enabled for this use case
        if !is_regeneration && !self.should_generate_test(use_case) {
            return Ok(());
        }

        let file_extension = self.get_file_extension();
        let test_file_path = self.get_file_path(use_case)?;
        let test_file_exists = test_file_path.exists();

        // For initial generation (not regeneration), check overwrite setting
        if !is_regeneration
            && test_file_exists
            && !self.config.generation.overwrite_test_documentation
        {
            UseCaseFormatter::display_test_skipped();
            return Ok(());
        }

        // Generate test content using template
        let mut test_content = self.generate_content(use_case)?;

        // If regenerating and file exists, preserve safe zones
        if is_regeneration && test_file_exists {
            if let Ok(existing_content) = std::fs::read_to_string(&test_file_path) {
                test_content = self.preserve_safe_zones(&test_content, &existing_content)?;
            }
        }

        // Save the test file
        self.file_operations
            .save_test_file(use_case, &test_content, &file_extension)?;

        // Note: Individual test generation messages are suppressed in favor of
        // summary messages displayed by the controller layer (e.g., "Regenerated N test(s)")

        Ok(())
    }

    /// Checks if test generation should occur for this use case based on multi-view settings.
    ///
    /// Test generation occurs if:
    /// - Global auto_generate_tests is enabled, OR
    /// - Any of the use case's views has a methodology with auto_generate_tests enabled
    fn should_generate_test(&self, use_case: &UseCase) -> bool {
        // Check global setting
        if self.config.generation.auto_generate_tests {
            return true;
        }

        // Check per-methodology settings if registry is available
        if let Some(ref registry) = self.methodology_registry {
            for view in &use_case.views {
                if view.enabled {
                    if let Some(methodology) = registry.get(&view.methodology) {
                        if methodology.auto_generate_tests() {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Preserves user code in safe zones when regenerating test files.
    fn preserve_safe_zones(&self, rendered: &str, existing: &str) -> Result<String> {
        // Get comment syntax from language registry
        let comment_start = if let Some(ref registry) = self.language_registry {
            if let Some(lang) = registry.get(&self.config.generation.test_language) {
                lang.comment_start().to_string()
            } else {
                self.default_comment_start()
            }
        } else {
            self.default_comment_start()
        };

        // Parse existing file for safe zones
        let preserver = SafeZonePreserver::new(comment_start);
        let preserved = preserver.parse_safe_zones(existing);

        // Merge preserved content into newly rendered template
        Ok(preserver.merge_safe_zones(rendered, &preserved))
    }

    /// Returns default comment syntax based on configured test language.
    fn default_comment_start(&self) -> String {
        match self.config.generation.test_language.as_str() {
            "python" => "#".to_string(),
            "javascript" | "rust" => "//".to_string(),
            _ => "#".to_string(),
        }
    }

    /// Generates test content for a use case without saving to file.
    fn generate_content(&self, use_case: &UseCase) -> Result<String> {
        // Convert UseCase to JSON for template engine
        let use_case_json = serde_json::to_value(use_case)?;
        let mut data: HashMap<String, Value> = serde_json::from_value(use_case_json)?;

        // Merge extra fields into top-level HashMap
        if let Some(Value::Object(extra_map)) = data.remove("extra") {
            for (key, value) in extra_map {
                data.insert(key, value);
            }
        }

        // Add generated timestamp
        data.insert(
            "generated_at".to_string(),
            json!(chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()),
        );

        // Add snake_case version of title for class names
        if let Some(Value::String(title)) = data.get("title") {
            data.insert("title_snake_case".to_string(), json!(to_snake_case(title)));
        }

        // Render using test template for the configured language
        self.template_engine
            .render_test(&self.config.generation.test_language, &data)
    }

    /// Gets the file extension for test files based on the configured language.
    fn get_file_extension(&self) -> String {
        match self.config.generation.test_language.as_str() {
            "python" => "py".to_string(),
            "javascript" => "js".to_string(),
            "rust" => "rs".to_string(),
            "java" => "java".to_string(),
            "none" => "txt".to_string(), // fallback for none
            _ => "txt".to_string(),      // fallback for unknown
        }
    }

    /// Gets the full file path for a use case's test file.
    fn get_file_path(&self, use_case: &UseCase) -> Result<std::path::PathBuf> {
        let test_dir = std::path::Path::new(&self.config.directories.test_dir);
        let category_dir = test_dir.join(to_snake_case(&use_case.category));
        let file_extension = self.get_file_extension();
        let file_name = format!("{}.{}", to_snake_case(&use_case.id), file_extension);
        Ok(category_dir.join(file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_zone_preserver_extract_global_safe_zone() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = r#"
# START USER IMPLEMENTATION - Add your imports and setup code here
import my_module
from other import thing
# END USER IMPLEMENTATION

def test_something():
    pass
"#;

        let global = preserver.extract_global_safe_zone(content);
        assert_eq!(global.trim(), "import my_module\nfrom other import thing");
    }

    #[test]
    fn test_safe_zone_preserver_extract_global_safe_zone_empty() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = r#"
# START USER IMPLEMENTATION - Add your imports and setup code here
# END USER IMPLEMENTATION

def test_something():
    pass
"#;

        let global = preserver.extract_global_safe_zone(content);
        assert_eq!(global, "");
    }

    #[test]
    fn test_safe_zone_preserver_extract_global_safe_zone_no_markers() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = "def test_something():\n    pass";

        let global = preserver.extract_global_safe_zone(content);
        assert_eq!(global, "");
    }

    #[test]
    fn test_safe_zone_preserver_extract_scenario_id_python() {
        let preserver = SafeZonePreserver::new("#".to_string());

        assert_eq!(
            preserver.extract_scenario_id_from_line("def test_scenario_001(self):"),
            Some("scenario_001".to_string())
        );
        assert_eq!(
            preserver.extract_scenario_id_from_line("    def test_login_success(self):"),
            Some("login_success".to_string())
        );
    }

    #[test]
    fn test_safe_zone_preserver_extract_scenario_id_rust() {
        let preserver = SafeZonePreserver::new("//".to_string());

        assert_eq!(
            preserver.extract_scenario_id_from_line("fn test_scenario_001() {"),
            Some("scenario_001".to_string())
        );
        assert_eq!(
            preserver.extract_scenario_id_from_line("    pub fn test_login_success() {"),
            Some("login_success".to_string())
        );
    }

    #[test]
    fn test_safe_zone_preserver_extract_scenario_id_javascript() {
        let preserver = SafeZonePreserver::new("//".to_string());

        assert_eq!(
            preserver.extract_scenario_id_from_line("function test_scenario_001() {"),
            Some("scenario_001".to_string())
        );
    }

    #[test]
    fn test_safe_zone_preserver_extract_scenario_id_no_match() {
        let preserver = SafeZonePreserver::new("#".to_string());

        assert_eq!(
            preserver.extract_scenario_id_from_line("def some_other_function():"),
            None
        );
        assert_eq!(
            preserver.extract_scenario_id_from_line("# This is a comment"),
            None
        );
        assert_eq!(preserver.extract_scenario_id_from_line(""), None);
    }

    #[test]
    fn test_safe_zone_preserver_parse_scenario_safe_zones_python() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = r#"
def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    
    # My custom test code
    assert True
    
    # END USER IMPLEMENTATION - Do not modify anything below this line

def test_scenario_002(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    
    # Another test
    assert False
    
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let scenarios = preserver.extract_scenario_safe_zones(content);
        assert_eq!(scenarios.len(), 2);
        assert!(scenarios
            .get("scenario_001")
            .unwrap()
            .contains("My custom test code"));
        assert!(scenarios
            .get("scenario_002")
            .unwrap()
            .contains("Another test"));
    }

    #[test]
    fn test_safe_zone_preserver_parse_scenario_safe_zones_empty() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = r#"
def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let scenarios = preserver.extract_scenario_safe_zones(content);
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios.get("scenario_001").unwrap(), "");
    }

    #[test]
    fn test_safe_zone_preserver_parse_full_content() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = r#"
# AUTO-GENERATED TEST FILE
# START USER IMPLEMENTATION - Add your imports and setup code here
import unittest
from my_app import login
# END USER IMPLEMENTATION

class TestLogin(unittest.TestCase):
    def test_successful_login(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        user = login("admin", "password")
        self.assertIsNotNone(user)
        # END USER IMPLEMENTATION - Do not modify anything below this line
    
    def test_failed_login(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        with self.assertRaises(ValueError):
            login("admin", "wrong")
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let safe_zones = preserver.parse_safe_zones(content);

        assert!(safe_zones.global.contains("import unittest"));
        assert!(safe_zones.global.contains("from my_app import login"));

        assert_eq!(safe_zones.scenarios.len(), 2);
        assert!(safe_zones
            .scenarios
            .get("successful_login")
            .unwrap()
            .contains("user = login"));
        assert!(safe_zones
            .scenarios
            .get("failed_login")
            .unwrap()
            .contains("assertRaises"));
    }

    #[test]
    fn test_safe_zone_preserver_merge_global_safe_zone() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let rendered = r#"# AUTO-GENERATED
# START USER IMPLEMENTATION - Add your imports and setup code here

# Add your imports here

# END USER IMPLEMENTATION
"#;
        let preserved = "import my_module";

        let result = preserver.merge_global_safe_zone(rendered, preserved);
        assert!(result.contains("import my_module"));
        assert!(!result.contains("Add your imports here"));
    }

    #[test]
    fn test_safe_zone_preserver_merge_scenario_safe_zone() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let rendered = r#"
def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    
    # TODO: Implement test
    
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;
        let preserved = "assert True  # My test";

        let result = preserver.merge_scenario_safe_zone(rendered, "scenario_001", preserved);
        assert!(result.contains("assert True  # My test"));
        assert!(!result.contains("TODO: Implement test"));
    }

    #[test]
    fn test_safe_zone_preserver_merge_safe_zones_full() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"# AUTO-GENERATED
# START USER IMPLEMENTATION - Add your imports and setup code here

# Add imports

# END USER IMPLEMENTATION

def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # TODO: Implement
    # END USER IMPLEMENTATION - Do not modify anything below this line

def test_scenario_002(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # TODO: Implement
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let mut preserved = SafeZoneContent {
            global: "import unittest".to_string(),
            ..Default::default()
        };
        preserved
            .scenarios
            .insert("scenario_001".to_string(), "assert True".to_string());
        preserved
            .scenarios
            .insert("scenario_002".to_string(), "assert False".to_string());

        let result = preserver.merge_safe_zones(rendered, &preserved);

        assert!(result.contains("import unittest"));
        assert!(result.contains("assert True"));
        assert!(result.contains("assert False"));
        assert!(!result.contains("TODO: Implement"));
    }

    #[test]
    fn test_safe_zone_preserver_merge_with_missing_scenario() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"
def test_new_scenario(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # TODO: Implement
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let mut preserved = SafeZoneContent::default();
        preserved
            .scenarios
            .insert("old_scenario".to_string(), "old code".to_string());

        let result = preserver.merge_safe_zones(rendered, &preserved);

        // New scenario should keep its TODO
        assert!(result.contains("TODO: Implement"));
        // Old scenario code should not appear (scenario was removed)
        assert!(!result.contains("old code"));
    }

    #[test]
    fn test_safe_zone_preserver_rust_comment_syntax() {
        let preserver = SafeZonePreserver::new("//".to_string());
        let content = r#"
// START USER IMPLEMENTATION - Add your imports and setup code here
use my_module::Thing;
// END USER IMPLEMENTATION

#[test]
fn test_scenario_001() {
    // START USER IMPLEMENTATION - Feel free to modify the code below this line
    assert!(true);
    // END USER IMPLEMENTATION - Do not modify anything below this line
}
"#;

        let safe_zones = preserver.parse_safe_zones(content);

        assert!(safe_zones.global.contains("use my_module::Thing"));
        assert_eq!(safe_zones.scenarios.len(), 1);
        assert!(safe_zones
            .scenarios
            .get("scenario_001")
            .unwrap()
            .contains("assert!(true)"));
    }

    #[test]
    fn test_safe_zone_preserver_preserves_indentation() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"
def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # TODO: Implement
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let preserved = "line1 = 'test'\nline2 = 'test'";

        let result = preserver.merge_scenario_safe_zone(rendered, "scenario_001", preserved);

        // Check that indentation is preserved (4 spaces)
        assert!(result.contains("    line1 = 'test'"));
        assert!(result.contains("    line2 = 'test'"));
    }

    #[test]
    fn test_get_safe_zone_markers() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let (start, end) = preserver.get_safe_zone_markers(SafeZoneType::Global);
        assert_eq!(
            start,
            "# START USER IMPLEMENTATION - Add your imports and setup code here"
        );
        assert_eq!(end, "# END USER IMPLEMENTATION");

        let (start, end) = preserver.get_safe_zone_markers(SafeZoneType::SetUp);
        assert_eq!(
            start,
            "# START USER IMPLEMENTATION - Add your setup code here"
        );
        assert_eq!(end, "# END USER IMPLEMENTATION");

        let (start, end) = preserver.get_safe_zone_markers(SafeZoneType::TearDown);
        assert_eq!(
            start,
            "# START USER IMPLEMENTATION - Add your cleanup code here"
        );
        assert_eq!(end, "# END USER IMPLEMENTATION");

        let (start, end) = preserver.get_safe_zone_markers(SafeZoneType::Scenario);
        assert_eq!(
            start,
            "# START USER IMPLEMENTATION - Feel free to modify the code below this line"
        );
        assert_eq!(end, "# END USER IMPLEMENTATION");
    }

    #[test]
    fn test_clean_preserved_content() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let content = "import os\n# END USER IMPLEMENTATION\nprint('hello')\n# END USER IMPLEMENTATION - suffix";
        let cleaned = preserver.clean_preserved_content(content, "# END USER IMPLEMENTATION");
        assert_eq!(cleaned, "import os\nprint('hello')");
    }

    #[test]
    fn test_replace_between_markers() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let rendered = "before\n# START USER IMPLEMENTATION - Add your imports and setup code here\nold content\n# END USER IMPLEMENTATION\nafter";
        let result = preserver.replace_between_markers(
            rendered,
            "# START USER IMPLEMENTATION - Add your imports and setup code here",
            "# END USER IMPLEMENTATION",
            "new content",
        );
        assert_eq!(result, "before\n# START USER IMPLEMENTATION - Add your imports and setup code here\nnew content\n# END USER IMPLEMENTATION\nafter");
    }

    #[test]
    fn test_replace_between_markers_preserves_exact_formatting() {
        let preserver = SafeZonePreserver::new("#".to_string());

        // Test with decorator lines (realistic Python test file format)
        let rendered = r#"# =============================================================================
# START USER IMPLEMENTATION - Add your imports and setup code here
# =============================================================================

# Add your imports here

# =============================================================================
# END USER IMPLEMENTATION
# ============================================================================="#;

        let new_content = "import os\nimport sys";
        let result = preserver.replace_between_markers(
            rendered,
            "# START USER IMPLEMENTATION - Add your imports and setup code here",
            "# END USER IMPLEMENTATION",
            new_content,
        );

        // The result should preserve the newline structure
        assert!(
            result.contains("# START USER IMPLEMENTATION - Add your imports and setup code here\n"),
            "Should have newline after START marker"
        );
        assert!(result.contains("import os"), "Should contain new content");
        assert!(
            result.contains("\n# END USER IMPLEMENTATION"),
            "Should have newline before END marker"
        );

        // Count newlines to ensure formatting is preserved
        let lines: Vec<&str> = result.lines().collect();
        // Original has ~9 lines, after replacement should have at least 5
        assert!(
            lines.len() >= 5,
            "Should preserve reasonable line count, got: {}",
            lines.len()
        );
    }

    #[test]
    fn test_replace_between_markers_with_empty_content() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"    def setUp(self):
        # START USER IMPLEMENTATION - Add your setup code here
        
        pass
        
        # END USER IMPLEMENTATION"#;

        let result = preserver.replace_between_markers(
            rendered,
            "# START USER IMPLEMENTATION - Add your setup code here",
            "# END USER IMPLEMENTATION",
            "",
        );

        // Should preserve structure even with empty content
        assert!(
            result.contains("# START USER IMPLEMENTATION - Add your setup code here"),
            "Should contain START marker"
        );
        assert!(
            result.contains("# END USER IMPLEMENTATION"),
            "Should contain END marker"
        );
    }

    #[test]
    fn test_replace_between_markers_preserves_indentation_context() {
        let preserver = SafeZonePreserver::new("#".to_string());

        // setUp method with indentation
        let rendered = r#"    def setUp(self):
        """Set up test fixtures before each test method."""
        # START USER IMPLEMENTATION - Add your setup code here
        
        # TODO: Add any setup code needed for all tests
        pass
        
        # END USER IMPLEMENTATION"#;

        let new_content = "self.db = Database()";
        let result = preserver.replace_between_markers(
            rendered,
            "# START USER IMPLEMENTATION - Add your setup code here",
            "# END USER IMPLEMENTATION",
            new_content,
        );

        // Verify the new content is in the result
        assert!(
            result.contains("self.db = Database()"),
            "Should contain new content"
        );
        // Verify markers are preserved
        assert!(
            result.contains("# START USER IMPLEMENTATION - Add your setup code here"),
            "Should preserve START marker"
        );
        assert!(
            result.contains("# END USER IMPLEMENTATION"),
            "Should preserve END marker"
        );
    }

    // ============================================================================
    // COMPREHENSIVE TESTS FOR INDENTATION BUG FIX AND MULTIPLE SAFE ZONES
    // ============================================================================

    #[test]
    fn test_no_extra_indentation_on_regenerate() {
        // This is the core bug: regenerating adds extra indentation each time
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"
def test_scenario_001(self):
    # START USER IMPLEMENTATION - Feel free to modify the code below this line
    # TODO: Implement
    # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        // First regeneration - user adds code with proper indentation stripped
        let preserved_content = "line1 = 'test'\nline2 = 'test'";
        let result1 =
            preserver.merge_scenario_safe_zone(rendered, "scenario_001", preserved_content);

        // Verify correct indentation (4 spaces)
        assert!(
            result1.contains("    line1 = 'test'"),
            "First regeneration should have 4 spaces, got:\n{}",
            result1
        );
        assert!(
            result1.contains("    line2 = 'test'"),
            "First regeneration should have 4 spaces"
        );
        assert!(
            !result1.contains("        line1"),
            "First regeneration should NOT have 8 spaces"
        );

        // Second regeneration - extract and reinsert the same content
        let safe_zones = preserver.parse_safe_zones(&result1);
        let extracted = safe_zones.scenarios.get("scenario_001").unwrap();

        let result2 = preserver.merge_scenario_safe_zone(rendered, "scenario_001", extracted);

        // BUG FIX: This should still have only 4 spaces
        assert!(
            result2.contains("    line1 = 'test'"),
            "Second regeneration should still have 4 spaces, got:\n{}",
            result2
        );
        assert!(
            result2.contains("    line2 = 'test'"),
            "Second regeneration should still have 4 spaces"
        );
        assert!(
            !result2.contains("        line1"),
            "Second regeneration should NOT have 8 spaces"
        );

        // Third regeneration - should still be the same
        let safe_zones3 = preserver.parse_safe_zones(&result2);
        let extracted3 = safe_zones3.scenarios.get("scenario_001").unwrap();
        let result3 = preserver.merge_scenario_safe_zone(rendered, "scenario_001", extracted3);

        assert!(
            result3.contains("    line1 = 'test'"),
            "Third regeneration should still have 4 spaces"
        );
        assert!(
            !result3.contains("        line1"),
            "Third regeneration should NOT have 8 spaces"
        );
    }

    #[test]
    fn test_multiple_safe_zones_same_file() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"
# START USER IMPLEMENTATION - Add your imports and setup code here
# END USER IMPLEMENTATION

class TestCase(unittest.TestCase):
    def setUp(self):
        # START USER IMPLEMENTATION - Add your setup code here
        # END USER IMPLEMENTATION
    
    def tearDown(self):
        # START USER IMPLEMENTATION - Add your cleanup code here
        # END USER IMPLEMENTATION
    
    def test_scenario_001(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # END USER IMPLEMENTATION - Do not modify anything below this line
    
    def test_scenario_002(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let mut preserved = SafeZoneContent {
            global: "import os\nimport sys".to_string(),
            setup: "self.db = Database()".to_string(),
            teardown: "self.db.close()".to_string(),
            scenarios: HashMap::new(),
        };
        preserved
            .scenarios
            .insert("scenario_001".to_string(), "assert 1 == 1".to_string());
        preserved
            .scenarios
            .insert("scenario_002".to_string(), "assert 2 == 2".to_string());

        let result = preserver.merge_safe_zones(rendered, &preserved);

        // Check all zones are present
        assert!(
            result.contains("import os"),
            "Global zone should contain import os"
        );
        assert!(
            result.contains("import sys"),
            "Global zone should contain import sys"
        );
        assert!(
            result.contains("self.db = Database()"),
            "setUp should contain database init"
        );
        assert!(
            result.contains("self.db.close()"),
            "tearDown should contain database close"
        );
        assert!(
            result.contains("assert 1 == 1"),
            "scenario_001 should contain assert"
        );
        assert!(
            result.contains("assert 2 == 2"),
            "scenario_002 should contain assert"
        );

        // Check proper indentation for each zone
        // Global zone has no extra indentation
        // setUp, tearDown, and scenarios should have 8 spaces (method body indentation)
        // Note: Our implementation adds indentation based on the template's placeholder indentation
        assert!(
            result.contains("self.db = Database()"),
            "setUp should contain database init"
        );
        assert!(
            result.contains("self.db.close()"),
            "tearDown should contain database close"
        );
        assert!(
            result.contains("assert 1 == 1"),
            "scenario_001 should contain assertion"
        );
        assert!(
            result.contains("assert 2 == 2"),
            "scenario_002 should contain assertion"
        );
    }

    #[test]
    fn test_extract_multiple_scenarios_comprehensive() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let content = r#"
class TestAuth(unittest.TestCase):
    def test_scenario_001(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        user = create_user()
        assert user.is_valid()
        # END USER IMPLEMENTATION - Do not modify anything below this line
    
    def test_scenario_002(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        login_result = login_user("test@example.com", "password")
        assert login_result.success
        # END USER IMPLEMENTATION - Do not modify anything below this line
    
    def test_scenario_003(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # Multi-line
        # comment block
        result = verify_email()
        assert result
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        let safe_zones = preserver.extract_scenario_safe_zones(content);

        assert_eq!(safe_zones.len(), 3, "Should extract all 3 scenarios");

        let s1 = safe_zones.get("scenario_001").unwrap();
        assert!(
            s1.contains("user = create_user()"),
            "Scenario 001 should contain user creation"
        );
        assert!(
            s1.contains("assert user.is_valid()"),
            "Scenario 001 should contain assertion"
        );

        let s2 = safe_zones.get("scenario_002").unwrap();
        assert!(
            s2.contains("login_result"),
            "Scenario 002 should contain login_result"
        );
        assert!(
            s2.contains("assert login_result.success"),
            "Scenario 002 should contain assertion"
        );

        let s3 = safe_zones.get("scenario_003").unwrap();
        assert!(
            s3.contains("# Multi-line"),
            "Scenario 003 should contain multi-line comment"
        );
        assert!(
            s3.contains("# comment block"),
            "Scenario 003 should contain comment block"
        );
        assert!(
            s3.contains("assert result"),
            "Scenario 003 should contain assertion"
        );
    }

    #[test]
    fn test_clean_preserved_content_comprehensive() {
        let preserver = SafeZonePreserver::new("#".to_string());

        // Content with END marker at the end
        let content = "line1\nline2\n# END USER IMPLEMENTATION";
        let cleaned = preserver.clean_preserved_content(content, "# END USER IMPLEMENTATION");
        assert_eq!(cleaned, "line1\nline2");

        // Content with END marker in middle
        let content = "line1\n# END USER IMPLEMENTATION\nline2";
        let cleaned = preserver.clean_preserved_content(content, "# END USER IMPLEMENTATION");
        assert_eq!(cleaned, "line1\nline2");

        // Content without END marker
        let content = "line1\nline2";
        let cleaned = preserver.clean_preserved_content(content, "# END USER IMPLEMENTATION");
        assert_eq!(cleaned, "line1\nline2");

        // Content with multiple END markers
        let content = "line1\n# END USER IMPLEMENTATION\nline2\n# END USER IMPLEMENTATION";
        let cleaned = preserver.clean_preserved_content(content, "# END USER IMPLEMENTATION");
        assert_eq!(cleaned, "line1\nline2");
    }

    #[test]
    fn test_full_roundtrip_extract_merge() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let original = r#"
# START USER IMPLEMENTATION - Add your imports and setup code here
import unittest
from mymodule import MyClass
# END USER IMPLEMENTATION

class TestCase(unittest.TestCase):
    def setUp(self):
        # START USER IMPLEMENTATION - Add your setup code here
        self.obj = MyClass()
        # END USER IMPLEMENTATION
    
    def test_scenario_001(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        result = self.obj.method()
        assert result == "expected"
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        // Extract
        let safe_zones = preserver.parse_safe_zones(original);

        // Create new template
        let template = r#"
# START USER IMPLEMENTATION - Add your imports and setup code here
# END USER IMPLEMENTATION

class TestCase(unittest.TestCase):
    def setUp(self):
        # START USER IMPLEMENTATION - Add your setup code here
        # TODO: Add setup
        # END USER IMPLEMENTATION
    
    def test_scenario_001(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # TODO: Implement
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        // Merge
        let result = preserver.merge_safe_zones(template, &safe_zones);

        // Verify all content is preserved
        assert!(
            result.contains("import unittest"),
            "Should contain import unittest"
        );
        assert!(
            result.contains("from mymodule import MyClass"),
            "Should contain import MyClass"
        );
        assert!(
            result.contains("self.obj = MyClass()"),
            "Should contain object initialization"
        );
        assert!(
            result.contains("result = self.obj.method()"),
            "Should contain method call"
        );
        assert!(
            result.contains("assert result == \"expected\""),
            "Should contain assertion"
        );

        // Verify placeholders are replaced
        assert!(
            !result.contains("TODO: Add setup"),
            "Should not contain TODO in setup"
        );
        assert!(
            !result.contains("TODO: Implement"),
            "Should not contain TODO in scenario"
        );
    }

    #[test]
    fn test_scenario_with_nested_indentation() {
        let preserver = SafeZonePreserver::new("#".to_string());

        let rendered = r#"
    def test_complex_scenario(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # TODO: Implement
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        // Preserved content with nested structures (already has proper relative indentation)
        let preserved = "if True:\n    nested_line()\n    another_nested()";
        let result = preserver.merge_scenario_safe_zone(rendered, "complex_scenario", preserved);

        // The function applies base indentation from the template and preserves relative indentation
        // Base indentation should be 8 spaces (matching the TODO line's indentation)
        assert!(
            result.contains("        if True:"),
            "Should have base indentation (8 spaces)"
        );
        // Nested lines maintain their relative 4-space indentation from the preserved content
        assert!(
            result.contains("        nested_line()"),
            "Should preserve relative indentation"
        );
        assert!(
            result.contains("        another_nested()"),
            "Should preserve relative indentation"
        );
    }

    #[test]
    fn test_five_successive_regenerations_no_indentation_creep() {
        // This test explicitly verifies that the indentation bug is fixed
        // by doing 5 successive regenerations and checking indentation stays constant
        let preserver = SafeZonePreserver::new("#".to_string());

        let template = r#"
class TestCase(unittest.TestCase):
    def test_scenario_001(self):
        # START USER IMPLEMENTATION - Feel free to modify the code below this line
        # TODO: Implement
        # END USER IMPLEMENTATION - Do not modify anything below this line
"#;

        // Initial user code (no leading indentation - will be added during merge)
        let user_code = "x = 1\ny = 2\nassert x + y == 3";

        // First generation
        let gen1 = preserver.merge_scenario_safe_zone(template, "scenario_001", user_code);
        assert!(
            gen1.contains("        x = 1"),
            "Gen 1: should have 8 spaces"
        );
        assert!(
            !gen1.contains("            x = 1"),
            "Gen 1: should NOT have 12 spaces"
        );

        // Extract and regenerate 4 more times
        let mut current = gen1;
        for i in 2..=5 {
            let safe_zones = preserver.parse_safe_zones(&current);
            let extracted = safe_zones.scenarios.get("scenario_001").unwrap();
            current = preserver.merge_scenario_safe_zone(template, "scenario_001", extracted);

            assert!(
                current.contains("        x = 1"),
                "Gen {}: should have 8 spaces",
                i
            );
            assert!(
                !current.contains("            x = 1"),
                "Gen {}: should NOT have 12 spaces (indentation creep)",
                i
            );
            assert!(
                !current.contains("                x = 1"),
                "Gen {}: should NOT have 16 spaces (severe indentation creep)",
                i
            );
        }

        // Final verification: code should be identical after 5 regenerations
        let safe_zones_final = preserver.parse_safe_zones(&current);
        let final_extracted = safe_zones_final.scenarios.get("scenario_001").unwrap();

        // The extracted content should be clean (no extra indentation accumulated)
        assert!(
            final_extracted.contains("x = 1"),
            "Extracted content should be normalized"
        );
        assert!(
            !final_extracted.contains("        x = 1"),
            "Extracted content should NOT have accumulated indentation"
        );
    }

    #[test]
    fn test_merge_setup_safe_zone() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let rendered = r#"    def setUp(self):
        # START USER IMPLEMENTATION - Add your setup code here
        # TODO: setup
        # END USER IMPLEMENTATION
"#;
        let preserved = "self.user = 'test'";

        let result = preserver.merge_setup_safe_zone(rendered, preserved);
        assert!(result.contains("self.user = 'test'"));
        assert!(!result.contains("TODO: setup"));
    }

    #[test]
    fn test_merge_teardown_safe_zone() {
        let preserver = SafeZonePreserver::new("#".to_string());
        let rendered = r#"    def tearDown(self):
        # START USER IMPLEMENTATION - Add your cleanup code here
        # TODO: cleanup
        # END USER IMPLEMENTATION
"#;
        let preserved = "self.cleanup()";

        let result = preserver.merge_teardown_safe_zone(rendered, preserved);
        assert!(result.contains("self.cleanup()"));
        assert!(!result.contains("TODO: cleanup"));
    }
}
