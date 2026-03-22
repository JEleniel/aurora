# Aurora Editor — Project Plan

> This plan covers implementation of the Aurora Editor (`aurora_editor`) and Aurora MCP Server (`aurora_mcp`), as specified in `docs/design/AuroraEditor.md` and `docs/design/AuroraMCP.md`. Work in `aurora_shared` and `aurora_cli` required to support those products is included.
>
> Completed items are grouped first for quick scanning. Remaining items are ordered by recommended execution sequence rather than original phase order. Task numbers remain stable so existing references and dependencies do not drift.

---

## Completed work

### Phase 0 — Foundation

1. [x] Extend `aurora_shared` model write API to be validation-gated and issue card IDs
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-001
    - Description: All writes to card and model files must be gated by schema and invariant validation. The current `Card::write()` and `Model::write()` write unconditionally. Replace them with a transactional write surface that validates, rejects invalid states, and returns structured errors instead of panicking. Card IDs are issued by the application — callers never supply an ID for a new card. This prevents collisions, enforces well-formed IDs, and removes the burden from users and agents alike.
    - Deliverables:
        - `Card::create()` generates and assigns a new card ID; no external ID input is accepted for creation.
        - `Card::write()` returns a `Result` and rejects schema or invariant violations.
        - `Model::write()` validates the entire model before persisting any file.
        - All existing tests pass; new tests cover write rejection paths and ID generation uniqueness.
    - Notes: Existing CLI callers must be updated to handle the new `Result` type.
    - Status: Completed

2. [x] Add OS-level exclusive lock on the audit log
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-009
    - Description: Implement single-writer enforcement via an OS-level exclusive file lock on `aurora/<MISSION_ID>/AuditLog.ndjson`. The audit log is used as the lock _target_ rather than locking every card file individually because it is a single, always-present file in every model home — a reliable sentinel. Holding it session-scoped (not per-write) makes it a global mutex for the whole model home: a second process that cannot acquire the lock is refused access entirely, which closes the TOCTOU race on card files without per-file locking. The lock must be held for the lifetime of the write session and released when the owning process exits or closes the model. All writers are cooperative (our own code), so advisory locking is sufficient.

        **Platform-specific semantics:**
        - **Linux / macOS**: Use `flock(2)` with `LOCK_EX | LOCK_NB`. The lock is associated with the open file description, not the process; it is released automatically by the OS when the file descriptor is closed or the process exits (including `SIGKILL`). Forking after acquiring the lock requires explicit care — the child inherits the file descriptor and therefore the lock; the child must close it immediately if it does not intend to hold it.
        - **Windows**: Use `LockFileEx` with `LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY`. On Windows the lock is per-process, not per-file-descriptor; a second open by the _same_ process would succeed on Linux but fail on Windows. The implementation must not attempt to re-acquire the lock from the same process.

        A non-blocking attempt must be used on all platforms so that the caller gets an immediate `AuroraError::ModelLocked` rather than blocking indefinitely.

        **Scope**: The lock is acquired only by the editor and MCP server. The CLI is read-only and never writes to the model, so it neither competes for the lock nor requires it.

    - Deliverables:
        - `AuditLog` acquires an exclusive, non-blocking OS lock on open and releases it on drop.
        - A second editor or MCP server opening the same model home fails with `AuroraError::ModelLocked` on all three target platforms.
        - The lock is confirmed to be released on process exit without explicit cleanup.
        - Tests verify single-writer enforcement across two handles in the same process.
        - Platform-conditional tests document the differing same-process re-acquisition behavior.
    - Notes: `fd-lock` or `fs2` are candidate crates; evaluate for Windows correctness before use. Do not rely on a separate `.lock` sentinel file — the audit log itself is the target.
    - Status: Completed

3. [x] Add undo/redo stack to `aurora_shared`
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-001
    - Description: Implement an in-memory command-history stack with 50-entry depth. Each mutating operation produces a reversible command object. The stack must support `undo()` and `redo()` operations and must integrate with the validation-gated write API so that undone and redone states are also validated before persisting.
    - Deliverables:
        - `EditHistory` type with `push(command)`, `undo()`, `redo()`, `can_undo()`, `can_redo()` methods.
        - Stack is bounded to 50 entries; oldest entry is dropped when full.
        - Undo applies the logical inverse of each command and re-validates before persisting.
        - Tests cover push, overflow, undo, redo, and validation-failure rollback.
    - Status: Completed

