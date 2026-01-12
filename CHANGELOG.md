# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Added [rustfmt.toml](rustfmt.toml) to enforce hard tabs and consistent Rust formatting.

### Changed

- Consolidated duplicated agent coding standards into `.github/instructions/Coding.instructions.md` and referenced it from developer agents.
- Standardized instruction file casing under [.github/instructions](.github/instructions) (AURORA → Aurora).
- Updated canonical card schema to v1.0.0: [schemas/Aurora.schema.json](schemas/Aurora.schema.json).
- Required cards to include `"$schema"` and tightened `snake_case` patterns for `id`, `card_type`, and `card_subtype`.
- Updated documentation and instructions to match the v2 model invariants.
- Updated `.gitignore` to include Rust build artifacts and configuration files.
- Rewrote [.github/instructions/Markdown.instructions.md](.github/instructions/Markdown.instructions.md) for clarity and consistency with `.markdownlint.json`.
- Updated [.github/instructions/Rust.instructions.md](.github/instructions/Rust.instructions.md) to require Rust 2024 and document the tab-based formatting override.
- Updated [.prettierrc.json](.prettierrc.json) to add an override for `.github/agents/*.agent.md`.

### Fixed

- Corrected a case-sensitive link typo in Architect agent documentation.
- Replaced outdated `PROGRESS.md` references with `AGENT_PROGRESS.md` in handoff guidance and docs.

### Removed

- Removed [docs/viewer.html](docs/viewer.html) (standalone HTML viewer) due to CORS issues and incompatibility with the current docs structure.

## 1.0.0 - 2026-01-08

### Added

- Initial public release
- All foundational files: repository scaffolding, documentation, and schema

## Releases

See [GitHub Releases](https://github.com/JEleniel/aurora/releases).
