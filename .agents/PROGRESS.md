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
    + Updates: Added matrix-based validation warnings, completion summaries in the CLI, Everything View dashed-edge styling with regression coverage, new validation tests for updated relationship matrix rules, a CLI instructions-root override with reporting, plus centered SVG node labels and disjoint-boundary overlap avoidance in the SVG renderer with regression coverage. Aligned validation fixtures with the canonical relationships matrix (including `Application implements Test`, `Component implements Class`, `Artifact persists to Data Store`, and `uses`), added a standalone aurora_shared lockfile, and clarified CLI help/logging around the canonical registry files. Embedded the canonical registries directly in `aurora_shared`, removed the CLI's dependency on external instruction files, and aligned view rendering with the styling guide (label ordering, icon sizing, colors, dotted note edges, and subtype-aware view filtering). Refined SVG layout again by restoring curved splines with relaxed spacing, narrowing hexagons, shifting diamond/hexagon icon offsets, and making State nodes slightly larger and circular.
    + Next Actions:
        - Re-render views to reflect embedded registries, updated label ordering, and dotted note edges; confirm SVG output matches styling guide.
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

- [ ] **Architect** (MIS-002) **Relationship Matrix Reference**
    + Status: Design
    + Updates: Added a minimal reference model under `docs/design/aurora/` that encodes the requested relationship matrix as concrete card links (intended as a vocabulary/edge-case fixture for validation and rendering).
    + Links:
        - [Aurora Mission Card (source)](../docs/design/aurora/MIS-002-Relationship_Matrix_Reference.jsjson)
        - [Aurora Mission Folder (all cards)](../docs/design/aurora/MIS-002/)
        - [Rendered Model Output (docs + views)](../docs/design/MIS-002-Relationship_Matrix_Reference/)
        - [Design Index](../docs/design/README.md)
    + Next Actions:
        - Decide whether MIS-002 should validate cleanly against the embedded registries (update the model’s verbs/card pairs) or should instead drive updates to the embedded relationship registry.
        - Address a Graphviz “bad label format” failure observed when rendering the “Entire Model” view (icon label contained `{}` for `Class`), so `aurora_cli render-all` succeeds end-to-end.

