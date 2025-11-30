# CLI Reference

This document provides a comprehensive reference for all Markdown Use Case Manager (mucm) CLI commands.

## Command Overview

```bash
mucm [OPTIONS] <COMMAND>
```

## Global Options

- `-i, --interactive`: Launch interactive mode
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Commands

### `init`

Initialize a new project with templates and configuration.

```bash
mucm init [OPTIONS]
```

**Options:**
- `--language <LANGUAGE>`: Programming language for test generation (rust, python, javascript)
- `--methodology <METHODOLOGY>`: Default methodology (developer, tester, business, feature)
- `--storage <STORAGE>`: Storage backend (toml, sqlite)
- `--finalize`: Skip confirmation prompts

**Examples:**
```bash
mucm init
mucm init --language rust --methodology developer
mucm init --storage sqlite
```

### `create`

Create a new use case.

```bash
mucm create [OPTIONS] <TITLE> --category <CATEGORY>
```

**Arguments:**
- `<TITLE>`: Use case title

**Options:**
- `-c, --category <CATEGORY>`: Use case category
- `-d, --description <DESCRIPTION>`: Use case description
- `-m, --methodology <METHODOLOGY>`: Methodology to use (deprecated: use --views for multi-view support)
- `--views <VIEWS>`: Multiple views as comma-separated methodology:level pairs (e.g., `feature:normal,business:advanced`)

**Examples:**
```bash
mucm create "User Authentication" --category security
mucm create "Data Export" --category api --description "Export user data in various formats"

# Multi-view support - create use case with multiple methodology perspectives
mucm create "User Registration" --category auth --views developer:normal,tester:advanced
mucm create "Payment Processing" --category payment --views business:advanced,feature:normal
```

### `list`

List all use cases in the project.

```bash
mucm list
```

**Examples:**
```bash
mucm list
```

### `status`

Show project status and statistics.

```bash
mucm status
```

**Examples:**
```bash
mucm status
```

### `languages`

List available programming languages for test generation.

```bash
mucm languages
```

**Examples:**
```bash
mucm languages
```

### `methodologies`

List available methodologies.

```bash
mucm methodologies
```

**Examples:**
```bash
mucm methodologies
```

### `methodology-info`

Show detailed information about a specific methodology.

```bash
mucm methodology-info <NAME>
```

**Arguments:**
- `<NAME>`: Methodology name

**Examples:**
```bash
mucm methodology-info developer
```

### `regenerate`

Regenerate markdown files for use cases.

```bash
mucm regenerate [OPTIONS]
```

**Options:**
- `--use-case-id <USE_CASE_ID>`: Specific use case ID to regenerate
- `--methodology <METHODOLOGY>`: Methodology to use for regeneration
- `-a, --all`: Regenerate all use cases

**Examples:**
```bash
mucm regenerate --all
mucm regenerate --use-case-id UC-SEC-001 --methodology developer
```

## Field Management Commands

### Precondition Management

#### `precondition add`

Add a precondition to a use case.

```bash
mucm precondition add <USE_CASE_ID> <PRECONDITION>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<PRECONDITION>`: Precondition text

**Examples:**
```bash
mucm precondition add UC-SEC-001 "User must be authenticated"
mucm precondition add UC-API-001 "API key must be valid"
```

#### `precondition list`

List all preconditions for a use case.

```bash
mucm precondition list <USE_CASE_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)

**Examples:**
```bash
mucm precondition list UC-SEC-001
```

#### `precondition remove`

Remove a precondition from a use case.

```bash
mucm precondition remove <USE_CASE_ID> <INDEX>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<INDEX>`: Precondition index (1-based)

**Examples:**
```bash
mucm precondition remove UC-SEC-001 1
```

### Postcondition Management

#### `postcondition add`

Add a postcondition to a use case.

```bash
mucm postcondition add <USE_CASE_ID> <POSTCONDITION>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<POSTCONDITION>`: Postcondition text

**Examples:**
```bash
mucm postcondition add UC-SEC-001 "User session is established"
mucm postcondition add UC-API-001 "Response is sent to client"
```

#### `postcondition list`

List all postconditions for a use case.

```bash
mucm postcondition list <USE_CASE_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)

