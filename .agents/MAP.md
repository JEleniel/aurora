# CODEMAP

Quick index of “where things live” for agents working in this repository.

## Architecture model (Aurora)

- Source-of-truth model home: `docs/design/aurora/`
    + Root mission cards live directly under `docs/design/aurora/` (for example `MIS-001-Provide_Default_Tooling_for_AURORA.json`).
    + Mission-scoped cards live under `docs/design/aurora/<MISSION_ID>/` in per-card-type folders.
    + Folder names match `card_type` exactly (Title Case, spaces included, for example `Data Store/`).
    + Compact, agent-friendly snapshot(s): `docs/design/aurora/AGENT-<MISSION_ID>.json` (for example `AGENT-MIS-001.json`).
- Current mission (this repo):
    + Source mission card: `docs/design/aurora/MIS-001-Provide_Default_Tooling_for_AURORA.json`
    + Compact export: `docs/design/aurora/AGENT-MIS-001.json`
    + Rendered docs + views: `docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/`
- Schemas:
    + Canonical schemas and registries for tooling live under `.github/agents/aurora/`.
    + `schemas/` is a symlink to `.github/agents/aurora/`.
    + Card schema: `.github/agents/aurora/Aurora.card.schema.json`
    + Compact schema: `.github/agents/aurora/Aurora.compact.schema.json`
    + Audit schema: `.github/agents/aurora/Aurora.audit.schema.json`
    + Registry schemas: `.github/agents/aurora/Aurora.canonical.definitions.schema.json`, `.github/agents/aurora/View.Definitions.schema.json`

## Finding things fast

- Find a card by id (for example `REQ-014`):
    + Source model: `docs/design/aurora/<MISSION_ID>/**/<CARD_ID>-*.json`
    + Rendered Markdown (if generated): `docs/design/<Rendered_Model_Name>/**/<CARD_ID>-*.md`
- Find cards by type:
    + Source model: `docs/design/aurora/<MISSION_ID>/<Card Type>/`
    + Rendered Markdown: `docs/design/<Rendered_Model_Name>/<Card Type>/`

- Find a view:
    + SVG: `docs/design/<Rendered_Model_Name>/Views/<View>_View.view.svg`
    + DOT: `docs/design/<Rendered_Model_Name>/Views/source/<View>_View.view.dot`
    + Tip: if a view looks odd, inspect its `.view.dot` first; it’s the exact Graphviz input.

## Canonical registries

These define the vocabulary and view rules; prefer updating these over duplicating lists elsewhere:

- Card types + allowed relationships: `.github/agents/aurora/Aurora.canonical.definitions.json`
- View definitions: `.github/agents/aurora/View.Definitions.json`

## Rendering pipeline (Rust)

- Shared library (rendering + model logic): `tools/aurora_shared/`
    + View rendering entry points and Graphviz DOT/SVG generation live under `tools/aurora_shared/src/`.
    + DOT JSON → SVG rendering helpers live in `tools/aurora_shared/src/render/svg.rs`.
    + Graphviz HTML node labels (ID + type line) are generated in `tools/aurora_shared/src/render.rs` (`node_label`).
    + Embedded registries (card definitions, relationship rules, view definitions, styling guide) live in `tools/aurora_shared/src/registry.rs` and are compiled into the CLI.
    + DOT styling conventions are documented in:
        - `.github/instructions/details/Graphviz_View_Styling_Guide.md` (canonical)
        - `docs/design/Graphviz_View_Styling_Guide.md` (human-facing copy)
- CLI (validate/render/compact/bump): `tools/aurora_cli/`
    + Binary outputs typically appear at:
        - `target/debug/aurora_cli`
        - `target/release/aurora_cli`
    + CLI reminder: global options come before the subcommand (see `docs/design/Model_Quickstart.md`).
    + Common commands (examples):
        - `aurora_cli -i docs/design/aurora validate`
        - `aurora_cli -i docs/design/aurora render-all -o docs/design/`
        - `aurora_cli -i docs/design/aurora compact -o docs/design/aurora/AGENT-<MISSION_ID>.json`

## UI / editor

- Dioxus desktop editor (Rust): `tools/aurora_editor/`
    + Entry point: `tools/aurora_editor/src/main.rs`

## Documentation

- Human-facing design docs: `docs/design/`
    + Quickstart: `docs/design/Model_Quickstart.md`
    + Request templates: `docs/design/How_to_Talk_to_the_Model.md`
    + Canonical card types + relationships (Mermaid): `docs/design/Canonical.md`

## Workspace root

- Rust workspace root: `Cargo.toml`
    + CLI crate: `tools/aurora_cli/Cargo.toml`
    + Shared library: `tools/aurora_shared/Cargo.toml`
    + Editor crate: `tools/aurora_editor/Cargo.toml`
- Changelog (Keep a Changelog): `CHANGELOG.md`

## Generated outputs

- Render outputs commonly land under `docs/design/<Rendered_Model_Name>/`.
    + Example in this repo: `docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/`
- Graphviz artifacts:
    + SVG: `docs/design/<Rendered_Model_Name>/Views/<View>_View.view.svg`
    + DOT source: `docs/design/<Rendered_Model_Name>/Views/source/<View>_View.view.dot`