- [ ] **BackendDeveloper** (APP-002) **Tauri Editor Backend**
    + Status: implementation
    + Links:
        - [Design: APP-002 Editor](../docs/design/aurora/MIS-001/Application/APP-002-Editor.jsjson)
        - [Design: COM-003 Tauri Rust Backend](../docs/design/aurora/MIS-001/Component/COM-003-Tauri_Rust_Backend.jsjson)
        - [Design: INT-003 Editor Backend API](../docs/design/aurora/MIS-001/Interface/INT-003-Editor_Backend_API.jsjson)
        - [`tauri.conf.json`](../tools/aurora_editor/src-tauri/tauri.conf.json)
        - [`src/main.rs`](../tools/aurora_editor/src-tauri/src/main.rs)
        - [`src/commands.rs`](../tools/aurora_editor/src-tauri/src/commands.rs)
        - [`src/workspace.rs`](../tools/aurora_editor/src-tauri/src/workspace.rs)
        - [`aurora_shared` render + registry parsing](../tools/aurora_shared/src/render.rs)
    + Scope (design-aligned):
        - FEA-008 (REQ-010): interactive, safe create/update/delete operations (already present as commands; needs UI + hardening).
        - FEA-009 (REQ-011): structured diagnostics (already present; needs richer payload + eventing).
        - FEA-012/013 (REQ-014/015/016): workspace integration + safe file ops + trust gating (mostly present; needs watchers + tighter policy).
        - FEA-011/015 (REQ-013/017): markdown preview + sanitized rendering (backend needs preview-friendly API + security posture).
        - REQ-018: “postable” model packaging (ZIP example) (not implemented; compact export exists but is not ZIP).
    + Implementation checkpoints (step-by-step):
        - [ ] **BackendDeveloper** (APP-002-BE-01) **Stabilize the command contract**
            * Status: completed
            * Owner: BackendDeveloper
            * Links:
                - [`src-tauri/src/types.rs`](../tools/aurora_editor/src-tauri/src/types.rs)
                - [`src/lib/tauri.ts`](../tools/aurora_editor/src/lib/tauri.ts)
            * Next Action: Keep the typed command contract in `src/lib/tauri.ts` aligned with backend DTOs as new commands are added.
        - [ ] **BackendDeveloper** (APP-002-BE-02) **Add “preview-first” rendering commands**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-013 Preview Rendered Markdown](../docs/design/aurora/MIS-001/Requirement/REQ-013-Preview_Rendered_Markdown.jsjson)
                - [Design: REQ-017 Sanitized Rendering](../docs/design/aurora/MIS-001/Requirement/REQ-017-Sanitized_Rendering.jsjson)
            * Next Action: Add backend commands that return render output as data (strings / structured payload) suitable for in-app preview (avoid “write to disk then read” as the primary UX path).
        - [ ] **BackendDeveloper** (APP-002-BE-03) **Workspace watchers + event stream**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-014 Workspace Integration](../docs/design/aurora/MIS-001/Requirement/REQ-014-Workspace_Integration.jsjson)
            * Next Action: Implement a watcher that tracks changes under the active workspace/model home and emits events to the UI (model-homes changed, snapshot stale, validation stale). Keep watcher scope constrained to the workspace root.
        - [ ] **BackendDeveloper** (APP-002-BE-04) **Diagnostics enrichment and navigation hooks**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-011 Show Validation Diagnostics](../docs/design/aurora/MIS-001/Requirement/REQ-011-Show_Validation_Diagnostics.jsjson)
            * Next Action: Ensure diagnostics always include enough context for UI navigation (card id when possible, relative path when possible). Add helper endpoints for “diagnostics for card” or “open card by id” flows if needed.
        - [ ] **BackendDeveloper** (APP-002-BE-05) **Editing guardrails + rollback UX**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-010 Edit Models Interactively](../docs/design/aurora/MIS-001/Requirement/REQ-010-Edit_Models_Interactively.jsjson)
                - [`create_card/update_card/delete_card`](../tools/aurora_editor/src-tauri/src/commands.rs)
            * Next Action: Make edit failures “actionable” for UI (structured error response including validation errors). Confirm rollback paths are fully atomic and cannot leave partial writes.
        - [ ] **BackendDeveloper** (APP-002-BE-06) **ZIP export/import for postable models**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-018 Store the Model in a Postable Format](../docs/design/aurora/MIS-001/Requirement/REQ-018-Store_the_Model_in_a_Postable_Format.jsjson)
                - [`write_compact_model`](../tools/aurora_shared/src/render.rs)
            * Next Action: Implement a ZIP (or equivalent archive) export in `aurora_shared` and expose it via the editor backend (trust-gated). Ensure safe path handling (no traversal, no symlink escape) and deterministic archive contents.
        - [ ] **BackendDeveloper** (APP-002-BE-07) **Security posture for webview rendering**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [Design: REQ-016 Workspace Trust Gating](../docs/design/aurora/MIS-001/Requirement/REQ-016-Workspace_Trust_Gating.jsjson)
                - [Design: REQ-017 Sanitized Rendering](../docs/design/aurora/MIS-001/Requirement/REQ-017-Sanitized_Rendering.jsjson)
                - [`tauri.conf.json`](../tools/aurora_editor/src-tauri/tauri.conf.json)
            * Next Action: Replace `csp: null` with a locked-down CSP for non-dev builds and define a minimal capability set for the editor webview. Document which operations are blocked in “untrusted” mode.
        - [ ] **BackendDeveloper** (APP-002-BE-08) **Backend test strategy (unit-first)**
            * Status: pending
            * Owner: BackendDeveloper
            * Links:
                - [`tools/aurora_shared/src/validation.rs`](../tools/aurora_shared/src/validation.rs)
                - [`tools/aurora_editor/src-tauri/src/workspace.rs`](../tools/aurora_editor/src-tauri/src/workspace.rs)
            * Next Action: Add focused unit tests around the editor backend’s workspace path safety helpers, trust gating, and edit rollback behavior. Prefer tests that don’t require a full Tauri runtime.

