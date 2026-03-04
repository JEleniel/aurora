# Standalone, Cross-Platform Editor Specifications

## Purpose

This document specifies requirements and UX expectations for a standalone Aurora editor. It is intended as a prompt for the Architect to model and refine into an official design.

## Terminology

- A "model home" is a directory that contains one or more complete Aurora models plus the schemas and reference registries required to validate and render it. Storage format and locations are defined in [Aurora/](../../Aurora/).
- "Validate" refers to the same validation performed by Aurora tooling through the `aurora_shared` library. If an edit would cause the model to fail validation, the edit must be blocked.
- A "typical model home" is defined as approximately 2000 cards and 3500 links per model.
- "Index persistence" means storing the editor's index on disk so that subsequent opens can reuse it (subject to validation) and become interactive faster.

## Behavior Summary

- The editor MUST remain responsive on large models by keeping memory bounded (small working set + searchable index).
- All writes MUST be validation-gated.
- The editor MUST prevent concurrent editing of the same model (single active editor instance).
- Agent features (if enabled) MUST be tool-mediated: the agent can only do what the editor's model tool allows.

## Requirements

### Platform

- The editor MUST run on the major desktop platforms: Linux, Microsoft Windows, and Apple macOS. Support will focus on the current versions as of release.

### Scalability & Memory

- The editor MUST be able to work with models of any size (1 card to unbounded) and any complexity (1 link per card to unbounded), within the minimum memory bound.
    - The minimum supported platform has 8GiB of RAM.
    - The editor MAY take advantage of more memory when available to improve performance.
    - Operations MAY take longer on large models, but the editor MUST remain responsive and correct (progress feedback, cancellation, no runaway memory growth).

### Loading & Indexing

- The editor MUST become interactive "almost instantly" (target: <2 seconds) on the minimum target platform for typical model homes.
    - If the time to read model files exceeds this budget, the editor MUST still open quickly and provide visible progress while loading continues.
- The editor MUST NOT load the entire model into memory at once.
    - The editor MUST build and maintain a searchable index to support fast navigation and search while keeping memory bounded.
    - Full indexing SHOULD be available almost instantly for typical model homes.
    - The editor SHOULD persist the index on disk to accelerate subsequent opens.
- Minimum search/index coverage MUST include:
    - Card type
    - Card subtype
    - Card ID
    - Card name
    - Link adjacency (at least: outbound adjacency list)
    - Attribute property names
- Indexing MUST update on save.
    - Index updates are asynchronous.
    - Index updates are expected to be inexpensive and should not block the UI thread.
    - Full-text search is optional (nice to have).

### Model Home Inputs

- The editor MUST load the schemas and reference files included with a model home and use those for all interaction with that model. This allows the editor to work across multiple versions and customizations of Aurora.
    - The editor MUST use `schemas/*` for validation.
    - The editor MUST use `reference/Aurora.modelconfiguration.json` for canonical card registries and view definitions.
    - The editor MUST use `reference/Aurora.viewconfiguration.json` for canonical appearance defaults (including `available_icons`).

### Editing, Validation, and Linting

- The editor MUST prevent edits that would break a model.
    - "Break a model" means an edit that would violate invariants or cause the model to fail validation.
    - This includes schema invalidity and invariant violations.
    - This does not include warning-only checks (e.g., naming and relationship verbs). Those remain warnings.
- Validation is performed when a change would normally be written.
    - The exact write points are to be defined by the Architect as usage flows are modeled.
- All edits MUST be automatically logged.
    - This includes all create/change/delete operations for cards and links, regardless of whether the edit originated from the UI or from an agent.
    - Audit entries MUST be appended by the model tool (not by the agent and not by direct file writes).
    - Each audit entry MUST include attribution.

### Views & Root Selection

- The editor MUST be able to render any view defined in the model configuration file and ad hoc views.
- View rendering (layout selection, depth assignment, packing, and orthogonal edge routing) MUST follow the requirements in `docs/design/ViewLayouts.md`.
    - Architectural placement and ownership boundaries for rendering are defined in `docs/design/ViewRenderingArchitecture.md`.
