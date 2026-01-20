# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Added [rustfmt.toml](rustfmt.toml) to enforce hard tabs and consistent Rust formatting.
- Added a `compact` CLI subcommand that exports the model as a single JSON file (sans `audit_trail` metadata) for agent-friendly consumption.
- Added a `full` CLI subcommand that validates, renders all docs, and writes the compact model in a single run.
- Added [schemas/Aurora.compact.schema.json](schemas/Aurora.compact.schema.json) to validate compact model exports.

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
- Mermaid view rendering now generates boundary subgraphs plus node, edge, and class mappings during model rendering ([tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs)).
- `render-all` now renders views before cards and the generated README always lists the applicable view links, preventing the views section from being blank.
- The `compact` command now retains each card's `$schema` reference while stripping `audit_trail` blocks, ensuring the export passes schema validation and still preserves link relationship metadata.
- Compact exports now set the top-level `$schema` pointer to the mission's local `Aurora.compact.schema.json` via a relative path so agents can validate models without hardcoded absolute locations.
- Replaced the placeholder CLI implementations for `render-*`, `compact`, and `bump-*` with concrete logic that streams Markdown summaries, produces compact JSON exports, and updates audit history entries when versions change.
- Mission loader now enforces `MIS-###-Sanitized_Name` filenames and reports schema violations with precise file:line detail ([tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs)).
- Schema loader now detects the aurora root by looking for local schema files, tolerates trailing slashes/parent paths, and builds validators from a single buffered parse ([tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs)).
- Card discovery enforces the `<Mission>/<CardType>/<ID>.json` layout, validates filename/directory/prefix consistency, and surfaces per-field line+column errors ([tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs)).
- Compact exports stream via `CompactModelBorrowed`/`CompactCardBorrowed`, avoiding redundant cloning while keeping `Model` immutable ([tools/aurora_cli/src/aurora/model/compact_model.rs](tools/aurora_cli/src/aurora/model/compact_model.rs), [tools/aurora_cli/src/aurora/model/compact_card.rs](tools/aurora_cli/src/aurora/model/compact_card.rs)).

### Fixed

- Corrected a case-sensitive link typo in Architect agent documentation.
- Replaced outdated `PROGRESS.md` references with `AGENT_PROGRESS.md` in handoff guidance and docs.
- Resolved Mermaid bracket mismatches for deployment, node instance, and process/actor nodes so rendered views use the intended shapes.
- Fixed the nested `aurora::model` exports so the CLI can resolve `CompactModel` and the bump argument structs ([tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs), [tools/aurora_cli/src/aurora/model/model_args.rs](tools/aurora_cli/src/aurora/model/model_args.rs), [tools/aurora_cli/src/aurora/model/compact_model.rs](tools/aurora_cli/src/aurora/model/compact_model.rs), [tools/aurora_cli/src/cli.rs](tools/aurora_cli/src/cli.rs)).
- Fixed `aurora_cli` model detection when invoked from subdirectories by resolving the default `--input` relative to the nearest git root and expanding Aurora home discovery to search upwards and include `docs/design/aurora/` ([tools/aurora_cli/src/lib.rs](tools/aurora_cli/src/lib.rs), [tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs)).

### Removed

- Removed [docs/viewer.html](docs/viewer.html) (standalone HTML viewer) due to CORS issues and incompatibility with the current docs structure.

## 1.0.0 - 2026-01-08

### Added

- Initial public release
- All foundational files: repository scaffolding, documentation, and schema

## Releases

See [GitHub Releases](https://github.com/JEleniel/aurora/releases).
