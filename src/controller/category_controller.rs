//! # Category Controller
//!
//! This module provides the controller for category management operations.
//! It handles category creation, listing, collision detection, and auto-resolution.
//!
//! ## Responsibilities
//!
//! - Category creation with full name and abbreviation
//! - Category listing for selection
//! - Abbreviation suggestion from category names
//! - Collision detection and auto-resolution
//! - Case-insensitive uniqueness validation

use crate::config::Config;
use crate::controller::dto::{DisplayResult, SelectionOptions};
use crate::core::{Category, TomlCategoryRepository};
use anyhow::Result;

/// Controller for category operations and management.
///
/// Manages category operations including creation, listing, and collision resolution.
/// Acts as the coordination layer between CLI commands and category persistence.
pub struct CategoryController {
    /// Repository for category persistence
    repository: TomlCategoryRepository,
}

impl CategoryController {
    /// Create a new category controller instance.
    ///
    /// Initializes the controller with the TOML category repository.
    ///
    /// # Returns
    /// A new CategoryController instance ready for use
    ///
    /// # Errors
    /// Returns error if the configuration cannot be loaded
    pub fn new() -> Result<Self> {
        let _config = Config::load()?;
        // Categories are stored in .config/.mucm/ directory relative to current working directory
        let config_dir = std::env::current_dir()?
            .join(Config::CONFIG_DIR)
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid config directory path"))?
            .to_string();

        Ok(Self {
            repository: TomlCategoryRepository::new(config_dir),
        })
    }

    /// List all categories for display or selection.
    ///
    /// Returns categories sorted by full name with their abbreviations.
    ///
    /// # Returns
    /// SelectionOptions containing formatted category entries ("Name (ABBR)")
    pub fn list_categories(&self) -> Result<SelectionOptions> {
        let mut categories = self.repository.load_all()?;
        categories.sort_by(|a, b| a.full_name.cmp(&b.full_name));

        let formatted: Vec<String> = categories
            .iter()
            .map(|cat| format!("{} ({})", cat.full_name, cat.abbreviation))
            .collect();

        Ok(SelectionOptions::new(formatted))
    }

    /// Get all categories with their names and abbreviations.
    ///
    /// # Returns
    /// Vector of (full_name, abbreviation) tuples
    pub fn get_all_categories(&self) -> Result<Vec<(String, String)>> {
        let categories = self.repository.load_all()?;
        Ok(categories
            .iter()
            .map(|cat| (cat.full_name.clone(), cat.abbreviation.clone()))
            .collect())
    }

    /// Suggest an abbreviation for a category name.
    ///
    /// Takes the first 3 letters (uppercase) of the name as a starting point.
    ///
    /// # Arguments
    /// * `name` - The category name
    ///
    /// # Returns
    /// Suggested abbreviation (minimum 3 uppercase letters)
    pub fn suggest_abbreviation(&self, name: &str) -> String {
        Category::suggest_abbreviation(name)
    }

    /// Check if a category name exists (case-insensitive).
    ///
    /// # Arguments
    /// * `name` - The category name to check
    ///
    /// # Returns
    /// True if the name is already in use
    pub fn exists_name(&self, name: &str) -> Result<bool> {
        self.repository.exists_name(name)
    }

    /// Check if an abbreviation exists (case-insensitive).
    ///
    /// # Arguments
    /// * `abbreviation` - The abbreviation to check
    ///
    /// # Returns
    /// True if the abbreviation is already in use
    pub fn exists_abbreviation(&self, abbreviation: &str) -> Result<bool> {
        self.repository.exists_abbreviation(abbreviation)
    }

    /// Find a category by abbreviation.
    ///
    /// # Arguments
    /// * `abbreviation` - The abbreviation to search for
    ///
    /// # Returns
    /// The category if found, None otherwise
    pub fn find_by_abbreviation(&self, abbreviation: &str) -> Result<Option<Category>> {
        self.repository.find_by_abbreviation(abbreviation)
    }

    /// Create a new category.
    ///
    /// Validates that both name and abbreviation are unique (case-insensitive).
    ///
    /// # Arguments
    /// * `full_name` - The full category name
    /// * `abbreviation` - The abbreviation (will be converted to uppercase)
    ///
    /// # Returns
    /// DisplayResult with success/failure status and message
    pub fn create_category(
        &self,
        full_name: String,
        abbreviation: String,
    ) -> Result<DisplayResult> {
        // Create and validate the category
        let category = match Category::new(full_name.clone(), abbreviation.clone()) {
            Ok(cat) => cat,
            Err(e) => {
                return Ok(DisplayResult {
                    success: false,
                    message: format!("Invalid category: {}", e),
                });
            }
        };

        // Check for duplicates
        if self.repository.exists_name(&category.full_name)? {
            return Ok(DisplayResult {
                success: false,
                message: format!(
                    "Category '{}' already exists (case-insensitive check)",
                    category.full_name
                ),
            });
        }

        if self
            .repository
            .exists_abbreviation(&category.abbreviation)?
        {
            return Ok(DisplayResult {
                success: false,
                message: format!(
                    "Abbreviation '{}' is already in use (case-insensitive check)",
                    category.abbreviation
                ),
            });
        }

        // Save the category
        self.repository.add(category)?;

        Ok(DisplayResult {
            success: true,
            message: format!(
                "Category '{}' ({}) created successfully",
                full_name, abbreviation
            ),
        })
    }

