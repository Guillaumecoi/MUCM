/// CLI Runner - Core business logic adapter.
///
/// The CliRunner serves as the main interface between the CLI layer and the
/// application's business logic. It provides high-level operations for use case
/// management, project initialization, and methodology handling.
///
/// This runner delegates to specialized controllers:
/// - `ProjectController`: Handles project-level operations (init, config, templates)
/// - `UseCaseController`: Manages individual use cases (CRUD, regeneration)
///
/// The runner maintains lazy-loaded controllers to avoid unnecessary initialization
/// and provides a clean, error-handling facade for CLI command handlers.
use anyhow::Result;

use crate::controller::{DisplayResult, ProjectController, UseCaseController};

/// CLI runner that delegates to controllers
/// This is a thin adapter between CLI interface and business logic
pub struct CliRunner {
    use_case_controller: Option<UseCaseController>,
}

impl CliRunner {
    /// Create a new CLI runner instance with uninitialized controllers.
    pub fn new() -> Self {
        Self {
            use_case_controller: None,
        }
    }

    /// Sanitize an optional string input by trimming whitespace and filtering empty strings.
    ///
    /// Returns None if the input is None or contains only whitespace.
    /// Returns Some(trimmed_string) if the input contains non-whitespace characters.
    fn sanitize_optional_string(input: Option<String>) -> Option<String> {
        input
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
    }

    /// Sanitize a required string input by trimming whitespace.
    ///
    /// Preserves internal whitespace but removes leading/trailing whitespace.
    fn sanitize_required_string(input: String) -> String {
        input.trim().to_string()
    }

    /// Ensure the use case controller is loaded.
    fn ensure_use_case_controller(&mut self) -> Result<&mut UseCaseController> {
        if self.use_case_controller.is_none() {
            self.use_case_controller = Some(UseCaseController::new()?);
        }
        Ok(self
            .use_case_controller
            .as_mut()
            .expect("controller was just initialized"))
    }

    /// Sanitize a string input by trimming whitespace.
    ///
    /// Returns None if the input contains only whitespace, Some(trimmed_string) otherwise.
    fn sanitize_string(input: String) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Initialize a new use case manager project with storage backend choice (configuration phase).
    ///
    /// Creates the initial project structure and configuration files with the specified storage backend.
    /// This is the first step of initialization - templates are copied later
    /// in `finalize_init()` to allow config review.
    ///
    /// # Arguments
    /// * `language` - Optional programming language for code templates
    /// * `methodologies` - List of methodologies to enable
    /// * `storage` - Storage backend to use (toml or sqlite)
    ///
    /// # Returns
    /// Returns a DisplayResult with success message.
    pub fn init_project(
        &mut self,
        language: Option<String>,
        methodologies: Vec<String>,
        storage: String,
    ) -> Result<DisplayResult> {
        // Sanitize inputs: trim whitespace and filter out empty strings
        let sanitized_language = Self::sanitize_optional_string(language);
        let sanitized_methodologies: Vec<String> = methodologies
            .into_iter()
            .filter_map(Self::sanitize_string)
            .collect();

        // Use first methodology as default, or "feature" if none provided
        let default_methodology = sanitized_methodologies
            .first()
            .cloned()
            .unwrap_or_else(|| "feature".to_string());

        let params = crate::controller::InitProjectParams {
            language: sanitized_language,
            methodologies: Some(sanitized_methodologies),
            storage: Some(storage),
            default_methodology: Some(default_methodology),
            use_case_dir: None,
            test_dir: None,
            actor_dir: None,
            data_dir: None,
            default_scenario_template: None,
        };
        let result = ProjectController::init_project(params)?;
        Ok(result)
    }

    /// Finalize project initialization (template copying phase).
    ///
    /// Completes the initialization by copying template files based on the
    /// previously created configuration. This should be called after reviewing
    /// the generated config files.
    ///
    /// # Returns
    /// Returns a DisplayResult with completion message.
    pub fn finalize_init(&mut self) -> Result<DisplayResult> {
        let result = ProjectController::finalize_init()?;
        Ok(result)
    }

