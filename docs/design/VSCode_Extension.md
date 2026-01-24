# Aurora VS Code Extension Design (APP-003)

## Definitions

- Files: Model files may be stored in either `*.json` or `*.jsjson` files. A single model MUST only use one or the other across the entire model. Anywhere a `*.json` file is referenced in this document it also refers to the `*.jsjson` equivalent.
- `*.jsjson` files are functionally identical to `*.json` files.
- Model Home: Models MUST be located in a folder named `aurora/`. User input may point to the parent of a Model Home, in which case the real Model Home should be discovered and used.
- Card Schema: `Aurora.schema.json` in the Model Home.
- Compact Model: optional file `AGENT-<MISSION_ID>.jsjson` in the Model Home.
- Compact Model Schema: `Aurora.compact.schema.json` in the Model Home.
- APP-003: The Aurora VS Code extension application. In the current model, this is named **VSCode Extension** (`APP-003`).

## Scope and intent

APP-003 provides an in-editor workflow for working with Aurora models inside VS Code:

- Open and browse an Aurora model from a workspace.
- Validate the model (schema + invariants + canonical vocabulary).
- Edit cards and relationships with safe, trust-aware file writes.
- Preview human-friendly outputs (Markdown + views) and optionally render them to disk.
- Optionally expose Aurora operations via an MCP surface for other in-editor tooling.

This document is a design sketch: it describes intended behaviors, constraints, and recommended implementation structure. It does not assert that specific code exists yet.

## Architecture (conceptual)

APP-003 is structured as a set of collaborating components (as modeled today):

- **Shared Library** (`COM-001`)
	+ Core model operations: load, validate, transform, and write Aurora cards.
	+ Shared across tooling surfaces (CLI, editor, extension).
- **VSCode Extension Host** (`COM-013`)
	+ Activation, configuration, command registration, and workspace integration.
	+ Owns the “source of truth” state for the opened model in the extension runtime.
- **Webview UI Host** (`COM-014`)
	+ Interactive UI for browsing and editing models.
	+ Communicates with the extension host via a message interface.
- **Workspace Adapter** (`COM-015`)
	+ File open / watch / save behaviors.
	+ Enforces trust-aware defaults and confines writes to safe locations.
- **MCP Server** (`COM-008`)
	+ Optional server surface exposing model operations through an MCP interface.
	+ Uses the shared library for implementation.

Key interface surfaces:

- **VSCode Commands Interface** (`INT-005`)
	+ Command palette and context-menu entry points.
- **VSCode Webview Messaging API** (`INT-004`)
	+ Message-based protocol between the extension host and webview UI.
- **MCP Interface** (`INT-002`)
	+ Exposed by `COM-008`.

## Capabilities

### Workspace-level experience

- Discover and open a model home (`aurora/`) in the current workspace.
- Persist user intent (e.g., “last opened model home”) per workspace.
- Watch for file changes and keep the in-memory model representation in sync.
- Provide trust-aware behavior when the workspace is untrusted.

### Commands and entry points

- Command palette actions:
	+ Open model.
	+ Validate model.
	+ Render views / render markdown.
	+ Export compact model.
	+ Package model as ZIP.
	+ Show model health / diagnostics.
- Context-menu actions:
	+ Validate selected model home.
	+ Render outputs for selected model.

### Webview UI

- Browse and focus the model (tree + graph-like navigation).
- Inspect a card (rendered Markdown view + structured JSON/editor form).
- Create/edit/delete cards and links with immediate validation feedback.
- Offer safe “quick fixes” where possible (e.g., rename a non-canonical verb).

### Shared model operations

These are best implemented in `COM-001` and consumed by the extension host:

- Parse and round-trip cards without losing unknown fields.
- Validate against schemas and invariants.
- Validate canonical vocabulary:
	+ Canonical card palette.
	+ Canonical relationship verbs and allowed source/target pairs.
- Deterministic rendering (same input produces stable output):
	+ Human-friendly per-card markdown.
	+ Common views.
	+ Compact model export.
	+ ZIP packaging.

## User flows (happy paths)

### Open a model

- User runs “Aurora: Open Model”.
	+ User selects either:
		- A model home folder (`aurora/`), or
		- A workspace folder that contains (or contains a child) `aurora/`.
	+ Extension discovers the model home (see “Model discovery”).
	+ Extension loads cards into memory, populates indexes, and starts file watchers.
	+ Extension opens the main webview UI for browsing/editing.

### Edit a card safely

- User selects a card in the UI.
- UI requests the card payload from the extension host.
- User edits fields and outgoing links.
- Extension host:
	+ Validates changes (incremental).
	+ Writes changes atomically (temp + rename) via the workspace adapter.
	+ Appends a single audit event for the user action.
	+ Re-validates impacted neighborhoods and returns updated diagnostics.

### Validate and navigate diagnostics