    /// Detect if an abbreviation collides with existing categories.
    ///
    /// # Arguments
    /// * `abbreviation` - The abbreviation to check
    ///
    /// # Returns
    /// Option containing the conflicting category if collision detected
    pub fn detect_collision(&self, abbreviation: &str) -> Result<Option<Category>> {
        self.repository.find_by_abbreviation(abbreviation)
    }

    /// Suggest collision resolution for two conflicting category names.
    ///
    /// Attempts to create distinguishing abbreviations by adding an extra letter.
    ///
    /// # Arguments
    /// * `new_name` - The new category name being created
    /// * `existing_category` - The existing category with conflicting abbreviation
    ///
    /// # Returns
    /// Option containing (new_abbrev, updated_existing_abbrev) if resolution possible
    pub fn suggest_collision_resolution(
        &self,
        new_name: &str,
        existing_category: &Category,
    ) -> Option<(String, String)> {
        Category::suggest_collision_resolution(new_name, &existing_category.full_name)
    }

    /// Update an existing category's abbreviation.
    ///
    /// Used when resolving collisions manually or via auto-resolution.
    ///
    /// # Arguments
    /// * `old_abbreviation` - Current abbreviation
    /// * `new_abbreviation` - New abbreviation to use
    ///
    /// # Returns
    /// DisplayResult with success/failure status and message
    pub fn update_abbreviation(
        &self,
        old_abbreviation: &str,
        new_abbreviation: &str,
    ) -> Result<DisplayResult> {
        match self
            .repository
            .update_abbreviation(old_abbreviation, new_abbreviation)
        {
            Ok(()) => Ok(DisplayResult {
                success: true,
                message: format!(
                    "Abbreviation updated from '{}' to '{}'",
                    old_abbreviation, new_abbreviation
                ),
            }),
            Err(e) => Ok(DisplayResult {
                success: false,
                message: format!("Failed to update abbreviation: {}", e),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_controller() -> (CategoryController, TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".config/.mucm");
        fs::create_dir_all(&config_path).unwrap();

        // Set up minimal config
        let config_content = format!(
            r#"
[project]
name = "Test Project"
description = "Test"

[storage]
backend = "toml"

[directories]
use_case_dir = "docs/use-cases"
test_dir = "tests"
actor_dir = "docs/actors"
data_dir = "{}"

[templates]
methodologies = ["developer"]
default_methodology = "developer"

[generation]
test_language = "none"
auto_generate_tests = false
overwrite_test_documentation = false

[metadata]
created = true
last_updated = true
"#,
            temp_dir.path().join("data").to_str().unwrap()
        );

        fs::write(
            temp_dir.path().join(".config/.mucm/mucm.toml"),
            config_content,
        )
        .unwrap();

        // Change to temp directory so Config::load() finds it and stays there
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let controller = CategoryController::new().unwrap();

        // DON'T restore directory yet - return it so tests can restore after they're done

        (controller, temp_dir, original_dir)
    }

    #[test]
    fn test_suggest_abbreviation() {
        let (controller, _temp, original_dir) = setup_test_controller();
        assert_eq!(controller.suggest_abbreviation("Authentication"), "AUT");
        assert_eq!(controller.suggest_abbreviation("User Management"), "USE");

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_create_category() {
        let (controller, _temp, original_dir) = setup_test_controller();

        let result = controller
            .create_category("Authentication".to_string(), "AUT".to_string())
            .unwrap();

        assert!(result.success);
        assert!(result.message.contains("created successfully"));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_create_duplicate_name_fails() {
        let (controller, _temp, original_dir) = setup_test_controller();

        controller
            .create_category("Authentication".to_string(), "AUT".to_string())
            .unwrap();

        let result = controller
            .create_category("AUTHENTICATION".to_string(), "AUTH".to_string())
            .unwrap();

        assert!(!result.success);
        assert!(result.message.contains("already exists"));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_create_duplicate_abbreviation_fails() {
        let (controller, _temp, original_dir) = setup_test_controller();

        controller
            .create_category("Authentication".to_string(), "AUT".to_string())
            .unwrap();

        let result = controller
            .create_category("Authorization".to_string(), "aut".to_string())
            .unwrap();

        assert!(!result.success);
        assert!(result.message.contains("already in use"));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_detect_collision() {
        let (controller, _temp, original_dir) = setup_test_controller();

        controller
            .create_category("Authentication".to_string(), "AUT".to_string())
            .unwrap();

        let collision = controller.detect_collision("AUT").unwrap();
        assert!(collision.is_some());
        assert_eq!(collision.unwrap().full_name, "Authentication");

        let no_collision = controller.detect_collision("AUTH").unwrap();
        assert!(no_collision.is_none());

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_list_categories() {
        let (controller, _temp, original_dir) = setup_test_controller();

        let result1 = controller
            .create_category("Authentication".to_string(), "AUT".to_string())
            .unwrap();
        assert!(result1.success, "First create failed: {}", result1.message);

        let result2 = controller
            .create_category("User Management".to_string(), "USR".to_string())
            .unwrap();
        assert!(result2.success, "Second create failed: {}", result2.message);

        let options = controller.list_categories().unwrap();
        assert_eq!(
            options.items.len(),
            2,
            "Expected 2 categories, got: {:?}",
            options.items
        );
        // Should be sorted alphabetically
        assert!(options.items[0].contains("Authentication"));

        env::set_current_dir(original_dir).unwrap();
    }
}
