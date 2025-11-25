//! # Interactive Project Initialization
//!
//! Guided project setup wizard for interactive CLI mode.
//! Walks users through configuring their project with language, methodologies, and templates.

use anyhow::Result;
use inquire::{Confirm, MultiSelect, Select};

use crate::cli::interactive::runner::InteractiveRunner;
use crate::cli::interactive::selectors::{
    get_available_languages, get_available_methodologies, get_methodology_descriptions,
};
use crate::cli::interactive::ui::UI;

/// Handle project initialization workflow
pub struct Initialization;

impl Initialization {
    /// Check if project is initialized, offer to initialize if not
    pub fn check_and_initialize() -> Result<()> {
        // Try to load config
        if crate::config::Config::load().is_err() {
            UI::clear_screen()?;
            UI::show_init_wizard_header()?;

            let should_init = Confirm::new("Would you like to initialize a new project?")
                .with_default(true)
                .prompt()?;

            if !should_init {
                UI::show_warning(
                    "Exiting without initializing. Run 'mucm init' to initialize later.",
                )?;
                return Err(anyhow::anyhow!("Project not initialized"));
            }

            // Run the initialization wizard
            Self::run_initialization_wizard()?;
        }

        Ok(())
    }

    /// Run the full initialization wizard
    fn run_initialization_wizard() -> Result<()> {
        let mut runner = InteractiveRunner::new();

        // Step 1: Select programming language
        UI::show_step(
            1,
            "Project Programming Language",
            "Select the primary programming language for your project.\nThis is used for test scaffolding generation.",
        )?;

        let languages = get_available_languages(&mut runner)?;
        let mut language_options = vec!["none".to_string()];
        language_options.extend(languages);

        let language_selection = Select::new("Programming language:", language_options)
            .with_help_message("Choose 'none' if you don't need test scaffolding")
            .prompt()?;

        let language = if language_selection == "none" {
            None
        } else {
            Some(language_selection)
        };

        // Step 2: Select methodologies
        UI::show_step(
            2,
            "Use Case Methodologies",
            "Select which methodologies you plan to use for documenting use cases.\n💡 You can always add or remove methodologies later!",
        )?;

        let methodology_infos = get_available_methodologies(&mut runner)?;

        if methodology_infos.is_empty() {
            UI::show_error("No methodologies available. This is unexpected.")?;
            return Err(anyhow::anyhow!("No methodologies found"));
        }

        let methodology_display = get_methodology_descriptions(&methodology_infos);

        let selected = MultiSelect::new("Select methodologies:", methodology_display.clone())
            .with_help_message(
                "Use Space to select/deselect, Enter to confirm. Select at least one.",
            )
            .prompt()?;

        if selected.is_empty() {
            UI::show_error("You must select at least one methodology.")?;
            return Err(anyhow::anyhow!("No methodology selected"));
        }

        // Extract methodology names from display strings
        let selected_methodologies: Vec<String> = selected
            .iter()
            .map(|display| {
                // Extract the methodology name (before the first space or dash)
                display.split_whitespace().next().unwrap().to_lowercase()
            })
            .collect();

        // Step 3: Select default methodology
        UI::show_step(
            3,
            "Default Methodology",
            "Choose which methodology to use by default when creating use cases.",
        )?;

        let default_methodology = if selected_methodologies.len() == 1 {
            selected_methodologies[0].clone()
        } else {
            let methodology_display = get_methodology_descriptions(&methodology_infos);

            let default_options: Vec<String> = selected_methodologies
                .iter()
                .filter_map(|m| {
                    methodology_display
                        .iter()
                        .find(|d| d.to_lowercase().starts_with(m))
                        .cloned()
                })
                .collect();

            let default_display = Select::new("Default methodology:", default_options)
                .with_help_message("This will be used when no methodology is specified")
                .prompt()?;

            default_display
                .split_whitespace()
                .next()
                .unwrap()
                .to_lowercase()
        };

        // Step 4: Configure directories
        UI::show_step(
            4,
            "Directory Configuration",
            "Configure where use cases, tests, personas, and data will be stored.\nPress Enter to use default values.",
        )?;

        let use_case_dir = inquire::Text::new("Use case directory:")
            .with_default("docs/use-cases")
            .with_help_message("Where markdown use case files will be stored")
            .prompt()?;

        let test_dir = inquire::Text::new("Test directory:")
            .with_default("tests/use-cases")
            .with_help_message("Where test files will be generated")
            .prompt()?;

        let persona_dir = inquire::Text::new("Persona directory:")
            .with_default("docs/personas")
            .with_help_message("Where persona markdown files will be stored")
            .prompt()?;

        let data_dir = inquire::Text::new("Data directory:")
            .with_default("use-cases-data")
            .with_help_message("Where TOML/SQLite data files will be stored")
            .prompt()?;

        // Step 5: Select storage backend
        UI::show_step(
            5,
            "Storage Backend",
            "Choose how use case data will be stored.\n\
            TOML: Simple file-based storage, great for version control\n\
            SQLite: Database storage, better for complex queries and large projects",
        )?;

        let storage_options = vec![
            "toml - Simple file-based storage (recommended for most projects)",
            "sqlite - Database storage (better for complex queries)",
        ];

        let storage_selection = Select::new("Storage backend:", storage_options)
            .with_help_message("TOML is simpler and git-friendly, SQLite offers better querying")
            .prompt()?;

        let storage_backend = if storage_selection.starts_with("toml") {
            "toml"
        } else {
            "sqlite"
        };

        // Step 6: Select scenario template
        UI::show_step(
            6,
            "Scenario Template",
            "Choose how scenarios will be rendered in your use case documentation.\n\
            Standard: Traditional text-based scenario descriptions\n\
            Mermaid: Visual sequence diagrams showing actor interactions",
        )?;

        let template_options = vec![
            "scenarios/scenario.hbs - Standard text-based scenarios",
            "scenarios/scenario_mermaid.hbs - Mermaid sequence diagrams",
        ];

        let template_selection = Select::new("Default scenario template:", template_options)
            .with_help_message("You can override this per-methodology in methodology.toml")
            .prompt()?;

        let scenario_template = if template_selection.starts_with("scenarios/scenario_mermaid") {
            Some("scenarios/scenario_mermaid.hbs".to_string())
        } else {
            Some("scenarios/scenario.hbs".to_string())
        };

        // Show summary
        show_configuration_summary(
            &language,
            &selected_methodologies,
            &default_methodology,
            storage_backend,
            &use_case_dir,
            &test_dir,
            &persona_dir,
            &data_dir,
        )?;

        // Confirm settings
        let confirm = Confirm::new("Are these settings correct?")
            .with_default(true)
            .with_help_message(
                "Choose 'Yes' to proceed (templates and directories will be created). Choose 'No' to start over.",
            )
            .prompt()?;

        if !confirm {
            UI::show_warning("Restarting initialization wizard...\n")?;
            return Self::run_initialization_wizard();
        }

        // Ask if they want to create standard system actors
        let create_actors =
            Confirm::new("Create standard system actors (Database, API, Web Server, etc.)?")
                .with_default(true)
                .with_help_message(
                    "These are commonly used external systems that interact with your use cases",
                )
                .prompt()?;

        // Create config with directories
        create_config_with_directories(
            &mut runner,
            language,
            selected_methodologies,
            storage_backend,
            use_case_dir.clone(),
            test_dir.clone(),
            persona_dir.clone(),
            data_dir.clone(),
            scenario_template,
            create_actors,
        )?;

        Ok(())
    }
}

