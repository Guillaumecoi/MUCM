# Documentation Index

Welcome to the MUCM (Markdown Use Case Manager) documentation!

## Quick Start

New to MUCM? Start here:

1. **[Getting Started Guide](guides/getting-started.md)** - Installation and first steps
2. **[Choosing a Methodology](guides/choosing-a-methodology.md)** - Pick the right approach for your team
3. **[Interactive Mode Guide](guides/interactive-mode.md)** - Learn the guided interface

## User Guides

### CLI Interfaces

- **[CLI Reference](guides/cli/cli-reference.md)** - Complete command reference for standard CLI
- **[Interactive Mode](guides/cli/interactive-mode.md)** - Menu-driven interface guide

### Workflows

- **[Use Case Management](../README.md#getting-started)** - Creating and managing use cases
- **[Scenario Management](guides/workflows/scenario-management.md)** - Working with scenarios and steps
- **[Actor Management](guides/workflows/actor-management.md)** - Managing actors and personas
- **[Choosing a Methodology](guides/workflows/choosing-a-methodology.md)** - Methodology comparison and selection

### Customization

- **[Configuration Guide](guides/customization/configuration.md)** - Project setup and settings
- **[Template Customization](guides/customization/template-customization.md)** - Customizing Handlebars templates

### Testing

- **[Testing Guide](guides/testing/testing.md)** - Running tests and CI setup

## Technical Documentation

### For Contributors

- **[Architecture Overview](architecture.md)** - System design and technical architecture
- **[Contributing Guide](../CONTRIBUTING.md)** - Development workflow and standards

## Methodology Documentation

MUCM supports four methodologies, each with its own focus:

| Methodology | Best For | Documentation |
|-------------|----------|---------------|
| **Developer** | Engineering teams, technical implementation | `mucm methodology-info developer` |
| **Tester** | QA teams, test-driven development | `mucm methodology-info tester` |
| **Business** | Product managers, stakeholder communication | `mucm methodology-info business` |
| **Feature** | Agile teams, user story workflows | `mucm methodology-info feature` |

Run `mucm methodologies` to list all available methodologies.

## Quick Reference

### Common Commands

```bash
# Initialize project
mucm init

# Create use case
mucm create "Title" --category category

# List all use cases
mucm list

# Interactive mode
mucm -i

# Get help
mucm --help
mucm <command> --help
```

### Storage Options

- **TOML** (default) - Human-readable, git-friendly, < 100 use cases
- **SQLite** - High performance, 100+ use cases, complex queries

See [Configuration Guide](guides/configuration.md#storage-backend) for details.

## Documentation Structure

```
docs/
├── README.md                          # This file
├── architecture.md                    # Technical architecture
└── guides/
    ├── getting-started.md             # Quick start guide
    ├── cli/
    │   ├── cli-reference.md           # Standard CLI reference
    │   └── interactive-mode.md        # Interactive mode guide
    ├── workflows/
    │   ├── choosing-a-methodology.md  # Methodology selection
    │   ├── actor-management.md        # Actors and personas
    │   └── scenario-management.md     # Scenarios and steps
    ├── customization/
    │   ├── configuration.md           # Configuration options
    │   └── template-customization.md  # Template system
    └── testing/
        └── testing.md                 # Testing workflows
```

## External Resources

- **GitHub Repository:** [Guillaumecoi/MUCM](https://github.com/Guillaumecoi/MUCM)
- **Issue Tracker:** [GitHub Issues](https://github.com/Guillaumecoi/MUCM/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Guillaumecoi/MUCM/discussions)
- **License:** MIT License

## Getting Help

- **Questions?** Check the relevant guide above or open a Discussion
- **Bug reports?** File an Issue with reproduction steps
- **Feature requests?** Open an Issue with your use case
- **Contributing?** Read [CONTRIBUTING.md](../CONTRIBUTING.md)

---

**Version:** 0.1.0  
**Last Updated:** December 2025