**Examples:**
```bash
mucm postcondition list UC-SEC-001
```

#### `postcondition remove`

Remove a postcondition from a use case.

```bash
mucm postcondition remove <USE_CASE_ID> <INDEX>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<INDEX>`: Postcondition index (1-based)

**Examples:**
```bash
mucm postcondition remove UC-SEC-001 1
```

### Reference Management

#### `reference add`

Add a reference relationship to a use case.

```bash
mucm reference add [OPTIONS] <USE_CASE_ID> <TARGET_ID> <RELATIONSHIP>
```

**Arguments:**
- `<USE_CASE_ID>`: Source use case ID (e.g., UC-SEC-001)
- `<TARGET_ID>`: Target use case ID (e.g., UC-AUTH-001)
- `<RELATIONSHIP>`: Relationship type (dependency, extension, inclusion, alternative)

**Options:**
- `-d, --description <DESCRIPTION>`: Optional description of the relationship

**Examples:**
```bash
mucm reference add UC-SEC-001 UC-AUTH-001 dependency "Requires authentication"
mucm reference add UC-API-001 UC-SEC-001 extension "Extends security features"
```

#### `reference list`

List all references for a use case.

```bash
mucm reference list <USE_CASE_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)

**Examples:**
```bash
mucm reference list UC-SEC-001
```

#### `reference remove`

Remove a reference from a use case.

```bash
mucm reference remove <USE_CASE_ID> <TARGET_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Source use case ID (e.g., UC-SEC-001)
- `<TARGET_ID>`: Target use case ID to remove (e.g., UC-AUTH-001)

**Examples:**
```bash
mucm reference remove UC-SEC-001 UC-AUTH-001
```

## Scenario Management Commands

All scenario management commands are nested under `usecase scenario` for better organization.

### `usecase scenario add`

Add a new scenario to a use case.

```bash
mucm usecase scenario add [OPTIONS] <USE_CASE_ID> <TITLE> --scenario-type <TYPE>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<TITLE>`: Scenario title

**Options:**
- `-t, --scenario-type <TYPE>`: Scenario type (main, alternative, exception)
- `-d, --description <DESCRIPTION>`: Optional scenario description  
- `-p, --persona <PERSONA_ID>`: Assign a persona to the scenario

**Examples:**
```bash
mucm usecase scenario add UC-SEC-001 "Happy Path Login" --scenario-type main
mucm usecase scenario add UC-SEC-001 "Login with 2FA" --scenario-type alternative
mucm usecase scenario add UC-SEC-001 "Invalid Password" --scenario-type exception
mucm usecase scenario add UC-SEC-001 "Admin Login" --scenario-type main --persona admin-user
```

### `usecase scenario edit`

Edit an existing scenario's properties.

```bash
mucm usecase scenario edit <USE_CASE_ID> <SCENARIO_ID> [OPTIONS]
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)

**Options:**
- `--title <TITLE>`: Update scenario title
- `--description <DESCRIPTION>`: Update description
- `--scenario-type <TYPE>`: Update scenario type (main, alternative, exception)
- `--status <STATUS>`: Update status (planned, in-progress, completed, deprecated)

**Examples:**
```bash
mucm usecase scenario edit UC-SEC-001 UC-SEC-001-S01 --title "Updated Login Flow"
mucm usecase scenario edit UC-SEC-001 UC-SEC-001-S01 --status in-progress
mucm usecase scenario edit UC-SEC-001 UC-SEC-001-S02 --description "New description" --status completed
```

### `usecase scenario delete`

Delete a scenario from a use case.

```bash
mucm usecase scenario delete <USE_CASE_ID> <SCENARIO_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID to delete (e.g., UC-SEC-001-S01)

**Examples:**
```bash
mucm usecase scenario delete UC-SEC-001 UC-SEC-001-S03
```

### `usecase scenario list`

List all scenarios for a use case.

```bash
mucm usecase scenario list <USE_CASE_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)

**Examples:**
```bash
mucm usecase scenario list UC-SEC-001
```

### Scenario Step Management

#### `usecase scenario step add`

Add a step to a scenario.

```bash
mucm usecase scenario step add <USE_CASE_ID> <SCENARIO_ID> <DESCRIPTION> [OPTIONS]
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)
- `<DESCRIPTION>`: Step description