/// Show configuration summary
fn show_configuration_summary(
    language: &Option<String>,
    selected_methodologies: &[String],
    default_methodology: &str,
    storage_backend: &str,
    use_case_dir: &str,
    test_dir: &str,
    persona_dir: &str,
    data_dir: &str,
) -> Result<()> {
    println!("\n✨ Configuration Summary:");
    println!(
        "   Language: {}",
        language.as_ref().unwrap_or(&"none".to_string())
    );
    println!("   Methodologies: {}", selected_methodologies.join(", "));
    println!("   Default: {}", default_methodology);
    println!("   Storage: {}", storage_backend);
    println!("   Use case dir: {}", use_case_dir);
    println!("   Test dir: {}", test_dir);
    println!("   Persona dir: {}", persona_dir);
    println!("   Data dir: {}\n", data_dir);
    Ok(())
}

/// Create project configuration with directories
fn create_config_with_directories(
    runner: &mut InteractiveRunner,
    language: Option<String>,
    selected_methodologies: Vec<String>,
    storage_backend: &str,
    use_case_dir: String,
    test_dir: String,
    persona_dir: String,
    data_dir: String,
    scenario_template: Option<String>,
    create_standard_actors: bool,
) -> Result<()> {
    match runner.initialize_project(
        language,
        selected_methodologies,
        storage_backend.to_string(),
        use_case_dir,
        test_dir,
        persona_dir,
        data_dir,
        scenario_template,
    ) {
        Ok(message) => {
            UI::show_success(&message)?;

            // Save the preference in config
            if let Ok(mut config) = crate::config::Config::load() {
                config.actor.auto_create_standard_actors = create_standard_actors;
                let _ = crate::config::Config::save_config_only(&config);
            }

            // Create actors if requested
            if create_standard_actors {
                use crate::controller::ActorController;
                let actor_controller = ActorController::new()?;
                let result = actor_controller.init_standard_actors()?;

                if result.success {
                    UI::show_success(&result.message)?;
                } else {
                    UI::show_warning(&format!(
                        "Could not create standard actors: {}",
                        result.message
                    ))?;
                }
            }

            Ok(())
        }
        Err(e) => {
            UI::show_error(&format!("Failed to initialize project: {}", e))?;
            Err(e)
        }
    }
}
