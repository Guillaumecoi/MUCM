# Contributing to Markdown Use Case Manager

Thank you for your interest in contributing to MUCM! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors. Please be kind, professional, and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (install from [rustup.rs](https://rustup.rs/))
- **Git**: For version control
- **cargo-nextest**: Recommended for running tests

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/Guillaumecoi/MD-usecase-manager.git
   cd MD-usecase-manager
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Install locally (optional)**
   ```bash
   cargo install --path .
   ```

4. **Install development tools**
   ```bash
   # cargo-nextest for better test isolation
   cargo install cargo-nextest
   
   # cargo-audit for security checks (optional)
   cargo install cargo-audit
   ```

## Development Workflow

### Making Changes

1. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes**
   - Write clear, concise code
   - Follow existing code style and patterns
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**
   ```bash
   # Run all tests with nextest (recommended)
   cargo nextest run
   
   # Or use standard test runner
   cargo test --lib
   
   # Run specific test module
   cargo test --lib controller::tests::use_case_controller_tests
   ```

4. **Check code quality**
   ```bash
   # Format code
   cargo fmt
   
   # Run clippy (linter)
   cargo clippy --all-targets --all-features -- -D warnings
   
   # Build in release mode
   cargo build --release
   ```

## Code Style

### Formatting

- Use `cargo fmt` to format all code
- Follow Rust standard formatting conventions
- Maximum line length: 100 characters (default rustfmt)

### Linting

- All code must pass `cargo clippy` without warnings
- Run with: `cargo clippy --all-targets --all-features -- -D warnings`
- Address all clippy suggestions before submitting PR

### Code Conventions

- **Documentation**: Add doc comments (`///`) for public APIs
- **Error handling**: Use `anyhow::Result` for functions that can fail
- **Naming**: Follow Rust naming conventions
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants
- **Testing**: Use `#[serial]` attribute for tests that modify global state
- **Modules**: Keep modules focused and cohesive

## Testing Guidelines

### Test Structure

- **Unit tests**: In same file as code, in `#[cfg(test)]` module
- **Integration tests**: In `tests/` directory
- **Benchmarks**: In `benches/` directory

### Running Tests

```bash
# Recommended: Use cargo-nextest for better isolation
cargo nextest run

# Run specific test file
cargo nextest run --test persistence_unified_tests

# Run with standard test runner (may have issues with serial tests)
cargo test --lib

# Run benchmarks
cargo bench
```

### Test Requirements

- All new features must have tests
- Bug fixes should include regression tests
- Maintain or improve code coverage
- Tests must pass on all platforms (Linux, macOS, Windows)

## Commit Messages

### Format

Follow conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, no code change)
- `refactor`: Code refactoring (no feature change)
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates
- `ci`: CI/CD changes

### Examples

```
feat(cli): add support for batch use case creation

Implement new command to create multiple use cases from JSON file.
Includes validation and progress reporting.

Closes #123
```

```
fix(persistence): handle null values in SQLite backend

Previously, null preconditions caused deserialization errors.
Added proper null handling in persistence layer.
```

```
docs: update contributing guide with commit conventions
```

## Pull Request Process

### Before Submitting

1. **Update tests**: Ensure all tests pass
2. **Update docs**: Update README, docs, or code comments as needed
3. **Update CHANGELOG**: Add entry under `[Unreleased]` section
4. **Format and lint**: Run `cargo fmt` and `cargo clippy`
5. **Commit changes**: Use conventional commit messages

### Submitting PR

1. **Push your branch**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**
   - Use the PR template (auto-filled)
   - Provide clear description of changes
   - Link related issues
   - Add screenshots/examples if applicable

3. **Code Review**
   - Address reviewer feedback promptly
   - Keep discussions focused and professional
   - Update PR based on suggestions

4. **Merge Requirements**
   - All CI checks must pass (format, lint, tests)
   - At least one approval required
   - No merge conflicts with base branch
   - CHANGELOG.md updated

## Versioning Strategy

This project follows [Semantic Versioning](https://semver.org/):

### Pre-1.0 (Current: 0.1.0)

- Version format: `0.x.y`
- **Minor (x)**: Breaking changes, major features
- **Patch (y)**: Bug fixes, small improvements, non-breaking features

### Version Bump Guidelines

- **Code changes**: Require version bump in `Cargo.toml`
- **Documentation only**: No version bump needed
- **Breaking changes**: Increment minor (0.1.0 → 0.2.0)
- **Bug fixes**: Increment patch (0.1.0 → 0.1.1)
- **New features**: Increment minor or patch depending on scope

### Post-1.0 (Future)

When the project reaches 1.0:
- Follow strict semver: `MAJOR.MINOR.PATCH`
- Document migration guides for breaking changes

## Release Process

Releases are automated via GitHub Actions:

1. **Prepare release**
   - Update version in `Cargo.toml`
   - Update `CHANGELOG.md` (move Unreleased to new version)
   - Commit: `chore: bump version to x.y.z`

2. **Create tag**
   ```bash
   git tag -a v0.1.1 -m "Release v0.1.1"
   git push origin v0.1.1
   ```

3. **Automated process**
   - CI builds binaries for Linux, macOS, Windows
   - GitHub release created with artifacts
   - Published to crates.io (if code changed)

## Documentation

### Code Documentation

- Add doc comments (`///`) for all public APIs
- Include examples in doc comments where helpful
- Document panics, errors, and safety invariants

### User Documentation

- Update `README.md` for user-facing changes
- Add guides in `docs/guides/` for new features
- Update CLI reference in `docs/reference/`

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue with reproduction steps
- **Feature requests**: Open a GitHub Issue with use case description
- **Security**: Email security@example.com (update with actual email)

## Project Structure

```
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── cli/                 # CLI argument parsing and interactive mode
│   ├── config/              # Configuration management
│   ├── controller/          # Business logic controllers
│   ├── core/                # Core domain, application, infrastructure
│   └── presentation/        # Output formatting
├── tests/                   # Integration tests
├── benches/                 # Benchmark tests
├── docs/                    # User documentation
├── source-templates/        # Handlebars templates
└── example/                 # Example project
```

## Architecture

The project follows Clean Architecture principles:

- **Core**: Domain entities, use cases, repository interfaces
- **Controller**: Business logic coordination
- **Infrastructure**: Persistence (TOML, SQLite), file system
- **Presentation**: CLI, formatting, output

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to MUCM! Your efforts help make documentation better for everyone. 🎉
