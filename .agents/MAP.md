# CODEMAP

Quick index of “where things live” for agents working in this repository.

## Architecture model (Aurora)

- Source-of-truth model home (cards): `docs/design/aurora/`
    + Note: this directory is populated (for example MIS-001 source cards + audit log).
    + Intended structure (per Aurora invariants):
        - Root mission cards live directly under `docs/design/aurora/`.
        - Mission-scoped cards live under `docs/design/aurora/<MISSION_ID>/` in per-card-type folders.
        - Folder names match `card_type` exactly (Title Case; spaces preserved, for example `Data Store/`).
        - Audit log lives at `docs/design/aurora/<MISSION_ID>/AuditLog.json`.
        - Compact export lives at `docs/design/aurora/<MISSION_ID>/Compact.json`.
- Schemas:
    + Canonical schemas and registries live under `.github/agents/aurora/`.
    + Convenience mirror exists under `schemas/` (kept in sync with `.github/agents/aurora/`).
    + Card schema: `schemas/Aurora.card.schema.json`
    + Compact schema: `schemas/Aurora.compact.schema.json`
    + Audit schema: `schemas/Aurora.audit.schema.json`
    + Registries: `schemas/Aurora.canonical.definitions.json`, `schemas/View.Definitions.json`

## Finding things fast

- Find a card by id (for example `REQ-014`):
    + Source model: `docs/design/aurora/<MISSION_ID>/**/<CARD_ID>-*.json`
- Find cards by type:
    + Source model: `docs/design/aurora/<MISSION_ID>/<Card Type>/`

- Find a view (when generated):
    + SVG: `docs/design/<Rendered_Model_Name>/Views/<View>_View.view.svg`
    + DOT: `docs/design/<Rendered_Model_Name>/Views/source/<View>_View.view.dot`
    + Tip: inspect the `.view.dot` first; it’s the exact Graphviz input.

## Canonical registries

These define the vocabulary and view rules; prefer updating these over duplicating lists elsewhere:

- Card types + allowed relationships: `.github/agents/aurora/Aurora.canonical.definitions.json`
- View definitions: `.github/agents/aurora/View.Definitions.json`

## Rendering pipeline (Rust)

- Shared library (rendering + model logic): `tools/aurora_shared/`
    + View rendering entry points and Graphviz DOT/SVG generation live under `tools/aurora_shared/src/`.
    + Hierarchical layout for view graphs lives in `tools/aurora_shared/src/render/layout.rs` (`layout_model`).
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
    + Note: `aurora_cli` is currently not usable in this workspace/environment (see `.agents/TECHNOLOGIES.md`).

## UI / editor

- Desktop editor: `tools/aurora_editor/`
    + Status: present in this workspace; treat feature/UX specifics as evolving.

## Documentation

- Human-facing design docs: `docs/design/`
    + Canonical card types + relationships (reference): `docs/design/Aurora.canonical.definitions.md`

## Workspace root

- Rust workspace root: `Cargo.toml`
    + CLI crate: `tools/aurora_cli/Cargo.toml`
    + Shared library: `tools/aurora_shared/Cargo.toml`
- Changelog (Keep a Changelog): `CHANGELOG.md`

## Generated outputs

- Render outputs commonly land under `docs/design/<Rendered_Model_Name>/` (when generated).
- Graphviz artifacts (when generated):
    + SVG: `docs/design/<Rendered_Model_Name>/Views/<View>_View.view.svg`
    + DOT source: `docs/design/<Rendered_Model_Name>/Views/source/<View>_View.view.dot`
