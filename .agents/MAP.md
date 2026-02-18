# Map

Quick index of “where things live”.

## Canonical specs

- Source of truth
	+ Schemas: `.github/agents/aurora/schemas/`
	+ Reference assets (registry, SVG template): `.github/agents/aurora/reference/`
- Mirror (kept in sync): `Aurora_Specs/`

## Models and artifacts

- Model home (cards + per-mission folders): `docs/design/aurora/`
	+ Example mission roots:
		* `docs/design/aurora/MIS-001-Model_Aurora_With_Aurora.json`
		* `docs/design/aurora/MIS-002-Example_Canonical_Coverage_Model.json`
	+ Mission folders (cards grouped by type): `docs/design/aurora/<MISSION_ID>/`
	+ Audit log (per mission): `docs/design/aurora/<MISSION_ID>/AuditLog.ndjson`
- Checked-in rendered examples: `docs/design/MIS-002/Views/*.svg`

## Rust workspace

- Workspace root: `Cargo.toml`
- CLI: `tools/aurora_cli/`
- Shared library: `tools/aurora_shared/`
	+ Model types: `tools/aurora_shared/src/aurora/`
	+ Rendering: `tools/aurora_shared/src/render/`
	+ Embedded registries/defaults: `tools/aurora_shared/src/registry/`
- SVG prep tool: `tools/svg_prep/`
- Editor: `tools/aurora_editor/`

Last updated: 2026-02-18
