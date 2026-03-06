# Agent Memory

- This file is a local redundant memory ledger for agent tasks in this repository.
- Keep concise, durable facts that aid future work.
- `aurora_cli upgrade` now resolves model home from input, syncs required files from embedded `.github/aurora/{schemas,reference}` assets, and prunes outdated files in `schemas/` and `reference/` before loading models.
- Task 1 complete in `aurora_shared`: `Card::create` now issues registry-based IDs, `Card::write` infers and validates `$schema` from the target path, and `Model::write` stages transactional temp-file writes with rollback on failure.
- Task 2 complete in `aurora_shared`: `Aurora::try_load_for_update` acquires non-blocking per-mission `AuditLog.ndjson` locks via `fs2`, normalizes same-process contention with an in-process registry, and maps lock contention to `AuroraError::ModelLocked` while `Aurora::try_load` remains read-only.
- Task 3 complete in `aurora_shared`: `EditHistory` stores up to 50 reversible snapshot-based `EditCommand`s, `undo`/`redo` reapply model snapshots through `Model::write()`, and failed validation rolls back both in-memory state and on-disk persistence.
- Task 5 complete in the workspace: `tokio` is now a workspace dependency and `aurora_shared` exports a shared background runtime with `spawn_background`, `spawn_blocking_background`, and `block_on_background` helpers for later backup/index work.
- Task 4 complete in `aurora_shared`: `BackupManager` creates timestamped ZIP archives under `aurora/backups/`, excludes the backup directory from archive contents, and supports both editor (`MIS-...`) and MCP (`MCP-MIS-...`) naming via the shared background runtime.