4. [x] Add model backup system to `aurora_shared`
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-009
    - Description: When the editor or MCP server opens a model in write mode, create a single timestamped ZIP archive of the model home under `aurora/backups/` before any writes occur. Archive creation must be non-blocking. Named `<MISSION_ID>-<timestamp>.zip` for the editor and `MCP-<MISSION_ID>-<timestamp>.zip` for the MCP server. The CLI does not trigger backup creation.
    - Deliverables:
        - `BackupManager` creates one timestamped ZIP of the model directory on editor/MCP open.
        - Backup creation is async and does not block model readiness.
        - CLI open path does not invoke `BackupManager`.
        - Tests verify archive creation and naming for both editor and MCP prefixes.
    - Notes: Requires async runtime (Task 5).
    - Dependencies: Task 5
    - Status: Completed

5. [x] Add async runtime to the workspace
    - Priority: 0 (Critical)
    - Cards: SYS-001
    - Description: The editor's background I/O (fs monitoring, backup, autosave) requires an async runtime. Add `tokio` to the workspace dependencies and integrate it as the executor for `aurora_shared` background operations.
    - Deliverables:
        - `tokio` added to `[workspace.dependencies]` in the root `Cargo.toml`.
        - `aurora_shared` background operations use `tokio::spawn` or `block_in_place` where appropriate.
        - Compilation succeeds; no regressions in existing tests.
    - Status: Completed

6. [x] Add configuration safety backup to `aurora_shared`
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-009
    - Description: Before the first write to any model configuration or view configuration file, create a single ZIP of the `reference/` and `schemas/` directories and store it in `aurora/backups/` (e.g., `MIS-001-config-backup.zip`). This backup is created once — if it already exists it must not be overwritten, as the goal is to preserve the originals exactly as shipped. Creation is synchronous; failure must block the configuration write entirely.
    - Deliverables:
        - `ConfigBackupManager` checks for an existing backup before each configuration write.
        - If no backup exists, creates the ZIP synchronously before any file is modified.
        - If the backup already exists, skips creation silently.
        - Failure to create the backup returns an error and aborts the write with no partial changes.
        - Tests verify: backup created on first write, skipped on subsequent writes, write blocked on backup failure.
    - Status: Completed

7. [x] Implement SVGTemplate defs extraction in `aurora_shared`
    - Priority: 1 (High)
    - Cards: SYS-001, FEA-006
    - Description: Implement a read-only utility that opens `reference/SVGTemplate.svgz` from a model home and extracts the available icon and shape `<defs>` entries. The result is a registry of named asset IDs used by the appearance customization UI to present available options without requiring the full Aurora library. This is distinct from the svg_prep build pipeline — it inspects the template rather than processing it.
    - Deliverables:
        - Function or struct that decompresses `SVGTemplate.svgz` and returns available icon IDs and shape IDs from `<defs>`.
        - Result is consumed by the appearance customization UI (Task 18).
        - Tests verify extraction against a known template fixture with a predictable set of defs.
    - Notes: May share decompression infrastructure with Task 27 (svg_prep integration); evaluate before duplicating.
    - Status: Completed

### Phase 1 — Search Index

8. [x] Implement in-memory search index in `aurora_shared`
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-003
    - Description: Implement a searchable in-memory index over the model home using `tantivy`. The index is rebuilt on startup from the card files on disk and kept live via filesystem monitoring (inotify on Linux, FSEvents on macOS, ReadDirectoryChangesW on Windows). Filesystem monitoring replaces any explicit update or remove API: changes on disk are detected automatically and the index is updated without application intervention. No persistence is required — rebuilding from disk is fast enough to beat even MCP server initialization time. Coverage: card type, subtype, ID, name, outbound link adjacency, and attribute property names.
    - Deliverables:
        - `ModelIndex` struct with `open(model_home)` and `search(query)` methods.
        - Index is built in-memory on `open`; no index files written to disk.
        - Filesystem watcher detects card file create, modify, and delete events and updates the index automatically.
        - `search()` returns ranked `CardRef` results with ID, type, subtype, and name.
        - Tests cover initial build, watcher-triggered update, card deletion, and search ranking.
    - Notes: Uses `tantivy` for search and a platform-appropriate fs-watch crate (e.g., `notify`).
    - Dependencies: Task 5
    - Status: Completed