**Options:**
- `-a, --actor <ACTOR>`: Actor performing the step (optional)
- `-o, --order <ORDER>`: Step order (1-based, optional - appends if not specified)

**Examples:**
```bash
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "User enters credentials"
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "System validates credentials" --order 2
mucm usecase scenario step add UC-SEC-001 UC-SEC-001-S01 "Display welcome message" --actor system
```

#### `usecase scenario step edit`

Edit an existing step in a scenario.

```bash
mucm usecase scenario step edit <USE_CASE_ID> <SCENARIO_ID> <STEP_ORDER> [OPTIONS]
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)
- `<STEP_ORDER>`: Step order number (1-based)

**Options:**
- `-d, --description <DESCRIPTION>`: Update step description
- `-a, --actor <ACTOR>`: Update actor

**Examples:**
```bash
mucm usecase scenario step edit UC-SEC-001 UC-SEC-001-S01 1 --description "Updated step text"
mucm usecase scenario step edit UC-SEC-001 UC-SEC-001-S01 2 --actor admin
```

#### `usecase scenario step remove`

Remove a step from a scenario.

```bash
mucm usecase scenario step remove <USE_CASE_ID> <SCENARIO_ID> <STEP_ORDER>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)
- `<STEP_ORDER>`: Step order to remove (1-based)

**Examples:**
```bash
mucm usecase scenario step remove UC-SEC-001 UC-SEC-001-S01 3
```

### Scenario Persona Management

#### `usecase scenario assign-persona`

Assign a persona to a scenario.

```bash
mucm usecase scenario assign-persona <USE_CASE_ID> <SCENARIO_ID> <PERSONA_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)
- `<PERSONA_ID>`: Persona ID to assign

**Examples:**
```bash
mucm usecase scenario assign-persona UC-SEC-001 UC-SEC-001-S01 admin-user
mucm usecase scenario assign-persona UC-API-001 UC-API-001-S02 developer
```

#### `usecase scenario unassign-persona`

Remove persona assignment from a scenario.

```bash
mucm usecase scenario unassign-persona <USE_CASE_ID> <SCENARIO_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID (e.g., UC-SEC-001)
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)

**Examples:**
```bash
mucm usecase scenario unassign-persona UC-SEC-001 UC-SEC-001-S01
```

### Scenario Reference Management

#### `usecase scenario reference add`

Add a reference from one scenario to another scenario or use case.

```bash
mucm usecase scenario reference add [OPTIONS] <USE_CASE_ID> <SCENARIO_ID> <TARGET_ID> \
  --ref-type <TYPE> --relationship <RELATIONSHIP>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID containing the source scenario
- `<SCENARIO_ID>`: Source scenario ID (e.g., UC-SEC-001-S01)
- `<TARGET_ID>`: Target scenario or use case ID

**Options:**
- `-t, --ref-type <TYPE>`: Reference type (scenario, usecase)
- `-r, --relationship <RELATIONSHIP>`: Relationship type (includes, extends, depends-on, alternative-to)
- `-d, --description <DESCRIPTION>`: Optional description of the reference

**Examples:**
```bash
# Reference another scenario
mucm usecase scenario reference add UC-SEC-001 UC-SEC-001-S01 UC-SEC-001-S02 \
  --ref-type scenario --relationship extends \
  --description "Extends with 2FA verification"

# Reference a use case
mucm usecase scenario reference add UC-API-001 UC-API-001-S01 UC-SEC-001 \
  --ref-type usecase --relationship depends-on \
  --description "Requires user authentication"

# Include another scenario
mucm usecase scenario reference add UC-SEC-001 UC-SEC-001-S01 UC-SEC-001-S03 \
  --ref-type scenario --relationship includes
```

#### `usecase scenario reference remove`

Remove a reference from a scenario.

```bash
mucm usecase scenario reference remove <USE_CASE_ID> <SCENARIO_ID> <TARGET_ID> \
  --relationship <RELATIONSHIP>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID containing the scenario
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)
- `<TARGET_ID>`: Target ID to remove

**Options:**
- `-r, --relationship <RELATIONSHIP>`: Relationship type of the reference to remove

