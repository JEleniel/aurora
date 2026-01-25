# Progress Plan

- [ ] **Architect** (MIS-001) **Provide Default Tooling for AURORA**
    + Status: Design
    + Links:
        - [Design Sketch: CLI (APP-001)](../docs/design/CLI.md)
        - [Design Sketch: Editor (APP-002)](../docs/design/Editor.md)
        - [Design Sketch: VS Code Extension (APP-003)](../docs/design/VSCode_Extension.md)
        - [Aurora Mission Card](../docs/design/aurora/MIS-001-Provide_Default_Tooling_for_AURORA.jsjson)
        - [Card Definitions](../.github/instructions/Card_Definitions.md)
        - [View Definitions](../.github/instructions/View_Definitions.md)
        - [Relationship Definitions](../.github/instructions/Relationship_Definitions.md)
        - [SYS-001 Default Tooling Platform](../docs/design/aurora/MIS-001/System/SYS-001-Default_Tooling_Platform.jsjson)
        - [DRI-001 Produce Common Architectural Artifacts](../docs/design/aurora/MIS-001/Driver/DRI-001-Produce_Common_Architectural_Artifacts.jsjson)
        - [DRI-002 Explain Entire Model in Simple Language](../docs/design/aurora/MIS-001/Driver/DRI-002-Explain_Entire_Model_in_Simple_Language.jsjson)
        - [DRI-003 One Model to Rule Them All](../docs/design/aurora/MIS-001/Driver/DRI-003-One_Model_to_Rule_Them_All.jsjson)
        - [DRI-004 Human and Agent Friendly](../docs/design/aurora/MIS-001/Driver/DRI-004-Human_and_Agent_Friendly.jsjson)
        - [DRI-005 Open and Unencumbered](../docs/design/aurora/MIS-001/Driver/DRI-005-Open_and_Unencumbered.jsjson)
        - [APP-001 Command Line Tools](../docs/design/aurora/MIS-001/Application/APP-001-Command_Line_Tools.jsjson)
        - [APP-002 Editor](../docs/design/aurora/MIS-001/Application/APP-002-Editor.jsjson)
        - [APP-003 VSCode Extension](../docs/design/aurora/MIS-001/Application/APP-003-VSCode_Extension.jsjson)
        - [COM-001 Shared Library](../docs/design/aurora/MIS-001/Component/COM-001-Shared_Library.jsjson)
        - [COM-003 Tauri Rust Backend](../docs/design/aurora/MIS-001/Component/COM-003-Tauri_Rust_Backend.jsjson)
        - [COM-013 VSCode Extension Host](../docs/design/aurora/MIS-001/Component/COM-013-VSCode_Extension_Host.jsjson)
        - [COM-014 Webview UI Host](../docs/design/aurora/MIS-001/Component/COM-014-Webview_UI_Host.jsjson)
        - [COM-015 Workspace Adapter](../docs/design/aurora/MIS-001/Component/COM-015-Workspace_Adapter.jsjson)
        - [CAP-006 Interactive Authoring](../docs/design/aurora/MIS-001/Capability/CAP-006-Interactive_Authoring.jsjson)
        - [CAP-007 In-Editor Visualization](../docs/design/aurora/MIS-001/Capability/CAP-007-In-Editor_Visualization.jsjson)
        - [CAP-008 Workspace Integration](../docs/design/aurora/MIS-001/Capability/CAP-008-Workspace_Integration.jsjson)
        - [CAP-009 Safe and Trusted Operation](../docs/design/aurora/MIS-001/Capability/CAP-009-Safe_and_Trusted_Operation.jsjson)
        - [FEA-005 Render Views](../docs/design/aurora/MIS-001/Feature/FEA-005-Render_Views.jsjson)
        - [FEA-009 Show Validation Diagnostics](../docs/design/aurora/MIS-001/Feature/FEA-009-Show_Validation_Diagnostics.jsjson)
        - [FEA-010 Visualize Model Graph](../docs/design/aurora/MIS-001/Feature/FEA-010-Visualize_Model_Graph.jsjson)
        - [FEA-011 Preview Rendered Markdown](../docs/design/aurora/MIS-001/Feature/FEA-011-Preview_Rendered_Markdown.jsjson)
        - [FEA-013 Safe File Operations](../docs/design/aurora/MIS-001/Feature/FEA-013-Safe_File_Operations.jsjson)
        - [FEA-014 Workspace Trust Gating](../docs/design/aurora/MIS-001/Feature/FEA-014-Workspace_Trust_Gating.jsjson)
        - [FEA-015 Sanitized Rendering](../docs/design/aurora/MIS-001/Feature/FEA-015-Sanitized_Rendering.jsjson)
        - [INT-003 Editor Backend API](../docs/design/aurora/MIS-001/Interface/INT-003-Editor_Backend_API.jsjson)
        - [INT-004 VSCode Webview Messaging API](../docs/design/aurora/MIS-001/Interface/INT-004-VSCode_Webview_Messaging_API.jsjson)
        - [INT-005 VSCode Commands Interface](../docs/design/aurora/MIS-001/Interface/INT-005-VSCode_Commands_Interface.jsjson)
        - [DEP-001 Local Development](../docs/design/aurora/MIS-001/Deployment/DEP-001-Local_Development.jsjson)
        - [DEP-002 User Environment](../docs/design/aurora/MIS-001/Deployment/DEP-002-User_Environment.jsjson)
        - [DTS-001 Workspace File System](../docs/design/aurora/MIS-001/Data Store/DTS-001-Workspace_File_System.jsjson)
        - [AST-001 Aurora Models](../docs/design/aurora/MIS-001/Asset/AST-001-Aurora_Models.jsjson)
        - [THR-001 Script Injection via Markdown](../docs/design/aurora/MIS-001/Threat/THR-001-Script_Injection_via_Markdown.jsjson)
        - [RIS-001 Injected Content in Rendered Output](../docs/design/aurora/MIS-001/Risk/RIS-001-Injected_Content_in_Rendered_Output.jsjson)
        - [CTL-001 Workspace Trust Gate](../docs/design/aurora/MIS-001/Control/CTL-001-Workspace_Trust_Gate.jsjson)
        - [REQ-010 Edit Models Interactively](../docs/design/aurora/MIS-001/Requirement/REQ-010-Edit_Models_Interactively.jsjson)
        - [REQ-011 Show Validation Diagnostics](../docs/design/aurora/MIS-001/Requirement/REQ-011-Show_Validation_Diagnostics.jsjson)
        - [REQ-016 Workspace Trust Gating](../docs/design/aurora/MIS-001/Requirement/REQ-016-Workspace_Trust_Gating.jsjson)
        - [REQ-018 Store the Model in a Postable Format](../docs/design/aurora/MIS-001/Requirement/REQ-018-Store_the_Model_in_a_Postable_Format.jsjson)
        - [REQ-019 Render Canonical View Set](../docs/design/aurora/MIS-001/Requirement/REQ-019-Render_Canonical_View_Set.jsjson)
        - [STR-003 Store the Model in a Postable Format](../docs/design/aurora/MIS-001/Story/STR-003-Store_the_Model_in_a_Postable_Format.jsjson)
        - [STO-001 Store the Model in a Postable Format (tombstone)](../docs/design/aurora/MIS-001/Story/STO-001-Store_the_Model_in_a_Postable_Format.jsjson)
        - [PRO-001 Model Authoring Workflow](../docs/design/aurora/MIS-001/Process/PRO-001-Model_Authoring_Workflow.jsjson)
        - [PRO-002 In-Tool Model Editing Workflow](../docs/design/aurora/MIS-001/Process/PRO-002-In-Tool_Model_Editing_Workflow.jsjson)
        - [ACT-001 Architect](../docs/design/aurora/MIS-001/Actor/ACT-001-Architect.jsjson)
        - [ACT-002 User](../docs/design/aurora/MIS-001/Actor/ACT-002-User.jsjson)
        - [ACT-003 Malicious Workspace Author](../docs/design/aurora/MIS-001/Actor/ACT-003-Malicious_Workspace_Author.jsjson)
        - [TES-001 Validate Model Command](../docs/design/aurora/MIS-001/Test/TES-001-Validate_Model_Command.jsjson)
        - [TES-002 Render All Command](../docs/design/aurora/MIS-001/Test/TES-002-Render_All_Command.jsjson)
        - [TES-003 Compact Command](../docs/design/aurora/MIS-001/Test/TES-003-Compact_Command.jsjson)
        - [CNS-001 Text Based Data Format](../docs/design/aurora/MIS-001/Constraint/CNS-001-Text_Based_Data_Format.jsjson)
        - [CNS-002 Directed, Locally Cyclic Graph (Invariant)](../docs/design/aurora/MIS-001/Constraint/CNS-002-Directed__Locally_Cyclic_Graph.jsjson)
        - [CNS-003 No Inherent Semantic Meaning](../docs/design/aurora/MIS-001/Constraint/CNS-003-No_Inherent_Semantic_Meaning.jsjson)
        - [CNS-004 No Orphans](../docs/design/aurora/MIS-001/Constraint/CNS-004-No_Orphans.jsjson)
        - [CNS-005 Graphviz DOT View Conventions](../docs/design/aurora/MIS-001/Constraint/CNS-005-Graphviz_DOT_View_Conventions.jsjson)
    + Next Actions:
        - Keep derived artifacts (`docs/design/MIS-001/**` and `docs/design/AGENT-MIS-001.jsjson`) synchronized with the source model (`docs/design/aurora/**`). This workspace snapshot still lacks the Rust workspace members under `tools/*` (for example `tools/aurora_cli`), so updates are currently manual.
        - Keep the canonical view renderer aligned with the current artifact contract (SVG-first view artifacts with DOT sources under `Views/source/`; no per-view Markdown embedding).
        - Reconcile supported card types in the Editor/VSCode tooling surfaces vs. newly added security/runtime card types (AST/THR/RIS/CTL/DEP/NOD/DTS).
        - Continue decomposing Editor and VSCode Extension into concrete implementation tasks, interfaces, and tests.

