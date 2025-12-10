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

**Documentation that travels with your code.** MUCM is a documentation compiler that turns structured data (TOML or SQLite) into beautifully formatted markdown documentation. Write once, generate multiple perspectives for developers, testers, business analysts, and product managers.

### The Problem
- **Online tools create barriers** - contributors need accounts, logins, permissions just to read/edit docs
- **Documentation lives separately from code** - not in your repo, not in pull requests, not in your workflow
- **Keeping it structured manually is painful** - consistent formatting across dozens of use cases is tedious
- **Vendor lock-in** - your documentation is trapped in proprietary systems

git clone https://github.com/Guillaumecoi/MUCM
cd MUCM
- **Customizable templates without coding** - Handlebars templates you can edit to format output exactly how you want
- **Version-controlled source of truth** in your repo, no servers, no online services, zero barriers
- **Auto-generated test scaffolding** with safe zones so your code survives regeneration
- **Smart cross-references** that automatically link use cases, actors, and preconditions
- **CI/CD ready** - generate docs automatically in GitHub Actions, no installation needed for contributors.
- **Multi-view documentation** tailored for different teams (same data, different perspectives)

### See It In Action

**[→ E-Commerce Demo](examples/ecommerce-demo/)** - Complete authentication system with use cases, personas, tests, and Mermaid diagrams. Built in 10 minutes. Zero manual markdown.

## Features


**🔧 Extend Everything**  
Create custom methodologies for your industry. Add support for new test languages. Build your own template fields. MUCM adapts to your workflow, not the other way around.

**🎨 Customize Templates Without Coding**  
Edit Handlebars templates to format your documentation exactly how you want. Change headings, reorder sections, add custom fields - no programming experience required. Copy the templates to your project, tweak them in any text editor, and regenerate. Your documentation, your style.

**🔗 Smart Cross-References That Never Break**  
Type `||UC:UC-AUTH-001:depend` in any precondition - MUCM automatically generates clickable markdown links to the right use case. Rename a file? Move a category? All references update automatically. No more hunting for broken links.

**🧪 Test Scaffolding That Survives Regeneration**  
Generate test files in Python, Rust, Java or JavaScript (and soon many more) with protected "safe zones" for your code. Write your test logic once between the markers - regenerate docs 100 times and your implementation stays intact. Only the scaffolding updates.

**📊 Interactive Diagrams Built From Your Scenarios**  
Describe your workflow in scenario steps - MUCM generates Mermaid sequence diagrams automatically. Actors, interactions, all visualized. Change a step, regenerate, diagram updates. No drawing tools needed.

**👥 Rich Actor Profiles**  
Create realistic personas (background, motivations, technical skills) and system actors (APIs, databases, external services). Every actor gets an emoji, a full profile page, and appears consistently across all scenarios.

**🎯 Four Methodology Views (Or Mix Them)**  
Generate documentation tailored for different audiences from the same source:
- **Developer** - API specs, data models, security requirements, technical architecture
- **Tester** - Test scenarios, coverage areas, automation status, quality gates
- **Business** - ROI analysis, stakeholder requirements, business value, success metrics
- **Feature** - User stories, acceptance criteria, agile workflows, sprint planning

**⚡ Two Ways to Work**  
- **Interactive Mode** (`mucm -i`) - Guided menus, perfect for getting started or complex edits
- **CLI Mode** - Lightning-fast commands for automation, scripting, and CI/CD pipelines

**💾 Choose Your Storage**  
- **TOML (Recommended)** - Human-readable files, beautiful diffs, perfect for code review and version control
- **SQLite (Experimental)** - Database backend for projects with 100+ use cases, complex queries, and better performance


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
│       ├── README.md                         # Category overview (auto-generated)
│       └── UC-SEC-001/                       # Each use case has its own folder
│           ├── README.md                     # Main documentation (single view)
│           ├── UC-SEC-001-feature-normal.md  # Or multiple methodology views
│           └── UC-SEC-001-developer-normal.md
├── personas/
│   ├── admin-user.md                         # Persona documentation
│   └── auth-service.md                       # System actor documentation

tests/use-cases/
└── security/
    └── uc_sec_001.rs                         # Test scaffolding (optional)
```

## Documentation

**Start Here:**
- **[E-Commerce Demo](examples/ecommerce-demo/)** - See MUCM in action with a real project
- **[Getting Started](docs/guides/getting-started.md)** - Installation and your first use case
- **[CLI Reference](docs/guides/cli/cli-reference.md)** - All commands and options

**Guides:**
- **[Interactive Mode](docs/guides/cli/interactive-mode.md)** - Menu-driven interface walkthrough
- **[Choosing a Methodology](docs/guides/workflows/choosing-a-methodology.md)** - Pick the right view
- **[Template Customization](docs/guides/customization/template-customization.md)** - Make MUCM yours

**Deep Dive:**
- **[Architecture](docs/architecture.md)** - Technical design and implementation
- **[Full Documentation Index](docs/README.md)** - Everything else

## Community & Support

**Questions?** Join the [discussions](https://github.com/Guillaumecoi/MD-usecase-manager/discussions)  
**Found a bug?** [Open an issue](https://github.com/Guillaumecoi/MD-usecase-manager/issues)  
**Want to contribute?** See our [Contributing Guide](CONTRIBUTING.md)

We especially encourage:
- New methodology templates for different industries
- Additional programming language support for test generation
- Documentation improvements and examples
- Feature requests and bug reports
- Feature development, code contributions and bug fixes

## License

MIT License - see [LICENSE](LICENSE) for details.