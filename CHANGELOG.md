# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Expose manual use-case status to templates + tests/docs ([#72](https://github.com/Guillaumecoi/MUCM/pull/72))

## [0.3.0] - 2025-12-09

### Added

- **ci:** Add automated changelog generation with git-cliff ([#62](https://github.com/Guillaumecoi/MUCM/pull/62))

### Fixed

- Resolve Indentation Accumulation Bug + Template Standardization ([#64](https://github.com/Guillaumecoi/MUCM/pull/64))

## [0.2.0] - 2025-12-07

### Added

- Exception scenario enhancements (loop-back, alternative extensions, UI cleanup) ([#58](https://github.com/Guillaumecoi/MUCM/pull/58))
- Implement folder-based use case structure ([#44](https://github.com/Guillaumecoi/MUCM/pull/44))

### Changed

- Implement boilerplate reduction and enhance actor rendering ([#38](https://github.com/Guillaumecoi/MUCM/pull/38))

### Documentation

- Improve README marketing copy and add comprehensive e-commerce demo ([#61](https://github.com/Guillaumecoi/MUCM/pull/61))

### Fixed

- Correct link to UC-AUTH-001 documentation in README
- Preconditions/postconditions rendering as [object] ([#34](https://github.com/Guillaumecoi/MUCM/pull/34))
- Prevent dry-run from mutating use case state ([#32](https://github.com/Guillaumecoi/MUCM/pull/32))
- Switch to single coverage upload without CLI code
- Remove component_management section to avoid flag conflicts
- Remove redundant path filtering from codecov flags
- Nest cli status under project in codecov.yml
- Use lcov to generate truly separate coverage reports
- Use single coverage report with Codecov flag filtering
- Upload coverage with all flags in single request
- Upload coverage with all flags in single request
- Correct codecov flag configuration
- Update CI and test badge links in README

### Miscellaneous

- Trigger coverage workflow
- Add master branch to workflow triggers
- Trigger CI workflows

## [0.1.0] - 2025-11-27

### Changed

- Rename crate from markdown-use-case-manager to mucm

### Fixed

- Add release permissions and include source files in package

[Unreleased]: https://github.com/Guillaumecoi/MUCM/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Guillaumecoi/MUCM/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Guillaumecoi/MUCM/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Guillaumecoi/MUCM/releases/tag/v0.1.0

