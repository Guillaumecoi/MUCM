# Actor Management Guide

This guide covers managing actors in the Markdown Use Case Manager, including both **personas** (human users) and **system actors** (databases, APIs, services).

## Overview

Actors represent entities that interact with your system in use case scenarios:

- **Personas**: Human users with backgrounds, motivations, and technical experience (aligned with Sommerville software engineering framework)
- **System Actors**: Technical components like databases, APIs, external services

All actors have:
- Unique ID
- Display name
- Emoji for visual identification
- Metadata (created/updated timestamps)
- Custom fields (flexible key-value storage)

## Creating Personas

Create a persona to represent a human user:

```bash
mucm actor create-persona <id> "<name>"
```

**Example:**
```bash
mucm actor create-persona teacher "Sarah Williams"
```

Personas are initialized with Sommerville-aligned fields defined in your `mucm.toml` config:
- **Background**: Personal circumstances, family, location
- **Job Role**: Title and responsibilities
- **Education**: Degrees, training, certifications
- **Technical Experience**: Comfort level with technology
- **Motivation for Product**: Why they want to use the system

These fields can be filled in by editing the TOML/SQLite data directly or through the interactive CLI.

## Creating System Actors

Create system actors for technical components:

```bash
mucm actor create-system <id> "<name>" <type> [--emoji <emoji>]
```

**Actor Types:**
- `System` - General system components (web servers, load balancers)
- `Database` - Data storage systems
- `ExternalService` - Third-party APIs and services

**Examples:**
```bash
# Database with default emoji (💾)
mucm actor create-system user-db "User Database" Database

# API with custom emoji
mucm actor create-system payment-api "Payment API" ExternalService --emoji 💳

# Web server with default emoji (🖥️)
mucm actor create-system web "Web Server" System
```

## Initializing Standard Actors

Quickly create a set of commonly used system actors:

```bash
mucm actor init-standard
```

This creates:
- 💾 Database
- 🖥️ Web Server
- 🌐 API
- 💳 Payment Gateway
- 📧 Email Service
- ⚡ Cache
- 📬 Message Queue
- 🔐 Auth Service
- 📦 Storage
- ⚖️ Load Balancer

Existing actors are skipped (not overwritten).

## Listing Actors

List all actors or filter by type:

```bash
# List all actors
mucm actor list

# List only personas
mucm actor list --actor-type Persona

# List only system actors
mucm actor list --actor-type System
mucm actor list --actor-type Database
mucm actor list --actor-type ExternalService
```

**Output Example:**
```
👤 Personas (2):
  👨‍💼 Admin User - admin [Persona]
  🙂 Customer - customer [Persona]

⚙️  System Actors (3):
  💾 Database - database [Database]
  🌐 API - api [System]
  📧 Email Service - email-service [ExternalService]
```

## Viewing Actor Details

Show complete details for an actor:

```bash
mucm actor show <id>
```

**Example:**
```bash
mucm actor show teacher
```

## Updating Actors

### Update Actor Emoji

Change the emoji used for visual identification:

```bash
mucm actor update-emoji <id> <emoji>
```

**Examples:**
```bash
mucm actor update-emoji teacher 👩‍🏫
mucm actor update-emoji api 🚀
```

### Update Persona Fields

Persona custom fields (background, education, etc.) can be updated by:
1. Editing the TOML file in `use-cases-data/personas/`
2. Editing the SQLite database (if using SQLite backend)
3. Using the interactive CLI (future feature)

## Deleting Actors

Remove an actor from the project:

```bash
mucm actor delete <id>
```

**Example:**
```bash
mucm actor delete old-service
```

⚠️ **Warning**: This permanently removes the actor data.

## Using Actors in Scenarios

Actors appear in use case scenarios as participants in business flows. When creating scenarios, you can:

1. Reference actors by ID in scenario steps
2. The system automatically displays their emojis in generated documentation
3. Personas can be set as primary actors for use cases

**Example Scenario Step:**
```
1. 👨‍💼 admin logs in to the system
2. 💾 database verifies credentials
3. 🌐 api returns user profile
```

## Actor Templates

Actor documentation is generated from `source-templates/actor.hbs`:

- **Personas**: Shows Sommerville fields (background, education, motivation)
- **System Actors**: Shows description, responsibilities, integration points

Generated markdown files are stored in the `actor_dir` configured in `mucm.toml` (default: `docs/actors/`).

## Configuration

### Persona Fields

Customize persona fields in `mucm.toml`:

```toml
[actor.persona_fields]
background = { type = "text", required = false, description = "...", example = "..." }
job_role = { type = "string", required = false, description = "...", example = "..." }
# Add your custom fields here
```

### Storage Backend

Actors are stored using the configured backend:

```toml
[storage]
backend = "toml"  # or "sqlite"
```

- **TOML**: Human-readable files in `use-cases-data/personas/` and `use-cases-data/actors/`
- **SQLite**: Database storage in `use-cases-data/mucm.db` (⚠️ experimental)

## Best Practices

1. **Use Descriptive IDs**: kebab-case, e.g., `senior-developer`, `payment-gateway`
2. **Choose Appropriate Emojis**: Make actors instantly recognizable in diagrams
3. **Fill Persona Details**: Rich personas lead to better requirements and design decisions
4. **Standard Actors First**: Run `init-standard` before creating custom system actors
5. **Consistent Naming**: Use clear, domain-appropriate names for actors

## Backward Compatibility

The `persona` command is still available for legacy workflows:

```bash
mucm persona create <id> "<name>"  # Same as: mucm actor create-persona
mucm persona list                   # Same as: mucm actor list --actor-type Persona
mucm persona show <id>             # Same as: mucm actor show <id>
mucm persona delete <id>           # Same as: mucm actor delete <id>
```

## Related Guides

- [CLI Reference](cli-reference.md) - Complete command documentation
- [Getting Started](getting-started.md) - Initial setup and first steps
- [Scenario Management](scenario-management.md) - Creating use case scenarios with actors
