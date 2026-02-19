# Project Brief

Aurora (Agent-Unified Representation of Requirements and Architecture) is a machine-readable architecture modeling system. Models are expressed as a directed graph of JSON “cards” rooted at a single `Mission` card and validated against canonical schemas/registries. Rust tooling can validate models and render view artifacts (for example SVG diagrams).

## What’s in this repo

- Rust workspace under `tools/`
	+ `tools/aurora_cli/`: CLI for validating, compacting, and rendering models.
	+ `tools/aurora_shared/`: shared library (model, registry, renderers) used by the CLI and other tools.
	+ `tools/svg_prep/`: SVG/template preparation utilities.
	+ `tools/aurora_editor/`: editor UI (evolving).

- Canonical specs (source of truth)
	+ Schemas: `.github/agents/aurora/schemas/`
	+ Registry + template: `.github/agents/aurora/reference/`
	+ Convenience mirror: `Aurora_Specs/`

- Architecture models and design artifacts
	+ Model home: `docs/design/aurora/` (example missions: `MIS-001`, `MIS-002`).
	+ Checked-in rendered examples: `docs/design/MIS-002/Views/`.

## Key reference files

- Model configuration registry: `.github/agents/aurora/reference/Aurora.modelconfiguration.json`
- SVG template (repo assets): `assets/masters/SVGTemplate.svg` (source) and `assets/templates/SVGTemplate.svgz` (generated)
- SVG template (model homes at runtime): `<MODEL_HOME>/reference/SVGTemplate.svgz` (preferred) or `<MODEL_HOME>/reference/SVGTemplate.svg` (fallback)
- CLI contract: `docs/Aurora_CLI_Contract.md`

Last updated: 2026-02-19
