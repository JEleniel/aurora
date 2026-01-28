# Progress Plan

- [ ] **Architect** (MIS-001) **Provide Default Tooling for AURORA**
    + Status: Design
    + Updates: Refined relationship guidance (validates targets, includes/uses notes), refreshed the matrix at `.github/instructions/details/Relationships_Matrix.md`, and added new `persists to`, `provides`, and `reverse proxies` entries plus Application support in `limits`.
    + Links:
        - [Aurora Mission Card (source)](../docs/design/aurora/MIS-001-Provide_Default_Tooling_for_AURORA.jsjson)
        - [Aurora Mission Folder (all cards)](../docs/design/aurora/MIS-001/)
        - [Compact Export (agent snapshot)](../docs/design/aurora/AGENT-MIS-001.jsjson)
        - [Rendered Model Output (docs + views)](../docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/)
        - [SYS-001 Default Tooling Platform](../docs/design/aurora/MIS-001/System/SYS-001-Default_Tooling_Platform.jsjson)
        - [APP-001 Command Line Tools](../docs/design/aurora/MIS-001/Application/APP-001-Command_Line_Tools.jsjson)
        - [APP-002 Editor](../docs/design/aurora/MIS-001/Application/APP-002-Editor.jsjson)
        - [APP-003 VSCode Extension](../docs/design/aurora/MIS-001/Application/APP-003-VSCode_Extension.jsjson)
        - [Design Sketch: CLI (APP-001)](../docs/design/CLI.md)
        - [Design Sketch: Editor (APP-002)](../docs/design/Editor.md)
        - [Design Sketch: VS Code Extension (APP-003)](../docs/design/VSCode_Extension.md)
        - [Card Definitions](../.github/instructions/details/Card_Definitions.md)
        - [View Definitions](../.github/instructions/details/View_Definitions.md)
        - [Relationship Definitions](../.github/instructions/details/Relationship_Definitions.md)
    + Next Actions:
        - Keep derived artifacts (`docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/**` and `docs/design/aurora/AGENT-MIS-001.jsjson`) synchronized with the source model (`docs/design/aurora/**`) by re-running `aurora_cli` after model edits.
        - Keep the canonical view renderer aligned with the current artifact contract (SVG-first view artifacts with DOT sources under `Views/source/`; no per-view Markdown embedding).
        - Reconcile supported card types in the Editor/VSCode tooling surfaces vs. newly added security/runtime card types (AST/THR/RIS/CTL/DEP/NOD/DTS).
        - Continue decomposing Editor and VSCode Extension into concrete implementation tasks, interfaces, and tests.

- [ ] **BackendDeveloper** (APP-001) **Bootstrap Aurora CLI and shared tooling**
    + Status: Coding
    + Links:
        - [Source: `tools/aurora_cli/src/main.rs`](../tools/aurora_cli/src/main.rs)
        - [Library & tests: `tools/aurora_shared/src/render.rs`](../tools/aurora_shared/src/render.rs)
    + Updates: Added matrix-based validation warnings, completion summaries in the CLI, Everything View dashed-edge styling with regression coverage, new validation tests for updated relationship matrix rules, a CLI instructions-root override with reporting, plus centered SVG node labels and disjoint-boundary overlap avoidance in the SVG renderer with regression coverage. Aligned validation fixtures with the canonical relationships matrix (including `Application implements Test`, `Component implements Class`, `Artifact persists to Data Store`, and `uses`), added a standalone aurora_shared lockfile, and clarified CLI help/logging around the canonical registry files.
    + Next Actions:
        - Validate updated CLI render output after ID-prefixed headers and left-aligned icon labels; re-render artifacts if needed.
        - Re-render views to confirm the dot-only layout selection matches expected output.
        - Keep the card/view output layout consistent across CLI, Editor, and VS Code (SVG-first view artifacts and DOT sources under `Views/source/`).
        - Align visual cues across hosts (Unicode icon prefixes, subtype line format, and the "Zoom & pan" link cue).
        - Re-render views after SVG shape/icon updates to confirm Card_Definitions palette alignment.
        - Regenerate view artifacts any time the view renderer changes (to keep `docs/design/*/Views/` in sync).
        - Continue hardening the canonical view renderer with regression coverage around boundaries/notes/root filtering.
        - Ensure boundary cluster DOT output uses graph attribute statements so Graphviz accepts clusters with multiple attributes.
        - Expand shared library coverage (schema validation, workspace packaging) so Editor/VSCode hosts can reuse the same logic.
        - Flesh out version bump workflows and additional CLI surfaces once shared semantics are defined.
        - Keep multi-mission model home handling in the CLI covered with regression tests.

- [ ] **BackendDeveloper** (APP-002) **Tauri Editor Backend**
    + Status: Coding
    + Links:
        - [`tauri.conf.json`](../tools/aurora_editor/src-tauri/tauri.conf.json)
        - [`src/main.rs`](../tools/aurora_editor/src-tauri/src/main.rs)
        - [`src/commands.rs`](../tools/aurora_editor/src-tauri/src/commands.rs)
        - [`static/index.html`](../tools/aurora_editor/static/index.html)
    + Next Actions:
        - Wire up the UI to the expanded backend command surface (workspace setup, model operations, card CRUD).
        - Implement workspace watchers for live reload and diagnostics updates.
        - Add integration tests or a mocked Tauri command harness once system dependencies (webkit/libsoup) are available locally.
        - Surface backend validation rollback errors in the editor UI when edits are rejected.

- [ ] **UIDeveloper** (APP-002) **Tauri Editor UI**
    + Status: Coding
    + Links:
        - [`index.html`](../tools/aurora_editor/index.html)
        - [`src/App.svelte`](../tools/aurora_editor/src/App.svelte)
        - [`src/main.ts`](../tools/aurora_editor/src/main.ts)
        - [`src/styles/app.css`](../tools/aurora_editor/src/styles/app.css)
        - [`src/lib/components`](../tools/aurora_editor/src/lib/components)
    + UI Notes:
        - Workspace connect + quick actions (validate/render/compact) are the primary flow.
        - Prefer TailwindCSS for styling, falling back to custom CSS only when needed.
        - Diagnostics + model home selection should remain the "always visible" feedback loop.
        - Keep payload casing consistent with backend command schemas (snake_case).
        - Trim render output paths before invocation and surface guidance for ~ path handling.
        - Workspace selection uses the folder dialog and auto-connects to discover model homes; default output path is `docs/design/`.
        - Header is condensed to a 4rem bar with a 2.4rem H1; status pills are right-aligned and stacked content is minimized.
    + Accessibility Notes:
        - Maintain keyboard-only parity and minimum hit-target sizing on core actions.
        - Workspace and output path hints are plain text and readable at default zoom.
    + Dependency Notes:
        - Using Tauri v2 `invoke` via `@tauri-apps/api/core` and a permissive dev capability for local iteration.
        - Keep `@tailwindcss/vite` and `tailwindcss` declared locally to avoid cross-package Vite type conflicts.
        - Use `@tauri-apps/plugin-dialog` with the matching Rust plugin to drive workspace folder selection.
    + Next Actions:
        - Add keyboard navigation and status announcements for validation events.
        - Populate the left navigation with live model inventory and recent activity.
        - Introduce inline error summaries near workspace inputs.