### Phase 2 — `aurora_editor` crate scaffold

9. [x] Create `aurora_editor` crate and workspace member
    - Priority: 0 (Critical)
    - Cards: SYS-001, FEA-008
    - Description: Create the `tools/aurora_editor/` crate, register it in the workspace, and establish the binary entry point with CLI argument parsing (model home path, logging level). Wire up structured logging (file-based, same conventions as CLI).
    - Deliverables:
        - `tools/aurora_editor/Cargo.toml` with appropriate dependencies (UI framework, logging, async runtime).
        - `main.rs` parses arguments and initializes logging and the async runtime.
        - `cargo build -p aurora_editor` succeeds with no warnings.
    - Notes: The UI framework approved in the ADR is Dioxus (desktop target).
    - Dependencies: Task 5
    - Status: Completed

10. [x] Implement model-home loading with bounded working set
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-001, CAP-003
    - Description: The editor must not fully materialize all cards on startup. Implement a session layer that builds the index (Task 8) immediately and defers full card deserialization to on-demand access. Interactive readiness must not wait for full model load. Target: interactive within 2 seconds on a typical model home (2000 cards, 3500 links).
    - Deliverables:
        - Editor reaches interactive state before all cards are deserialized.
        - Card detail is fetched on demand via index-backed lazy access.
        - Startup load time ≤ 2 s measured against a synthetic 2000-card / 3500-link fixture.
        - Exclusive lock is acquired on startup; second instance is refused with clear UX.
    - Dependencies: Task 2, Task 8, Task 9
    - Status: Completed

11. [x] Implement configuration and settings management
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-007
    - Description: Implement user-facing configuration: autosave toggle, theme preference (system / light / dark), base font size, editor identity (name used for audit log attribution), and agent provider endpoints and API keys. Configuration is stored in the OS-standard user config directory. Secrets (API keys) must use the OS keychain where available; log files must be scrubbed of secrets.
    - Deliverables:
        - `EditorConfig` struct with all required fields; persisted as versioned JSON.
        - Theme preference supports system-detected, forced light, and forced dark.
        - API keys stored in OS keychain; fallback to environment variable with a warning.
        - Settings UI panel with fields for all user-configurable options.
        - Secrets do not appear in log output.
    - Notes: Applies to both `aurora_editor` and `aurora_mcp`.
    - Dependencies: Task 9
    - Status: Completed

12. [x] Implement first-run configuration wizard
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-007
    - Description: On first launch (no config file present), display a setup wizard before opening any model. The wizard collects the minimum required preferences so the editor is correctly configured from the start. After the wizard completes, the config file is written and normal startup proceeds.
    - Deliverables:
        - Wizard is shown when no `EditorConfig` file exists.
        - Wizard collects: autosave preference, theme preference, editor identity (name/email for audit log attribution).
        - Completed wizard writes an initial `EditorConfig` to the OS-standard user config directory.
        - Subsequent launches skip the wizard.
        - Tests cover wizard completion and config file creation.
    - Dependencies: Task 11
    - Status: Completed

### Phase 3 — Completed editor UI slice

17. [x] Implement bottom panel — audit log and diagnostics
    - Priority: 2 (Medium)
    - Cards: SYS-001, FEA-011, FEA-008
    - Description: Bottom panel hosts two tabs: an audit log viewer (reverse-chronological entries for the selected card) and a diagnostics/validation messages pane (warnings and errors for the current model state). Entries in the audit log must link back to the relevant card in the graph view.
    - Deliverables:
        - Audit log tab shows entries from `AuditLog` filtered to the selected card.
        - Diagnostics tab shows current `validation_errors` and `validation_warnings`.
        - Clicking an audit entry or diagnostic navigates to the relevant card.
    - Dependencies: Task 13, Task 14
    - Status: Completed

