//! # Configuration Workflow
//!
//! Interactive configuration management for project settings.
//! Handles general project configuration excluding methodologies and use cases.

use anyhow::Result;
use inquire::{Confirm, Select, Text};

use crate::cli::interactive::ui::UI;
use crate::config::{Config, TemplateManager};
use crate::core::LanguageRegistry;

/// Configuration workflow handler
pub struct ConfigWorkflow;

impl ConfigWorkflow {
    /// Configure project information
    pub fn configure_project_info(config: &mut Config) -> Result<()> {
        println!("\n📋 Project Information");
        println!("────────────────────────");

        config.project.name = Text::new("Project name:")
            .with_default(&config.project.name)
            .prompt()?;

        config.project.description = Text::new("Project description:")
            .with_default(&config.project.description)
            .prompt()?;

        Ok(())
    }

    /// Configure directory settings
    pub fn configure_directories(config: &mut Config) -> Result<()> {
        println!("\n📁 Directory Settings");
        println!("───────────────────────");

        config.directories.use_case_dir = Text::new("Use case directory:")
            .with_default(&config.directories.use_case_dir)
            .with_help_message("Where to store generated use case markdown files")
            .prompt()?;

        config.directories.test_dir = Text::new("Test directory:")
            .with_default(&config.directories.test_dir)
            .with_help_message("Where to generate test scaffolding")
            .prompt()?;

        config.directories.actor_dir = Text::new("Actor directory:")
            .with_default(&config.directories.actor_dir)
            .with_help_message("Where to store generated persona markdown files")
            .prompt()?;

        config.directories.data_dir = Text::new("Data directory:")
            .with_default(&config.directories.data_dir)
            .with_help_message("Source of truth: TOML files and SQLite database")
            .prompt()?;

        Ok(())
    }

    /// Configure generation settings
    pub fn configure_generation(config: &mut Config) -> Result<()> {
        println!("\n🔧 Generation Settings");
        println!("────────────────────────");

        let templates_dir = TemplateManager::find_source_templates_dir()?;
        let languages = LanguageRegistry::discover_available(&templates_dir)?;
        let mut language_options = vec!["none".to_string()];
        language_options.extend(languages);

        config.generation.test_language = Select::new("Test language:", language_options)
            .with_help_message("Programming language for test generation")
            .prompt()?;

        config.generation.auto_generate_tests = Confirm::new("Auto-generate tests?")
            .with_default(config.generation.auto_generate_tests)
            .prompt()?;

        Ok(())
    }

    /// Configure metadata settings
    pub fn configure_metadata(config: &mut Config) -> Result<()> {
        println!("\n📊 Metadata Configuration");
        println!("────────────────────────────");

        config.metadata.created = Confirm::new("Auto-set 'created' timestamp?")
            .with_default(config.metadata.created)
            .prompt()?;

        config.metadata.last_updated = Confirm::new("Auto-update 'last_updated' timestamp?")
            .with_default(config.metadata.last_updated)
            .prompt()?;

        println!("\n💡 To configure additional fields (author, reviewer, status, priority, etc.),");
        println!("   edit [extra_fields] section in .config/.mucm/mucm.toml after saving.\n");

        Ok(())
    }

    /// View current configuration
    pub fn view_config(config: &Config) -> Result<()> {
        UI::clear_screen()?;
        println!("\n📄 Current Configuration");
        println!("═══════════════════════════\n");

        println!("📋 Project Information");
        println!("  Name: {}", config.project.name);
        println!("  Description: {}\n", config.project.description);

        println!("📁 Directory Settings");
        println!(
            "  Use Cases: {} (generated)",
            config.directories.use_case_dir
        );
        println!("  Tests: {}", config.directories.test_dir);
        println!("  Actors: {} (generated)", config.directories.actor_dir);
        println!(
            "  Data: {} (source of truth)\n",
            config.directories.data_dir
        );

        println!("🔧 Generation Settings");
        println!("  Test Language: {}", config.generation.test_language);
        println!(
            "  Auto-generate Tests: {}\n",
            config.generation.auto_generate_tests
        );

        println!("📚 Methodologies");
        println!("  Default: {}", config.templates.default_methodology);
        if config.templates.methodologies.is_empty() {
            println!("  Available: (none configured)");
        } else {
            println!("  Available: {}", config.templates.methodologies.join(", "));
        }
        println!();

        println!("📊 Metadata Configuration");
        println!("  Auto-set 'created': {}", config.metadata.created);
        println!(
            "  Auto-update 'last_updated': {}\n",
            config.metadata.last_updated
        );

        println!("💾 Storage");
        println!("  Backend: {}\n", config.storage.backend);

        UI::pause_for_input()?;
        Ok(())
    }

    /// Configure default scenario template
    pub fn configure_scenario_template(config: &mut Config) -> Result<()> {
        use crate::controller::ProjectController;

        println!("\n🎨 Scenario Template Configuration");
        println!("──────────────────────────────────");

        // Get available templates
        let available_templates = ProjectController::get_available_scenario_templates()?;

        if available_templates.is_empty() {
            println!("⚠️  No scenario templates found in project templates.");
            println!("Using default: scenarios/scenario.hbs\n");
            UI::pause_for_input()?;
            return Ok(());
        }

        // Create display options with descriptions
        let mut template_options: Vec<String> = Vec::new();
        for template in &available_templates {
            let description = if template.contains("mermaid") {
                "Mermaid sequence diagrams"
            } else {
                "Standard text-based scenarios"
            };
            template_options.push(format!("{} - {}", template, description));
        }

        // Show current default
        println!(
            "Current default: {}\n",
            config.templates.default_scenario_template
        );

        // Prompt for selection
        let selection = Select::new("Default scenario template:", template_options)
            .with_help_message("Methodology levels can override this in their methodology.toml")
            .prompt()?;

        // Extract template path from selection (before the " - " separator)
        let template_path = selection.split(" - ").next().unwrap().to_string();

        // Update config
        config.templates.default_scenario_template = template_path.clone();

        println!(
            "\n✅ Default scenario template updated to: {}",
            template_path
        );
        println!("💡 Methodology levels can still override this by setting scenario_template in methodology.toml\n");

        Ok(())
    }

    /// Save configuration
    pub fn save_config(config: &Config) -> Result<()> {
        config.save_in_dir(".")?;

        println!("\n💾 Syncing template files with configuration...");

        // Sync template files with current config (copy new, remove old)
        use crate::controller::ProjectController;
        match ProjectController::sync_templates(false) {
            Ok(_) => {
                UI::show_success("✅ Configuration saved and templates synced!")?;
            }
            Err(e) => {
                eprintln!("⚠️  Warning: Templates sync failed: {}", e);
                println!("Configuration was saved but templates may be out of sync.\n");
            }
        }

        Ok(())
    }
}
