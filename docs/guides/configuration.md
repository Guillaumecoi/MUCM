# Configuration Guide

Let's set up MUCM exactly how you want it! Your settings live in a file called `mucm.toml`.

## Where's My Config File?

After you run `mucm init`, you'll find your settings here:
`.config/.mucm/mucm.toml`

## Basic Setup

Here's what a simple config looks like:

```toml
[project]
name = "My Awesome Project"
description = "What this project is about"
author = "Your Name"

[templates]
methodology = "developer"           # Which style to use

[directories]
use_case_dir = "docs/use-cases"     # Where your use cases go
```

## The Main Settings

### Project Info
```toml
[project]
name = "My Project"                 # Shows up in your docs
description = "A cool project"      # What it's about
version = "1.0.0"                  # Version number
author = "You!"                    # Who made it
```

### Pick Your Style
```toml
[templates]
methodology = "developer"           # Options: developer, tester, business, feature
```

**Which one should you pick?**
- `developer` - Engineering teams focused on technical implementation and architecture
- `tester` - QA teams focused on testing, quality assurance, and test coverage
- `business` - Product managers and stakeholders focused on business value and requirements
- `feature` - Agile teams working with user stories and feature development

### Folder Setup
```toml
[directories]
use_case_dir = "docs/use-cases"     # Your use cases (markdown) go here
test_dir = "tests/use-cases"        # Test files go here (if you want them)
toml_dir = "data/use-cases"         # (Optional) Store TOML source files separately
```

**Note:** By default, both TOML source files and generated markdown files are stored in `use_case_dir`. If you set `toml_dir`, TOML files will be stored there instead, keeping source data separate from generated documentation. See [Separate TOML Directory Guide](./separate-toml-directory.md) for details.

### File Naming
```toml
[formatting]
id_format = "UC-{category}-{number:03}"     # How IDs look: UC-AUTH-001
filename_format = "{id}.md"                 # How files are named
```

**Want different IDs?**
- `"UC-{category}-{number:03}"` → `UC-AUTH-001`
- `"UC{number:04}"` → `UC0001` 
- `"{category}-{number}"` → `AUTH-1`

### Test Generation
```toml
[generation]
test_language = "rust"                      # Or "python" or "none"
auto_generate_tests = true                  # Create test files automatically
```

### Storage Backend
```toml
[storage]
backend = "toml"                            # Options: "toml" (default) or "sqlite"
database_path = ".config/mucm/usecases.db"  # SQLite database location (only used when backend = "sqlite")
```

**Choosing Your Storage Backend:**

**TOML (Default)**
- Human-readable files you can edit directly
- Great for version control and code review
- Perfect for small to medium projects (< 100 use cases)
- No database setup required

**SQLite**
- High-performance database storage
- Optimized for large projects (100+ use cases)
- Faster queries and bulk operations
- Automatic schema migrations
- Transaction support for data integrity

**How to Choose:**
- Start with TOML (default) for most projects
- Switch to SQLite when you have 100+ use cases or need better query performance
- Use `mucm init --backend sqlite` to start with SQLite from the beginning

## Common Setups

### For Developers and Engineers
```toml
[templates]
methodology = "developer"

[generation]
test_language = "rust"                     # Or python, javascript
auto_generate_tests = true
```

### For Product Managers and Business Analysts
```toml
[templates]
methodology = "business"
use_extended_metadata = true               # More detailed fields

[generation]
test_language = "none"                     # Focus on requirements, not tests

[storage]
backend = "sqlite"                         # Better for large business projects
```

### For QA Teams and Testers
```toml
[templates]
methodology = "tester"
use_extended_metadata = true

[generation]
test_language = "python"
auto_generate_tests = true
test_framework = "pytest"
```

### For Agile Feature Teams
```toml
[templates]
methodology = "feature"

[generation]
test_language = "javascript"
auto_generate_tests = true
```

## Custom Fields

### Global Persona Fields

Add custom fields that apply to all personas in your project:

```toml
[persona_fields]
department = { label = "Department", type = "string", required = false }
experience_level = { label = "Experience Level", type = "string", required = false }
pain_points = { label = "Pain Points", type = "array", required = false }
technical_skills = { label = "Technical Skills", type = "array", required = false }
```

**Field types:** `"string"`, `"number"`, `"boolean"`, `"array"`

These fields will appear in your persona TOML files and rendered documentation.

### Methodology-Specific Use Case Fields

Each methodology can have its own custom fields defined in its config. These live in the methodology directory:

**Location:** `source-templates/methodologies/{methodology-name}/config.toml`

**Example:**
```toml
[custom_fields]
risk_level = { label = "Risk Level", type = "string", required = true }
business_value = { label = "Business Value", type = "number", required = false }
compliance_tags = { label = "Compliance Tags", type = "array", required = false }
```

This allows different methodologies to track different information:
- **Business** methodology might need budget and stakeholder info
- **Developer** methodology might need technical debt and refactoring notes  
- **Tester** methodology might need test coverage and defect tracking

## Need Help?

**Broke something?** Run `mucm status` to check if your config is valid.

**Want to start over?** Delete the `.config` folder and run `mucm init` again.

**Can't decide?** Use `mucm -i` and it'll help you pick the right settings.