---

## Remaining work — recommended execution order

### Close out already-finished foundation with formal review

30. [ ] Code review — Phases 0 and 1 (`aurora_shared` foundation)
    - Priority: 0 (Critical)
    - Description: Apply the Code Review Checklist to all changes in Phases 0 and 1: validation-gated writes, ID issuance, exclusive lock, undo/redo, model backup, async runtime, configuration safety backup, SVGTemplate defs extraction, and search index.
    - Deliverables:
        - All Secure Code checklist items addressed.
        - No source file exceeds 500 lines; no function exceeds 50 lines.
        - All tests are meaningful and cover edge cases and failure paths.
        - Audit of trust boundaries for file I/O and lock acquisition.
    - Notes: Recommended immediately; every prerequisite is already complete and later work should build on reviewed foundations rather than assumptions.
    - Dependencies: Task 1, Task 2, Task 3, Task 4, Task 5, Task 6, Task 7, Task 8
    - Status: Not Started

### Finish the editor core interaction loop

13. [ ] Implement the 4-region UI shell
    - Priority: 0 (Critical)
    - Cards: SYS-001, FEA-008
    - Description: Complete the outer UI shell already scaffolded in the desktop app: left sidebar (20% width), right sidebar (20%), bottom panel (20% height), and top-center main area (remainder). Preserve bounded resize behavior and live theme switching, and finish the remaining cross-platform polish and accessibility verification.
    - Deliverables:
        - 4-region layout renders correctly on Linux, Windows, and macOS.
        - Regions are resizable; constraints prevent collapse below minimum usable size.
        - All three theme modes (system, light, dark) are applied correctly and switch without restart.
        - Keyboard navigation reaches all interactive controls.
        - Color contrast meets WCAG AA for all foreground/background pairings in all three theme modes.
    - Notes: Shell composition, live theme updates, bounded resize rails, and keyboard resize affordances are already in place; remaining work is verification, polish, and any cross-platform fixes discovered during that sweep.
    - Dependencies: Task 9
    - Status: In Progress

14. [ ] Implement the centered graph view (main area)
    - Priority: 0 (Critical)
    - Cards: SYS-001, FEA-008, CAP-005
    - Description: Finish the central graph view on top of the existing focused-graph renderer and hotspot plumbing. The selected card is centered; adjacent cards radiate outward (TheBrain-style). The view renders live SVG output produced by `aurora_shared`'s rendering pipeline. Pan and zoom must be supported. Clicking a node navigates (centers) to that card.
    - Deliverables:
        - Graph view renders a focused subgraph centered on the selected card.
        - Pan and zoom operate smoothly with mouse and keyboard.
        - Clicking a node re-centers the view on that card.
        - View re-renders reactively when card data changes.
        - SVG rendering delegates to `aurora_shared::render`; no duplicate layout logic.
    - Notes: Focused graph rendering, hotspot extraction, and keyboard zoom scaffolding already exist; remaining work is navigation completion, reactive refresh behavior, and cross-platform interaction polish.
    - Dependencies: Task 10, Task 13
    - Status: In Progress

15. [ ] Implement left sidebar — card browser, search, and breadcrumb
    - Priority: 1 (High)
    - Cards: SYS-001, FEA-008, CAP-003
    - Description: Finish the left sidebar using the existing navigation/search scaffold. Search queries the `ModelIndex` (Task 8). Results show card type, subtype, ID, and name. Selecting a result navigates the graph view to that card. A breadcrumb at the bottom of the sidebar shows the navigation path from the root card to the currently selected card, and each crumb is clickable.
    - Deliverables:
        - Search input issues queries against `ModelIndex` and displays ranked results.
        - Results are filterable by card type.
        - Selecting a result updates the graph view.
        - Search is non-blocking; UI does not freeze during query.
        - Breadcrumb at the bottom of the sidebar reflects the current navigation path.
        - Clicking a breadcrumb crumb navigates to that card.
    - Notes: Mission-root navigation, live search, type filtering, and breadcrumb scaffolding already exist; remaining work is completion of navigation behaviors, UX polish, and performance verification under realistic model sizes.
    - Dependencies: Task 8, Task 13, Task 14
    - Status: In Progress

