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
	+ Next Actions:
		- Keep derived artifacts (`docs/design/MIS-001/**` and `docs/design/AGENT-MIS-001.jsjson`) synchronized with the source model (`docs/design/aurora/**`). This workspace snapshot still lacks the Rust workspace members under `tools/*` (for example `tools/aurora_cli`), so updates are currently manual.
		- Reconcile supported card types in the Editor/VSCode tooling surfaces vs. newly added security/runtime card types (AST/THR/RIS/CTL/DEP/NOD/DTS).
		- Continue decomposing Editor and VSCode Extension into concrete implementation tasks, interfaces, and tests.

- [ ] **BackendDeveloper** (APP-001) **Bootstrap Aurora CLI and shared tooling**
	+ Status: Coding
	+ Links:
		- [Source: `tools/aurora_cli/src/main.rs`](../tools/aurora_cli/src/main.rs)
		- [Library & tests: `tools/aurora_shared/src/render.rs`](../tools/aurora_shared/src/render.rs)
	+ Next Actions:
		- Flesh out version bump workflows and additional CLI surfaces once shared semantics are defined.
		- Expand shared library coverage (schema validation, workspace packaging) so Editor/VSCode hosts can reuse the same logic.

- [ ] **BackendDeveloper** (APP-002) **Tauri Editor Backend**
	+ Status: Coding
	+ Links:
		- [`tauri.conf.json`](../tools/aurora_editor_backend/tauri.conf.json)
		- [`src/main.rs`](../tools/aurora_editor_backend/src/main.rs)
		- [`src/commands.rs`](../tools/aurora_editor_backend/src/commands.rs)
		- [`static/index.html`](../tools/aurora_editor_backend/static/index.html)
	+ Next Actions:
		- Wire up add/delete card workflows plus audit/version bump policies exposed to the UI.
		- Implement workspace watchers and trust gating before enabling background edits.
		- Add integration tests or mocked Tauri command harness once system dependencies (webkit/libsoup) are available locally.