- [ ] **BackendDeveloper** (APP-001) **Bootstrap Aurora CLI and shared tooling**
    + Status: Coding
    + Links:
        - [Source: `tools/aurora_cli/src/main.rs`](../tools/aurora_cli/src/main.rs)
        - [Library & tests: `tools/aurora_shared/src/render.rs`](../tools/aurora_shared/src/render.rs)
    + Next Actions:
        - Socialize the new card/view output layout with downstream tooling (Editor, VS Code), including SVG-first view artifacts and DOT sources under `Views/source/`.
        - Surface the Unicode icon prefixes exposed by `aurora_shared::render_views` inside Editor/VS Code previews so users see the same visual cues as the canonical DOT/SVG outputs.
        - Regenerate view artifacts after the renderer update lands (subtype line format now implemented; COM-009/COM-010/COM-011 use lowercase `struct`; white background; black edge lines/labels).
        - Harden the canonical view renderer (`tools/aurora_shared/src/render.rs`) with boundary membership heuristics and regression coverage (linked Note/Boundary inclusion, root-derived filtering, root connectivity checks with view-defined roots, and note parent gating are complete).
        - Mirror the new "Zoom & pan" SVG link cue inside the Editor and VS Code surfaces so the guidance is consistent across hosts.
        - Flesh out version bump workflows and additional CLI surfaces once shared semantics are defined.
        - Expand shared library coverage (schema validation, workspace packaging) so Editor/VSCode hosts can reuse the same logic.
        - Confirm downstream tooling can render HTML labels with `<br />` line breaks and subtype lines in previews.

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

