# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- HTML model viewer with Mermaid diagram rendering: [docs/viewer.html](docs/viewer.html) and supporting assets under [docs/assets](docs/assets).
- Example Aurora model cards (mobile ordering) under [docs/aurora](docs/aurora).
- View documentation pages under [docs/views](docs/views) (Component, Deployment, Requirements, StateMachine, Story, Views).

### Changed

- Standardized example model folder name to [docs/aurora](docs/aurora) for case-sensitive filesystems.
- Standardized instruction file casing under [.github/instructions](.github/instructions) (AURORA → Aurora).
- Updated canonical card schema to v1.2.0: [schemas/Aurora.schema.json](schemas/Aurora.schema.json).
- Updated documentation and instructions to match the v2 model invariants and viewer usage.
- Updated `.gitignore` to include Rust build artifacts and configuration files.

### Fixed

- Corrected a case-sensitive link typo in Architect agent documentation.

### Removed

- (none)

## 1.0.0 - 2026-01-08

### Added

- Initial public release
- All foundational files: repository scaffolding, documentation, and schema

## Releases

See [GitHub Releases](https://github.com/JEleniel/aurora/releases).