    /// Create a new use case using the project's default methodology.
    ///
    /// Generates a new use case with the specified details, using whatever
    /// methodology is configured as default for the project.
    ///
    /// # Arguments
    /// * `title` - The use case title
    /// * `category` - The category for organization
    /// * `description` - Optional detailed description
    ///
    /// # Returns
    /// Returns a DisplayResult with success message.
    pub fn create_use_case(
        &mut self,
        title: String,
        category: String,
        description: Option<String>,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        let category_clean = Self::sanitize_required_string(category);
        let category_abbreviation = category_clean
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        let params = crate::controller::CreateUseCaseParams {
            title: Self::sanitize_required_string(title),
            category: category_clean,
            category_abbreviation,
            description: Self::sanitize_optional_string(description),
            methodology: None,
            views: None,
            priority: None,
            extra_fields: None,
        };
        controller.create_use_case(params)
    }

    /// Create a new use case with a specific methodology.
    ///
    /// Generates a new use case with the specified details, overriding the
    /// project's default methodology with the provided one.
    ///
    /// # Arguments
    /// * `title` - The use case title
    /// * `category` - The category for organization
    /// * `description` - Optional detailed description
    /// * `methodology` - The methodology to use for documentation generation
    ///
    /// # Returns
    /// Returns a DisplayResult with success message.
    pub fn create_use_case_with_methodology(
        &mut self,
        title: String,
        category: String,
        description: Option<String>,
        methodology: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        let category_clean = Self::sanitize_required_string(category);
        let category_abbreviation = category_clean
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        let params = crate::controller::CreateUseCaseParams {
            title: Self::sanitize_required_string(title),
            category: category_clean,
            category_abbreviation,
            description: Self::sanitize_optional_string(description),
            methodology: Some(Self::sanitize_required_string(methodology)),
            views: None,
            priority: None,
            extra_fields: None,
        };
        controller.create_use_case(params)
    }

    /// Create a new use case with multiple views.
    ///
    /// Creates a use case that can be rendered in multiple methodology/level combinations.
    /// The views parameter should be a comma-separated list of methodology:level pairs.
    ///
    /// # Arguments
    /// * `title` - The use case title
    /// * `category` - The category to organize under
    /// * `description` - Optional description
    /// * `views` - Comma-separated methodology:level pairs (e.g., "feature:simple,business:normal")
    ///
    /// # Returns
    /// Returns a DisplayResult with success message.
    pub fn create_use_case_with_views(
        &mut self,
        title: String,
        category: String,
        description: Option<String>,
        views: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        let category_clean = Self::sanitize_required_string(category);
        let category_abbreviation = category_clean
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(3)
            .collect::<String>()
            .to_uppercase();
        let params = crate::controller::CreateUseCaseParams {
            title: Self::sanitize_required_string(title),
            category: category_clean,
            category_abbreviation,
            description: Self::sanitize_optional_string(description),
            methodology: None,
            views: Some(Self::sanitize_required_string(views)),
            priority: None,
            extra_fields: None,
        };
        controller.create_use_case(params)
    }

