// File operation utilities for persistence layer
use crate::config::Config;
use crate::core::{to_snake_case, UseCase};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Helper functions for file operations
pub struct FileOperations {
    config: Config,
}

impl FileOperations {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Save a test file
    /// This should create test files in the configured test_dir with proper structure
    pub fn save_test_file(
        &self,
        use_case: &UseCase,
        test_content: &str,
        file_extension: &str,
    ) -> Result<()> {
        // Create test directory with category subdirectory if it doesn't exist
        let test_dir = Path::new(&self.config.directories.test_dir);
        let category_dir = test_dir.join(to_snake_case(&use_case.category));
        fs::create_dir_all(&category_dir)?;

        // Generate filename: snake_case of use case ID with extension
        let file_name = format!("{}.{}", to_snake_case(&use_case.id), file_extension);
        let test_path = category_dir.join(file_name);

        // Write the test file
        fs::write(&test_path, test_content)?;

        Ok(())
    }

    /// Save overview file
    pub fn save_overview(&self, content: &str) -> Result<()> {
        let use_case_dir = Path::new(&self.config.directories.use_case_dir);
        fs::create_dir_all(use_case_dir)?;
        let overview_path = use_case_dir.join("README.md");
        fs::write(&overview_path, content)?;
        println!("Generated overview at: {}", overview_path.display());
        Ok(())
    }

    /// Save category overview file
    pub fn save_category_overview(&self, category_path: &str, content: &str) -> Result<()> {
        let category_dir = Path::new(&self.config.directories.use_case_dir).join(category_path);
        fs::create_dir_all(&category_dir)?;
        let readme_path = category_dir.join("README.md");
        fs::write(&readme_path, content)?;
        println!("Generated category overview at: {}", readme_path.display());
        Ok(())
    }

    /// Check if a test file exists for a given use case
    pub fn test_file_exists(&self, use_case: &UseCase, file_extension: &str) -> bool {
        let test_dir =
            Path::new(&self.config.directories.test_dir).join(to_snake_case(&use_case.category));
        let test_file_name = format!("{}.{}", to_snake_case(&use_case.id), file_extension);
        let test_path = test_dir.join(test_file_name);
        test_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_test_file_exists() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test config with the temp directory as test_dir
        let mut config = Config::default();
        config.directories.test_dir = temp_path.to_string_lossy().to_string();

        // Create FileOperations instance
        let file_ops = FileOperations::new(config);

        // Create a test use case
        let use_case = UseCase::new(
            "UC-TEST-001".to_string(),
            "Test Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "A test use case".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        // Test that file doesn't exist initially
        assert!(!file_ops.test_file_exists(&use_case, "py"));

        // Create the category directory and test file manually
        let category_dir = temp_path.join("test");
        fs::create_dir_all(&category_dir).expect("Failed to create category dir");
        let test_file_path = category_dir.join("uc_test_001.py");
        fs::write(&test_file_path, "test content").expect("Failed to write test file");

        // Test that file now exists
        assert!(file_ops.test_file_exists(&use_case, "py"));

        // Test with different extension
        assert!(!file_ops.test_file_exists(&use_case, "js"));

        // Test with different use case
        let other_use_case = UseCase::new(
            "UC-OTHER-001".to_string(),
            "Other Use Case".to_string(),
            "Test".to_string(),
            "TES".to_string(),
            "Another test use case".to_string(),
            "Medium".to_string(),
        )
        .unwrap();
        assert!(!file_ops.test_file_exists(&other_use_case, "py"));
    }

    #[test]
    fn test_save_test_file_creates_category_directory() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test config with the temp directory as test_dir
        let mut config = Config::default();
        config.directories.test_dir = temp_path.to_string_lossy().to_string();

        // Create FileOperations instance
        let file_ops = FileOperations::new(config);

        // Create a test use case
        let use_case = UseCase::new(
            "UC-TEST-002".to_string(),
            "Test Use Case 2".to_string(),
            "Feature".to_string(),
            "FEA".to_string(),
            "A test use case".to_string(),
            "Medium".to_string(),
        )
        .unwrap();

        // Save a test file
        let test_content = "# Generated test file\nprint('Hello, World!')\n";
        file_ops
            .save_test_file(&use_case, test_content, "py")
            .expect("Failed to save test file");

        // Check that the category directory was created
        let category_dir = temp_path.join("feature");
        assert!(category_dir.exists());
        assert!(category_dir.is_dir());

        // Check that the test file was created with correct content
        let test_file_path = category_dir.join("uc_test_002.py");
        assert!(test_file_path.exists());
        assert!(test_file_path.is_file());

        let read_content = fs::read_to_string(&test_file_path).expect("Failed to read test file");
        assert_eq!(read_content, test_content);
    }
}