- [ ] **UIDeveloper** (APP-002) **Tauri Editor UI**
    + Status: implementation
    + Links:
        - [`index.html`](../tools/aurora_editor/index.html)
        - [`src/App.svelte`](../tools/aurora_editor/src/App.svelte)
        - [`src/main.ts`](../tools/aurora_editor/src/main.ts)
        - [`src/styles/app.css`](../tools/aurora_editor/src/styles/app.css)
        - [`src/lib/components`](../tools/aurora_editor/src/lib/components)
        - [Design: APP-002 Editor](../docs/design/aurora/MIS-001/Application/APP-002-Editor.jsjson)
        - [Design: COM-004 Explorer Tree View](../docs/design/aurora/MIS-001/Component/COM-004-Explorer_Tree_View.jsjson)
        - [Design: COM-005 Mind-Map Like View](../docs/design/aurora/MIS-001/Component/COM-005-Mind-Map_Like_View.jsjson)
        - [Design: COM-006 Edit View](../docs/design/aurora/MIS-001/Component/COM-006-Edit_View.jsjson)
        - [Design: COM-007 Markdown View](../docs/design/aurora/MIS-001/Component/COM-007-Markdown_View.jsjson)
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
    + Implementation checkpoints (step-by-step):
        - [ ] **UIDeveloper** (APP-002-UI-01) **Promote “model snapshot” to a first-class UI state**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [`load_model_snapshot` command](../tools/aurora_editor/src-tauri/src/commands.rs)
            * Next Action: Add a shared store that holds (workspace, trust, selected model home, model snapshot, validation report, last render/export results) and refreshes deterministically.
        - [ ] **UIDeveloper** (APP-002-UI-02) **Explorer Tree View (browse + search)**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: COM-004 Explorer Tree View](../docs/design/aurora/MIS-001/Component/COM-004-Explorer_Tree_View.jsjson)
                - [Design: FEA-007 Browse and Search Models](../docs/design/aurora/MIS-001/Feature/FEA-007-Browse_and_Search_Models.jsjson)
            * Next Action: Render a tree/list grouped by card_type (and optionally folder path) with search/filter. Selecting a card should update the “focused card” across all panes.
        - [ ] **UIDeveloper** (APP-002-UI-03) **Diagnostics UX that users can act on**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: REQ-011 Show Validation Diagnostics](../docs/design/aurora/MIS-001/Requirement/REQ-011-Show_Validation_Diagnostics.jsjson)
            * Next Action: Replace the “top 3 findings” summary with a full diagnostics list, severity filtering, and click-to-focus-card (or click-to-open-path) behavior.
        - [ ] **UIDeveloper** (APP-002-UI-04) **Edit View (card CRUD + link editing)**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: COM-006 Edit View](../docs/design/aurora/MIS-001/Component/COM-006-Edit_View.jsjson)
                - [Design: REQ-010 Edit Models Interactively](../docs/design/aurora/MIS-001/Requirement/REQ-010-Edit_Models_Interactively.jsjson)
            * Next Action: Implement an edit form for the focused card (including links + attributes). Hook it to backend create/update/delete and surface rollback/validation errors inline.
        - [ ] **UIDeveloper** (APP-002-UI-05) **Mind-map / graph navigation view**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: COM-005 Mind-Map Like View](../docs/design/aurora/MIS-001/Component/COM-005-Mind-Map_Like_View.jsjson)
                - [Design: REQ-012 Visualize Model Graph](../docs/design/aurora/MIS-001/Requirement/REQ-012-Visualize_Model_Graph.jsjson)
            * Next Action: Add an interactive graph view (zoom/pan + focus-on-card + neighbor expansion). Keep it driven from the model snapshot to avoid backend coupling.
        - [ ] **UIDeveloper** (APP-002-UI-06) **Markdown View with sanitization**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: COM-007 Markdown View](../docs/design/aurora/MIS-001/Component/COM-007-Markdown_View.jsjson)
                - [Design: REQ-013 Preview Rendered Markdown](../docs/design/aurora/MIS-001/Requirement/REQ-013-Preview_Rendered_Markdown.jsjson)
                - [Design: REQ-017 Sanitized Rendering](../docs/design/aurora/MIS-001/Requirement/REQ-017-Sanitized_Rendering.jsjson)
            * Next Action: Render Markdown previews in-app using a sanitizer-first pipeline. Navigation between cards should work from markdown links (at minimum: resolve `CARD_ID` links).
        - [ ] **UIDeveloper** (APP-002-UI-07) **Trust gating UX**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: REQ-016 Workspace Trust Gating](../docs/design/aurora/MIS-001/Requirement/REQ-016-Workspace_Trust_Gating.jsjson)
            * Next Action: Make trust state always visible and enforce it consistently (disable/hide render/export/edit actions when untrusted; show explicit rationale).
        - [ ] **UIDeveloper** (APP-002-UI-08) **Accessibility + keyboard-first navigation**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Design: REQ-010 Edit Models Interactively](../docs/design/aurora/MIS-001/Requirement/REQ-010-Edit_Models_Interactively.jsjson)
            * Next Action: Ensure full keyboard parity for: model selection, search, focus card, edit/save, diagnostics navigation, and graph interactions.
        - [ ] **UIDeveloper** (APP-002-UI-09) **Polish + end-to-end acceptance checks**
            * Status: pending
            * Owner: UIDeveloper
            * Links:
                - [Rendered model output](../docs/design/MIS-001-Provide_Default_Tooling_for_AURORA/)
            * Next Action: Validate the full user flow against the MIS-001 model: connect workspace → select model home → browse → validate → fix issues → render views → preview markdown → export compact/zip.

    + Notes / risks:
        - The source model references `docs/design/Editor.md` via `APP-002.attributes.design_docs`, but that file is currently missing in-repo. Treat the Aurora cards and rendered output as authoritative until the design sketch is added.
