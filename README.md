
<div align="center">
  <p align="center">
    <a href="https://github.com/Guillaumecoi/MD-usecase-manager/actions/workflows/ci.yml">
      <img src="https://github.com/Guillaumecoi/MD-usecase-manager/workflows/CI/badge.svg" alt="CI Status">
    </a>
    <a href="https://github.com/Guillaumecoi/MD-usecase-manager/actions/workflows/test.yml">
      <img src="https://github.com/Guillaumecoi/MD-usecase-manager/workflows/Tests/badge.svg" alt="Test Status">
    </a>
    <a href="https://codecov.io/gh/Guillaumecoi/MD-usecase-manager">
      <img src="https://codecov.io/gh/Guillaumecoi/MD-usecase-manager/branch/main/graph/badge.svg" alt="Coverage">
    </a>
    <a href="https://github.com/Guillaumecoi/MD-usecase-manager/actions/workflows/security.yml">
      <img src="https://github.com/Guillaumecoi/MD-usecase-manager/workflows/Security%20Audit/badge.svg" alt="Security">
    </a>
    <a href="LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
  </p>

  <img src="https://capsule-render.vercel.app/api?type=soft&color=0:667eea,100:764ba2&height=160&section=header&text=Markdown%20Use%20Case%20Manager&fontSize=42&fontColor=ffffff&fontAlignY=40&desc=Documentation%20that%20travels%20with%20your%20code&descSize=26&descAlignY=70" style="border-radius: 25px;">
</div>

## Why MUCM?

Keep your use case documentation **in your repository**, not scattered across external tools. MUCM generates clean markdown files that live alongside your code—version controlled, searchable, and always in sync.

**No cloud dependencies. No vendor lock-in. Just markdown.**

Perfect for teams that value documentation as code and want their requirements to travel with the codebase.

## Key Features

### Modular Template System
- **Dynamic methodology templates** - Choose between Developer, Tester, Business, and Feature-focused approaches
- **Language-specific test generation** - Python, Rust, and JavaScript support with customizable test templates
- **Fully customizable templates** - Modify Handlebars templates to match your team's needs
- **Mix and match** - Different methodologies for different use case categories

### Dual Storage Backends

**TOML (Default)**
- Human-readable source of truth that lives in your repository
- View and edit directly in GitHub/GitLab without special tools
- Perfect for code review and version control
- Ideal for small to medium projects (< 100 use cases)
- Manual editing friendly

**SQLite**
- High-performance database for large projects (100+ use cases)
- Complex queries and relationship tracking
- Transaction support for data integrity
- CLI-driven workflow (harder to edit manually)
- Not easily viewable on GitHub/GitLab web interface

### Flexible Workflow
- **Interactive mode** - Guided workflows with smart suggestions and auto-completion
- **Script mode** - Automation-friendly for CI/CD pipelines
- **Field management** - Add, list, and remove preconditions, postconditions, and use case references
- **Both modes available** - Choose based on your context

### Professional Documentation
- **Extended metadata** - Personas, prerequisites, business value, acceptance criteria
- **Use case dependencies** - Reference and link related use cases
- **Status tracking** - Progress from planning to deployment with automatic rollup
- **Markdown export** - Works with any static site generator or documentation platform

## Quick Start

### Installation

```bash
git clone https://github.com/Guillaumecoi/markdown-use-case-manager
cd markdown-use-case-manager
cargo install --path .
```

### Create Your First Project

**Option 1: Interactive Mode (Recommended)**
```bash
mucm -i
# Follow the guided setup wizard
```

**Option 2: CLI Mode**
```bash
# Initialize project
mucm init --methodology developer

# Create a use case
mucm create "User Authentication" --category security

# View your work
mucm list
```

![interactive terminal screenshot](images/interactive.png)

### What You Get

```
docs/use-cases/
├── README.md                    # Auto-generated overview
└── security/
    └── UC-SEC-001.md           # Clean markdown documentation

tests/use-cases/
└── security/
    └── uc_sec_001.rs           # Test scaffolding
```

Standard markdown files that work with any static site generator or documentation platform.

## Core Concepts

### Scenarios & Steps
Break down use cases into scenarios with actor-based steps:
```bash
mucm usecase scenario add UC-SEC-001 "Successful login"
mucm usecase scenario step add UC-SEC-001 "successful-login" \
  --action "User enters valid credentials" --actor User
```

### Dependencies & References
Link related use cases:
```bash
mucm reference add UC-API-001 UC-AUTH-001 dependency \
  "API access requires authentication"
```

### Actors & Personas
Simple actors for quick scenarios, detailed personas for user context:
```bash
mucm actor create AdminUser --function "System administrator"
mucm persona create "Power User Sarah" --tech-level 8
```

**[→ Full CLI Reference](docs/guides/cli/cli-reference.md)**

## Methodologies

Each methodology generates documentation optimized for different audiences:

| Methodology | Focus | Best For |
|-------------|-------|----------|
| **Developer** | APIs, data models, technical specs | Engineering teams |
| **Tester** | Test scenarios, coverage, quality | QA teams |
| **Business** | ROI, stakeholder value, requirements | Product managers |
| **Feature** | User stories, acceptance criteria | Agile teams |

```bash
mucm init --methodology developer
# or mix multiple views
mucm create "Payment" --category billing --views developer:normal,tester:advanced
```

**[→ Methodology Selection Guide](docs/guides/workflows/choosing-a-methodology.md)**

## Storage Options

**TOML (Recommended)**
- ✅ Human-readable files in your repository
- ✅ Git-friendly (easy diffs, code review)
- ✅ Works great for most projects

**SQLite (⚠️ Experimental)**
- 🔬 Database storage for 100+ use cases
- 🔬 Complex queries and better performance
- ⚠️ Still under active development

```bash
mucm init --backend sqlite    # Use SQLite backend
```

**[→ Configuration Guide](docs/guides/customization/configuration.md)**

## Customization

Everything is customizable via Handlebars templates:

```
source-templates/
├── methodologies/          # Add your own methodology
│   ├── developer/
│   ├── tester/
│   ├── business/
│   └── feature/
└── languages/              # Add language support
    ├── rust/
    ├── python/
    └── javascript/
```

Create custom fields, modify templates, or build entirely new methodologies.

**[→ Template Customization Guide](docs/guides/customization/template-customization.md)**

## Documentation

- **[Getting Started](docs/guides/getting-started.md)** - Installation and first steps
- **[CLI Reference](docs/guides/cli/cli-reference.md)** - All commands and options
- **[Interactive Mode](docs/guides/cli/interactive-mode.md)** - Menu-driven interface
- **[Architecture](docs/architecture.md)** - Technical design and implementation
- **[Contributing](CONTRIBUTING.md)** - How to contribute

**[→ Full Documentation Index](docs/README.md)**

## Contributing

Contributions welcome! We especially encourage:
- 🎨 New methodology templates for different industries
- 🔧 Additional programming language support
- 📚 Documentation improvements
- 🐛 Bug reports and fixes

```bash
git clone https://github.com/Guillaumecoi/MD-usecase-manager.git
cd MD-usecase-manager
cargo build
cargo nextest run    # Requires: cargo install cargo-nextest
```

**[→ Contributing Guide](CONTRIBUTING.md)**

## License

MIT License - see [LICENSE](LICENSE) for details.