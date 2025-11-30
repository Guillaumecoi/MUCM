# Testing Guide

## Running Tests

This project includes comprehensive test coverage with integration tests for controllers, CLI, and core functionality.

### Recommended: Using Nextest

For the best test experience, use **cargo nextest** which provides superior test isolation:

```bash
# Install nextest (one time)
cargo install cargo-nextest

# Run all tests
cargo nextest run

# Run specific test modules
cargo nextest run controller::tests
cargo nextest run cli::interactive::tests
```

**Results with nextest:** ✅ 530 tests pass, 2 skipped

### Alternative: Standard Test Runner

You can also use the standard Rust test runner:

```bash
# Run all library tests
cargo test --lib

# Run specific test modules
cargo test --lib controller::tests::use_case_controller_tests
cargo test --lib controller::tests::project_controller_tests
cargo test --lib cli::interactive::tests
```

**Note:** Some tests may fail when run with `cargo test` due to test isolation issues around global state (current working directory). This is a known limitation of the standard test runner. All tests pass reliably with nextest.



## Test Organization

### Controller Tests (`src/controller/tests.rs`)
- **UseCaseController tests:** 20 tests covering use case creation, scenarios, and field management
- **ProjectController tests:** 7 tests covering project initialization and configuration

### CLI Tests (`src/cli/interactive/tests.rs`)
- **InteractiveRunner tests:** 11 tests covering workflow coordination and user interactions
- **Workflow tests:** Integration tests for complete CLI workflows

### Core Tests
- **Domain entities:** Tests for use case, scenario, and metadata models
- **Application services:** Business logic and workflow tests
- **Infrastructure:** Persistence (TOML, SQLite), template engine, and registry tests

## Test Isolation

Integration tests modify global state (current working directory) and are marked with `#[serial]` to run sequentially. The `serial_test` crate ensures tests don't interfere with each other, but nextest provides additional process-level isolation for even better reliability.

## Continuous Integration

In CI pipelines, use nextest for faster and more reliable test execution:

```yaml
# Example GitHub Actions
- name: Install nextest
  run: cargo install cargo-nextest
  
- name: Run tests
  run: cargo nextest run
```

## Test Coverage

Current coverage:
- ✅ Controller layer: Use case and project operations
- ✅ CLI layer: Interactive workflows and commands
- ✅ Domain layer: Entity models and business rules
- ✅ Application layer: Service coordination and workflows
- ✅ Infrastructure layer: Persistence, templates, and registries
- ⏳ Standard CLI commands: Basic coverage (can be expanded)

Total: **530 tests** with comprehensive scenario and metadata testing.