16. [ ] Implement right sidebar — card detail and editor
    - Priority: 1 (High)
    - Cards: SYS-001, FEA-008, CAP-001
    - Description: Extend the current read-only inspector into a full inline editor. All edits must pass through the validation-gated write API; invalid states are indicated inline and save is blocked. The attribute editor does not need to be a full JSON builder: provide dedicated form controls for the scalar types (boolean, integer, number, text) and let the user enter a text value that may itself be a valid JSON object — it is accepted as-is without further parsing.
    - Deliverables:
        - All standard card fields are editable via inline form controls.
        - Attribute editor provides boolean, integer, number, and text inputs; text fields accept plain strings or raw JSON objects.
        - Saving triggers the validation-gated write API; invalid states are rejected with inline error display.
        - Successful saves update the index (via fs watcher), the graph view, and the audit log.
        - Create and delete actions for cards and links are accessible from this panel; new card IDs are assigned by the application.
    - Notes: The read-only inspector, link navigation, attributes, diagnostics, and registry warnings already exist; editing, mutation flows, and save plumbing are the remaining scope.
    - Dependencies: Task 1, Task 3, Task 13, Task 14
    - Status: In Progress

17. [ ] Implement autosave and manual save
    - Priority: 0 (Critical)
    - Cards: SYS-001, CAP-001
    - Description: Implement Apple-style immediate autosave: changes are persisted to disk as soon as they pass validation, without requiring an explicit save action. A manual save action must also be available. Autosave must be configurable (disable for users who prefer explicit save). Each save produces a new audit log entry.
    - Deliverables:
        - Passing-validation changes are written to disk immediately in autosave mode.
        - Autosave can be disabled; when disabled, a "modified" indicator is shown.
        - Each save produces an audit log entry with the configured editor identity.
        - The fs watcher picks up the saved file and updates the index automatically.
    - Notes: Land the smallest end-to-end validated save path first, then layer the autosave toggle and dirty-state UX on top of it.
    - Dependencies: Task 1, Task 8, Task 16
    - Status: Not Started

18. [ ] Implement undo/redo in the editor
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-001
    - Description: Wire the `EditHistory` stack (Task 3) to the editor UI. Undo and redo must be accessible via standard keyboard shortcuts and menu items.
    - Deliverables:
        - Undo and redo are accessible via ⌘Z / ⌃Z and ⌘⇧Z / ⌃Y.
        - Stack depth (up to 50) is reflected in the UI (grayed when unavailable).
        - Undo of a save creates a new audit log entry recording the reversion.
    - Notes: Scope the first pass to card editing, then extend the same command-history affordances to configuration edits once Task 18 exists.
    - Dependencies: Task 3, Task 19
    - Status: Not Started

### Complete model-authoring surfaces before agent and MCP work

26. [ ] Add `version` field to `ModelConfiguration` schema and struct
    - Priority: 1 (High)
    - Cards: SYS-001
    - Description: `AuroraEditor.md` explicitly requires a `version` property on `ModelConfiguration`. Add it to the JSON Schema, the `ModelConfiguration` struct, and the reference configuration files. Update the CLI upgrade path to handle configs that omit the field.
    - Deliverables:
        - `Aurora.modelconfiguration.schema.json` includes a `version` field.
        - `ModelConfiguration` struct has a `version: Option<String>` field.
        - Reference configuration files include a `version` value.
        - CLI upgrade logic handles absence of `version` without error.
        - All existing tests pass.
    - Notes: Do this before further configuration UI and tool-surface work so every remaining config path targets the same stable schema.
    - Status: Not Started

