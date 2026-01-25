# CODEMAP

This file is for agents working in this repository. It’s a quick index of “where things live” so you can navigate fast.

## Architecture model (Aurora)

- Source-of-truth model home: `docs/design/aurora/`
  - Root mission cards live directly under `docs/design/aurora/`.
  - Mission-scoped cards live under `docs/design/aurora/<MISSION_ID>/` in per-card-type folders.
- Schemas:
  - `docs/design/aurora/Aurora.schema.jsjson`
  - `docs/design/aurora/Aurora.compact.schema.jsjson`
  - Duplicates are also under `schemas/`.

## Canonical registries (instructions)

These define the vocabulary and view rules; prefer updating these over duplicating lists elsewhere:

- Card palette: `.github/instructions/Card_Definitions.md`
- View registry: `.github/instructions/View_Definitions.md`
- Relationship verbs: `.github/instructions/Relationship_Definitions.md`

## Rendering pipeline (Rust)

- Shared library (rendering + model logic): `tools/aurora_shared/`
  - View rendering entry points and Graphviz DOT/SVG generation live under `tools/aurora_shared/src/`.
  - DOT styling conventions are documented in:
    - `.github/instructions/Graphviz_View_Styling_Guide.md` (canonical)
- CLI (validate/render/compact/bump): `tools/aurora_cli/`

## UI / editor

- Svelte + Vite frontend: `tools/aurora_editor/`
- Tauri backend: `tools/aurora_editor/src-tauri/`

## Documentation

- Human-facing design docs: `docs/design/`
  - Quickstart: `docs/design/Model_Quickstart.md`
  - Request templates: `docs/design/How_to_Talk_to_the_Model.md`

## Workspace root

- Cargo workspace root: `Cargo.toml`
- Changelog (Keep a Changelog): `CHANGELOG.md`

## Generated outputs

- Render outputs commonly land under `docs/design/<Rendered_Model_Name>/`.
- Graphviz artifacts:
  - SVG: `docs/design/<Rendered_Model_Name>/Views/*.view.svg`
  - DOT source: `docs/design/<Rendered_Model_Name>/Views/source/*.view.dot`
