//! # Interactive Settings Configuration
//!
//! Menu-driven settings configuration for interactive CLI mode.
//! Coordinates between specialized workflow modules for different configuration domains.

use anyhow::Result;

use crate::cli::interactive::ui::UI;
use crate::config::Config;

use crate::cli::interactive::workflows::config::ConfigWorkflow;
use crate::cli::interactive::workflows::methodology::MethodologyWorkflow;

use super::common::{display_menu, MenuOption};

/// Handle settings configuration
pub struct Settings;

impl Settings {
    /// Interactive settings configuration menu
    pub fn configure() -> Result<()> {
        UI::clear_screen()?;
        UI::show_section_header("Configuration Settings", "⚙️")?;

        // Load current config
        let mut config = Config::load()?;

        loop {
            let options = Self::create_settings_options();

            if display_menu("What would you like to configure?", &options, &mut config)? {
                break; // Exit the settings menu
            }
        }

        Ok(())
    }

    /// Create the settings menu options
    fn create_settings_options() -> Vec<MenuOption<Config>> {
        vec![
            MenuOption::new("Project Information", |config| {
                ConfigWorkflow::configure_project_info(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Directory Settings", |config| {
                ConfigWorkflow::configure_directories(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Generation Settings", |config| {
                ConfigWorkflow::configure_generation(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Metadata Configuration", |config| {
                ConfigWorkflow::configure_metadata(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Scenario Template", |config| {
                ConfigWorkflow::configure_scenario_template(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Methodology Management", |config| {
                Self::manage_methodologies()?;
                // Reload config to pick up methodology changes saved by controller
                *config = Config::load()?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("View Configuration", |config| {
                ConfigWorkflow::view_config(config)?;
                Ok(false) // Continue menu
            }),
            MenuOption::new("Save & Exit", |config| {
                ConfigWorkflow::save_config(config)?;
                Ok(true) // Exit the settings menu
            }),
        ]
    }

    /// Handle methodology management submenu
    fn manage_methodologies() -> Result<()> {
        loop {
            UI::show_section_header("Methodology Management", "📚")?;

            let options = vec!["Add Methodologies", "Remove Methodologies", "Back"];
            let choice = inquire::Select::new("What would you like to do?", options).prompt()?;

            match choice {
                "Add Methodologies" => {
                    MethodologyWorkflow::add_methodologies()?;
                    UI::pause_for_input()?;
                }
                "Remove Methodologies" => {
                    MethodologyWorkflow::remove_methodologies()?;
                    UI::pause_for_input()?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }
}