27. [ ] Implement model configuration customization UI
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-001
    - Description: Provide a dedicated editor panel for customizing the Aurora canon stored in `reference/Aurora.modelconfiguration.json` and `reference/Aurora.viewconfiguration.json`. All writes go through the configuration safety backup (Task 6) before any file is modified. Card type names and acronyms are validated for uniqueness at input time; duplicates are rejected, not warned. Acronyms are auto-generated from the card type name and may be overridden subject to the same rejection. Relationship duplicates (same source + target + label) are rejected. In-use card type deletion is blocked with a clear error. The appearance editor shows a live preview using the shared render pipeline; icon and shape options are sourced from the SVGTemplate defs (Task 7). View definition management constrains root card type selection to currently valid roots; subsequent invalidation of a root is surfaced as a standard validation error.
    - Deliverables:
        - Card type list with add, edit, and remove actions.
        - Acronym auto-generated on type name entry; override field validates uniqueness on change, rejects duplicates at input time.
        - Type name field validates uniqueness on change; duplicates are rejected at input time.
        - Relationship editor rejects duplicate source + target + label triples at input time.
        - In-use card type deletion is blocked with a clear validation error.
        - Appearance editor: shape, fill, stroke, text colour, and icon controls with live preview card rendering via the shared pipeline.
        - Icon and shape selectors populated from SVGTemplate defs (Task 7).
        - View definition editor: add/edit/remove views; root card type selector constrained to currently valid roots.
        - All writes go through `ConfigBackupManager` (Task 6) before any file is modified.
    - Notes: Prefer after Task 26 so the customization UI writes the final versioned configuration shape from day one.
    - Dependencies: Task 6, Task 7, Task 13, Task 26
    - Status: Not Started

28. [ ] Implement view rendering panel
    - Priority: 2 (Medium)
    - Cards: SYS-001, FEA-008, CAP-005
    - Description: Provide an on-demand view rendering panel that can render any configured view or an ad-hoc view for the open model using `aurora_shared::render`. The root safety rule must be enforced before rendering. Ad-hoc views must support all three layout families (vertical tree, horizontal tree, radial subtree), with the user selecting the desired family. Output is displayed as a scrollable, zoomable SVG.
    - Deliverables:
        - View selector lists all views from the `ViewRegistry` plus an ad-hoc option.
        - Ad-hoc view allows selection of root card(s), included card types, and layout family.
        - Selecting a view triggers a render via `aurora_shared::render` and displays the output SVG inline.
        - Root safety rule violations are reported as a validation error; render is blocked.
        - Renders are triggered asynchronously; UI remains responsive.
    - Notes: Sequencing this after Task 18 reduces churn because configured views and view constraints will already have a stable editing surface.
    - Dependencies: Task 10, Task 13, Task 18
    - Status: Not Started

29. [ ] Integrate `svg_prep` functionality into `aurora_shared`
    - Priority: 2 (Medium)
    - Cards: FEA-006
    - Description: Per the future roadmap, `svg_prep` functionality should be available inside `aurora_shared` so the editor and CLI can use it directly without spawning a subprocess. Move the core SVG preparation logic into `aurora_shared` as a library module. The standalone `svg_prep` binary can remain as a thin wrapper.
    - Deliverables:
        - Core SVG preparation logic lives in `aurora_shared::render::svg_prep` (or equivalent module).
        - `aurora_cli` and `aurora_editor` call the shared module directly.
        - `svg_prep` binary is a thin wrapper over the shared module.
        - All existing `svg_prep` tests pass.
    - Notes: Reuse the SVGTemplate loading and defs-extraction work from Task 7 where practical; no second bespoke SVG pipeline, please and thank you.
    - Dependencies: Task 7
    - Status: Not Started

### Build the shared model tool surface, then layer MCP and chat on top

