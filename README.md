<div align="center">
  <p align="center">
    <a href="https://github.com/Guillaumecoi/MUCM/actions/workflows/ci.yml">
      <img src="https://github.com/Guillaumecoi/MUCM/workflows/CI/badge.svg" alt="CI Status">
    </a>
    <a href="https://github.com/Guillaumecoi/MUCM/actions/workflows/test.yml">
      <img src="https://github.com/Guillaumecoi/MUCM/workflows/Tests/badge.svg" alt="Test Status">
    </a>
    <a href="https://codecov.io/gh/Guillaumecoi/MUCM">
      <img src="https://codecov.io/gh/Guillaumecoi/MUCM/branch/master/graph/badge.svg" alt="Coverage">
    </a>
    <br>
    <a href="https://github.com/Guillaumecoi/MUCM/actions/workflows/security.yml">
      <img src="https://github.com/Guillaumecoi/MUCM/workflows/Security%20Audit/badge.svg" alt="Security">
    </a>
    <a href="https://crates.io/crates/mucm">
      <img src="https://img.shields.io/crates/v/mucm.svg?logo=rust" alt="Crates.io">
    </a>
    <a href="LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
  </p>

  <img src="https://capsule-render.vercel.app/api?type=soft&color=0:667eea,100:764ba2&height=160&section=header&text=Markdown%20Use%20Case%20Manager&fontSize=42&fontColor=ffffff&fontAlignY=40&desc=Documentation%20that%20travels%20with%20your%20code&descSize=26&descAlignY=70" style="border-radius: 25px;">
</div>

## Why MUCM?

Keep your use case documentation **in your repository**, not scattered across external tools. MUCM stores structured data (TOML/SQLite) and generates clean markdown files that live alongside your code—version controlled, searchable, and always in sync.

**No cloud dependencies. No vendor lock-in. Your data, your repo.**

Perfect for teams that value documentation as code and want their requirements to travel with the codebase.

## Features

### 🧩 Extensible
- Custom fields per methodology
- Handlebars templates you can modify
- Test generation for multiple programming languages
- Combine multiple methodologies in one use case

### 🎨 Four Methodology Templates
Choose the perspective that fits your team:
- **Developer** - API design, data models, technical architecture
- **Tester** - Test scenarios, coverage metrics, quality assurance
- **Business** - ROI, stakeholder requirements, business value
- **Feature** - User stories, acceptance criteria, agile workflows

### 🎭 Scenarios & Actors
- **Four scenario types**: main, alternative, exception, extension
- **Extension scenarios**: branch from main flow at specific steps
- **Rich actors**: personas with backgrounds + system actors (Database, API, etc.)
- Actor-based steps with emoji identification

### 🗄️ Flexible Storage
- **TOML (Recommended)** - Human-readable, git-friendly, perfect for most projects  
- **SQLite (⚠️ Experimental)** - Database storage for 100+ use cases, still under active development

### 💬 Two Interfaces
**Interactive Mode** (`mucm -i`) - Menu-driven, great for getting started  
**CLI Mode** - Fast commands for automation and scripting

### 📊 Smart Status Tracking
Six status levels (Planned → Deployed) with automatic rollup from scenarios to use cases

## Quick Start

### Installation

#### Using Cargo (Recommended)
```bash
cargo install mucm
```

#### From Source
```bash
git clone https://github.com/Guillaumecoi/markdown-use-case-manager
cd markdown-use-case-manager
cargo install --path .
```

### Create Your First Project

**Option 1: Interactive Mode (Recommended)**
```bash
mucm      # or mucm -i or mucm --interactive
```
Than follow the prompts to set up your project, choose a methodology, and create your first use case, etc... 

![interactive terminal screenshot](images/interactive.png)

**[→ Interactive Mode Guide](docs/guides/workflows/interactive-mode.md)**

**Option 2: CLI Mode**

Or use direct commands (Better for automation)

```bash
# Initialize project
mucm init --methodology developer

# Create a use case
mucm create "User Authentication" --category security

...
```

**[→ Standard CLI guide](docs/guides/cli/cli-reference.md)**

### What You Get

**Configuration & Templates:**

After initialization, MUCM creates configuration files and copies template assets into your project:
All these files live in `.config/.mucm/` and can be customized as needed.

```
.config/.mucm/
├── mucm.toml                   # Project configuration
└── template-assets/            # Customizable templates (copied from source-templates)
    ├── methodologies/          # Settings and templates per methodology
    │   ├── developer/
    │   ├── feature/
    │   └── ...
    └── languages/              # Settings and templates per programming language
        ├── rust/
        ├── python/
        └── ...
```

**Input (Your Data):**

Your single source of truth for use cases is stored in structured toml files (or SQLite). That way, your data is always version controlled and easy to edit.

```
use-cases-data/
├── actors/
│   ├── admin-user.toml         # Persona definitions
│   └── auth-service.toml       # System actor definitions
└── security/
    └── UC-SEC-001.toml         # Use case source data
```
Or if you use SQLite backend:
```use-cases-data/
└── mucm.db                     # SQLite database file with all data
```

**Output (Generated Documentation):**

MUCM generates clean, organized markdown files for your use cases.

```
docs/
├── use-cases/
│   ├── README.md                             # Auto-generated overview
│   └── security/
|       ├── UC-SEC-001-feature-normal.md      # Clean markdown documentation
│       └── UC-SEC-001-developer-normal.md    # Support multiple methodologies outputs per use case
├── actors/
│   ├── admin-user.md                         # Persona documentation
│   └── auth-service.md                       # System actor documentation

tests/use-cases/
└── security/
    └── uc_sec_001.rs                         # Test scaffolding (optional)
```

## Core Concepts

### Scenarios & Steps
Break down use cases into **main** (happy path), **alternative**, **exception**, and **extension** scenarios. Extensions can branch from main scenarios at specific steps and return later. Each scenario contains actor-based steps.

**[→ Scenario Management Guide](docs/guides/workflows/scenario-management.md)**

### Rich Actor System
Create **personas** (human users with backgrounds, roles, technical experience) and **system actors** (Database, API, ExternalService types) with emoji identification. Reference actors in scenario steps.

**[→ Actor Management Guide](docs/guides/workflows/actor-management.md)**

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
│       ├── mythology.toml        # Methodology settings
│       ├── uc_normal.hbs         # Use case template (normal view)
│       ├── uc_advanced.hbs       # Use case template (advanced view)
│       └── uc_custom.hbs         # You can add more views yourself
│   └── ...
└── languages/              # Add language support
    ├── rust/
        ├── info.toml              # Language configuration
        └── test.hbs               # Test generation template
    └── ...
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
git clone https://github.com/Guillaumecoi/MUCM.git
cd MUCM
cargo build
cargo nextest run    # Requires: cargo install cargo-nextest
```

**[→ Contributing Guide](CONTRIBUTING.md)**

## License

MIT License - see [LICENSE](LICENSE) for details.