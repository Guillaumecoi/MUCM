# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive GitHub Actions CI/CD pipeline
- Automated quality gates (format, lint, build checks)
- Cross-platform testing with cargo-nextest
- Code coverage tracking with Codecov integration
- Weekly security audits with cargo-audit
- Automated cross-platform release builds (Linux, macOS, Windows)
- Intelligent crates.io publishing (skips doc-only changes)
- Contributing guide (CONTRIBUTING.md)
- Pull request template
- CI/CD status badges in README

## [0.1.0] - 2025-11-27

### Added
- Initial release of Markdown Use Case Manager (MUCM)
- CLI tool for managing use cases in markdown format
- Support for multiple methodologies: Developer, Tester, Business, Feature
- Dual storage backends: TOML (default) and SQLite
- Interactive mode with guided workflows
- Script mode for automation and CI/CD
- Template system with Handlebars
- Language-specific test generation (Python, Rust, JavaScript)
- Actor management (system components/services)
- Scenario management with preconditions/postconditions
- Use case dependencies and references
- Status tracking (PLANNED, IN_PROGRESS, IMPLEMENTED, TESTED, DEPLOYED, DEPRECATED)
- Extended metadata support (personas, business value, acceptance criteria)
- Markdown export compatible with static site generators
- Category management with collision detection
- Field management (preconditions, postconditions, references)
- Comprehensive test suite with 100+ tests
- Benchmark suite for TOML vs SQLite performance
- Clean Architecture implementation with modular design

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- N/A (initial release)

[Unreleased]: https://github.com/Guillaumecoi/MD-usecase-manager/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Guillaumecoi/MD-usecase-manager/releases/tag/v0.1.0