    /// List all use cases in the project.
    ///
    /// Displays information about all existing use cases, including their
    /// titles, categories, and current status.
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or an error if listing fails.
    pub fn list_use_cases(&mut self) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.list_use_cases()
    }

    /// Display the current project status.
    ///
    /// Shows information about the project's initialization state,
    /// configuration, and available use cases.
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or an error if status retrieval fails.
    pub fn show_status(&mut self) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.show_status()
    }

    /// Display available programming languages.
    ///
    /// Shows the list of supported programming languages for code templates.
    ///
    /// # Returns
    /// Returns a formatted string listing available languages.
    pub fn show_languages() -> Result<String> {
        ProjectController::show_languages()
    }

    /// List all available methodologies.
    ///
    /// Retrieves and formats information about all supported documentation
    /// methodologies, including their names and descriptions.
    ///
    /// # Returns
    /// Returns a formatted string with methodology information.
    pub fn list_methodologies(&mut self) -> Result<String> {
        let methodologies = ProjectController::get_available_methodologies()?;

        if methodologies.is_empty() {
            return Ok("No methodologies available.".to_string());
        }

        let mut result = String::from("Available methodologies:\n");
        for info in methodologies {
            result.push_str(&format!("  - {}: {}\n", info.name, info.description));
        }

        Ok(result)
    }

    /// Get detailed information about a specific methodology.
    ///
    /// Retrieves comprehensive information about the requested methodology,
    /// including its display name, description, when to use it, and key features.
    ///
    /// # Arguments
    /// * `methodology` - The name of the methodology to query
    ///
    /// # Returns
    /// Returns a formatted string with methodology details, or a not-found message.
    pub fn get_methodology_info(&mut self, methodology: String) -> Result<String> {
        use crate::config::Config;
        use crate::core::{Methodology, MethodologyRegistry};

        let sanitized_methodology = Self::sanitize_required_string(methodology);

        // Always load methodology metadata (info.toml) from source templates
        let templates_dir = Config::get_metadata_load_dir()?;
        let registry = MethodologyRegistry::new_dynamic(&templates_dir)?;

        match registry.get(&sanitized_methodology) {
            Some(methodology) => {
                let mut result = format!(
                    "=== {} ===\n\n{}\n\n",
                    methodology.title(),
                    methodology.description()
                );

                result.push_str("When to Use:\n");
                for item in methodology.when_to_use() {
                    result.push_str(&format!("  • {}\n", item));
                }

                result.push_str("\nKey Features:\n");
                for item in methodology.key_features() {
                    result.push_str(&format!("  • {}\n", item));
                }

                result.push_str("\nAvailable Levels:\n");
                for level in methodology.levels() {
                    result.push_str(&format!(
                        "  • {} ({}): {}\n",
                        level.name, level.filename, level.description
                    ));
                }

                result.push_str(&format!(
                    "\nPreferred Style: {}\n",
                    methodology.preferred_style()
                ));

                Ok(result)
            }
            None => Ok(format!(
                "Methodology '{}' not found.",
                sanitized_methodology
            )),
        }
    }

    /// Regenerate a use case with a different methodology.
    ///
    /// Updates an existing use case to use a new methodology, regenerating
    /// its documentation accordingly.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to regenerate
    /// * `methodology` - The new methodology to apply
    ///
    /// # Returns
    /// Returns a DisplayResult with success message.
    pub fn regenerate_use_case_with_methodology(
        &mut self,
        use_case_id: String,
        methodology: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.regenerate_use_case_with_methodology(
            Self::sanitize_required_string(use_case_id),
            Self::sanitize_required_string(methodology),
        )
    }

    /// Regenerate documentation for a single use case.
    ///
    /// Regenerates the markdown documentation for the specified use case
    /// using its current methodology.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case to regenerate
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or an error if regeneration fails.
    pub fn regenerate_use_case(&mut self, use_case_id: String) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.regenerate_use_case(&Self::sanitize_required_string(use_case_id))
    }

    /// Regenerate documentation for all use cases.
    ///
    /// Regenerates markdown documentation for all use cases in the project
    /// using their current methodologies.
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or an error if any regeneration fails.
    pub fn regenerate_all_use_cases(&mut self) -> Result<()> {
        let controller = self.ensure_use_case_controller()?;
        controller.regenerate_all_use_cases()
    }

    /// Add a precondition to a use case.
    ///
    /// Adds a new precondition to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `precondition` - The precondition text to add
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or precondition cannot be added
    pub fn add_precondition(
        &mut self,
        use_case_id: String,
        precondition: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.add_precondition(
            Self::sanitize_required_string(use_case_id),
            Self::sanitize_required_string(precondition),
        )
    }

    /// List preconditions for a use case.
    ///
    /// Retrieves and displays all preconditions for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with preconditions list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_preconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.list_preconditions(Self::sanitize_required_string(use_case_id))
    }

    /// Remove a precondition from a use case.
    ///
    /// Removes the precondition at the specified index from the use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the precondition to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn remove_precondition(
        &mut self,
        use_case_id: String,
        index: usize,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.remove_precondition(Self::sanitize_required_string(use_case_id), index)
    }

    /// Add a postcondition to a use case.
    ///
    /// Adds a new postcondition to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `postcondition` - The postcondition text to add
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or postcondition cannot be added
    pub fn add_postcondition(
        &mut self,
        use_case_id: String,
        postcondition: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.add_postcondition(
            Self::sanitize_required_string(use_case_id),
            Self::sanitize_required_string(postcondition),
        )
    }

    /// List postconditions for a use case.
    ///
    /// Retrieves and displays all postconditions for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with postconditions list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_postconditions(&mut self, use_case_id: String) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.list_postconditions(Self::sanitize_required_string(use_case_id))
    }

    /// Remove a postcondition from a use case.
    ///
    /// Removes the postcondition at the specified index from the use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `index` - The 1-based index of the postcondition to remove
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or index is invalid
    pub fn remove_postcondition(
        &mut self,
        use_case_id: String,
        index: usize,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.remove_postcondition(Self::sanitize_required_string(use_case_id), index)
    }

    /// Add a reference to a use case.
    ///
    /// Adds a new reference relationship to the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `target_id` - The ID of the target use case
    /// * `relationship` - The type of relationship (dependency, extension, inclusion, alternative)
    /// * `description` - Optional description of the relationship
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or reference cannot be added
    pub fn add_reference(
        &mut self,
        use_case_id: String,
        target_id: String,
        relationship: String,
        description: Option<String>,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.add_reference(
            Self::sanitize_required_string(use_case_id),
            Self::sanitize_required_string(target_id),
            Self::sanitize_required_string(relationship),
            Self::sanitize_optional_string(description),
        )
    }

    /// List references for a use case.
    ///
    /// Retrieves and displays all references for the specified use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    ///
    /// # Returns
    /// DisplayResult with references list
    ///
    /// # Errors
    /// Returns error if use case not found
    pub fn list_references(&mut self, use_case_id: String) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.list_references(Self::sanitize_required_string(use_case_id))
    }

    /// Remove a reference from a use case.
    ///
    /// Removes the reference to the specified target use case.
    ///
    /// # Arguments
    /// * `use_case_id` - The ID of the use case
    /// * `target_id` - The ID of the target use case to remove reference to
    ///
    /// # Returns
    /// DisplayResult with success message
    ///
    /// # Errors
    /// Returns error if use case not found or reference doesn't exist
    pub fn remove_reference(
        &mut self,
        use_case_id: String,
        target_id: String,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.remove_reference(
            Self::sanitize_required_string(use_case_id),
            Self::sanitize_required_string(target_id),
        )
    }

    /// List use cases that use a specific persona
    ///
    /// # Arguments
    /// * `persona_id` - The persona identifier to search for
    ///
    /// Clean up orphaned methodology fields from use cases
    ///
    /// Scans methodology_fields and removes entries for methodologies not used by any enabled view.
    ///
    /// # Arguments
    /// * `use_case_id` - Optional specific use case to clean. If None, cleans all use cases.
    /// * `dry_run` - If true, shows what would be cleaned without making changes
    ///
    /// # Returns
    /// DisplayResult with summary of cleanup operation
    ///
    /// # Errors
    /// Returns error if use case not found or cleanup fails
    pub fn cleanup_methodology_fields(
        &mut self,
        use_case_id: Option<String>,
        dry_run: bool,
    ) -> Result<DisplayResult> {
        let controller = self.ensure_use_case_controller()?;
        controller.cleanup_methodology_fields(use_case_id, dry_run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test sanitization of optional strings
    #[test]
    fn test_sanitize_optional_string() {
        // Test None input
        assert_eq!(CliRunner::sanitize_optional_string(None), None);

        // Test empty string
        assert_eq!(
            CliRunner::sanitize_optional_string(Some("".to_string())),
            None
        );

        // Test whitespace-only string
        assert_eq!(
            CliRunner::sanitize_optional_string(Some("   ".to_string())),
            None
        );

        // Test string with leading/trailing whitespace
        assert_eq!(
            CliRunner::sanitize_optional_string(Some("  hello  ".to_string())),
            Some("hello".to_string())
        );

        // Test string with internal whitespace (should be preserved)
        assert_eq!(
            CliRunner::sanitize_optional_string(Some("  hello world  ".to_string())),
            Some("hello world".to_string())
        );

        // Test string with no whitespace
        assert_eq!(
            CliRunner::sanitize_optional_string(Some("hello".to_string())),
            Some("hello".to_string())
        );
    }

    /// Test sanitization of required strings
    #[test]
    fn test_sanitize_required_string() {
        // Test string with leading/trailing whitespace
        assert_eq!(
            CliRunner::sanitize_required_string("  hello  ".to_string()),
            "hello".to_string()
        );

        // Test string with internal whitespace (should be preserved)
        assert_eq!(
            CliRunner::sanitize_required_string("  hello world  ".to_string()),
            "hello world".to_string()
        );

        // Test string with no whitespace
        assert_eq!(
            CliRunner::sanitize_required_string("hello".to_string()),
            "hello".to_string()
        );

        // Test empty string
        assert_eq!(
            CliRunner::sanitize_required_string("".to_string()),
            "".to_string()
        );

        // Test whitespace-only string
        assert_eq!(
            CliRunner::sanitize_required_string("   ".to_string()),
            "".to_string()
        );
    }
}
