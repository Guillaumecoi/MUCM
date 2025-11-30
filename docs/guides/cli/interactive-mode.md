# Interactive Mode Guide

Interactive mode provides a guided, menu-driven interface for all MUCM operations. It's the easiest way to work with use cases, especially when you're getting started or prefer a more visual workflow.

## Launching Interactive Mode

```bash
# Start interactive mode
mucm interactive

# Or use the shorthand
mucm -i

# Or simply as interactive mode is the default
mucm
```

## Main Menu

When you launch interactive mode, you'll see the main menu:

```
What would you like to do?

📝 Manage Use Cases
👤 Manage Actors
📄 Regenerate All Documentation
⚙️ Project Settings
🚪 Exit
```

## Menu Sections

### 📝 Manage Use Cases

Access all use case operations in a submenu:

**Operations:**
- **Create New Use Case** - Add a new use case with title, category, and methodology
- **Edit Use Case** - Modify fields, add scenarios, manage steps and references
- **List All Use Cases** - View all use cases in the project
- **Show Project Status** - Overview of use case statuses and statistics

When editing a use case, you can manage:
- Basic fields (title, description, status)
- Scenarios (create main scenarios, add extension scenarios)
- Scenario steps (add, edit, remove steps with actors)
- References (dependencies, extensions, includes)
- Preconditions and postconditions

---

### 👤 Manage Actors

Access all actor and persona operations in a submenu:

**Operations:**
- **Create New Actor** - Choose between Persona (human user) or System actor (Database, API, ExternalService, Custom)
- **Edit Actor** - Update actor name, emoji, or type
- **List All Actors** - View all personas and system actors
- **Show Actor Details** - Display complete actor information
- **Delete Actor** - Remove an actor from the project

Actor creation is streamlined:
1. Select actor type (Persona, System, Database, ExternalService, Custom)
2. Enter name (ID is auto-generated from name)
3. Choose emoji for visual identification

---

### 📄 Regenerate All Documentation

Regenerate markdown files from TOML/SQLite data for all use cases. Use this when:
- You've manually edited TOML files
- Template changes need to be applied
- Documentation seems out of sync

---

### ⚙️ Project Settings

Configure project-wide settings:
- Default methodology (developer, tester, business, feature)
- Storage backend (TOML or SQLite)  
- Test generation language (Rust, Python, JavaScript, or none)
- Custom fields and templates

---

## Navigation Tips

### General Navigation
- **Arrow Keys** - Move between options
- **Enter** - Select an option
- **Esc or Ctrl+C** - Go back or exit
- **Tab** - Auto-complete (where applicable)

### Input Fields
- **Text Input** - Type freely, press Enter when done
- **Selection Lists** - Use arrow keys, Enter to select
- **Multi-Select** - Space to toggle, Enter to confirm
- **Numeric Input** - Enter numbers only

### Common Shortcuts
- Type use case ID directly when prompted (e.g., `UC-SEC-001`)
- Press `?` for help in most menus
- Press `q` to quit from list views

## Common Workflows

### Creating Your First Use Case

1. Launch interactive mode: `mucm -i`
2. Select "📝 Manage Use Cases"
3. Choose "Create New Use Case"
4. Follow prompts:
   - **Title**: "User Login"
   - **Category**: Create new or select existing (e.g., "Authentication" → generates "AUTH" abbreviation)
   - **Methodologies**: Select one or multiple with their levels
     - Example: "developer:normal" and "tester:advanced"
   - **Additional fields** (optional): Description, priority, author, reviewer
   - **Preconditions/Postconditions** (optional): Conditions that must be true before/after
   - **Create scenario?**: Option to immediately add a scenario
5. Use case created with multiple methodology views (e.g., `developer-normal.md`, `tester-advanced.md`)

### Adding Scenarios to a Use Case

1. From main menu: "📝 Manage Use Cases"
2. Select "Edit Use Case"
3. Choose your use case (e.g., UC-AUTH-001)
4. Navigate to scenario management within the use case editor
5. Create main scenario or extension scenario
6. Add steps with actors:
   - "User enters valid email"
   - "System validates credentials" (actor: database)
   - "System redirects to dashboard"

### Creating Actors

1. From main menu: "👤 Manage Actors"
2. Select "Create New Actor"
3. Choose actor type:
   - Persona (human user)
   - System (application component)
   - Database
   - ExternalService
4. Enter name (e.g., "Admin User" or "Payment API")
5. Choose emoji
6. Actor ID is auto-generated

### Viewing Project Status

1. From main menu: "📝 Manage Use Cases"
2. Select "Show Project Status"
3. Review statistics and use case breakdown

## When to Use Interactive Mode

### ✅ Use Interactive Mode When:
- Getting started with MUCM
- Creating complex use cases with many fields
- Exploring available options
- Learning MUCM commands
- Guided workflow is helpful
- Working on a single use case
- Prefer visual menus over command flags

### ⚠️ Use CLI Mode When:
- Automating tasks in CI/CD
- Scripting bulk operations
- You know exact commands
- Working with multiple use cases at once
- Performance is critical
- Integrating with other tools

## Tips for Effective Use

1. **Start with Interactive Mode** - Learn MUCM's capabilities through guided menus
2. **Use Tab Completion** - Save time with auto-completion where available
3. **Review Before Confirming** - Always check the summary before final confirmation
4. **Learn the IDs** - Note down use case IDs for quick access later
5. **Combine Both Modes** - Use interactive for complex operations, CLI for quick tasks

## Next Steps

- Review the [CLI Reference](cli-reference.md) to learn command equivalents
- Check [Configuration Guide](configuration.md) for advanced settings
- Explore [Choosing a Methodology](choosing-a-methodology.md) for methodology guidance
- See [Template Customization](template-customization.md) for advanced templates

## Getting Help

- Press `?` in most menus for context-sensitive help
- Use `mucm --help` for CLI command reference
- Check the documentation at `docs/` in your project
