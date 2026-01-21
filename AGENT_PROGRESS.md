# AGENT_PROGRESS

## Project Brief

AURORA is a deterministic, JSON-based architectural modeling format where Cards (JSON files) are connected by directed Links away from a single root `Mission` card.

- Feature: Maintain authoritative schema and modeling guidance
    + Status: In Progress
    + Primary schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
    + Compact schema: [schemas/Aurora.compact.schema.json](schemas/Aurora.compact.schema.json)
    + Modeling guidance: [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md)

## Active Context Summary

- Branch: `v2.0.0`
- Current work:
	+ Implement `tools/aurora_cli` from the design model in [docs/design/aurora/](docs/design/aurora/).
	+ Expand the Viewer/Editor design in [docs/design/Aurora Viewer and Editor.md](docs/design/Aurora%20Viewer%20and%20Editor.md) into an implementation-ready mission model (MIS-002) under `docs/design/aurora/`.
    + Bootstrap `tools/aurora_editor` (Tauri + SvelteKit) backed by a reusable `tools/aurora_lib` crate shared with the CLI.
- Design model entrypoints:
	+ CLI mission: [docs/design/aurora/MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json](docs/design/aurora/MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json)
	+ Viewer/Editor mission: [docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json](docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json)
- Key product decisions captured in the design:
	+ Default active mission/tab is the lowest `MIS-###`.
	+ Tree view shows only card JSON files (no schemas / non-card files).
	+ Editor is constrained: IDs immutable; `card_type` change creates a new card and marks the old as `Deleted` (with confirmation).
	+ Invalid edits are blocked; link target selection is invariant-safe.
	+ File watcher conflict handling is field-level with deterministic rules (first-writer-wins on conflicts, merge non-conflicting changes).
	+ MCP endpoint supports CRUD plus change events; SVG export uses Mermaid SVG rendering; model content is never executed; filenames/paths are validated.
- Last known build signal:
	+ `cargo build --release` succeeds (local terminal state).

### Current high-priority risks / follow-ups

- Safe path handling: validate all derived path segments (mission id, card type, view names) as single safe segments before using them in filesystem paths.
- Schema compilation: ensure schema files are parsed as JSON values (not as JSON strings) before compilation/validation.
- Card discovery/layout enforcement: ensure discovery checks the actual entry filename (not the root folder name) and reports malformed/unknown JSON files rather than silently skipping.
- View definitions: ensure card type strings match exact Title Case names (no embedded newlines).
- Determinism: avoid `HashMap` iteration for output ordering; sort by card id.
- Template contracts: keep Markdown template placeholders aligned with renderer replacements.

## Patterns

- Deterministic directed-graph model rooted at a single `Mission` card
- Links are descriptive verbs; semantics emerge when rendering views, not from the link type alone

## Technologies

- JSON Schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
- Static docs site under [docs/](docs/) (note: `docs/viewer.html` removed due to CORS issues and incompatibility with the current structure)
- Mermaid diagrams for examples
- Rust crates used by `tools/aurora_cli`: `clap`, `chrono`, `indexmap`, `jsonschema`, `pathdiff`, `percent-encoding`, `semver`, `serde`, `serde_json`, `tempfile` (dev), `thiserror`, `walkdir`, `whoami`
- Desktop editor stack: [Tauri 2.x](https://tauri.app/) backend under `tools/aurora_editor/src-tauri` plus a [SvelteKit](https://kit.svelte.dev/) frontend managed via `pnpm` (`pnpm@10.28.1`, `vite@6.x`).
- SvelteKit preprocessing/build relies on `@sveltejs/vite-plugin-svelte@6.x` (explicit dev dependency) so Vite can own `dev/build/preview` commands under the Tauri pipeline.
- Shared Rust crate `tools/aurora_lib` exposes Aurora card types, IO helpers, and validation that both the CLI and GUI consume.

## Master Project Plan and Progress Tracker

- **Core schemas and instructions** (P1)
    + Status: In Progress
    + Owner: Architect
    + Links: [schemas/Aurora.schema.json](schemas/Aurora.schema.json), [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md)
    + Next Action: Keep schema + instructions + examples aligned; update guidance when invariants evolve.
- **aurora_cli correctness + determinism** (P1)
    + Status: In Progress
    + Owner: BackendDeveloper
    + Links: [tools/aurora_cli/src/](tools/aurora_cli/src/), [docs/design/aurora/MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json](docs/design/aurora/MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json)
    + Next Action: Address the high-priority follow-ups (safe path segments, schema parse, view definition normalization, deterministic ordering) and keep regression tests current.
- **Aurora Viewer/Editor mission model (MIS-002)** (P1)
    + Status: In Progress
    + Owner: Architect
    + Links: [docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json](docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json)
    + Next Action: Complete the model depth needed for implementation agents (components/interfaces/process/state machine/threats) and validate with `aurora_cli`.
- **Aurora Editor scaffolding** (P1)
    + Status: In Progress
    + Owner: BackendDeveloper
    + Links: [tools/aurora_editor/](tools/aurora_editor/), [tools/aurora_lib/](tools/aurora_lib/), [docs/design/Aurora Viewer and Editor.md](docs/design/Aurora%20Viewer%20and%20Editor.md)
    + Next Action: Flesh out the invocation surface (additional commands, file system flows) and connect the UI to real model payloads once MCP contracts are finalized; build tooling now runs via `vite build` and a square-icon AppImage configuration so `pnpm tauri build` succeeds end-to-end.
- **Docs and indexes** (P2)
    + Status: In Progress
    + Owner: TechnicalWriter
    + Links: [docs/README.md](docs/README.md), [docs/design/README.md](docs/design/README.md)
    + Next Action: Keep documentation in sync with current output layout and add links for new missions.
