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

## Rework pending

The incomplete plan items have been removed for rework. The completed foundation above is the only retained content in this draft.
