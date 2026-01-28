# Aurora Editor (APP-002) Design Sketch

This document is the human-readable design sketch for the Aurora Editor (APP-002). The authoritative, machine-readable design is the Aurora model under `docs/design/aurora/`.

## Purpose

The Aurora Editor is a desktop application that provides an interactive experience for creating, editing, validating, and navigating Aurora models. The Editor is optimized for discoverability and safe incremental iteration with immediate validation feedback.

## Architecture (high level)

The Editor is a Tauri (v2) desktop app with:

- A UI layer (Svelte) responsible for interaction, navigation, and presentation.
- A Rust backend responsible for file I/O, model parsing/validation, and rendering/export operations.

The key contract between the UI and backend is the `INT-003 Editor Backend API` interface:

- Source card: `docs/design/aurora/MIS-001/Interface/INT-003-Editor_Backend_API.jsjson`

## Core user flows

- Connect a workspace and discover model homes.
- Select a model home and load a model snapshot (cards + links).
- Validate the model and present actionable diagnostics.
- Browse/search the model and focus a card.
- Edit a focused card safely (create/update/delete with rollback on failure).
- Preview rendered Markdown for the focused card (and optionally related neighborhood).
- Render views and/or export the model in portable formats.

## Model representation

The Editor operates on Aurora cards and links.

- A model has a single root `Mission` card.
- Every non-`Mission`, non-`Note` card must have at least one incoming link.
- All links point away from the `Mission` (local cycles allowed only for bounded flows such as state machines).

The Editor should primarily operate on the source-of-truth JSJSON cards under `docs/design/aurora/**`.

## UI composition (design intent)

The Editor application (APP-002) comprises UI components for primary panes:

- Explorer Tree View (`COM-004`): browse and search the model.
- Mind-Map Like View (`COM-005`): graph-based navigation over the model snapshot.
- Edit View (`COM-006`): safe editing of cards and links.
- Markdown View (`COM-007`): in-app preview of human-readable Markdown renderings.

The UI should keep a single “focused card” concept shared across panes.

## Backend responsibilities

### Safe file operations

The backend is responsible for enforcing workspace-root confinement for all file I/O and for implementing atomic writes.

### Validation and diagnostics

The backend performs schema and invariant validation and produces diagnostics that the UI can navigate:

- Include card id when applicable.
- Include relative path when applicable.
- Preserve severity and actionable messages.

### Preview-first rendering

To support in-app preview and avoid a poor “write to disk then read” UX, the backend should provide preview-first APIs that return:

- Rendered Markdown (as strings) suitable for UI display.
- View render artifacts (at minimum SVG string payloads, potentially also DOT sources) for UI embedding.

This is a prerequisite for the Markdown View and “sanitized rendering” requirements.

## Trust gating and security

The Editor must enforce a trust boundary between untrusted workspaces and privileged operations.

- When the workspace is untrusted, the UI should disable/hide mutation and rendering/export operations.
- The backend must also enforce trust gating (UI gating is not sufficient).

For rendering in a webview, the project must use a sanitizer-first pipeline and a locked-down CSP in non-dev builds.

## Derived artifacts

The CLI and Editor backend can generate derived artifacts (human-readable Markdown and rendered views). These outputs are conveniences and should not replace the source model as the system of record.

- Rendered output folder example: `docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/`

## Open questions

- Which previews must be supported first: per-card Markdown, per-view SVG, or both?
- How should “clickable” navigation inside Markdown previews be represented (e.g., `CARD_ID` link conventions)?
- What is the minimal CSP that supports the required UI features while keeping the preview surfaces safe?
