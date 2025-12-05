//! CLI commands for managing actors (personas and system actors)

use crate::cli::args::ActorCommands;
use crate::controller::ActorController;
use crate::core::{ActorType, Persona};
use crate::presentation::DisplayResultFormatter;
use anyhow::Result;
use std::str::FromStr;

/// Handle actor commands
pub fn handle_actor_command(command: ActorCommands) -> Result<()> {
    let controller = ActorController::new()?;

    match command {
        ActorCommands::CreatePersona { id, name, function } => {
            let result = controller.create_persona(id, name, function)?;
            DisplayResultFormatter::display(&result);
            Ok(())
        }
        ActorCommands::CreateSystem {
            id,
            name,
            actor_type,
            emoji,
        } => {
            let result = controller.create_system_actor(id, name, actor_type, emoji)?;
            DisplayResultFormatter::display(&result);
            Ok(())
        }
        ActorCommands::InitStandard => {
            let result = controller.init_standard_actors()?;
            DisplayResultFormatter::display(&result);
            Ok(())
        }
        ActorCommands::UpdateEmoji { id, emoji } => {
            let result = controller.update_emoji(id, emoji)?;
            DisplayResultFormatter::display(&result);
            Ok(())
        }
        ActorCommands::List { actor_type } => {
            let type_filter = if let Some(type_str) = actor_type {
                Some(ActorType::from_str(&type_str).map_err(|e| anyhow::anyhow!(e))?)
            } else {
                None
            };
            list_actors_with_controller(&controller, type_filter)
        }
        ActorCommands::Show { id } => show_actor_with_controller(&controller, &id),
        ActorCommands::UseCases { id } => list_use_cases_for_actor(&id),
        ActorCommands::Delete { id } => {
            let result = controller.delete_actor(id)?;
            DisplayResultFormatter::display(&result);
            Ok(())
        }
    }
}

/// List actors using the controller
fn list_actors_with_controller(
    controller: &ActorController,
    type_filter: Option<ActorType>,
) -> Result<()> {
    let actors = controller.list_actors(type_filter)?;

    if actors.is_empty() {
        println!("No actors found.");
        println!("\n💡 Tip: Create standard system actors with: mucm actor init-standard");
        return Ok(());
    }

    // Group by type
    let personas: Vec<_> = actors
        .iter()
        .filter(|a| a.actor_type == ActorType::Persona)
        .collect();
    let systems: Vec<_> = actors
        .iter()
        .filter(|a| a.actor_type != ActorType::Persona)
        .collect();

    if !personas.is_empty() {
        println!("👥 Personas ({}):", personas.len());
        for actor in personas {
            println!("  {} {} - {}", actor.emoji, actor.name, actor.id);
        }
        println!();
    }

    if !systems.is_empty() {
        println!("⚙️  System Actors ({}):", systems.len());
        for actor in systems {
            println!(
                "  {} {} - {} [{}]",
                actor.emoji, actor.name, actor.id, actor.actor_type
            );
        }
        println!();
    }

    Ok(())
}

/// Show actor details using the controller
fn show_actor_with_controller(controller: &ActorController, id: &str) -> Result<()> {
    let actor = controller.get_actor(id)?;

    println!("{} {}", actor.emoji, actor.name);
    println!("ID: {}", actor.id);
    println!("Type: {}", actor.actor_type);
    println!("Created: {}", actor.metadata.created_at);
    println!("Last Updated: {}", actor.metadata.updated_at);

    if !actor.extra.is_empty() {
        println!("\nFields:");
        for (key, value) in &actor.extra {
            match value {
                serde_json::Value::String(s) if !s.is_empty() => {
                    println!("  {}: {}", key, s);
                }
                serde_json::Value::Array(arr) if !arr.is_empty() => {
                    println!("  {}:", key);
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            println!("    - {}", s);
                        }
                    }
                }
                serde_json::Value::Number(n) => {
                    println!("  {}: {}", key, n);
                }
                serde_json::Value::Bool(b) => {
                    println!("  {}: {}", key, b);
                }
                _ => {} // Skip empty or null values
            }
        }
    }

    Ok(())
}

/// List use cases that reference an actor
fn list_use_cases_for_actor(id: &str) -> Result<()> {
    use crate::controller::UseCaseController;

    let uc_controller = UseCaseController::new()?;
    let use_cases = uc_controller.get_all_use_cases()?;

    let filtered: Vec<_> = use_cases
        .iter()
        .filter(|uc| {
            // Check if actor is referenced in any scenario step (as actor or receiver)
            uc.scenarios.iter().any(|s| {
                s.steps
                    .iter()
                    .any(|step| step.acting_actor() == id || (step.receiving_actor() == Some(id)))
            })
        })
        .collect();

    if filtered.is_empty() {
        println!("No use cases reference actor '{}'", id);
        return Ok(());
    }

    println!(
        "Use cases referencing actor '{}' ({}):\n",
        id,
        filtered.len()
    );
    for uc in filtered {
        println!("  {} - {}", uc.id, uc.title);
        // Show which scenarios reference this actor
        let referencing_scenarios: Vec<_> = uc
            .scenarios
            .iter()
            .filter(|s| {
                s.steps
                    .iter()
                    .any(|step| step.acting_actor() == id || (step.receiving_actor() == Some(id)))
            })
            .collect();
        for scenario in referencing_scenarios {
            println!("    └─ {}", scenario.title);
        }
    }

    Ok(())
}

/// List personas (legacy format - for internal use)
fn _list_personas_legacy(personas: &[Persona]) {
    if personas.is_empty() {
        println!("No personas found.");
        return;
    }

    println!("Personas ({}):\n", personas.len());
    for persona in personas {
        println!("  {} {} - {}", persona.emoji(), persona.name, persona.id);

        // Show a few key extra fields if they exist
        if let Some(role) = persona.extra.get("role") {
            if let Some(role_str) = role.as_str() {
                println!("     Role: {}", role_str);
            }
        }
        if let Some(dept) = persona.extra.get("department") {
            if let Some(dept_str) = dept.as_str() {
                println!("     Department: {}", dept_str);
            }
        }
        println!();
    }
}

// Note: Tests for actor commands are in integration tests
// See tests/persona_management_integration_test.rs and related