- User runs “Aurora: Validate Model” or validation runs on-save with debounce.
- Extension host returns a list of diagnostics.
	+ Diagnostics include: severity, card ID, field path, message, remediation.
- UI displays diagnostics and provides navigation.
	+ Clicking a diagnostic focuses the relevant card and field.

### Render outputs

- User runs “Aurora: Render Views” or “Aurora: Render Markdown”.
- Extension host:
	+ Validates before rendering.
	+ Writes outputs to a chosen output folder (or a configured default).
	+ Reports progress and summary.

## Model discovery

Discovery rules are consistent with other Aurora tooling:

- If a selected folder:
	+ Contains `Aurora.schema.json` (or `Aurora.schema.jsjson`), it is a model home.
	+ Else if it contains a child `aurora/` folder that contains the schema, that child is the model home.
	+ Else, show an actionable error with remediation.
- If multiple candidate model homes exist beneath a chosen folder:
	+ Prompt the user to choose (present mission cards discovered in each).

## Data model & state (extension runtime)

### In-memory representation

- Parse each card into a strongly typed structure.
	+ Preserve unknown fields in an “extras” map so round-tripping is lossless.
- Maintain derived indexes:
	+ `by_id`: card ID → card.
	+ `outgoing`: card ID → outgoing links.
	+ `incoming`: card ID → incoming links (derived).
	+ `by_type`: card_type → list of card IDs.
	+ `search`: token index over ID/name/description.
- Maintain per-file dirty state and a concurrency strategy.
	+ Only one write-in-flight per file.
	+ Coalesce rapid edits into one save where possible.

### Webview state

- Webview UI is a cache and presentation layer.
	+ The extension host is the authority.
	+ All mutations must be validated server-side (host-side) before persisting.

## Persistence & audit trail

### Writes

- All writes are confined:
	+ Card edits: within the model home.
	+ Render outputs: within an explicit output folder.
- Writes are atomic:
	+ Write temp file.
	+ Rename over the original.
- File formatting:
	+ Prefer stable pretty-print for JSON (or JSJSON).
	+ Preserve file extension choice for the model.

### Audit events

- Every user action that changes the model produces one audit history entry.
- The audit entry includes:
	+ Editor identity.
	+ RFC3339 timestamp.
	+ Event kind.
- Avoid duplicate audit entries when one action touches multiple fields.

Open question:

- How should interactive tools bump `audit_trail.version` (patch/minor/major)? Decide and apply consistently.

## Validation & diagnostics

### Validation modes

- Fast (incremental):
	+ Schema shape (required fields present).
	+ Link targets exist.
	+ “Mission has outgoing-only links” and “all cards reachable from mission”.
	+ Relationship verb is canonical and allowed for the source/target types.
- Full:
	+ Full invariants.
	+ Render staleness checks (if configured).

### Diagnostics UX

- Diagnostics are first-class and actionable.
	+ Provide quick fixes only when they are safe and unambiguous.
	+ Never auto-delete data; prefer “tombstone” behaviors.

## Security and trust boundaries

### Untrusted inputs

Treat all model files as untrusted data:

- Never execute content from model files.
- Ensure previews (Markdown, Mermaid) are rendered safely.
	+ No remote fetch by default.
	+ Sanitize any HTML in previews.

### Workspace trust

- Respect VS Code workspace trust.
	+ In untrusted workspaces, disable operations that write files or execute external tooling.
	+ Allow safe read-only operations (browse + validate) when possible.

### Path safety

- Prevent path traversal in:
	+ Rendering.
	+ ZIP packaging.
	+ Any “write output” operations.

## Performance considerations

- Large models:
	+ Virtualize long lists (tree views, search results).
	+ Limit graph rendering to a bounded neighborhood (1–3 generations) by default.
- Incremental updates:
	+ On file watcher events, re-parse only impacted cards.
	+ Recompute incoming edges for affected IDs only.
- Debounced validation:
	+ Validate on-save and on-edit, but debounce to avoid UI stalls.

## Accessibility (WCAG)

- Full keyboard navigation.
- Screen-reader-friendly labels for:
	+ Cards.
	+ Relationships.
	+ Diagnostics.
- High-contrast support and respect OS font scaling.

## Testing strategy

- Unit tests for the shared library:
	+ Schema validation.
	+ Invariant checks.
	+ Relationship registry enforcement.
	+ Deterministic rendering.
- Integration tests for extension host:
	+ Workspace discovery.
	+ Safe writing rules.
	+ Diagnostics routing.
- Webview UI tests (if applicable):
	+ Message protocol handling.
	+ Accessibility checks.

## Open questions

- Should APP-003 support multiple open model homes in one workspace, or exactly one at a time?
- Do we run the MCP server by default, or only on-demand?
- Where should default render outputs go if the user doesn’t pick an output folder (e.g., `docs/design/`), and should that be opt-in?
- Should “delete card” mean tombstone (`status: Deleted`) or physical deletion (file removal)?