**Examples:**
```bash
mucm usecase scenario reference remove UC-SEC-001 UC-SEC-001-S01 UC-SEC-001-S02 \
  --relationship extends

mucm usecase scenario reference remove UC-API-001 UC-API-001-S01 UC-SEC-001 \
  --relationship depends-on
```

#### `usecase scenario reference list`

List all references for a scenario.

```bash
mucm usecase scenario reference list <USE_CASE_ID> <SCENARIO_ID>
```

**Arguments:**
- `<USE_CASE_ID>`: Use case ID containing the scenario
- `<SCENARIO_ID>`: Scenario ID (e.g., UC-SEC-001-S01)

**Examples:**
```bash
mucm usecase scenario reference list UC-SEC-001 UC-SEC-001-S01
mucm usecase scenario reference list UC-API-001 UC-API-001-S02
```

### Scenario Reference Relationship Types

When adding scenario references, use one of these relationship types:

- `includes`: The scenario includes functionality from the target
- `extends`: The scenario extends the target with additional functionality
- `depends-on`: The scenario depends on the target being completed first
- `alternative-to`: The scenario provides an alternative path to the target

### `actor`

Manage actors (personas and system actors).

> **New in v0.1.0**: Unified actor management replacing separate persona commands. The `persona` command is still available for backward compatibility.

#### `actor create-persona`

Create a new persona (human user).

```bash
mucm actor create-persona <ID> <NAME>
```

**Arguments:**
- `<ID>`: Unique identifier (kebab-case, e.g., senior-developer)
- `<NAME>`: Display name (quoted if contains spaces)

**Examples:**
```bash
mucm actor create-persona teacher "Sarah Williams"
mucm actor create-persona admin-user "Admin User"
```

Personas are initialized with Sommerville-aligned fields (background, job role, education, technical experience, motivation) that can be filled in by editing the TOML/SQLite data.

#### `actor create-system`

Create a new system actor (database, API, service).

```bash
mucm actor create-system <ID> <NAME> <TYPE> [--emoji <EMOJI>]
```

**Arguments:**
- `<ID>`: Unique identifier (kebab-case)
- `<NAME>`: Display name
- `<TYPE>`: Actor type (System, Database, ExternalService)

**Options:**
- `--emoji <EMOJI>`: Custom emoji (defaults provided per type)

**Examples:**
```bash
# Database with default emoji (💾)
mucm actor create-system user-db "User Database" Database

# API with custom emoji
mucm actor create-system payment-api "Payment API" ExternalService --emoji 💳

# Web server with default emoji (🖥️)
mucm actor create-system web "Web Server" System
```

#### `actor init-standard`

Initialize a set of standard system actors.

```bash
mucm actor init-standard
```

Creates 10 commonly used actors: Database 💾, Web Server 🖥️, API 🌐, Payment Gateway 💳, Email Service 📧, Cache ⚡, Message Queue 📬, Auth Service 🔐, Storage 📦, Load Balancer ⚖️.

Existing actors are skipped (not overwritten).

**Examples:**
```bash
mucm actor init-standard
# ✅ Initialized 10 standard system actors
```

#### `actor update-emoji`

Update an actor's emoji.

```bash
mucm actor update-emoji <ID> <EMOJI>
```

**Arguments:**
- `<ID>`: Actor ID to update
- `<EMOJI>`: New emoji

**Examples:**
```bash
mucm actor update-emoji teacher 👩‍🏫
mucm actor update-emoji api 🚀
```

#### `actor list`

List all actors or filter by type.

```bash
mucm actor list [--actor-type <TYPE>]
```

**Options:**
- `--actor-type <TYPE>`: Filter by type (Persona, System, Database, ExternalService)

**Examples:**
```bash
mucm actor list                          # All actors
mucm actor list --actor-type Persona     # Only personas
mucm actor list --actor-type System      # Only system actors
```

#### `actor show`

Show detailed information about a specific actor.

```bash
mucm actor show <ID>
```

**Arguments:**
- `<ID>`: Actor ID

**Examples:**
```bash
mucm actor show teacher
mucm actor show database
```

#### `actor delete`

Delete an actor.

```bash
mucm actor delete <ID>
```

**Arguments:**
- `<ID>`: Actor ID to delete

