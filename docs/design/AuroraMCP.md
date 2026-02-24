# Aurora MCP Server Requirements

This document specifies requirements and expectations for Aurora MCP server. It is intended as a prompt for the Architect to model and refine into an official design.

## Terminology

- A "model home" is a directory that contains one or more complete Aurora models plus the schemas and reference registries required to validate and render it. Storage format and locations are defined in [Aurora/](../../Aurora/).
- "Validate" refers to the same validation performed by Aurora tooling through the `aurora_shared` library. If an edit would cause the model to fail validation, the edit must be blocked.
- A "typical model home" is defined as approximately 2000 cards and 3500 links per model.
- "Index persistence" means storing the server's index on disk so that subsequent opens can reuse it (subject to validation) and become interactive faster.

## Behavior Summary

- The server MUST remain responsive on large models by keeping memory bounded (small working set + searchable index).
- All writes MUST be validation-gated.
- The server MUST prevent concurrent editing of the same model (single active server/server instance).

## Requirements

### Platform

- The server MUST run on the major desktop platforms: Linux, Microsoft Windows, and Apple macOS. Support will focus on the current versions as of release.

### Scalability & Memory

- The server MUST be able to work with models of any size (1 card to unbounded) and any complexity (1 link per card to unbounded), within the minimum memory bound.
    - The minimum supported platform has 8GiB of RAM.
    - The server MAY take advantage of more memory when available to improve performance.
    - Operations MAY take longer on large models, but the server MUST remain responsive and correct (progress feedback, cancellation, no runaway memory growth).

### Loading & Indexing

- The server MUST become interactive "almost instantly" (target: <2 seconds) on the minimum target platform for typical model homes.
    - If the time to read model files exceeds this budget, the server MUST still open quickly and provide progress feedback while loading continues.
- The server MUST NOT load the entire model into memory at once.
    - The server MUST build and maintain a searchable index to support fast navigation and search while keeping memory bounded.
    - Full indexing SHOULD be available almost instantly for typical model homes.
    - The server SHOULD persist the index on disk to accelerate subsequent opens.
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

- The server MUST load the schemas and reference files included with a model home and use those for all interaction with that model. This allows the server to work across multiple versions and customizations of Aurora.
    - The server MUST use `schemas/*` for validation.
    - The server MUST use `reference/Aurora.modelconfiguration.json` for canonical registries, appearance, and view definitions.

### Views and rendering

- If the MCP surface exposes view rendering, the server MUST use the shared renderer behavior defined in `docs/design/ViewLayouts.md`.
    - Architectural placement and ownership boundaries for rendering are defined in `docs/design/ViewRenderingArchitecture.md`.

### Editing, Validation, and Linting

- The server MUST prevent edits that would break a model.
    - "Break a model" means an edit that would violate invariants or cause the model to fail validation.
    - This includes schema invalidity and invariant violations.
    - This does not include warning-only checks (e.g., naming and relationship verbs). Those remain warnings.
- Validation is performed when a change would normally be written.
    - The exact write points are to be defined by the Architect as usage flows are modeled.
- All edits MUST be automatically logged.
    - This includes all create/change/delete operations for cards and links, regardless of whether the edit originated from the UI or from an agent.
    - Audit entries MUST be appended by the model tool (not by the agent and not by direct file writes).
    - Each audit entry MUST include attribution.

### Backups, Pack, and Unpack

- On load, the server MUST begin creating a timestamped backup ZIP in `aurora/backups/` (e.g., `MCP-MIS-001-20260210T061800Z.zip`) of the entire model home, stored per conventions defined in [Aurora/](../../Aurora/).
    - Backup creation MUST be asynchronous.
    - Backup creation MUST NOT block interactivity or the first update.
    - If backup creation fails, the server MUST notify the agent but continue loading.
    - A configurable number of ZIPs will be retained, default 5. Older ones will be automatically deleted.
    - Restore procedure: close the server and restore the model home from the ZIP as a full rollback snapshot.
    - Backups MUST include the entire model home as stored on disk, including the compact model (generated artifacts are outside of the model home).