- View roots MUST follow the documented "root safety" rule:
    - A view root is a card that has descendants.
    - No descendant may link back to the root.
- If a view has no viable roots, the editor MUST present a clear explanation and a way to choose a different view or adjust view definitions.

### Backups, Pack, and Unpack

- On load, the editor MUST begin creating a timestamped backup ZIP in `aurora/backups/` (e.g., `MIS-001-20260210T061800Z.zip`) of the entire model home, stored per conventions defined in [Aurora/](../../Aurora/).
    - Backup creation MUST be asynchronous.
    - Backup creation MUST NOT block UI interactivity or the first update.
    - If backup creation fails, the editor MUST warn the user but continue loading.
    - A configurable number of ZIPs will be retained, default 5. Older ones will be automatically deleted.
    - Restore procedure: close the editor and restore the model home from the ZIP as a full rollback snapshot.
    - Backups MUST include the entire model home as stored on disk, including the compact model (generated artifacts are outside of the model home).
- The editor MUST be able to pack and unpack the model into a single ZIP-compressed file.

### Concurrency & Safety (Single Writer)

- Multiple instances on the same model are not supported.
- The editor MUST prevent accidental concurrent editing using OS-level locking.
    - The active editor holds an exclusive lock (write handle) to the mission audit log at `aurora/<MISSION_ID>/AuditLog.ndjson`.
    - If the exclusive lock cannot be acquired because it is already held, the model is considered locked.
    - This relies on the OS to release locks on crash, minimizing "stale lock" cleanup.
    - The editor MUST warn the user and refuse to open the model when locked.

### UX

- The primary working screen is divided into four resizable regions:
    - A left sidebar defaulted to 20% width, full height.
    - A bottom-center panel defaulted to 20% height, full width.
    - A right sidebar defaulted to 20% width, full height.
    - A top-center panel filling the remaining space.
- Left sidebar navigation (VS Code-like):
    - Context-aware navigation of the model and tool-specific views.
    - Tree view aligned to the on-disk layout, with extraneous information removed.
    - Search entry point.
    - Pinned and recent items.
- Primary navigation tree:
    - Root is the Mission.
    - Each card type has a folder; each card is shown within its type folder.
    - The display text is `ID: Name`, not the filename (e.g., `MIS-001: A Mission to the Editor`).
- Bottom panel:
    - Details and interaction with the currently selected card.
    - The save affordance and validation errors are visible here (exact interaction flow to be defined by the Architect).
- Top-center view (TheBrain-like centered graph):
    - Selecting a card centers it as the working node.
    - Parents are shown above and descendants below.
    - Siblings are shown to the left and right; siblings are determined lexicographically by Card ID.
    - Clicking a visible node re-centers it.
    - Keyboard navigation is supported (at minimum: move focus, center focused node, back/forward navigation).

### Autosave, Undo/Redo, and Crash Recovery

- Immediate autosave (Apple-style) by default, with an option for manual save mode.
    - Autosave granularity is to be defined by the Architect.
- Undo/redo spans 50 edits deep (current target; to be defined by the Architect pending testing and memory constraints).
- With autosave enabled: on a crash, the model may at worst contain an orphan card that needs to be linked; cards are written in a single operational call.
    - If an orphan is found during load, offer the option to link or delete it to correct the model.
- With autosave disabled: the saved model is always valid; unsaved changes are lost on crash.
- User confirmation required before closing or exiting when unsaved changes exist.
- Since the model is multi-file, atomic writes are not possible. Transactional writes should be simulated; if one in a sequence fails, the whole sequence is rolled back.

### Accessibility & Theming

- WCAG AA compliant accessibility.
- Theme selection:
    - Use the system mode (light/dark) when available.
    - If the system mode cannot be determined, fall back to dark mode.
    - User configuration takes precedence.
- Adjustable base font size with a default of 16px; all other UI elements scale relative to this setting.

