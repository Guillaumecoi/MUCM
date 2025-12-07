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

use anyhow::{anyhow, Result};
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

    /// Detects indentation from a line of text.
    /// Returns a string of spaces matching the indentation, or default spaces if line has no indentation.
    fn detect_indentation(line: &str, default: usize) -> String {
        let indent_size = line.len() - line.trim_start().len();
        " ".repeat(if indent_size > 0 {
            indent_size
        } else {
            default
        })
    }

    /// Indents content with the given indentation string.
    /// Empty lines remain empty, non-empty lines get the indent prefix.
    fn indent_content(content: &str, indent: &str) -> Vec<String> {
        content
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect()
    }

    /// Validates safe zone structure in content.
    /// Checks that START markers have corresponding END markers.
    fn validate_safe_zone_structure(&self, content: &str, zone_type: SafeZoneType) -> Result<()> {
        let (start_marker, end_marker) = self.get_safe_zone_markers(zone_type);

        let start_count = content.matches(&start_marker).count();
        let end_count = content.matches(&end_marker).count();

        if start_count != end_count {
            return Err(anyhow!(
                "Mismatched safe zone markers for {:?}: {} START markers but {} END markers",
                zone_type,
                start_count,
                end_count
            ));
        }

        Ok(())
    }

    /// Parses an existing test file and extracts all safe zone content.
    ///
    /// Returns a SafeZoneContent with the global safe zone and per-scenario safe zones.
    /// Validates safe zone structure before parsing.
    fn parse_safe_zones(&self, content: &str) -> SafeZoneContent {
        // Validate structure (log warnings but don't fail - be permissive)
        if let Err(e) = self.validate_safe_zone_structure(content, SafeZoneType::Global) {
            eprintln!("Warning: {}", e);
        }
        if let Err(e) = self.validate_safe_zone_structure(content, SafeZoneType::SetUp) {
            eprintln!("Warning: {}", e);
        }
        if let Err(e) = self.validate_safe_zone_structure(content, SafeZoneType::TearDown) {
            eprintln!("Warning: {}", e);
        }

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
    fn extract_between_markers(
        &self,
        content: &str,
        start_marker: &str,
        end_marker: &str,
    ) -> String {
        if let Some(start_pos) = content.find(start_marker) {
            let content_after_start = &content[start_pos + start_marker.len()..];
            if let Some(end_pos) = content_after_start.find(end_marker) {
                let extracted = &content_after_start[..end_pos];
                // Trim leading/trailing whitespace but preserve internal formatting
                return extracted.trim().to_string();
            }
        }
        String::new()
    }

    /// Extracts safe zone content from a slice of lines until end marker is found.
    fn extract_safe_zone_from_lines(
        &self,
        lines: &[&str],
        start_idx: usize,
        end_marker: &str,
    ) -> String {
        let mut content_lines = Vec::new();

        for line in &lines[start_idx..] {
            if line.contains(end_marker) {
                break;
            }
            content_lines.push(*line);
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
        self.replace_between_markers(rendered, &start_marker, &end_marker, preserved_content)
    }

    /// Merges the setUp method safe zone content into the rendered template.
    fn merge_setup_safe_zone(&self, rendered: &str, preserved_content: &str) -> String {
        let (start_marker, end_marker) = self.get_safe_zone_markers(SafeZoneType::SetUp);
        self.replace_between_markers(rendered, &start_marker, &end_marker, preserved_content)
    }

    /// Merges the tearDown method safe zone content into the rendered template.
    fn merge_teardown_safe_zone(&self, rendered: &str, preserved_content: &str) -> String {
        let (start_marker, end_marker) = self.get_safe_zone_markers(SafeZoneType::TearDown);
        self.replace_between_markers(rendered, &start_marker, &end_marker, preserved_content)
    }

    /// Merges a scenario-specific safe zone into the rendered template.
    fn merge_scenario_safe_zone(
        &self,
        rendered: &str,
        scenario_id: &str,
        preserved_content: &str,
    ) -> String {
        // Find the test function for this scenario ID
        let lines: Vec<&str> = rendered.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut i = 0;

        let (start_marker, _) = self.get_safe_zone_markers(SafeZoneType::Scenario);
        // Note: End marker for scenarios includes additional suffix (not using standard end marker)
        let end_marker = format!(
            "{} END USER IMPLEMENTATION - Do not modify anything below this line",
            self.comment_start
        );

        while i < lines.len() {
            result_lines.push(lines[i].to_string());

            // Check if this line defines a test for our scenario ID
            if let Some(found_id) = self.extract_scenario_id_from_line(lines[i]) {
                if found_id == scenario_id {
                    // Found the right test function, now find the safe zone
                    i += 1;
                    while i < lines.len() {
                        result_lines.push(lines[i].to_string());

                        if lines[i].contains(&start_marker) {
                            // Skip the empty/placeholder content until end marker
                            i += 1;
                            let mut skipped_lines = Vec::new();
                            while i < lines.len() && !lines[i].contains(&end_marker) {
                                skipped_lines.push(lines[i]);
                                i += 1;
                            }

                            // Insert preserved content (with proper indentation from first skipped line)
                            if let Some(first_skipped) = skipped_lines.first() {
                                let indent_str = Self::detect_indentation(first_skipped, 4);
                                result_lines
                                    .extend(Self::indent_content(preserved_content, &indent_str));
                            } else {
                                // No indentation reference, just add the content as-is
                                result_lines.extend(preserved_content.lines().map(String::from));
                            }

                            // Add the end marker line (if we found it)
                            if i < lines.len() {
                                result_lines.push(lines[i].to_string());
                            }
                            break;
                        }
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
    fn replace_between_markers(
        &self,
        text: &str,
        start_marker: &str,
        end_marker: &str,
        new_content: &str,
    ) -> String {
        if let Some(start_pos) = text.find(start_marker) {
            let before_start = &text[..start_pos + start_marker.len()];
            let after_start = &text[start_pos + start_marker.len()..];

            if let Some(end_pos) = after_start.find(end_marker) {
                let after_end = &after_start[end_pos..];

                // Determine indentation from the next line after start marker
                let between_content = &after_start[..end_pos];
                let indent = if let Some(first_line) = between_content.lines().nth(1) {
                    Self::detect_indentation(first_line, 4)
                } else {
                    "    ".to_string() // Default 4 spaces
                };

                // Format new content with proper indentation
                let formatted_content = if new_content.is_empty() {
                    format!("\n{}\n", indent)
                } else {
                    let indented_lines = Self::indent_content(new_content, &indent);
                    format!("\n{}\n{}", indented_lines.join("\n"), indent)
                };

                return format!("{}{}{}", before_start, formatted_content, after_end);
            }
        }
        text.to_string()
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
}
