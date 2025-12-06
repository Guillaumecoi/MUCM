# Architecture Documentation

This document provides an overview of MUCM's architecture, design decisions, and technical implementation.

## Table of Contents

- [Overview](#overview)
- [Clean Architecture](#clean-architecture)
- [Storage Backends](#storage-backends)
- [Template System](#template-system)
- [Testing Strategy](#testing-strategy)
- [Design Decisions](#design-decisions)

## Overview

MUCM (Markdown Use Case Manager) is built with clean architecture principles, emphasizing:
- **Separation of concerns** - Clear boundaries between layers
- **Testability** - Comprehensive unit and integration tests
- **Flexibility** - Pluggable storage backends and templates
- **Maintainability** - Modular, well-documented code

### Technology Stack

- **Language:** Rust (stable)
- **CLI Framework:** clap (for argument parsing)
- **Template Engine:** Handlebars
- **Storage:** TOML (via `toml` crate) and SQLite (via `rusqlite`)
- **Testing:** cargo test / cargo nextest
- **Error Handling:** anyhow for application errors

---

## Clean Architecture

The codebase follows clean architecture layering:

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library root
├── cli/                 # Presentation Layer (CLI interface)
├── controller/          # Application Layer (business logic)
├── core/                # Core Layers
│   ├── domain/          # Domain Layer (entities, value objects)
│   ├── application/     # Application Layer (use cases)
│   └── infrastructure/  # Infrastructure Layer (persistence, file I/O)
├── config/              # Configuration management
└── presentation/        # Output formatting
```

### Layer Responsibilities

#### 1. Domain Layer (`src/core/domain/`)

Pure business entities with no external dependencies.

**Contents:**
- `UseCase` - Core use case entity
- `Scenario` - Scenario entity with steps
- `Actor` / `Persona` - User archetypes
- `Status` - Use case/scenario status enum
- Value objects and domain rules

**Characteristics:**
- No database dependencies
- No I/O operations
- Pure Rust structs and enums
- Business rule validation

**Example:**
```rust
pub struct UseCase {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub status: Status,
    pub scenarios: Vec<Scenario>,
    // ...
}
```

#### 2. Application Layer (`src/core/application/`)

Use case implementations (application-specific business rules).

**Contents:**
- Use case services
- Application-level validation
- Workflow orchestration

**Characteristics:**
- Coordinates domain entities
- Implements application workflows
- Technology-agnostic
- Depends only on domain layer

#### 3. Infrastructure Layer (`src/core/infrastructure/`)

Technical implementations for persistence and external services.

**Contents:**
- Repository implementations (TOML, SQLite)
- File system operations
- Template rendering
- External service integrations

**Characteristics:**
- Implements repository interfaces
- Handles I/O operations
- Technology-specific code
- Swappable implementations

**Example:**
```rust
pub trait UseCaseRepository {
    fn save(&mut self, use_case: &UseCase) -> anyhow::Result<()>;
    fn load(&self, id: &str) -> anyhow::Result<UseCase>;
    fn list_all(&self) -> anyhow::Result<Vec<UseCase>>;
    // ...
}

pub struct TomlRepository { /* ... */ }
pub struct SqliteRepository { /* ... */ }
```

#### 4. Controller Layer (`src/controller/`)

Application business logic coordination.

**Contents:**
- `UseCaseController` - Use case operations
- `ScenarioController` - Scenario management
- `ActorController` - Actor/persona management
- `ProjectController` - Project initialization

**Characteristics:**
- Orchestrates domain and infrastructure
- Implements business workflows
- Error handling and validation
- Transaction management

**Example:**
```rust
pub struct UseCaseController {
    repository: Box<dyn UseCaseRepository>,
    template_manager: TemplateManager,
}

impl UseCaseController {
    pub fn create_use_case(&mut self, title: &str, category: &str) 
        -> anyhow::Result<UseCase> {
        // Validation, ID generation, persistence, rendering
    }
}
```

#### 5. Presentation Layer (`src/cli/` and `src/presentation/`)

User interface and output formatting.

**Contents:**
- CLI argument parsing (`args.rs`)
- Interactive mode (`interactive/`)
- Standard CLI mode (`standard/`)
- Output formatters (`presentation/formatters/`)

**Characteristics:**
- User input handling
- Output formatting
- CLI interaction logic
- No business rules

---

## Storage Backends

MUCM supports two storage backends with a unified interface.

### Repository Pattern

Both backends implement the same trait:

```rust
pub trait UseCaseRepository {
    fn save(&mut self, use_case: &UseCase) -> anyhow::Result<()>;
    fn load(&self, id: &str) -> anyhow::Result<UseCase>;
    fn list_all(&self) -> anyhow::Result<Vec<UseCase>>;
    fn delete(&mut self, id: &str) -> anyhow::Result<()>;
    // ... additional methods
}
```

### TOML Backend

**File:** `src/core/infrastructure/persistence/toml_repository.rs`

**How it Works:**
1. Each use case stored as individual `.toml` file
2. File path: `use-cases-data/{category}/{id}.toml`
3. Markdown files generated in folder structure: `docs/use-cases/{category}/{id}/`
4. Serialization via `serde` and `toml` crate
5. Direct file I/O for read/write

**Advantages:**
- Human-readable source of truth
- Git-friendly (easy diffs, code review)
- Manual editing possible
- No database setup required
- Works great on GitHub/GitLab web interface

**Disadvantages:**
- Slower for bulk operations (100+ use cases)
- No ACID transactions
- Limited query capabilities
- File system I/O overhead

**Best For:**
- Small to medium projects (< 100 use cases)
- Teams that want version control visibility
- Projects requiring manual TOML editing
- Documentation-centric workflows

### SQLite Backend

**File:** `src/core/infrastructure/persistence/sqlite_repository.rs`

**How it Works:**
1. Single database file (`.config/mucm/usecases.db`)
2. Schema with tables for use cases, scenarios, actors
3. JSON columns for flexible metadata
4. Indexed queries for performance

**Schema Overview:**
```sql
CREATE TABLE use_cases (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    data JSON NOT NULL,  -- Full use case data
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE scenarios (
    id TEXT PRIMARY KEY,
    use_case_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    data JSON NOT NULL,
    FOREIGN KEY (use_case_id) REFERENCES use_cases(id)
);
```

**Advantages:**
- Fast queries and bulk operations
- ACID transactions
- Complex queries (joins, aggregations)
- Automatic schema migrations
- Better performance at scale (100+ use cases)

**Disadvantages:**
- Not human-readable
- Requires database file management
- Not easily viewable on GitHub/GitLab
- Requires CLI for all operations

**Best For:**
- Large projects (100+ use cases)
- Performance-critical applications
- Complex querying needs
- CI/CD pipelines with bulk operations

### Backend Selection

Configured in `.config/.mucm/mucm.toml`:

```toml
[storage]
backend = "toml"  # or "sqlite"
database_path = ".config/mucm/usecases.db"  # SQLite only
```

Switch backends:

```bash
mucm init --backend sqlite    # New project with SQLite
# Or edit mucm.toml manually
```

**Migration Between Backends:**
- Export from one backend (read all use cases)
- Import to another backend (write all use cases)
- Future: `mucm migrate --from toml --to sqlite`

---

## Template System

MUCM uses Handlebars for flexible, customizable templates.

### Template Manager

**File:** `src/config/template_manager.rs`

**Responsibilities:**
- Load templates from `source-templates/`
- Register Handlebars helpers
- Render use cases to markdown
- Manage partials (scenario templates)

### Template Structure

```
source-templates/
├── methodologies/
│   └── {methodology}/
│       ├── methodology.toml        # Config and custom fields
│       ├── uc_normal.hbs          # Normal level template
│       └── uc_advanced.hbs        # Advanced level template
├── languages/
│   └── {language}/
│       ├── info.toml              # Language config
│       └── test.hbs               # Test template
├── scenarios/
│   ├── scenario.hbs               # Default scenario template
│   └── scenario_mermaid.hbs       # Mermaid diagram template
├── actor.hbs                      # Actor documentation
├── overview.hbs                   # README generation
└── config.toml                    # Global template config
```

### Template Rendering Flow

1. **Load Template** - Read `.hbs` file from methodology directory
2. **Load Methodology Config** - Parse `methodology.toml` for custom fields
3. **Prepare Data** - Convert `UseCase` struct to Handlebars context
4. **Render** - Apply template with Handlebars engine
5. **Write Output** - Save rendered markdown to `docs/use-cases/`

### Custom Helpers

Registered Handlebars helpers:

```rust
// String formatting
handlebars.register_helper("uppercase", uppercase_helper);
handlebars.register_helper("lowercase", lowercase_helper);
handlebars.register_helper("pascal_case", pascal_case_helper);
handlebars.register_helper("snake_case", snake_case_helper);

// Numeric
handlebars.register_helper("add", add_helper);
handlebars.register_helper("multiply", multiply_helper);

// Visual
handlebars.register_helper("status_emoji", status_emoji_helper);
handlebars.register_helper("actor_emoji", actor_emoji_helper);
```

### Multi-View Support

Use cases can have multiple methodology views:

```rust
pub struct UseCase {
    pub views: HashMap<String, MethodologyView>,
    // views["developer"] = MethodologyView { level: "normal", data: {...} }
    // views["tester"] = MethodologyView { level: "advanced", data: {...} }
}
```

Each view stores its own methodology-specific fields.

---

## Testing Strategy

### Test Organization

```
tests/
├── persistence_unified_tests.rs       # Storage backend tests
├── scenario_references_integration_test.rs
├── persona_management_integration_test.rs
├── template_rendering_tests.rs        # Template system tests
└── ...

src/
└── controller/
    └── tests.rs                      # Unit tests for controllers
```

### Test Types

#### 1. Unit Tests

Located in same file as implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_case_creation() {
        // Test domain logic
    }
}
```

#### 2. Integration Tests

Located in `tests/` directory:

```rust
#[test]
#[serial]  // Sequential execution for file system tests
fn test_create_and_load_use_case() {
    // Test full workflow
}
```

#### 3. Template Tests

Validate template rendering with test data:

```rust
#[test]
fn test_developer_methodology_renders() {
    let use_case = create_test_use_case();
    let markdown = template_manager.render(&use_case)?;
    assert!(markdown.contains("## API Endpoint"));
}
```

### Running Tests

```bash
# Recommended: cargo-nextest for better isolation
cargo nextest run

# Standard test runner
cargo test --lib

# Specific test module
cargo test --lib controller::tests

# Integration tests
cargo test --test persistence_unified_tests
```

### Test Isolation

- Use `#[serial]` attribute for tests that modify global state (file system, working directory)
- cargo-nextest provides superior test isolation
- Integration tests clean up temporary directories
- Mock dependencies where possible

### Continuous Integration

GitHub Actions run:
1. Format check (`cargo fmt --check`)
2. Lint check (`cargo clippy -- -D warnings`)
3. Test suite (`cargo nextest run`)
4. Coverage report (codecov)
5. Security audit (`cargo audit`)

---

## Design Decisions

### Why Rust?

- **Performance** - Fast CLI, efficient file/database operations
- **Safety** - Memory safety, no null pointers, strong type system
- **Ecosystem** - Excellent crates for CLI, serialization, databases
- **Single binary** - Easy distribution, no runtime dependencies

### Why Handlebars for Templates?

- **Logic-less** - Enforces separation between data and presentation
- **Familiar** - Similar to Mustache, widely understood
- **Extensible** - Custom helpers for domain-specific formatting
- **Safe** - No arbitrary code execution in templates

### Why Both TOML and SQLite?

- **TOML** - Human-readable, git-friendly, great for small projects
- **SQLite** - Performance, scalability, complex queries for larger projects
- **Choice** - Let users pick based on their needs
- **Abstraction** - Repository pattern makes adding backends easy

### Why Clean Architecture?

- **Testability** - Each layer can be tested independently
- **Flexibility** - Easy to swap implementations (storage, templates)
- **Maintainability** - Clear boundaries, single responsibility
- **Onboarding** - New contributors understand structure quickly

### Why Separate Controllers?

- **Single Responsibility** - Each controller manages one aggregate
- **Testability** - Controllers can be unit tested
- **Reusability** - Controllers used by both CLI and interactive modes
- **Clarity** - Clear entry points for business operations

---

## Future Architecture Considerations

### Potential Enhancements

1. **Plugin System**
   - Custom methodology plugins
   - Third-party template repositories
   - External storage backends

2. **Web Interface**
   - Browser-based use case editor
   - REST API for external integrations
   - Real-time collaboration

3. **Export Formats**
   - PDF generation
   - HTML static sites
   - Confluence/Jira integration

4. **Advanced Querying**
   - Full-text search across use cases
   - Dependency graph visualization
   - Impact analysis tools

### Architectural Patterns to Maintain

- Keep domain layer pure (no external dependencies)
- Use repository pattern for all persistence
- Maintain clean boundaries between layers
- Favor composition over inheritance
- Keep templates logic-less

---

## Contributing to Architecture

When contributing architectural changes:

1. **Respect Layer Boundaries** - Don't bypass abstraction layers
2. **Add Tests** - Maintain >80% code coverage
3. **Document Decisions** - Update this file for major changes
4. **Consider Backward Compatibility** - Provide migration paths
5. **Discuss First** - Open an issue for major architectural changes

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed contribution guidelines.

---

## Questions?

- **Architecture questions?** Open a GitHub Discussion
- **Bug reports?** File a GitHub Issue
- **Feature ideas?** Start with an Issue to discuss architecture impact