- [ ] **UIDeveloper** (APP-002) **Tauri Editor UI**
    + Status: Coding
    + Links:
        - [`index.html`](../tools/aurora_editor/index.html)
        - [`src/App.svelte`](../tools/aurora_editor/src/App.svelte)
        - [`src/main.ts`](../tools/aurora_editor/src/main.ts)
        - [`src/styles/app.css`](../tools/aurora_editor/src/styles/app.css)
        - [`src/lib/components`](../tools/aurora_editor/src/lib/components)
    + UI Flow Notes:
        - Landing hero shows backend + action status pills with last ping and command feedback.
        - Workspace control card collects root path, trust toggle, model home, and output paths.
        - Workspace connect supports Enter in the path field and reports backend availability before issuing commands.
        - Workspace path input trims whitespace, accepts `~/` shorthand, and resolves absolute paths within the workspace root.
        - Action buttons dispatch clicks reliably and surface backend offline errors in the status pill.
        - Quick actions trigger validation, render, and compact export flows using backend commands.
        - System pulse includes last-interaction telemetry to confirm UI responsiveness.
        - Action errors render detailed messages when Tauri responses include structured error payloads.
        - Editor invokes use camelCase payload keys to match backend command schemas.
        - Model home selector shows detection counts and handles mixed payload casing.
        - Model home selector includes a collapsible list of detected roots for troubleshooting and syncs selection to available models.
        - Diagnostics feed summarizes validation results and top findings.
    + Accessibility Notes:
        - Focus-visible outlines and high-contrast palette are in place; verify uppercase microcopy meets AAA contrast targets.
        - Workspace path field supports keyboard-only connect via Enter for AAA input parity.
        - Rounded controls maintain 44px minimum hit target in the action grid.
    + Dependency Notes:
        - Reviewed Svelte component structure and scoped styling guidance from current Svelte docs.
        - Confirmed Tauri v2 `invoke` usage via `@tauri-apps/api/core` for frontend calls.
        - Added a default Tauri capability to allow internal devtools toggling on the main window during development.
        - Allowed IPC access for the Vite dev server by permitting localhost URLs and wildcard paths in the default capability.
    + Next Actions:
        - Add keyboard navigation and status announcements for validation events.
        - Populate the left navigation with live model inventory and recent activity.
        - Introduce inline error summaries near workspace inputs.
