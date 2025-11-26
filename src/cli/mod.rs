/// CLI module for the Markdown Use Case Manager.
///
/// This module provides the command-line interface, handling argument parsing,
/// command dispatching, and user interaction modes. It integrates Clap for
/// argument parsing and coordinates between interactive and regular command modes.
///
/// ## Modules
/// - `args`: Defines CLI argument structures using Clap.
/// - `commands`: Thin command handlers that delegate to business logic.
/// - `interactive`: Interactive session management for guided usage.
/// - `runner`: Core business logic and file operations.
///
/// ## Flow
/// 1. Parse CLI arguments with Clap.
/// 2. Check for interactive mode (flag, subcommand, or no args).
/// 3. For regular commands, create a runner and dispatch to handlers.
/// 4. Handlers perform CLI-specific tasks and call runner methods.
// Private modules
mod args;
mod interactive;
mod standard;

use anyhow::Result;
use clap::Parser;

use crate::controller::DisplayResult;
use crate::presentation::DisplayResultFormatter;
use args::{Cli, Commands};
use interactive::run_interactive_session;
use standard::{
    handle_actor_command, handle_cleanup_command, handle_create_command, handle_init_command,
    handle_languages_command, handle_list_command, handle_list_methodologies_command,
    handle_methodology_info_command, handle_postcondition_add_command,
    handle_postcondition_list_command, handle_postcondition_remove_command,
    handle_precondition_add_command, handle_precondition_list_command,
    handle_precondition_remove_command, handle_reference_add_command,
    handle_reference_list_command, handle_reference_remove_command, handle_regenerate_command,
    handle_status_command, handle_usecase_scenario_command, CliRunner,
};

/// Execute a command with proper error handling and colored output
fn execute_command<F>(command_fn: F)
where
    F: FnOnce() -> Result<()>,
{
    match command_fn() {
        Ok(()) => {}
        Err(e) => {
            DisplayResultFormatter::display(&DisplayResult::error(e.to_string()));
            std::process::exit(1);
        }
    }
}

/// Main CLI entry point.
///
/// Parses command-line arguments and dispatches to the appropriate handler.
/// Supports both interactive mode (for guided usage) and direct command execution.
///
/// Interactive mode is activated when:
/// - The `--interactive` flag is used
/// - The `interactive` subcommand is specified
/// - No command is provided (defaults to interactive)
///
/// For regular commands, creates a CliRunner instance and delegates to
/// command-specific handlers in the `commands` module.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Check if interactive mode is requested
    if cli.interactive
        || matches!(cli.command, Some(Commands::Interactive))
        || cli.command.is_none()
    {
        return run_interactive_session();
    }

    // Handle regular commands
    let mut runner = CliRunner::new();

    let Some(command) = cli.command else {
        // This shouldn't happen due to clap validation, but handle gracefully
        anyhow::bail!("No command specified. Use --help for available commands.");
    };

    match command {
        Commands::Init {
            language,
            methodology,
            storage,
            finalize,
        } => {
            let methodologies: Vec<String> = methodology
                .as_ref()
                .map(|s| {
                    s.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_else(|| vec!["feature".to_string()]);
            execute_command(|| {
                handle_init_command(&mut runner, language, methodologies, storage, finalize)
            });
            Ok(())
        }
        Commands::Create {
            title,
            category,
            description,
            methodology,
            views,
        } => {
            execute_command(|| {
                handle_create_command(
                    &mut runner,
                    title,
                    category,
                    description,
                    methodology,
                    views,
                )
            });
            Ok(())
        }
        Commands::List => {
            execute_command(|| handle_list_command(&mut runner));
            Ok(())
        }
        Commands::Languages => {
            execute_command(handle_languages_command);
            Ok(())
        }
        Commands::Methodologies => {
            execute_command(|| handle_list_methodologies_command(&mut runner));
            Ok(())
        }
        Commands::MethodologyInfo { name } => {
            execute_command(|| handle_methodology_info_command(&mut runner, name));
            Ok(())
        }
        Commands::Regenerate {
            use_case_id,
            methodology,
            all,
        } => {
            execute_command(|| {
                handle_regenerate_command(&mut runner, use_case_id, methodology, all)
            });
            Ok(())
        }
        Commands::Status => {
            execute_command(|| handle_status_command(&mut runner));
            Ok(())
        }
        Commands::Precondition { command } => match command {
            args::PreconditionCommands::Add {
                use_case_id,
                precondition,
            } => {
                execute_command(|| {
                    handle_precondition_add_command(&mut runner, use_case_id, precondition)
                });
                Ok(())
            }
            args::PreconditionCommands::List { use_case_id } => {
                execute_command(|| handle_precondition_list_command(&mut runner, use_case_id));
                Ok(())
            }
            args::PreconditionCommands::Remove { use_case_id, index } => {
                execute_command(|| {
                    handle_precondition_remove_command(&mut runner, use_case_id, index)
                });
                Ok(())
            }
        },
        Commands::Postcondition { command } => match command {
            args::PostconditionCommands::Add {
                use_case_id,
                postcondition,
            } => {
                execute_command(|| {
                    handle_postcondition_add_command(&mut runner, use_case_id, postcondition)
                });
                Ok(())
            }
            args::PostconditionCommands::List { use_case_id } => {
                execute_command(|| handle_postcondition_list_command(&mut runner, use_case_id));
                Ok(())
            }
            args::PostconditionCommands::Remove { use_case_id, index } => {
                execute_command(|| {
                    handle_postcondition_remove_command(&mut runner, use_case_id, index)
                });
                Ok(())
            }
        },
        Commands::Reference { command } => match command {
            args::ReferenceCommands::Add {
                use_case_id,
                target_id,
                relationship,
                description,
            } => {
                execute_command(|| {
                    handle_reference_add_command(
                        &mut runner,
                        use_case_id,
                        target_id,
                        relationship,
                        description,
                    )
                });
                Ok(())
            }
            args::ReferenceCommands::List { use_case_id } => {
                execute_command(|| handle_reference_list_command(&mut runner, use_case_id));
                Ok(())
            }
            args::ReferenceCommands::Remove {
                use_case_id,
                target_id,
            } => {
                execute_command(|| {
                    handle_reference_remove_command(&mut runner, use_case_id, target_id)
                });
                Ok(())
            }
        },
        Commands::UseCase { command } => match command {
            args::UseCaseCommands::Scenario { command } => {
                handle_usecase_scenario_command(&mut runner, command)?;
                Ok(())
            }
        },
        Commands::Actor { command } => handle_actor_command(command),
        Commands::Cleanup {
            use_case_id,
            dry_run,
        } => {
            execute_command(|| handle_cleanup_command(&mut runner, use_case_id, dry_run));
            Ok(())
        }
        Commands::Interactive => {
            // This case is handled above, but included for completeness
            run_interactive_session()
        }
    }
}
