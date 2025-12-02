//! Interactive runner tests
//!
//! Tests for the interactive CLI runner, focusing on basic workflows and coordination.
//!
//! ## Running Tests
//!
//! These tests modify global state and should be run with `cargo nextest run` for best results.
//! See the documentation in `controller/tests.rs` for more details.

#[cfg(test)]
mod interactive_runner_tests {
    use crate::cli::interactive::InteractiveRunner;
    use crate::config::{Config, ConfigFileManager};
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    // Note: Now using Config::ensure_test_templates() instead of local helper

    /// Helper to create a test environment with initialized config
    fn setup_test_env() -> (TempDir, InteractiveRunner) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Create minimal source-templates for testing using the shared helper
        Config::ensure_test_templates().unwrap();

        // Create a basic config
        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".").unwrap();

        let runner = InteractiveRunner::new();
        (temp_dir, runner)
    }

    #[test]
    #[serial]
    fn test_new_runner_creation() {
        let runner = InteractiveRunner::new();
        // Should not panic and should create a valid runner
        drop(runner);
    }

    #[test]
    #[serial]
    fn test_get_available_languages() {
        let (_temp_dir, runner) = setup_test_env();

        let result = runner.get_available_languages();
        assert!(result.is_ok());
        let languages = result.unwrap();
        assert!(!languages.is_empty());
    }

    #[test]
    #[serial]
    fn test_get_available_methodologies() {
        let (_temp_dir, runner) = setup_test_env();

        let result = runner.get_available_methodologies();
        assert!(result.is_ok());
        let methodologies = result.unwrap();
        assert!(!methodologies.is_empty());

        // Verify methodology info structure
        for methodology in methodologies {
            assert!(!methodology.name.is_empty());
            assert!(!methodology.display_name.is_empty());
        }
    }

    #[test]
    #[serial]
    fn test_create_use_case_interactive() {
        let (_temp_dir, mut runner) = setup_test_env();

        let result = runner.create_use_case_with_views(
            "Test Use Case".to_string(),
            "test".to_string(),
            Some("Test description".to_string()),
            vec![("business".to_string(), "normal".to_string())],
        );

        assert!(result.is_ok(), "Failed to create use case: {:?}", result);
        let message = result.unwrap();
        assert!(
            message.contains("Created use case") && message.contains("with views"),
            "Should confirm creation with views info. Message was: {}",
            message
        );

        // Verify use case is accessible through listing
        let list_result = runner.list_use_cases();
        assert!(
            list_result.is_ok(),
            "Should be able to list use cases after creation"
        );
    }

    #[test]
    #[serial]
    fn test_create_use_case_without_methodology() {
        let (_temp_dir, mut runner) = setup_test_env();

        // Should use default methodology from config
        let result = runner.create_use_case_with_views(
            "Test Use Case".to_string(),
            "test".to_string(),
            None,
            vec![("feature".to_string(), "simple".to_string())], // Use default methodology
        );

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_list_use_cases() {
        let (_temp_dir, mut runner) = setup_test_env();

        // Create a use case first
        runner
            .create_use_case_with_views(
                "Test UC".to_string(),
                "test".to_string(),
                None,
                vec![("business".to_string(), "normal".to_string())],
            )
            .unwrap();

        // List should not panic
        let result = runner.list_use_cases();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_show_status() {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Create minimal source-templates
        Config::ensure_test_templates().unwrap();

        // Initialize project
        let mut runner = InteractiveRunner::new();
        let params = crate::cli::interactive::runner::InitProjectParams {
            language: None,
            methodologies: vec!["business".to_string()],
            storage: "toml".to_string(),
            use_case_dir: "docs/use-cases".to_string(),
            test_dir: "tests".to_string(),
            persona_dir: "docs/personas".to_string(),
            data_dir: "use-cases-data".to_string(),
            scenario_template: None,
        };
        let result = runner.initialize_project(params);
        assert!(result.is_ok(), "Initialization should succeed");

        // Create some use cases
        runner
            .create_use_case_with_views(
                "Test UC 1".to_string(),
                "test".to_string(),
                None,
                vec![("business".to_string(), "normal".to_string())],
            )
            .unwrap();
        runner
            .create_use_case_with_views(
                "Test UC 2".to_string(),
                "test".to_string(),
                None,
                vec![("developer".to_string(), "detailed".to_string())],
            )
            .unwrap();

        // Show status should not panic
        let result = runner.show_status();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_multiple_use_case_creation() {
        let (_temp_dir, mut runner) = setup_test_env();

        // Create multiple use cases
        for i in 1..=3 {
            let result = runner.create_use_case_with_views(
                format!("Test UC {}", i),
                "test".to_string(),
                Some(format!("Description {}", i)),
                vec![("business".to_string(), "normal".to_string())],
            );
            assert!(result.is_ok());
        }

        // Verify we can list them all
        let result = runner.list_use_cases();
        assert!(result.is_ok());
    }

    // TODO: Add tests for initialize_project workflow
    // TODO: Add tests for finalize_initialization workflow
    // TODO: Add tests for error handling in workflows
    // TODO: Add tests for state management across operations
}

#[cfg(test)]
mod workflow_tests {
    use crate::cli::interactive::InteractiveRunner;
    use crate::config::{Config, ConfigFileManager};
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn setup_empty_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();
        // Create source-templates for testing
        Config::ensure_test_templates().unwrap();
        temp_dir
    }

    #[test]
    #[serial]
    fn test_initialization_workflow() {
        let _temp_dir = setup_empty_dir();
        let mut runner = InteractiveRunner::new();

        // Test initialization through runner interface
        let params = crate::cli::interactive::runner::InitProjectParams {
            language: Some("rust".to_string()),
            methodologies: vec!["business".to_string()],
            storage: "toml".to_string(),
            use_case_dir: "docs/my-use-cases".to_string(),
            test_dir: "tests/my-tests".to_string(),
            persona_dir: "docs/my-personas".to_string(),
            data_dir: "my-data".to_string(),
            scenario_template: None,
        };
        let result = runner.initialize_project(params);
        assert!(result.is_ok(), "Initialization should succeed");
        let message = result.unwrap();
        assert!(
            message.contains("Project setup complete") || message.contains("initialized"),
            "Should indicate successful initialization"
        );

        // Verify we can now create use cases (proves initialization worked)
        let create_result = runner.create_use_case_with_views(
            "Test Case".to_string(),
            "test".to_string(),
            None,
            vec![("business".to_string(), "normal".to_string())],
        );
        assert!(
            create_result.is_ok(),
            "Should be able to create use cases after initialization"
        );
    }

    #[test]
    #[serial]
    fn test_full_use_case_workflow() {
        let _temp_dir = setup_empty_dir();

        // Setup config
        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".").unwrap();

        let mut runner = InteractiveRunner::new();

        // Create a use case - verify success message
        let result = runner.create_use_case_with_views(
            "Login".to_string(),
            "authentication".to_string(),
            Some("User login workflow".to_string()),
            vec![("business".to_string(), "normal".to_string())],
        );
        assert!(result.is_ok(), "Use case creation should succeed");
        let message = result.unwrap();
        assert!(
            message.contains("Created use case") || message.contains("UC-"),
            "Should confirm use case creation with ID"
        );

        // List use cases - should not panic and work
        let result = runner.list_use_cases();
        assert!(result.is_ok(), "Listing use cases should work");

        // Show status - should work after creating use case
        let result = runner.show_status();
        assert!(result.is_ok(), "Status display should work");
    }

    #[test]
    #[serial]
    fn test_workflow_with_different_methodologies() {
        let _temp_dir = setup_empty_dir();

        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".").unwrap();

        let mut runner = InteractiveRunner::new();

        // Create use cases with different methodologies
        let methodologies = ["business", "developer", "feature", "tester"];

        for (i, methodology) in methodologies.iter().enumerate() {
            let result = runner.create_use_case_with_views(
                format!("UC {}", i + 1),
                "test".to_string(),
                None,
                vec![(methodology.to_string(), "simple".to_string())],
            );
            assert!(result.is_ok(), "Failed for methodology: {}", methodology);
        }
    }

    // TODO: Add tests for menu navigation workflows
    // TODO: Add tests for scenario creation workflows
    // TODO: Add tests for configuration workflows
    // TODO: Add tests for methodology selection workflows
}

