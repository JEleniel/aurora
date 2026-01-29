# MIS-003 Executive Summary

This executive summary captures the mission intent plus 2 drivers and 7 requirements.

## Mission

- **ID:** `MIS-003`
- **Name:** CLI Tooling
- **Description:** Provide CLI tooling capable of managing, rendering, compressing, and updating Aurora model(s).

## Drivers

- `DRI-002` — Human Friendly Artifacts from Source of Truth. Avoid manual diagram maintenance by generating rendered documentation and agent-friendly exports directly from the authoritative Aurora card graph.
- `DRI-003` — Automated Guardrails for Model Integrity. Provide deterministic, repeatable validation and canonical registries so Aurora models remain structurally sound (reachable, internally consistent, and relationship-matrix compliant) before rendering or publishing.

## Requirements

- `REQ-002` — Discover and Load Models. The tooling SHALL discover one or more Aurora model homes from an input path (file, model-home directory, or ancestor directory) and load all card files under each model home. Discovery is schema-based (presence of Aurora.schema.*), supports a direct child 'aurora/' folder, and bounds directory walking (max depth) for predictable performance.
- `REQ-003` — Validate Models and Emit Diagnostics. The tooling SHALL validate loaded models for core invariants (unique ids, at least one Mission, valid link targets, reachability from the mission root) and SHALL emit structured diagnostics with severity (error|warning|info). Validation SHALL fail the command when errors are present and SHOULD still surface warnings (e.g., relationship matrix compliance).
- `REQ-004` — Render Card Markdown. The tooling SHALL render per-card Markdown documentation and a mission executive summary into a user-specified output directory. Rendered files SHOULD mirror the source card layout (card-type subfolders) and MUST be produced only after the model validates without errors.
- `REQ-005` — Render View Diagrams. The tooling SHALL render view diagrams (DOT sources plus SVG output) for the canonical view set. Rendering MUST use a Graphviz-compatible DOT renderer (default 'dot') and SHOULD allow overriding the executable via environment configuration (AURORA_DOT_COMMAND).
- `REQ-006` — Write Compact Agent Export. The tooling SHALL generate a single-file compact export suitable for agent consumption (AGENT-<MISSION_ID>.jsjson by default) by removing non-essential fields (e.g., $schema, audit trail metadata) while preserving the card graph structure.
- `REQ-007` — Bump Model Audit Versions. The CLI SHOULD support bumping model audit trail versions (patch, minor, major) with deterministic edits to affected cards and an appended audit history entry. The current aurora_cli surface exposes bump commands but does not yet implement them.
- `REQ-008` — Embed Canonical Registries. The tooling SHALL embed canonical registries for card types, relationship verbs, and view definitions so validation and rendering can operate deterministically without depending on external instruction files at runtime.
