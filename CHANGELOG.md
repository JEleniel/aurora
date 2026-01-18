# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Added [rustfmt.toml](rustfmt.toml) to enforce hard tabs and consistent Rust formatting.
- Added a `compact` CLI subcommand that exports the model as a single JSON file (sans `audit_trail` and link `relationship` fields) for agent-friendly consumption.

### Changed

- Consolidated duplicated agent coding standards into `.github/instructions/Coding.instructions.md` and referenced it from developer agents.
- Standardized instruction file casing under [.github/instructions](.github/instructions) (AURORA → Aurora).
- Updated canonical card schema to v2.0.0: [schemas/Aurora.schema.json](schemas/Aurora.schema.json).
- Aligned card validation with v2 instructions: `audit_trail`, `MIS-001`-style `id`, Title Case `card_type`, and `links[].target` by card id.
- Updated documentation and instructions to match the v2 model invariants.
- Updated `.gitignore` to include Rust build artifacts and configuration files.
- Rewrote [.github/instructions/Markdown.instructions.md](.github/instructions/Markdown.instructions.md) for clarity and consistency with `.markdownlint.json`.
- Updated [.github/instructions/Rust.instructions.md](.github/instructions/Rust.instructions.md) to require Rust 2024 and document the tab-based formatting override.
- Updated [.prettierrc.json](.prettierrc.json) to add an override for `.github/agents/*.agent.md`.
- Renamed the Motivation Traceability view to Requirements View and wired the CLI renderer/tests to the new name.
- Updated the view renderer to group Mermaid `class` assignments per type (single `class element1,element2 class_name` line per `classDef`).
- View renderer now labels each node with the card `id` (wrapped as `"``ID``"`) and eliminates double blank lines in generated Markdown sections.
- Applied the canon shape palette per card type (mission circles, driver rounded boxes, custom document/comment glyphs, etc.) when rendering Mermaid views.
- `render-all` now renders views before cards and the generated README always lists the applicable view links, preventing the views section from being blank.

### Fixed

- Corrected a case-sensitive link typo in Architect agent documentation.
- Replaced outdated `PROGRESS.md` references with `AGENT_PROGRESS.md` in handoff guidance and docs.
- Resolved Mermaid bracket mismatches for deployment, node instance, and process/actor nodes so rendered views use the intended shapes.

### Removed

- Removed [docs/viewer.html](docs/viewer.html) (standalone HTML viewer) due to CORS issues and incompatibility with the current docs structure.

## 1.0.0 - 2026-01-08

### Added

- Initial public release
- All foundational files: repository scaffolding, documentation, and schema

## Releases

See [GitHub Releases](https://github.com/JEleniel/aurora/releases).