**Examples:**
```bash
mucm actor delete old-service
```

⚠️ **Warning**: This permanently removes the actor data.

---

### `persona`

Legacy persona management (backward compatibility).

> **Deprecated**: Use `mucm actor` commands instead. These commands will continue to work but may be removed in future versions.

#### `persona create`

Create a new persona (same as `actor create-persona`).

```bash
mucm persona create <ID> <NAME>
```

#### `persona list`

List all personas (same as `actor list --actor-type Persona`).

```bash
mucm persona list
```

#### `persona show`

Show persona details (same as `actor show`).

```bash
mucm persona show <ID>
```

#### `persona use-cases`

List all use cases that use a specific persona in their scenarios.

```bash
mucm persona use-cases <PERSONA_ID>
```

**Arguments:**
- `<PERSONA_ID>`: Persona ID to search for

**Examples:**
```bash
# Find all use cases that use the 'admin' persona
mucm persona use-cases admin

# Example output:
# Use cases using persona 'admin':
#
# 1. UC-AUTH-001 - User Authentication (2 scenarios)
# 2. UC-ADMIN-001 - System Configuration (4 scenarios)
# 3. UC-USER-003 - Account Management (1 scenario)

# If no use cases use the persona:
mucm persona use-cases developer
# No use cases found using persona 'developer'
```

**Use Cases:**
- See which use cases reference a specific persona
- Understand persona usage across your project
- Identify unused personas (no use cases listed)
- Plan persona retirement by checking usage first

#### `persona delete`

Delete a persona.

```bash
mucm persona delete <PERSONA_ID>
```

**Arguments:**
- `<PERSONA_ID>`: Persona ID to delete (e.g., power-user-sarah)

**Examples:**
```bash
mucm persona delete power-user-sarah
```

### `cleanup`

Clean up orphaned methodology fields from TOML files. After changing views or methodologies, 
use cases may contain fields that are no longer used by any active view. This command removes 
those unused fields to keep TOML files clean.

```bash
mucm cleanup [OPTIONS] [USE_CASE_ID]
```

**Arguments:**
- `[USE_CASE_ID]`: Use case ID to clean (e.g., UC-SEC-001). If omitted, cleans all use cases

**Options:**
- `-n, --dry-run`: Show what would be cleaned without making changes

**Examples:**
```bash
# Preview cleanup for all use cases
mucm cleanup --dry-run

# Clean up all use cases
mucm cleanup

# Clean up specific use case
mucm cleanup UC-SEC-001

# Preview cleanup for specific use case
mucm cleanup UC-SEC-001 --dry-run
```

### `interactive`

Launch interactive mode for guided workflows.

```bash
mucm interactive
```

**Examples:**
```bash
mucm interactive
```

## Relationship Types

When adding references, use one of these relationship types:

- `dependency`: This use case depends on the target use case
- `extension`: This use case extends the functionality of the target use case
- `inclusion`: This use case includes the functionality of the target use case
- `alternative`: This use case provides an alternative to the target use case

## Exit Codes

- `0`: Success
- `1`: Error occurred

## Examples

### Complete Workflow

```bash
# Initialize project
mucm init --language rust --methodology developer

# Create use cases
mucm create "User Authentication" --category security
mucm create "Password Reset" --category security
mucm create "API Access" --category api

# Add preconditions and postconditions
mucm precondition add UC-SEC-001 "User has valid email address"
mucm postcondition add UC-SEC-001 "User is logged in"

# Add relationships
mucm reference add UC-API-001 UC-SEC-001 dependency "Requires authentication"

# View results
mucm list
mucm status
```

### Managing Complex Use Cases

```bash
# Create a complex use case with multiple dependencies
mucm create "Data Export Feature" --category api

# Add multiple preconditions
mucm precondition add UC-API-002 "User has export permissions"
mucm precondition add UC-API-002 "Data exists in system"
mucm precondition add UC-API-002 "Export format is supported"

# Add postconditions
mucm postcondition add UC-API-002 "Export file is generated"
mucm postcondition add UC-API-002 "Download link is provided"

# Link to related use cases
mucm reference add UC-API-002 UC-SEC-001 dependency "Authentication required"
mucm reference add UC-API-002 UC-API-001 extension "Extends API capabilities"
```