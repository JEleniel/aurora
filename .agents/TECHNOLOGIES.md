# Technologies

## Aurora Editor backend (APP-002)

The Aurora Editor UI talks to a Rust Tauri backend that lives in `tools/aurora_editor/src-tauri`. This section documents the backend stack, public command surface, and data contracts that the UI calls via `@tauri-apps/api/core`.

### Stack

- Rust 2024 crate: `tools/aurora_editor/src-tauri/Cargo.toml`.
- Tauri v2.5.3 (`tauri` + `tauri-build`).
- Tauri dialog plugin (`tauri-plugin-dialog` + `@tauri-apps/plugin-dialog`).
- Shared rendering/validation library: `tools/aurora_shared`.
- Serialization: `serde`, `serde_json`.
- Error propagation: `anyhow`.
- Logging: `tracing`, `tracing-subscriber`.

### Key files

| Area | Path | Purpose |
| --- | --- | --- |
| Entrypoint | `tools/aurora_editor/src-tauri/src/main.rs` | Tauri app bootstrap, command registration, logging init. |
| Commands | `tools/aurora_editor/src-tauri/src/commands.rs` | All `#[tauri::command]` handlers exposed to the UI. |
| DTOs | `tools/aurora_editor/src-tauri/src/types.rs` | Request/response structures shared by commands. |
| State | `tools/aurora_editor/src-tauri/src/state.rs` | Workspace state storage for the running app. |
| Workspace helpers | `tools/aurora_editor/src-tauri/src/workspace.rs` | Path normalization, trust checks, workspace-relative helpers. |
| Audit helpers | `tools/aurora_editor/src-tauri/src/audit.rs` | Audit trail creation and bump semantics for card edits. |
| Tauri config | `tools/aurora_editor/src-tauri/tauri.conf.json` | Dev/build wiring (`beforeDevCommand`, `devUrl`, window config). |
| UI wrapper | `tools/aurora_editor/src/lib/tauri.ts` | TypeScript invoke helpers and DTO mirrors. |

### Command surface (Tauri `invoke`)

All commands are registered in `main.rs` and implemented in `commands.rs`. Payloads are snake_case, matching the Rust struct field names.

| Command | Payload | Returns | Requires trusted workspace |
| --- | --- | --- | --- |
| `health_check` | None | `string` (`"ok"`) | No |
| `set_workspace` | `{ root, trusted }` | `WorkspaceInfo` | No |
| `workspace_status` | None | `WorkspaceInfo` | No (workspace must exist) |
| `discover_models` | None | `ModelHomeInfo[]` | No (workspace must exist) |
| `load_model_snapshot` | `{ model_home }` | `ModelSnapshot` | No |
| `validate_model_snapshot` | `{ model_home }` | `ValidationReport` | No |
| `render_card_markdown` | `{ request: RenderRequest }` | `RenderSummaryDto` | Yes |
| `render_views_bundle` | `{ request: RenderRequest }` | `RenderSummaryDto` | Yes |
| `render_all_assets` | `{ request: RenderRequest }` | `RenderSummaryDto` | Yes |
| `write_compact_export` | `{ request: CompactRequest }` | `string` (output path) | Yes |
| `create_card` | `{ request: CreateCardRequest }` | `CardRecord` | Yes |
| `update_card` | `{ request: UpdateCardRequest }` | `CardRecord` | Yes |
| `delete_card` | `{ request: DeleteCardRequest }` | `CardRecord` | Yes |

### Data contracts

Contracts live in `tools/aurora_editor/src-tauri/src/types.rs` and are mirrored by the UI in `tools/aurora_editor/src/lib/tauri.ts`.

- `WorkspaceInfo`: `{ root: string, trusted: boolean }`.
- `ModelHomeInfo`: `{ root: string, has_schema: boolean }`.
- `ModelSnapshot`: `{ home: string, cards: CardRecord[] }`.
- `CardRecord`: `{ card: Card, source_path: string }`.
- `RenderRequest`: `{ model_home: string, output_dir: string }`.
- `CompactRequest`: `{ model_home: string, output_path?: string | null }`.
- `RenderSummaryDto`: `{ cards_written: number, views_written: number, output_dir: string }`.
- `CreateCardRequest` / `UpdateCardRequest` / `DeleteCardRequest`: include `model_home`, `relative_path`, `editor`, and a `CardDraft` (plus optional `bump` for update/delete).

### Trust and validation rules

- The workspace must be configured before any model operations.
- `render_*`, `write_compact_export`, and card mutation commands require `workspace.trusted = true`.
- Model homes must live inside the workspace and include `Aurora.schema.json`.
- Card edits are validated when the existing model is already valid; failing edits are rolled back.

### Error handling expectations

- Command handlers return `Result<_, String>`; the UI should display the string as a user-friendly message.
- Errors are logged with `tracing` at the backend boundary; do not log secrets in the UI.

### Adding new commands

- Implement the handler in `src-tauri/src/commands.rs`.
- Register it in `src-tauri/src/main.rs` (`tauri::generate_handler!`).
- Add or update DTOs in `src-tauri/src/types.rs`.
- Mirror the payload/response in `src/lib/tauri.ts` to keep the UI strongly typed.