22. [ ] Define and implement the internal model tool surface
    - Priority: 1 (High)
    - Cards: SYS-001, CAP-008
    - Description: Define the tool interface used by agentic access. To minimize the number of tools presented to agents, operations are organized into command/subcommand sets. Groupings: `card` (subcommands: `upsert`, `delete`, `get`), `link` (subcommands: `create`, `delete`), `query` (subcommands: `find`, `adjacency`), `config` (subcommands: `card-type upsert`, `card-type delete`, `card-type list`, `view upsert`, `view delete`, `view list`). Card create and update are merged into a single `upsert`: no ID creates a new card with a system-assigned ID returned in the response; an existing ID updates the card. Config writes go through the configuration safety backup (Task 6). All mutating operations go through the validation-gated write API and produce audit log or configuration change entries. This surface is shared by the editor agent integration and the MCP server.
    - Deliverables:
        - Tools are organized into command/subcommand sets; total tool count is minimized.
        - `card upsert` without an ID creates a new card with a system-assigned ID and returns it.
        - `card upsert` with an existing ID updates the card.
        - `config card-type upsert` enforces acronym and name uniqueness and relationship duplicate rejection.
        - `config view upsert` constrains root card types to currently valid roots.
        - Read subcommands do not require confirmation.
        - Mutating subcommands require confirmation by default (configurable opt-out).
        - Tests cover every subcommand for success, validation-failure, and ID-issuance paths.
    - Notes: Sequence this after Tasks 18 and 26 if possible so configuration semantics are settled once and encoded once.
    - Dependencies: Task 1, Task 6, Task 8, Task 26
    - Status: Not Started

23. [ ] Create `aurora_mcp` crate and MCP server scaffold
    - Priority: 1 (High)
    - Cards: SYS-001
    - Description: Create `tools/aurora_mcp/` as a workspace member. The binary reads from stdin and writes to stdout using the MCP protocol (`rmcp` library). File-based logging only (no stdout). On startup: acquire the audit log exclusive lock (Task 2) and create a model backup (Task 4). The same model tool surface (Task 22) is exposed as MCP tools.
    - Deliverables:
        - `cargo build -p aurora_mcp` succeeds.
        - MCP handshake completes over stdio.
        - All model tool surface operations (Task 22) are exposed as MCP tools.
        - Exclusive lock prevents concurrent editor and MCP server access to the same model home.
        - Backup is created on startup.
        - Log output goes to file; stdout is reserved for MCP protocol only.
    - Dependencies: Task 2, Task 4, Task 22
    - Status: Not Started

24. [ ] Implement MCP secrets and configuration, including CLI key provisioning
    - Priority: 1 (High)
    - Cards: SYS-001
    - Description: MCP server configuration mirrors the editor (Task 11) but without the UI. API keys must be stored in the OS keychain. Because the MCP server is typically started by an agent (non-interactively) it cannot prompt for credentials; the MCP binary must therefore expose a separate CLI subcommand (e.g., `aurora_mcp keychain set <provider> <key>`) so a human operator can provision keys into the keychain before handing control to the agent. Log output is scrubbed of all secret values.
    - Deliverables:
        - Configuration file is read from the OS-standard user config directory.
        - `aurora_mcp keychain set <provider> <key>` stores the key in the OS keychain; intended for interactive human use only.
        - At runtime the server reads keys from the keychain; it does not accept keys via config files or environment variables.
        - Log scrubbing is verified by test: injected secrets must not appear in output.
    - Dependencies: Task 11, Task 24
    - Status: Not Started

25. [ ] Implement agentic chat sidebar in the editor
    - Priority: 2 (Medium)
    - Cards: SYS-001, CAP-008
    - Description: Implement the opt-in agent integration sidebar. Chat input is forwarded to the configured AI provider. The agent has access only to the model tool surface (Task 22); it has no shell access. Mutating tool calls require user confirmation by default. Secrets (API keys) must not appear in logs or UI.
    - Deliverables:
        - Agent sidebar is hidden by default; opt-in toggle in settings.
        - Chat input/output displayed in a scrollable conversation view.
        - Agent tool calls are routed through the model tool surface only.
        - Mutating calls display a confirmation dialog before execution.
        - Supported providers: Ollama (required), OpenAI (required), GitHub Models (required).
        - Provider endpoint and key configuration delegates to Task 11.
    - Notes: Sequence after the MCP/server-side tool surface proves out so the editor sidebar can reuse the same hardened contract instead of inventing its own.
    - Dependencies: Task 11, Task 22, Task 25
    - Status: Not Started

### Close with verification, docs, and release gates