- The server MUST be able to pack and unpack the model into a single ZIP-compressed file.

### Concurrency & Safety (Single Writer)

- Multiple instances on the same model are not supported.
- The server MUST prevent accidental concurrent editing using OS-level locking. The editor uses the same locking to prevent dcross access.
    - The active server holds an exclusive lock (write handle) to the mission audit log at `aurora/<MISSION_ID>/AuditLog.ndjson`.
    - If the exclusive lock cannot be acquired because it is already held, the model is considered locked.
    - This relies on the OS to release locks on crash, minimizing "stale lock" cleanup.
    - The server MUST warn the user and refuse to open the model when locked.

### Undo/Redo, and Crash Recovery

- Undo/redo spans 50 edits deep (current target; to be defined by the Architect pending testing and memory constraints).
- If an orphan is found during load, offer the option to link or delete it to correct the model.
- Since the model is multi-file, atomic writes are not possible. Transactional writes should be simulated; if one in a sequence fails, the whole sequence is rolled back.

### Logging

- Logging is required.
- File-based logging is required since stdio is used to communicate with the agent.

### Compatibility & Versioning

- Each model home includes a complete set of schema and configuration files snapshotted at model creation time.
- Compatibility is guaranteed across a major version of Aurora.
    - Per semver, all minor and patch changes are backward compatible.
    - Tooling of a given major version MUST be able to load model homes created within that major version.
- The server MAY offer to upgrade a model home on load, but it MUST be able to work with the model without upgrading.
- Incompatible models are detected via schema validation (behavior defined in Aurora/).
- `reference/Aurora.modelconfiguration.json` MUST include a `version` property so tooling can identify the exact registry version.
    - This requires an update to the matching schema.
    - This may require changes to the `aurora_shared` library.

### Tool Calls

- Tool calls and tool results MUST be structured, machine-readable JSON.
- The server MUST return progress, success, or failure responses on every call.

#### Minimum tool capabilities

The server MUST expose an API sufficient to support agentic tasks without full model materialization:

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

### Configuration and Secrets

- Secrets (e.g., API keys) MUST NOT be stored in plain text.
    - Prefer OS keychain or equivalent secure storage.

### Privacy and Control

- The server MUST allow the user to control what content is shared (selected card only, view context, or broader model context).
- Calls to agents MUST be scrubbed of secrets.
- Logs MUST NOT include secrets.

### Editing

- The server should operate in one of two (user choice) ways:
    - (Default) Agents MAY propose edits, but the server MUST require explicit user confirmation before applying changes to the model.
    - Agents MAY make edits freely as required to meet user needs. No confirmation is required for normal edits; deletes should still require confirmation.

## Architecture Notes (Prompt for the Architect)

This is not the official design, but it defines minimum boundaries the Architecture model should capture.

- Threading:
    - The main thread handles model communication (stdio) so that callers are never left waiting.
    - A second thread handles engine work (I/O, validation, rendering).
    - Indexing has one or more dedicated threads.
- Communication:
    - Threads communicate through signals.
    - Long-running work is cancellable and reports progress.
- State:
    - The server maintains a small in-memory working set.
    - The index enables navigation/search without fully materializing the model.
- Writes:
    - Writes are transactional on save.
    - Validation gates writes.

## ADRs

- Rust has been chosen as the implementation language due to:
    - Smallest memory footprint
    - Prevents numerous common pitfalls by default and design
    - Highest performance in preliminary testing
    - Compiles and runs on the most platforms
- The following libraries have been reviewed and approved for use:
    - This list is not exhaustive. Additional libraries may be used when justified and reviewed.
    - `anyhow`, `thiserror` for error handling
    - `sha2`, `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
    - `chrono` for time and date handling
    - `clap` for CLI interfaces
    - `config` for configuration file handling
    - `dirs` for standard config/data/cache directories
    - `fern`, `log` for logging
    - `serde` (and sublibraries), `serde_json` for serialization
    - `tokio` (and sublibraries) for async runtime
    - `url`, `urlencoding` for URL handling
    - `tantivy` for indexing and search
    - `rmcp` for the MCP surface
