use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mucm")]
#[command(version)]
pub struct Cli {
    /// Enable interactive mode
    #[arg(short, long)]
    pub interactive: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new use case manager project
    Init {
        /// Programming language for test templates (rust, python, javascript, etc.)
        #[arg(short, long)]
        language: Option<String>,
        /// Documentation methodologies (feature, business, developer, tester) - can specify multiple
        #[arg(short, long)]
        methodology: Option<String>,
        /// Storage backend (toml or sqlite)
        #[arg(long, short = 's', default_value = "toml")]
        storage: String,
        /// Finalize initialization by copying templates (run after reviewing config)
        /// Note: Finalize also syncs templates with config - if you manually edit mucm.toml
        /// to add/remove methodologies, run 'mucm init --finalize' to sync template files
        #[arg(long)]
        finalize: bool,
    },
    /// Create a new use case
    Create {
        /// Use case title
        title: String,
        /// Category (e.g., "Authentication", "API")
        #[arg(short, long)]
        category: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Documentation methodology (feature, business, developer, tester)
        /// Deprecated: Use --views for multi-view support
        #[arg(long)]
        methodology: Option<String>,
        /// Multiple views as comma-separated methodology:level pairs
        /// Example: --views feature:simple,business:normal
        #[arg(long)]
        views: Option<String>,
    },
    /// Manage use cases and their scenarios
    UseCase {
        #[command(subcommand)]
        command: UseCaseCommands,
    },
    /// List all use cases
    List,
    /// List available programming languages for templates
    Languages,
    /// List available methodologies
    Methodologies,
    /// Show methodology information
    MethodologyInfo {
        /// Methodology name to get info for
        name: String,
    },
    /// Regenerate markdown documentation from TOML files
    ///
    /// Without arguments, regenerates all use cases with their current methodology.
    /// With a use case ID, regenerates just that use case.
    /// With --methodology, changes the methodology during regeneration.
    Regenerate {
        /// Use case ID (e.g., UC-SEC-001). If omitted, regenerates all use cases.
        use_case_id: Option<String>,
        /// Documentation methodology (feature, business, developer, tester)
        /// If omitted, uses the use case's current methodology
        #[arg(long)]
        methodology: Option<String>,
        /// Regenerate all use cases (explicit flag, same as omitting use_case_id)
        #[arg(long, short)]
        all: bool,
    },
    /// Show project status
    Status,
    /// Manage use case preconditions
    Precondition {
        #[command(subcommand)]
        command: PreconditionCommands,
    },
    /// Manage use case postconditions
    Postcondition {
        #[command(subcommand)]
        command: PostconditionCommands,
    },
    /// Manage use case references
    Reference {
        #[command(subcommand)]
        command: ReferenceCommands,
    },
    /// Manage actors (personas and system actors)
    Actor {
        #[command(subcommand)]
        command: ActorCommands,
    },
    /// Clean up orphaned methodology fields from TOML files
    ///
    /// Scans all use case TOML files and removes methodology sections that are no longer
    /// used by any view in the use case. This helps maintain clean TOML files after
    /// removing views or changing methodologies.
    Cleanup {
        /// Use case ID to clean (e.g., UC-SEC-001). If omitted, cleans all use cases.
        use_case_id: Option<String>,
        /// Dry run mode - show what would be cleaned without making changes
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Reinitialize missing methodology fields in TOML files
    ///
    /// Scans all use case TOML files and ensures all fields defined in their enabled
    /// methodology views are present. Missing fields are initialized with empty values
    /// based on their type (arrays: [], numbers: 0, booleans: false, strings: "").
    /// Existing field values are never overwritten.
    Reinitialize {
        /// Use case ID to reinitialize (e.g., UC-SEC-001). If omitted, all use cases.
        use_case_id: Option<String>,
        /// Dry run mode - show what would be added without making changes
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Check/validate use case fields
    ///
    /// Validates that all use cases have:
    /// - All required fields filled with non-empty values
    /// - No irrelevant fields (fields not defined in current methodology configuration)
    ///
    /// Returns warnings only (not errors) to help identify potential issues.
    Check {
        /// Use case ID to validate (e.g., UC-SEC-001). If omitted, validates all.
        use_case_id: Option<String>,
    },
    /// Migrate data to new format (v0.2.0: actor ID format change)
    ///
    /// Updates actor files to the new ID naming convention where personas use
    /// "function-name" format (e.g., "regular-customer-sarah-chen") instead of
    /// just name. System actors remain unchanged.
    ///
    /// This command is safe to run multiple times - already migrated actors will be skipped.
    Migrate {
        /// Dry run mode - show what would be changed without modifying files
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Enter interactive mode
    Interactive,
    /// Sync templates from source to project configuration
    ///
    /// Updates methodology templates in .config/.mucm/templates/ with the latest
    /// versions from the source templates. Useful after updating the tool to get
    /// new template features like inline examples.
    SyncTemplates {
        /// Force overwrite existing template files (default: preserve customizations)
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PreconditionCommands {
    /// Add a precondition to a use case
    Add {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Precondition text
        precondition: String,
    },
    /// List preconditions for a use case
    List {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
    },
    /// Remove a precondition from a use case
    Remove {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Precondition index (1-based)
        index: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum PostconditionCommands {
    /// Add a postcondition to a use case
    Add {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Postcondition text
        postcondition: String,
    },
    /// List postconditions for a use case
    List {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
    },
    /// Remove a postcondition from a use case
    Remove {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Postcondition index (1-based)
        index: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum ReferenceCommands {
    /// Add a reference to a use case
    Add {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Target use case ID
        target_id: String,
        /// Relationship type (dependency, extension, inclusion, alternative)
        relationship: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List references for a use case
    List {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
    },
    /// Remove a reference from a use case
    Remove {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Target use case ID to remove
        target_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ActorCommands {
    /// Create a new persona
    CreatePersona {
        /// Persona ID (e.g., "primary-teacher", "admin-user")
        id: String,
        /// Persona name
        name: String,
        /// Persona function/role (e.g., "System Administrator", "End User")
        function: String,
    },
    /// Create a new system actor
    CreateSystem {
        /// Actor ID (e.g., "payment-api", "auth-database")
        id: String,
        /// Actor name
        name: String,
        /// Actor type (system, external_service, database, custom)
        #[arg(short = 't', long)]
        actor_type: String,
        /// Emoji for visual identification (optional)
        #[arg(short, long)]
        emoji: Option<String>,
    },
    /// Initialize standard system actors
    ///
    /// Creates commonly used system actors: Database, Web Server, API,
    /// Payment Gateway, Email Service, and Cache with default emojis.
    InitStandard,
    /// Update an actor's emoji
    UpdateEmoji {
        /// Actor ID
        id: String,
        /// New emoji
        emoji: String,
    },
    /// List all actors
    List {
        /// Filter by actor type (persona, system, external_service, database)
        #[arg(short = 't', long)]
        actor_type: Option<String>,
    },
    /// Show actor details
    Show {
        /// Actor ID
        id: String,
    },
    /// List all use cases that reference this actor
    UseCases {
        /// Actor ID
        id: String,
    },
    /// Delete an actor
    Delete {
        /// Actor ID
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum UseCaseCommands {
    /// Manage scenarios within a use case
    Scenario {
        #[command(subcommand)]
        command: UseCaseScenarioCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum UseCaseScenarioCommands {
    /// Add a scenario to a use case
    Add {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario title
        title: String,
        /// Scenario type (main, alternative, exception)
        #[arg(short, long)]
        scenario_type: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
        /// Optional persona ID to assign
        #[arg(short, long)]
        persona: Option<String>,
    },
    /// Edit an existing scenario
    Edit {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New type (main, alternative, exception)
        #[arg(long)]
        scenario_type: Option<String>,
        /// New status (Planned, InProgress, Implemented, Tested, Deployed)
        #[arg(long)]
        status: Option<String>,
    },
    /// Delete a scenario from a use case
    Delete {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
    },
    /// List scenarios for a use case
    List {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
    },
    /// Manage scenario steps
    Step {
        #[command(subcommand)]
        command: ScenarioStepCommands,
    },
    /// Assign a persona to a scenario
    AssignPersona {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Persona ID to assign
        persona_id: String,
    },
    /// Unassign persona from a scenario
    UnassignPersona {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
    },
    /// Manage scenario references
    Reference {
        #[command(subcommand)]
        command: UseCaseScenarioReferenceCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ScenarioStepCommands {
    /// Add a step to a scenario
    Add {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Step description
        description: String,
        /// Step order (1-based, optional - will be appended if not specified)
        #[arg(short, long)]
        order: Option<u32>,
    },
    /// Edit a scenario step
    Edit {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Step order (1-based)
        order: u32,
        /// New step description
        description: String,
    },
    /// Remove a step from a scenario
    Remove {
        /// Use case ID (e.g., UC-SEC-001)
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Step order to remove (1-based)
        order: u32,
    },
}

#[derive(Debug, Subcommand)]
pub enum UseCaseScenarioReferenceCommands {
    /// Add a reference from one scenario to another scenario or use case
    Add {
        /// Use case ID containing the source scenario
        use_case_id: String,
        /// Source scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Target ID (scenario or use case)
        target_id: String,
        /// Reference type: "scenario" or "usecase"
        #[arg(short = 't', long)]
        ref_type: String,
        /// Relationship: "includes", "extends", "depends-on", "alternative-to"
        #[arg(short, long)]
        relationship: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Remove a reference from a scenario
    Remove {
        /// Use case ID containing the scenario
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
        /// Target ID to remove
        target_id: String,
        /// Relationship type
        #[arg(short, long)]
        relationship: String,
    },
    /// List all references for a scenario
    List {
        /// Use case ID containing the scenario
        use_case_id: String,
        /// Scenario ID (e.g., UC-SEC-001-S01)
        scenario_id: String,
    },
}
