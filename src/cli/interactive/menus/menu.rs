//! # Interactive Menu System
//!
//! Menu navigation and selection logic for interactive CLI mode.
//! Provides user-friendly menu interfaces for workflow selection.

use anyhow::Result;

use crate::cli::interactive::menus::settings::Settings;
use crate::cli::interactive::ui::UI;
use crate::cli::interactive::workflows::actor::ActorWorkflow;
use crate::cli::interactive::workflows::initialization::Initialization;
use crate::cli::interactive::workflows::use_case::UseCaseWorkflow;
use crate::cli::standard::CliRunner;

use super::common::{display_menu, MenuOption};

/// Run the interactive session main loop
pub fn run_interactive_session() -> Result<()> {
    let mut runner = CliRunner::new();

    // Check if project is initialized, if not offer to initialize
    if Initialization::check_and_initialize().is_err() {
        return Ok(());
    }

    UI::show_welcome()?;

    loop {
        let options = create_main_menu_options();

        if display_menu("What would you like to do?", &options, &mut runner)? {
            return Ok(()); // Exit the session
        }

        // Pause before showing menu again
        UI::pause_for_input()?;
    }
}

/// Create the main menu options
///
/// Simple action-oriented menu:
/// - Manage Use Cases: All use case operations (create, edit, list, status)
/// - Manage Actors: All actor operations (personas and system actors)
/// - Project Settings: Configuration
fn create_main_menu_options() -> Vec<MenuOption<CliRunner>> {
    vec![
        MenuOption::new("📝 Manage Use Cases", |_| {
            if let Err(e) = UseCaseWorkflow::manage_use_cases() {
                UI::show_error(&format!("Error managing use cases: {}", e))?;
            }
            Ok(false) // Don't exit
        }),
        MenuOption::new("👤 Manage Actors", |_| {
            if let Err(e) = ActorWorkflow::manage_actors() {
                UI::show_error(&format!("Error managing actors: {}", e))?;
            }
            Ok(false) // Don't exit
        }),
        MenuOption::new("📄 Regenerate All Documentation", |_| {
            use crate::controller::UseCaseController;
            
            let mut controller = match UseCaseController::new() {
                Ok(c) => c,
                Err(e) => {
                    UI::show_error(&format!("Failed to initialize controller: {}", e))?;
                    return Ok(false);
                }
            };
            
            UI::show_info("Regenerating all documentation...")?;
            
            match controller.regenerate_all_markdown() {
                Ok(result) => {
                    UI::show_success(&result.message)?;
                }
                Err(e) => {
                    UI::show_error(&format!("Error regenerating documentation: {}", e))?;
                }
            }
            
            Ok(false) // Don't exit
        }),
        MenuOption::new("⚙️  Project Settings", |_| {
            if let Err(e) = Settings::configure() {
                UI::show_error(&format!("Error configuring settings: {}", e))?;
            }
            Ok(false) // Don't exit
        }),
        MenuOption::new("🚪 Exit", |_| {
            UI::show_goodbye()?;
            Ok(true) // Exit the session
        }),
    ]
}