### Logging

- Logging is required.
- Support for stdout, stderr, and optional file-based logging.

### Compatibility & Versioning

- Each model home includes a complete set of schema and configuration files snapshotted at model creation time.
- Compatibility is guaranteed across a major version of Aurora.
    - Per semver, all minor and patch changes are backward compatible.
    - Tooling of a given major version MUST be able to load model homes created within that major version.
- The editor MAY offer to upgrade a model home on load, but it MUST be able to work with the model without upgrading.
- Incompatible models are detected via schema validation (behavior defined in Aurora/).
- `reference/Aurora.modelconfiguration.json` MUST include a `version` property so tooling can identify the exact registry version.
    - This requires an update to the matching schema.
    - This may require changes to the `aurora_shared` library.

## Agent Integration

This section captures requirements for integrating AI assistants into the editor. Detailed interaction flows are to be defined by the Architect.

### Agentic chat sidebar (IDE agent mode)

- The editor MUST provide a sidebar panel that supports agentic chat workflows comparable to "agent mode" in IDE assistants.
    - The intent is multi-step assistance that can read context, propose a plan, and apply model edits via well-defined tools.
- The agentic sidebar MUST support at least:
    - Free-form chat (user prompts, assistant responses).
    - A visible, structured "activity" feed of tool actions (what was read, searched, proposed, and changed).
    - Explicit context controls (what the agent is allowed to see): selected card only, current view context, or user-selected scope.
    - A clear, optional "proposed changes" review surface before writes (diff-like preview for cards/links/attributes).
- The editor MUST provide a constrained "model tool" surface that is the only way an agent can read or modify the model.
    - This tool surface MAY be implemented using a tool protocol such as MCP, but the editor is authoritative for capability boundaries.
- The agent MUST NOT be able to execute any shell commands or spawn arbitrary OS processes.
    - All agent actions MUST be mediated through editor-defined tools that preserve invariants and validation gates.
    - The agent MUST NOT write directly to model files (cards, compact model, audit log). It only requests tool operations.

Tool calls and tool results MUST be structured, machine-readable JSON.

- On success, the tool MUST return a JSON object indicating the operation was saved.
- On failure, the tool MUST return a JSON object containing validation and/or invariant errors.

#### Minimum tool capabilities

The editor MUST expose an internal tool API sufficient to support agentic tasks without full model materialization:

- Non-mutating tools:
    - Find cards by id/name/type/subtype.
    - Fetch a card's normalized representation (attributes + links).
    - Fetch adjacency (outbound and inbound) and bounded neighborhood expansions.
    - Query model configuration.
    - Compute viable roots per the root safety rule.
    - Retrieve validation errors/warnings for a candidate edit.
    - Retrieve a list of possible next target card types based on the model configuration.
- Write tools (MUST be transactional and validation-gated):
    - Create/update/delete cards.
        - Update card operations MUST supply a complete replacement card payload. The tool is responsible for writing the card safely.
    - Create/update/delete links.
    - Apply a batch edit as a single operation (all-or-nothing), with a clear failure report.

Write tool calls MUST require that the editor holds the exclusive model lock.

### Providers

- MUST support Ollama endpoints.
- MUST support OpenAI endpoints.
- SHOULD support GitHub Models via REST API, including optional organizational attribution.

### Configuration and Secrets

- Provider configuration MUST be explicit (endpoint URL, model selection, timeouts).
- Secrets (e.g., API keys) MUST NOT be stored in plain text.
    - Prefer OS keychain or equivalent secure storage.
- The editor MUST provide an "offline" mode that prevents network calls.

If GitHub Models is supported:

- The editor MUST support authenticating to the GitHub Models REST API.
    - Tokens MUST be stored securely.
    - The editor MUST support providing credentials appropriate to the deployment (e.g., a personal token for individuals, or organization-managed credentials if required).

### Privacy and Control

