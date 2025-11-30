# Interactive Mode Guide

Interactive mode provides a guided, menu-driven interface for all MUCM operations. It's the easiest way to work with use cases, especially when you're getting started or prefer a more visual workflow.

## Launching Interactive Mode

```bash
# Start interactive mode
mucm interactive

# Or use the shorthand
mucm -i
```

## Main Menu

When you launch interactive mode, you'll see the main menu with these options:

```
┌─────────────────────────────────────┐
│  Markdown Use Case Manager (MUCM)   │
│         Interactive Mode            │
├─────────────────────────────────────┤
│  1. Project Management              │
│  2. Use Case Management             │
│  3. Scenario Management             │
│  4. Actor Management                │
│  5. Field Management                │
│  6. View & Status                   │
│  7. Settings                        │
│  8. Exit                            │
└─────────────────────────────────────┘
```

## Menu Sections

### 1. Project Management

Initialize and configure your project.

**Operations:**
- **Initialize Project** - Set up a new MUCM project
  - Choose methodology (developer, tester, business, feature)
  - Select storage backend (TOML or SQLite)
  - Configure test language (Rust, Python, JavaScript, or none)
  - Set project metadata

**When to Use:**
- First time setting up MUCM
- Starting a new project
- Reconfiguring project settings

---

### 2. Use Case Management

Create, edit, and manage use cases.

**Operations:**
- **Create New Use Case** - Add a new use case to your project
- **List Use Cases** - View all use cases
- **Edit Use Case** - Modify existing use case details
- **Delete Use Case** - Remove a use case
- **Regenerate Documentation** - Update markdown from TOML

**Typical Workflow:**
1. Select "Create New Use Case"
2. Enter title (e.g., "User Authentication")
3. Choose category (e.g., "Security")
4. Add description (optional)
5. Select methodology or views
6. Confirm creation

---

### 3. Scenario Management

Manage scenarios within use cases.

**Operations:**
- **Add Scenario** - Create a new scenario for a use case
- **List Scenarios** - View all scenarios in a use case
- **Edit Scenario** - Modify scenario details
- **Delete Scenario** - Remove a scenario
- **Manage Scenario Steps** - Add, edit, or remove individual steps
- **Assign Actors** - Link actors/personas to scenarios

**Typical Workflow:**
1. Select use case (e.g., UC-SEC-001)
2. Choose "Add Scenario"
3. Enter scenario name (e.g., "Successful login")
4. Set status (PLANNED, IN_PROGRESS, etc.)
5. Add steps one by one
6. Assign actors to relevant steps

---

### 4. Actor Management

Manage actors (simple roles) and personas (detailed user profiles).

**Operations:**
- **Create Actor** - Define a simple role-based actor
- **Create Persona** - Create detailed user profile
- **List Actors** - View all actors and personas
- **Edit Actor/Persona** - Modify actor or persona details
- **Delete Actor/Persona** - Remove actor or persona

**Persona Creation Workflow:**
1. Select "Create Persona"
2. Enter name (e.g., "Power User Sarah")
3. Add description
4. Set technical level (1-10)
5. Define goals
6. Add custom fields (if configured)

---

### 5. Field Management

Manage preconditions, postconditions, and references.

**Operations:**
- **Add Precondition** - Define condition required before use case
- **Remove Precondition** - Delete precondition
- **Add Postcondition** - Define expected outcome
- **Remove Postcondition** - Delete postcondition
- **Add Use Case Reference** - Link to related use case
- **Remove Reference** - Delete use case link

**Typical Workflow:**
1. Select use case
2. Choose field type (precondition, postcondition, reference)
3. Enter field value
4. For references, select relationship type:
   - Dependency
   - Extension
   - Inclusion
   - Alternative

---

### 6. View & Status

View project information and statistics.

**Operations:**
- **Project Status** - Overview of all use cases and their statuses
- **List Use Cases** - Detailed list view
- **View Use Case Details** - Inspect specific use case
- **Show Statistics** - Counts, completion rates, scenario distribution

**Status Display:**
```
Project: My Application
Use Cases: 12 total
  📋 PLANNED: 3
  🔄 IN_PROGRESS: 5
  ⚡ IMPLEMENTED: 2
  ✅ TESTED: 1
  🚀 DEPLOYED: 1
```

---

### 7. Settings

Configure project settings and preferences.

**Operations:**
- **Change Methodology** - Update default methodology
- **Update Storage Backend** - Switch between TOML and SQLite
- **Configure Test Generation** - Change test language settings
- **View Configuration** - Display current settings
- **Edit Configuration File** - Open mucm.toml for advanced editing

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
2. Select "2. Use Case Management"
3. Choose "Create New Use Case"
4. Follow prompts:
   - Title: "User Login"
   - Category: "Authentication"
   - Description: "Allow users to log in with email and password"
   - Methodology: "developer"
5. Confirm creation
6. Optionally add scenarios immediately

### Adding Scenarios to a Use Case

1. From main menu: "3. Scenario Management"
2. Enter use case ID (e.g., UC-AUTH-001)
3. Select "Add Scenario"
4. Enter scenario details:
   - Name: "Successful login with valid credentials"
   - Status: IN_PROGRESS
5. Add steps:
   - Step 1: "User enters valid email"
   - Step 2: "User enters correct password"
   - Step 3: "System validates credentials"
   - Step 4: "System redirects to dashboard"
6. Assign actors to steps (e.g., "User", "System")

### Managing Use Case Dependencies

1. From main menu: "5. Field Management"
2. Select "Add Use Case Reference"
3. Enter source use case ID
4. Enter target use case ID
5. Choose relationship type: "dependency"
6. Add description: "Requires user to be authenticated"

### Viewing Project Status

1. From main menu: "6. View & Status"
2. Select "Project Status"
3. Review statistics and use case breakdown
4. Optionally drill down into specific use cases

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

## Troubleshooting

### Interactive Mode Won't Start
```bash
# Ensure you're in a project directory
cd /path/to/your/project

# Or initialize first
mucm init
```

### Can't Find Use Case
- Use "List Use Cases" to see all available IDs
- Check that you're in the correct project directory
- Verify case-sensitivity of use case IDs

### Changes Not Appearing
- Regenerate documentation: `mucm regenerate --all`
- Check that TOML files were actually modified
- For SQLite backend, verify database is being updated

### Menu Not Displaying Correctly
- Ensure terminal supports Unicode
- Try resizing terminal window
- Check terminal color support

## Advanced Interactive Features

### Batch Operations
Some interactive menus support batch operations:
- Select multiple use cases for regeneration
- Bulk status updates
- Mass field additions

### Search and Filter
In list views:
- Type to search/filter
- Use `/` to enter search mode
- Press Enter to select from filtered results

### Context Memory
Interactive mode remembers:
- Last used use case ID
- Recent categories
- Previous selections

This makes repeated operations faster.

## Next Steps

- Review the [CLI Reference](cli-reference.md) to learn command equivalents
- Check [Configuration Guide](configuration.md) for advanced settings
- Explore [Choosing a Methodology](choosing-a-methodology.md) for methodology guidance
- See [Template Customization](template-customization.md) for advanced templates

## Getting Help

- Press `?` in most menus for context-sensitive help
- Use `mucm --help` for CLI command reference
- Check the documentation at `docs/` in your project
