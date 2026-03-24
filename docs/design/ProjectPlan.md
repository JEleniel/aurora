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

## Remaining work

### Phase 4 — Review gates

18. [x] P0: Reconfirm remaining editor and MCP scope
    - Description: Re-read the synced editor, MCP, and shared-library docs against the current codebase to lock the remaining backlog before implementation.
    - Deliverables:
        - Remaining gaps list aligned with the current as-built state.
        - Backlog order updated to reflect the current implementation baseline.
    - References: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
    - Depends on: none

19. [x] P0: Reconfirm shared architecture boundaries
    - Description: Revalidate the ownership split between `aurora_shared`, `aurora_editor`, and `aurora_mcp` before feature work begins.
    - Deliverables:
        - Boundary notes for model writes, tool calls, and archive operations.
        - Any required ADR updates identified.
    - References: `docs/design/ViewRenderingArchitecture.md`
    - Depends on: 18

### Phase 5 — Editor mutation surface

20. [x] P0: Add selected-card field editing
    - Description: Add in-place editing for the selected card's fields in the right-sidebar inspector.
    - Deliverables:
        - Scalar card fields are editable from the inspector.
        - Invalid edits are rejected before write.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 1, 19

21. [ ] P0: Add card-link editing actions
    - Description: Add create and delete controls for card links in the editor.
    - Deliverables:
        - Link create and delete actions are available from the editor UI.
        - Link writes are validation-gated and logged through the model layer.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 20

22. [ ] P0: Wire editor save and autosave
    - Description: Wire explicit save and autosave behavior to the shared write path for validated editor changes.
    - Deliverables:
        - Manual save persists the current validated editor state.
        - Autosave persists validated changes without restart.
        - Save feedback is surfaced in the editor shell.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 20

23. [ ] P1: Add editor undo and redo actions
    - Description: Expose the shared undo/redo stack through editor actions and shortcuts.
    - Deliverables:
        - Undo and redo are available in the editor UI.
        - Control enabled-state follows the shared history stack.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 22

24. [ ] P1: Add graph keyboard navigation
    - Description: Finish the focused graph keyboard path so the editor can move focus, center a node, and navigate back and forward from the graph view.
    - Deliverables:
        - Focus movement works from the graph canvas.
        - Center and history navigation are available from the keyboard.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 23

### Phase 6 — Model configuration and packaging

25. [ ] P0: Add card-type registry write support
    - Description: Add validation-gated write support for card-type registry entries.
    - Deliverables:
        - Card-type registry changes can be written safely.
        - Accepted and rejected card-type edits are covered by tests.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 19

26. [ ] P0: Add view-definition write support
    - Description: Add validation-gated write support for view definitions.
    - Deliverables:
        - View-definition changes can be written safely.
        - Accepted and rejected view-definition edits are covered by tests.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 19

27. [ ] P0: Add appearance-default write support
    - Description: Add validation-gated write support for appearance defaults.
    - Deliverables:
        - Appearance-default changes can be written safely.
        - Accepted and rejected appearance edits are covered by tests.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 19

28. [ ] P1: Add model-home pack and unpack workflow
    - Description: Add the shared ZIP transport flow for packing and unpacking a model home as a single archive.
    - Deliverables:
        - Pack operation writes a single ZIP archive for a model home.
        - Unpack operation restores a model home from an archive.
    - References: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
    - Depends on: 19

29. [ ] P0: Define model tool request and response types
    - Description: Add the request and response types that editor agents and MCP calls use to read or modify models.
    - Deliverables:
        - Shared tool request/response types are available in `aurora_shared`.
        - Batch-edit and validation-error paths are represented explicitly.
    - References: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
    - Depends on: 25, 26, 27

30. [ ] P1: Add the card-type registry editor
    - Description: Add the editor UI for creating, editing, and removing card-type registry entries.
    - Deliverables:
        - Card-type add/edit/remove controls are available.
        - Duplicate-name and duplicate-acronym conflicts are blocked at input time.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 25

31. [ ] P1: Add the appearance customization editor
    - Description: Add the editor UI for card appearance defaults and live preview rendering.
    - Deliverables:
        - Shape, fill, stroke, text, and icon controls are available.
        - The preview updates from the shared render pipeline.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 27

32. [ ] P1: Add the view-definition editor
    - Description: Add the editor UI for creating, editing, and removing view definitions.
    - Deliverables:
        - View add/edit/remove controls are available.
        - Root selection is constrained by the root-safety rule.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 26

### Phase 7 — MCP server and agent integration

33. [ ] P0: Create the `aurora_mcp` crate scaffold
    - Description: Add the MCP server crate to the workspace and wire its stdio runtime and logging setup.
    - Deliverables:
        - `tools/aurora_mcp` is registered as a workspace member.
        - The server starts on stdio with file-based logging.
    - References: `docs/design/AuroraMCP.md`
    - Depends on: 27

34. [ ] P0: Add MCP read-only tools
    - Description: Add the MCP query handlers for card lookup, adjacency, root selection, and validation introspection.
    - Deliverables:
        - Read-only tool calls return structured JSON responses.
        - Queries do not require full model materialization.
    - References: `docs/design/AuroraMCP.md`
    - Depends on: 33, 29

35. [ ] P0: Add MCP write and batch-edit tools
    - Description: Add the MCP mutation handlers for card edits, link edits, and batch edits.
    - Deliverables:
        - Mutation calls are validation-gated and logged.
        - Batch edits fail atomically on the first validation error.
    - References: `docs/design/AuroraMCP.md`
    - Depends on: 33, 29

36. [ ] P1: Add the editor agent sidebar
    - Description: Add the opt-in agent chat sidebar with context controls and a visible tool-activity feed.
    - Deliverables:
        - Agent chat UI is available in the editor shell.
        - Tool use and context scope are shown to the user.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 29, 35

37. [ ] P1: Add agent proposed-change review
    - Description: Add the diff-like approval flow that shows agent-proposed model changes before they are written.
    - Deliverables:
        - Proposed changes are reviewable before writes.
        - User acceptance or rejection is explicit.
    - References: `docs/design/AuroraEditor.md`
    - Depends on: 36

### Phase 8 — Verification and documentation

38. [ ] P0: Code review — remaining implementation
    - Description: Review the completed editor, shared-library, MCP, and agent-integration work against the code-review checklist.
    - Deliverables:
        - Open code-review findings are recorded and addressed.
        - Final implementation remains within the agreed boundaries and validation gates.
    - References: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
    - Depends on: 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37

39. [ ] P1: Documentation review — editor and MCP guides
    - Description: Review the user and operator documentation for the editor and MCP server after the implementation review closes.
    - Deliverables:
        - Remaining documentation gaps are identified and closed.
        - User-facing and operator-facing guides match the implemented behavior.
    - References: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
    - Depends on: 38