- Agent integration MUST be opt-in.
- The editor MUST allow the user to control what content is shared (selected card only, view context, or broader model context).
- Calls to agents MUST be scrubbed of secrets.
- Logs MUST NOT include secrets.

### Editing

- Agents should operate in one of two (user choice) ways:
    - (Default) Agents MAY propose edits, but the editor MUST require explicit user confirmation before applying changes to the model.
    - Agents MAY make edits freely as required to meet user needs. No confirmation is required for normal edits; deletes should still require confirmation.

## Architecture Notes (Prompt for the Architect)

This is not the official design, but it defines minimum boundaries the Architecture model should capture.

- Threading:
    - UI runs on the main thread.
    - The secondary thread handles engine work (I/O, validation, rendering).
    - Indexing has one or more dedicated threads.
- Communication:
    - UI and engine communicate via signals.
    - Long-running work is cancellable and reports progress.
- State:
    - The editor maintains a small in-memory working set.
    - The index enables navigation/search without fully materializing the model.
- Writes:
    - Writes are transactional at the "write points" (to be defined by the Architect).
    - Validation gates writes.

## Minimum Test Plan

- Loading:
    - Open a typical model home (2000 cards, 3500 links) and confirm the UI becomes interactive within 2 seconds on the minimum target platform.
    - Confirm progress is shown when full load exceeds the interactive budget.
- Indexing:
    - Confirm full indexing is available almost instantly for a typical model home.
    - Confirm index persistence accelerates subsequent opens.
    - Confirm search supports type, subtype, id, name, adjacency, and attribute property names.
    - Confirm indexing updates asynchronously on save.
- Validation:
    - Attempt an edit that would violate schema/invariants and confirm it is blocked.
    - Confirm warning-only checks remain warnings.
- Views:
    - Render configured views and confirm root selection obeys the root safety rule.
    - Confirm the editor explains when no viable roots exist.
- Backups:
    - Confirm a timestamped backup ZIP is created on load.
    - Simulate a backup failure and confirm the editor warns but continues.
    - Confirm retention keeps only the most recent N backups.
- Concurrency:
    - Open the same model home twice and confirm the second instance cannot acquire the audit log write handle and refuses to open the model.
- Theming:
    - Confirm system theme is used when available.
    - Confirm fallback to dark when system mode is unavailable.
- Agent integration:
    - Confirm the agentic sidebar can perform a non-mutating task (search and summarize) without loading the full model.
    - Confirm the activity feed shows which tools were used and what scope was accessed.
    - Confirm agent-proposed changes are shown in a diff-like preview prior to writes.
    - Attempt an agent-assisted edit that would violate schema/invariants and confirm it is rejected.
    - Confirm agent write actions require the editor to hold the exclusive model lock.
    - Confirm Ollama and OpenAI endpoints can be configured.
    - Confirm offline mode blocks network calls.
    - Confirm secrets are not logged and are stored securely.

## ADRs

- Rust has been chosen as the implementation language due to:
    - Smallest memory footprint
    - Prevents numerous common pitfalls by default and design
    - Highest performance in preliminary testing
    - Compiles and runs on the most platforms
- Dioxus has been chosen as the UI framework due to:
    - Mature and consistent across the target platforms
    - HTML + CSS for layout and styling
    - No virtual machine
    - 100% Rust codebase
    - Better load times and performance than Tauri (main competitor)
- The following libraries have been reviewed and approved for use:
    - This list is not exhaustive. Additional libraries may be used when justified and reviewed.
    - `dioxus` as the UI framework
    - `anyhow`, `thiserror` for error handling
    - `sha2`, `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
    - `chrono` for time and date handling
    - `clap` for CLI interfaces
    - `config` for configuration file handling
    - `dirs` for standard config/data/cache directories
    - `fern`, `log` for logging
    - `rig` for LLM access
    - `reqwest` for HTTP client calls
    - `serde` (and sublibraries), `serde_json` for serialization
    - `tokio` (and sublibraries) for async runtime
    - `url`, `urlencoding` for URL handling
    - `tantivy` for indexing and search