28. [ ] Implement testing strategy and integration test suite
    - Priority: 0 (Critical)
    - Description: Establish testing coverage across all components. Unit tests reside alongside source code (`*_tests.rs`). Integration tests cover full editor workflows and MCP protocol conformance. A performance benchmark validates the startup time requirement. Add tests alongside each remaining implementation task, then close the phase with cross-component suites and performance validation.
    - Deliverables:
        - Each `aurora_shared` module has a corresponding `*_tests.rs` covering success, validation-failure, and edge-case paths.
        - Integration test suite exercises: card create/update/delete, undo/redo, search, view render, autosave, configuration customization, and config backup behavior.
        - MCP server integration test: tool call/response cycle over a simulated stdio transport for every tool subcommand including `config` subcommands.
        - Performance benchmark: startup load time ≤ 2 s against a synthetic 2000-card / 3500-link fixture (Task 10 requirement).
        - CI configuration runs all tests on Linux, Windows, and macOS.
    - Dependencies: Task 18, Task 19, Task 20, Task 21, Task 22, Task 23, Task 24, Task 25, Task 26, Task 27
    - Status: Not Started

29. [x] Write user and operator documentation
    - Priority: 1 (High)
    - Description: Produce the documentation required to install, configure, and operate the editor and MCP server.
    - Deliverables:
        - User guide for the editor: installation, first-run wizard, navigation, card editing, model configuration customization, view rendering, agent sidebar.
        - MCP server setup guide: installation, keychain provisioning via CLI (`aurora_mcp keychain set`), model home configuration, integration with agent frameworks.
        - Architecture decision records updated to reflect final implementation choices (UI framework, search library, locking strategy, tool surface design).
    - Dependencies: Task 12, Task 18, Task 21, Task 23, Task 24, Task 25
    - Status: Not Started

30. [ ] Code review — Phases 2–5 (editor core, configuration customization, and agent integration)
    - Priority: 0 (Critical)
    - Description: Apply the Code Review Checklist to the `aurora_editor` crate (Tasks 9–23).
    - Deliverables:
        - WCAG AA accessibility verified for all interactive elements in all three theme modes.
        - No duplicate layout or rendering logic (all delegates to `aurora_shared`).
        - Secrets (API keys, keychain access) do not appear in logs or error messages.
        - All mutating paths validated; no unconditional writes.
        - Configuration customization UI enforces all uniqueness and duplicate rejection rules at input time.
        - Agent tool surface has no shell access path.
        - All mutating agent calls are confirmation-gated by default.
        - File and function size limits met.
    - Dependencies: Task 9, Task 10, Task 11, Task 12, Task 13, Task 14, Task 15, Task 16, Task 17, Task 18, Task 19, Task 20, Task 21, Task 22, Task 23
    - Status: Not Started

31. [ ] Code review — Phase 6 (MCP server)
    - Priority: 1 (High)
    - Description: Apply the Code Review Checklist to the `aurora_mcp` crate (Tasks 24–25).
    - Deliverables:
        - MCP server log scrubbing verified for secrets.
        - Trust boundaries between MCP protocol and model tool surface are explicit.
        - Keychain CLI subcommand is guarded against non-interactive invocation.
        - Exclusive lock behavior verified against both editor and MCP server.
        - `config` subcommands enforce the same validation rules as the editor UI.
    - Dependencies: Task 24, Task 25
    - Status: Not Started

32. [ ] Pre-release review — `aurora_editor` v1.0
    - Priority: 0 (Critical)
    - Description: Apply the Pre-Release Checklist before tagging the first editor release.
    - Deliverables:
        - All required tests pass on Linux, Windows, and macOS.
        - No unresolved P0 or P1 findings from code reviews.
        - Versioning in `Cargo.toml` is correct and consistent with release scope.
        - `Cargo.lock` committed and up to date.
        - Release notes and changelog entries are accurate and complete.
        - Distribution artifacts are reproducible from declared inputs.
        - No secrets present in release artifacts or configuration templates.
    - Dependencies: Task 28, Task 29, Task 30, Task 31, Task 32
    - Status: Not Started