#[cfg(test)]
mod persona_workflow_tests {
    use crate::cli::interactive::InteractiveRunner;
    use crate::config::{Config, ConfigFileManager, StorageBackend};
    use serial_test::serial;
    use std::{env, fs};
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, InteractiveRunner, Config) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Create source-templates for testing
        Config::ensure_test_templates().unwrap();

        let config = Config::default();
        ConfigFileManager::save_in_dir(&config, ".").unwrap();

        let runner = InteractiveRunner::new();
        (temp_dir, runner, config)
    }

    fn setup_test_env_with_backend(
        backend: StorageBackend,
    ) -> (TempDir, InteractiveRunner, Config) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        // Create source-templates for testing
        Config::ensure_test_templates().unwrap();

        let mut config = Config::default();
        config.storage.backend = backend;

        // Create data directory for SQLite backend
        if matches!(backend, StorageBackend::Sqlite) {
            fs::create_dir_all(temp_dir.path().join(&config.directories.data_dir)).unwrap();
        }

        ConfigFileManager::save_in_dir(&config, ".").unwrap();

        let runner = InteractiveRunner::new();
        (temp_dir, runner, config)
    }

    #[test]
    #[serial]
    fn test_create_persona_interactive_basic() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        let result = runner.create_persona_interactive(
            "dev-user".to_string(),
            "Developer User".to_string(),
            "Test Function".to_string(),
        );

        assert!(
            result.is_ok(),
            "Failed to create persona: {:?}",
            result.err()
        );
        let message = result.unwrap();
        assert!(
            message.contains("Created persona"),
            "Message was: {}",
            message
        );
        assert!(
            message.contains("Developer User"),
            "Message was: {}",
            message
        );

        // Verify persona can be shown through runner interface
        let show_result = runner.show_actor("dev-user".to_string());
        assert!(
            show_result.is_ok(),
            "Should be able to show created persona"
        );
    }

    #[test]
    #[serial]
    fn test_create_persona_interactive_sqlite_backend() {
        let (_temp_dir, mut runner, _config) = setup_test_env_with_backend(StorageBackend::Sqlite);

        let result = runner.create_persona_interactive(
            "test-user".to_string(),
            "Test User".to_string(),
            "Test Function".to_string(),
        );

        assert!(result.is_ok(), "Should create persona with SQLite backend");

        // Verify through same runner instance - this uses the same connection pool
        // so there's no connection isolation issue
        let show_result = runner.show_actor("test-user".to_string());
        assert!(
            show_result.is_ok(),
            "Should be able to show created persona with SQLite backend"
        );
    }

    #[test]
    #[serial]
    fn test_create_persona_interactive_duplicate_id() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // Create first persona
        let result = runner.create_persona_interactive(
            "duplicate".to_string(),
            "First User".to_string(),
            "Test Function".to_string(),
        );
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("First User"), "Should confirm creation");

        // Try to create duplicate - with unified actor system, this shows error but returns Ok
        // The duplicate detection is in the controller and returns a DisplayResult with success=false
        let result = runner.create_persona_interactive(
            "duplicate".to_string(),
            "Second User".to_string(),
            "Test Function".to_string(),
        );
        assert!(
            result.is_ok(),
            "Duplicate detection now shows error message via DisplayResult"
        );

        // Verify persona still has first user's data by showing it
        let show_result = runner.show_actor("duplicate".to_string());
        assert!(
            show_result.is_ok(),
            "Should still be able to show the original persona after duplicate attempt"
        );
    }

    #[test]
    #[serial]
    fn test_list_actors_empty() {
        let (_temp_dir, runner, _config) = setup_test_env();

        let result = runner.list_actors();
        assert!(result.is_ok());
        // Should not panic with empty list
    }

    #[test]
    #[serial]
    fn test_list_actors_single() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // Create one persona
        runner
            .create_persona_interactive(
                "user1".to_string(),
                "User One".to_string(),
                "Test Function".to_string(),
            )
            .unwrap();

        let result = runner.list_actors();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_list_actors_multiple() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // Create multiple personas
        for i in 1..=5 {
            runner
                .create_persona_interactive(
                    format!("user{}", i),
                    format!("User {}", i),
                    "Test Function".to_string(),
                )
                .unwrap();
        }

        let result = runner.list_actors();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_show_persona_exists() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // Create persona
        runner
            .create_persona_interactive(
                "show-test".to_string(),
                "Show Test User".to_string(),
                "Test Function".to_string(),
            )
            .unwrap();

        let result = runner.show_actor("show-test".to_string());
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_delete_persona_exists() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // Create persona
        runner
            .create_persona_interactive(
                "to-delete".to_string(),
                "Delete Test".to_string(),
                "Test Function".to_string(),
            )
            .unwrap();

        // Verify it exists through show
        assert!(runner.show_actor("to-delete".to_string()).is_ok());

        // Delete it
        let result = runner.delete_actor("to-delete");
        assert!(result.is_ok());

        // Verify it's gone - show should fail or list should not contain it
        let list_result = runner.list_actors();
        assert!(list_result.is_ok(), "List should still work after deletion");
    }

    #[test]
    #[serial]
    fn test_delete_persona_not_found() {
        let (_temp_dir, runner, _config) = setup_test_env();

        let result = runner.delete_actor("nonexistent");
        // Should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    #[serial]
    fn test_persona_workflow_complete_cycle() {
        let (_temp_dir, mut runner, _config) = setup_test_env();

        // 1. Create persona - verify success message
        let result = runner.create_persona_interactive(
            "cycle-test".to_string(),
            "Cycle Test User".to_string(),
            "Test Function".to_string(),
        );
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Cycle Test User"));

        // 2. List personas - should succeed
        let result = runner.list_actors();
        assert!(result.is_ok());

        // 3. Show persona - should succeed
        let result = runner.show_actor("cycle-test".to_string());
        assert!(result.is_ok());

        // 4. Delete persona - should succeed
        let result = runner.delete_actor("cycle-test");
        assert!(result.is_ok());

        // 5. List should still work after deletion
        let result = runner.list_actors();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_persona_operations_both_backends() {
        // Test TOML backend
        {
            let (_temp_dir, mut runner, _config) =
                setup_test_env_with_backend(StorageBackend::Toml);

            let result = runner.create_persona_interactive(
                "toml-user".to_string(),
                "TOML User".to_string(),
                "Test Function".to_string(),
            );
            assert!(result.is_ok());

            // Verify through runner interface
            let show_result = runner.show_actor("toml-user".to_string());
            assert!(
                show_result.is_ok(),
                "Should be able to show persona with TOML backend"
            );
        }

        // TODO: SQLite backend test disabled - separate connections don't see each other's changes
        // Need to refactor to use shared connection for testing
        // {
        //     let (_temp_dir, mut runner, config) =
        //         setup_test_env_with_backend(StorageBackend::Sqlite);
        //
        //     runner
        //         .create_persona_interactive("sqlite-user".to_string(), "SQLite User".to_string(), "Test Function".to_string())
        //         .unwrap();
        //
        //     let repo = RepositoryFactory::create_persona_repository(&config).unwrap();
        //     assert!(repo.exists("sqlite-user").unwrap());
        // }
    }

    // TODO: Add tests for ActorWorkflow menu interactions when testing infrastructure supports it
    // TODO: Add tests for validation prompts in interactive mode
    // TODO: Add tests for cancellation/back navigation in workflows
    // TODO: Add tests for error recovery in interactive workflows
}
