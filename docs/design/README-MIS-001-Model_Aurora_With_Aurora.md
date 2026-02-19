# MIS-001: Model Aurora With Aurora

**[Mission Card](MIS-001-Model_Aurora_With_Aurora.md)**

Define Aurora (Agent-Unified Representation of Requirements and Architecture) using Aurora itself: a deterministic, machine-consumable architecture model rooted at a single Mission card, backed by schemas and canonical registries, and usable by tools and agents.

## Views

![Compliance_Governance.svg](MIS-001/Views/Compliance_Governance.svg)

![Component.svg](MIS-001/Views/Component.svg)

![Context.svg](MIS-001/Views/Context.svg)

![Entire_Model.svg](MIS-001/Views/Entire_Model.svg)

![Landscape.svg](MIS-001/Views/Landscape.svg)

![Process.svg](MIS-001/Views/Process.svg)

![Requirements.svg](MIS-001/Views/Requirements.svg)

![Traceability.svg](MIS-001/Views/Traceability.svg)

![Use_Case.svg](MIS-001/Views/Use_Case.svg)

## Card Index

### Activity

- **[ATV-010 - Backup Model Home](MIS-001/Activity/ATV-010-Backup_Model_Home.md)**: Create a ZIP backup of the model home at load time, stored per conventions defined in Aurora_Specs.

- **[ATV-005 - Edit Canonical Definitions Registry](MIS-001/Activity/ATV-005-Edit_Canonical_Definitions_Registry.md)**: Maintain the canonical registry of card types and allowed outgoing relationships.

- **[ATV-009 - Load Model Home](MIS-001/Activity/ATV-009-Load_Model_Home.md)**: Load a model home for interactive use, using the schemas and reference files included with that model home.

- **[ATV-015 - Unpack Model Home](MIS-001/Activity/ATV-015-Unpack_Model_Home.md)**: Unpack a ZIP-compressed model home into the standard folder structure used by Aurora.

- **[ATV-014 - Pack Model Home](MIS-001/Activity/ATV-014-Pack_Model_Home.md)**: Pack a model home into a single ZIP-compressed file while preserving the standard model home folder structure.

- **[ATV-001 - Validate Model Home](MIS-001/Activity/ATV-001-Validate_Model_Home.md)**: Load cards from the model home and validate them against schemas, canonical registries, and graph invariants.

- **[ATV-006 - Edit View Definitions Registry](MIS-001/Activity/ATV-006-Edit_View_Definitions_Registry.md)**: Maintain the canonical registry of view definitions (roots, included types, and view intent).

- **[ATV-011 - Edit Model Interactively](MIS-001/Activity/ATV-011-Edit_Model_Interactively.md)**: Edit cards, links, and view definitions interactively with immediate feedback, keeping the model valid.

- **[ATV-016 - Apply Accessibility Preferences](MIS-001/Activity/ATV-016-Apply_Accessibility_Preferences.md)**: Apply dark mode defaults, WCAG AA accessibility behavior, and base font scaling preferences.

- **[ATV-008 - Prepare SVG References](MIS-001/Activity/ATV-008-Prepare_SVG_References.md)**: Run svg_prep to generate/update Icons.svg and refresh SVGTemplate.svg defs from source icons and shapes, then synchronize icon availability into Aurora.modelconfiguration.json.

- **[ATV-004 - Append Audit Log Entry](MIS-001/Activity/ATV-004-Append_Audit_Log_Entry.md)**: Append one line to the mission `AuditLog.ndjson` file for each change event, allowing multiple changed cards and related link changes in one entry.

- **[ATV-017 - Configure Logging](MIS-001/Activity/ATV-017-Configure_Logging.md)**: Configure logging sinks and formatting via fern, supporting stdout, stderr, and optional file-based logging.

- **[ATV-002 - Render View Artifacts](MIS-001/Activity/ATV-002-Render_View_Artifacts.md)**: Select roots/included card types per view definitions, traverse the subgraph, and render the view into diagram artifacts.

- **[ATV-003 - Write Compact Export](MIS-001/Activity/ATV-003-Write_Compact_Export.md)**: Write the compact export JSON document for transport and agent consumption.

- **[ATV-013 - Undo And Redo](MIS-001/Activity/ATV-013-Undo_And_Redo.md)**: Allow undo/redo of recent edits (target depth: 50) across interactive editing operations.

- **[ATV-012 - Autosave Changes](MIS-001/Activity/ATV-012-Autosave_Changes.md)**: Persist edits immediately by default (autosave), while still supporting an optional manual save mode.

- **[ATV-007 - Maintain Aurora Schemas](MIS-001/Activity/ATV-007-Maintain_Aurora_Schemas.md)**: Maintain the Aurora JSON schemas used for cards, audit logs, and compact export.

### Actor

- **[ACT-001 - Model Author](MIS-001/Actor/ACT-001-Model_Author.md)**: A human author (often the architect) who edits Aurora cards and keeps registries, references, and audit logs current.

### Application

- **[APP-003 - Aurora Editor](MIS-001/Application/APP-003-Aurora_Editor.md)**: Standalone cross-platform desktop editor for loading, exploring, editing, and packaging Aurora model homes while preserving deterministic validity, auditability, and view generation.

- **[APP-001 - Aurora CLI](MIS-001/Application/APP-001-Aurora_CLI.md)**: Command-line tooling for validating Aurora models, generating views, and exporting compact representations.

- **[APP-002 - SVG Prep](MIS-001/Application/APP-002-SVG_Prep.md)**: Command-line tool that builds Aurora SVG reference assets (Icons.svg, SVGTemplate.svg defs) from source icons/shapes and synchronizes the icon availability list in the canonical model configuration.

### Artifact

- **[ART-011 - Editor Log Output](MIS-001/Artifact/ART-011-Editor_Log_Output.md)**: Editor log output emitted to stdout, stderr, and/or an optional log file, used for troubleshooting and diagnostics.

- **[ART-002 - View Diagram Artifacts](MIS-001/Artifact/ART-002-View_Diagram_Artifacts.md)**: Rendered view artifacts (for example SVG diagrams and DOT sources) generated from a model and view definitions.

- **[ART-003 - Compact Export](MIS-001/Artifact/ART-003-Compact_Export.md)**: The compact JSON export of the model cards and links for transport and agent consumption.

- **[ART-008 - SVG Template](MIS-001/Artifact/ART-008-SVG_Template.md)**: The SVGTemplate.svg reference template used by the renderer to wrap generated drawings and provide shared defs and styling.

- **[ART-004 - Mission Audit Log](MIS-001/Artifact/ART-004-Mission_Audit_Log.md)**: The per-mission append-only `AuditLog.ndjson` file that records grouped card/link change events.

- **[ART-007 - Aurora Schemas](MIS-001/Artifact/ART-007-Aurora_Schemas.md)**: The Aurora JSON schemas used to validate cards, audit logs, and compact exports.

- **[ART-005 - Canonical Definitions Registry](MIS-001/Artifact/ART-005-Canonical_Definitions_Registry.md)**: The canonical registry JSON (Aurora.modelconfiguration.json) that defines card types, acronyms, allowed outgoing relationships, view definitions, and the available icon list used by tooling.

- **[ART-006 - View Definitions Registry](MIS-001/Artifact/ART-006-View_Definitions_Registry.md)**: The canonical registry JSON that defines views (roots, included card types, and view descriptions).

- **[ART-001 - Validation Report](MIS-001/Artifact/ART-001-Validation_Report.md)**: Diagnostics output describing schema, registry, and invariant validation results for a model home.

- **[ART-010 - Model Archive Zip](MIS-001/Artifact/ART-010-Model_Archive_Zip.md)**: A ZIP-compressed archive of a model home used for backups and for pack/unpack workflows, preserving the model home folder structure.

- **[ART-009 - Icons Reference Sheet](MIS-001/Artifact/ART-009-Icons_Reference_Sheet.md)**: The Icons.svg reference output produced from source icons, containing normalized icon defs and a proof-sheet layout used for verification and template integration.

### Capability

- **[CAP-002 - Generate Views](MIS-001/Capability/CAP-002-Generate_Views.md)**: Generate view artifacts from the model by selecting roots and included card types, traversing reachable subgraphs, and rendering diagrams without changing the underlying model.

- **[CAP-007 - Edit Models Interactively](MIS-001/Capability/CAP-007-Edit_Models_Interactively.md)**: Interactively navigate and edit Aurora models with guardrails that prevent invalid edits and preserve invariants.

- **[CAP-001 - Validate Aurora Models](MIS-001/Capability/CAP-001-Validate_Aurora_Models.md)**: Validate that an Aurora model conforms to schemas, canonical registries, and graph invariants (reachability, root direction, and no-orphan rules).

- **[CAP-008 - Persist And Package Models](MIS-001/Capability/CAP-008-Persist_And_Package_Models.md)**: Persist edits safely (autosave or manual save), support undo/redo, and pack/unpack model homes as ZIP archives.

- **[CAP-010 - Editor Observability](MIS-001/Capability/CAP-010-Editor_Observability.md)**: Emit logs suitable for debugging and operations, supporting multiple sinks (stdout/stderr/file) and consistent formatting.

- **[CAP-005 - Maintain Canonical Registries](MIS-001/Capability/CAP-005-Maintain_Canonical_Registries.md)**: Maintain the canonical registries (card definitions and view definitions) that govern model validation and view generation.

- **[CAP-009 - Accessible User Experience](MIS-001/Capability/CAP-009-Accessible_User_Experience.md)**: Provide an accessible and usable editor UI, including dark mode by default and adjustable typography.

- **[CAP-006 - Load Model Homes](MIS-001/Capability/CAP-006-Load_Model_Homes.md)**: Load and validate model homes using the schemas and references bundled with that model home, while supporting very large models without blocking the UI.

- **[CAP-004 - Maintain Audit Trail](MIS-001/Capability/CAP-004-Maintain_Audit_Trail.md)**: Maintain an append-only audit log per mission (`AuditLog.ndjson`) with grouped card/link changes.

- **[CAP-003 - Export Compact Model](MIS-001/Capability/CAP-003-Export_Compact_Model.md)**: Export an agent-friendly compact model representation containing the cards and their directed links in a single JSON document.

### Component

- **[COM-005 - Aurora Editor Binary](MIS-001/Component/COM-005-Aurora_Editor_Binary.md)**: The aurora_editor desktop executable providing an interactive UI and background engine for safe, validated model authoring.

- **[COM-007 - Aurora Editor Engine](MIS-001/Component/COM-007-Aurora_Editor_Engine.md)**: Background engine responsible for loading model homes, validating/linting, rendering views, packaging/unpackaging, and persistence behaviors without blocking the UI.

- **[COM-003 - svg_prep Binary](MIS-001/Component/COM-003-svgprep_Binary.md)**: The svg_prep executable used to generate/update Aurora SVG reference assets (Icons.svg and SVGTemplate.svg defs) and synchronize the icon list in Aurora.modelconfiguration.json.

- **[COM-001 - Aurora Shared Library](MIS-001/Component/COM-001-Aurora_Shared_Library.md)**: Shared Rust library providing registry-aware parsing, validation helpers, and rendering primitives used by Aurora tools (CLI and Editor).

- **[COM-002 - Aurora CLI Binary](MIS-001/Component/COM-002-Aurora_CLI_Binary.md)**: The aurora_cli executable that exposes validate/render/compact commands to users and pipelines.

- **[COM-006 - Aurora Editor UI](MIS-001/Component/COM-006-Aurora_Editor_UI.md)**: Dioxus-based desktop UI responsible for interactive navigation (mind-map-like centered views), editing surfaces, and accessibility/theming preferences.

### Constraint

- **[CNS-001 - Mission Has Outgoing Links Only](MIS-001/Constraint/CNS-001-Mission_Has_Outgoing_Links_Only.md)**: The Mission card serves as the root of the model graph and must only have outgoing links.

- **[CNS-002 - No Orphan Cards](MIS-001/Constraint/CNS-002-No_Orphan_Cards.md)**: Every non-Mission card must have one or more incoming links and be reachable from the Mission card.

- **[CNS-004 - Rust 2024](MIS-001/Constraint/CNS-004-Rust_2024.md)**: The standalone editor MUST be implemented in Rust 2024.

- **[CNS-003 - Standard Model File Layout](MIS-001/Constraint/CNS-003-Standard_Model_File_Layout.md)**: Models must use the standard Aurora folder layout: Mission card at model home, mission-scoped cards under `<MISSION_ID>/<Card Type>/`, and an append-only audit log at `<MISSION_ID>/AuditLog.ndjson`.

- **[CNS-005 - Dioxus UI](MIS-001/Constraint/CNS-005-Dioxus_UI.md)**: The standalone editor UI MUST be implemented using Dioxus.

- **[CNS-006 - Fern Logging](MIS-001/Constraint/CNS-006-Fern_Logging.md)**: The standalone editor MUST support logging with fern integration.

### Driver

- **[DRI-002 - Canonical Vocabulary](MIS-001/Driver/DRI-002-Canonical_Vocabulary.md)**: Provide a single, normative registry for card types and allowed relationship targets/verbs so models can be validated and interpreted consistently across tools.

- **[DRI-001 - Deterministic Interpretation](MIS-001/Driver/DRI-001-Deterministic_Interpretation.md)**: Eliminate ambiguous architectural meaning by enforcing invariant rules and schema-backed cards so that any interpretation (views, traceability, impact analysis) is reproducible.

- **[DRI-003 - Automated View Generation](MIS-001/Driver/DRI-003-Automated_View_Generation.md)**: Enable tools to generate consistent diagrams and documentation from the model without manually drawing or maintaining multiple sources of truth.

- **[DRI-004 - Interactive Safe Authoring](MIS-001/Driver/DRI-004-Interactive_Safe_Authoring.md)**: Enable a fast, responsive, and safe desktop authoring experience so humans can edit models without breaking invariants, while tools and agents can still trust the model as deterministic and valid.

### Feature

- **[FEA-002 - Render Views](MIS-001/Feature/FEA-002-Render_Views.md)**: Generate view diagrams and related artifacts from an Aurora model based on the view definitions registry.

- **[FEA-008 - Interactive Model Editing](MIS-001/Feature/FEA-008-Interactive_Model_Editing.md)**: Provide interactive model authoring with immediate feedback and enforcement so invalid edits are prevented before they can persist to disk.

- **[FEA-006 - Prepare SVG References](MIS-001/Feature/FEA-006-Prepare_SVG_References.md)**: Generate and update Aurora SVG reference assets (Icons.svg and SVGTemplate.svg defs) and keep the canonical icon availability list synchronized.

- **[FEA-007 - Load And Validate Model Homes](MIS-001/Feature/FEA-007-Load_And_Validate_Model_Homes.md)**: Load a model home quickly (including large models), using the schemas and references packaged with that model home, and validate in a way that keeps the UI responsive.

- **[FEA-001 - Validate Model](MIS-001/Feature/FEA-001-Validate_Model.md)**: Validate an Aurora model against schemas, registries, and graph invariants, producing actionable diagnostics.

- **[FEA-010 - Accessible Themed UI](MIS-001/Feature/FEA-010-Accessible_Themed_UI.md)**: Provide WCAG AA accessible UI behavior, dark mode by default, and adjustable base font sizing with proportional scaling.

- **[FEA-009 - Model Persistence And Recovery](MIS-001/Feature/FEA-009-Model_Persistence_And_Recovery.md)**: Persist changes safely (autosave or manual save), provide undo/redo, and support packing/unpacking model homes as ZIP archives.

- **[FEA-011 - Editor Logging](MIS-001/Feature/FEA-011-Editor_Logging.md)**: Emit structured logs for troubleshooting and diagnostics, with fern integration supporting stdout, stderr, and optional file output.

- **[FEA-003 - Export Compact Model](MIS-001/Feature/FEA-003-Export_Compact_Model.md)**: Export an Aurora compact model representation suitable for agent consumption and transport.

### Process

- **[PRO-008 - Persist And Package Model Home](MIS-001/Process/PRO-008-Persist_And_Package_Model_Home.md)**: Persist edits to disk safely (autosave or manual save), support undo/redo, and pack/unpack model homes as ZIP archives.

- **[PRO-006 - Load Model Home In Editor](MIS-001/Process/PRO-006-Load_Model_Home_In_Editor.md)**: Load a model home for interactive use by locating the model home schemas/references, creating a backup archive, and validating the model.

- **[PRO-001 - Validate Model](MIS-001/Process/PRO-001-Validate_Model.md)**: Validate a model home by loading cards, applying schema checks, validating canonical relationship constraints, and enforcing reachability and root-direction invariants.

- **[PRO-005 - Maintain Canonical Registries](MIS-001/Process/PRO-005-Maintain_Canonical_Registries.md)**: Maintain the canonical registries and schemas that define the Aurora vocabulary and view semantics.

- **[PRO-010 - Configure Editor Logging](MIS-001/Process/PRO-010-Configure_Editor_Logging.md)**: Configure and emit logs from the editor to stdout/stderr and optionally to a file sink.

- **[PRO-007 - Edit Model In Editor](MIS-001/Process/PRO-007-Edit_Model_In_Editor.md)**: Edit a model via the UI while enforcing validity, recording audit entries, and keeping the editor responsive.

- **[PRO-009 - Apply Editor Preferences](MIS-001/Process/PRO-009-Apply_Editor_Preferences.md)**: Apply user preferences related to accessibility and appearance (dark mode, font sizing) for the editor UI.

- **[PRO-002 - Render Views](MIS-001/Process/PRO-002-Render_Views.md)**: Render view artifacts by selecting roots, traversing reachable subgraphs, and generating diagrams and related assets from the model.

- **[PRO-003 - Export Compact Model](MIS-001/Process/PRO-003-Export_Compact_Model.md)**: Export a compact representation of the model cards and links into a single JSON file.

- **[PRO-004 - Record Audit Log Entries](MIS-001/Process/PRO-004-Record_Audit_Log_Entries.md)**: Record mission change events by appending one entry per event to `AuditLog.ndjson`; each entry may include multiple changed cards and link changes.

### Requirement

- **[REQ-017 - Prevent Invalid Edits](MIS-001/Requirement/REQ-017-Prevent_Invalid_Edits.md)**: The editor MUST prevent edits that would break a model (schema, registry constraints, invariants) and SHOULD provide style checking and linting.

- **[REQ-019 - Undo Redo Depth](MIS-001/Requirement/REQ-019-Undo_Redo_Depth.md)**: The editor SHOULD support undo/redo spanning 50 edits deep.

- **[REQ-018 - Autosave Default Manual Optional](MIS-001/Requirement/REQ-018-Autosave_Default_Manual_Optional.md)**: The editor MUST use immediate autosave by default and MUST provide an option for manual save mode.

- **[REQ-005 - Standard Model Home Layout](MIS-001/Requirement/REQ-005-Standard_Model_Home_Layout.md)**: Model homes MUST contain the shared Aurora schemas, and missions MUST follow the standard folder layout for cards and audit logs.

- **[REQ-006 - Audit Log Semantics](MIS-001/Requirement/REQ-006-Audit_Log_Semantics.md)**: Each mission MUST have an append-only `AuditLog.ndjson` audit log where each line records one change event with timestamp, editor, and a list of changed cards (including link changes when applicable).

- **[REQ-010 - Rendering Semantics](MIS-001/Requirement/REQ-010-Rendering_Semantics.md)**: Rendering MUST treat canonical registry style fields (shape/icon/fill/color) as non-normative hints; model validity MUST depend on normative fields (types, ids, relationships), not styling.

- **[REQ-014 - Backup Zip On Load](MIS-001/Requirement/REQ-014-Backup_Zip_On_Load.md)**: At load time, the editor MUST create a backup ZIP of the model home, stored per conventions defined in Aurora_Specs.

- **[REQ-015 - UI Engine Thread Separation](MIS-001/Requirement/REQ-015-UI_Engine_Thread_Separation.md)**: The editor UI and engine MUST operate on separate threads to ensure UI responsiveness and effective use of modern hardware.

- **[REQ-013 - Fast Streamed Loading](MIS-001/Requirement/REQ-013-Fast_Streamed_Loading.md)**: Models of any size SHOULD load almost instantly; the load method MUST traverse and validate in the time it takes to read files, and the editor MUST NOT load the entire model into memory at once.

- **[REQ-023 - Accessibility And Dark Mode](MIS-001/Requirement/REQ-023-Accessibility_And_Dark_Mode.md)**: The editor MUST be WCAG AA compliant, MUST support dark mode (default), and MUST support adjustable base font size with proportional scaling.

- **[REQ-016 - Interactive View Rendering](MIS-001/Requirement/REQ-016-Interactive_View_Rendering.md)**: The editor MUST be able to render any view defined in the model configuration file and MUST support ad-hoc views.

- **[REQ-002 - Directed Graph Invariants](MIS-001/Requirement/REQ-002-Directed_Graph_Invariants.md)**: Starting from the Mission, all links MUST traverse away from the Mission; every card must be reachable; and traversal must terminate in a leaf or a previously seen card (local loop).

- **[REQ-025 - Single Instance Model Semantics](MIS-001/Requirement/REQ-025-Single_Instance_Model_Semantics.md)**: Multiple instances on the same model are not supported; if concurrent edits occur, last write wins. Shared/networked models are not officially supported.

- **[REQ-003 - Canonical Definitions Registry](MIS-001/Requirement/REQ-003-Canonical_Definitions_Registry.md)**: A canonical registry MUST define the available card types, their id acronyms, and the allowed outgoing relationship targets and verbs for each card type.

- **[REQ-001 - Single Mission Root](MIS-001/Requirement/REQ-001-Single_Mission_Root.md)**: A model MUST include exactly one Mission card as the root intent, with only outgoing links, summarizing the high-level why of the model.

- **[REQ-022 - Editor Logging With Fern](MIS-001/Requirement/REQ-022-Editor_Logging_With_Fern.md)**: Logging MUST be supported with fern integration, including stdout, stderr, and optional file-based logging.

- **[REQ-024 - Major Version Compatibility](MIS-001/Requirement/REQ-024-Major_Version_Compatibility.md)**: Each model home MUST include a complete set of schema and configuration files snapshotted at model creation time; incompatible models MUST be detected via schema validation.

- **[REQ-021 - Pack And Unpack Model Zips](MIS-001/Requirement/REQ-021-Pack_And_Unpack_Model_Zips.md)**: The editor MUST support packing and unpacking model homes as a single ZIP-compressed file.

- **[REQ-008 - Compact Model Format](MIS-001/Requirement/REQ-008-Compact_Model_Format.md)**: A compact, single-file representation MUST be supported for transport and agent consumption, using the compact schema and retaining card ids, types, fields, and links.

- **[REQ-011 - Standalone Cross Platform Editor](MIS-001/Requirement/REQ-011-Standalone_Cross_Platform_Editor.md)**: The Aurora Editor MUST run on the major desktop platforms (Linux, Microsoft Windows, and Apple macOS).

- **[REQ-007 - View Definitions And Root Safety](MIS-001/Requirement/REQ-007-View_Definitions_And_Root_Safety.md)**: View definitions MUST specify root card types and included card types; view roots MUST exclude cards participating in cycles (root safety rule).

- **[REQ-004 - Schema-Backed Card Format](MIS-001/Requirement/REQ-004-SchemaBacked_Card_Format.md)**: Cards MUST conform to the Aurora card schema (required fields, identifier format, and link structure) and be stored as pretty-printed JSON.

- **[REQ-012 - Model Home Local Schemas And References](MIS-001/Requirement/REQ-012-Model_Home_Local_Schemas_And_References.md)**: The editor MUST load and use the schemas and reference files included with the selected model home, allowing the editor to operate across multiple Aurora versions and customizations.

- **[REQ-009 - Default Tooling Support](MIS-001/Requirement/REQ-009-Default_Tooling_Support.md)**: Default tooling SHOULD validate models, generate human-readable outputs, generate views, and export compact models from the source cards.

- **[REQ-020 - Crash Safety Semantics](MIS-001/Requirement/REQ-020-Crash_Safety_Semantics.md)**: With autosave enabled, on crash the model may at worst contain an orphan card that needs to be linked; with autosave disabled, the saved model MUST always be valid and unsaved changes are lost on crash.

### Stakeholder

- **[STK-001 - Architects And Agents](MIS-001/Stakeholder/STK-001-Architects_And_Agents.md)**: Humans and automation (LLMs, agents, validation/render tooling) that need a shared, unambiguous representation of architecture and requirements.

### Story

- **[STR-001 - Deterministic Modeling Experience](MIS-001/Story/STR-001-Deterministic_Modeling_Experience.md)**: As an architect, I want to express architecture and requirements as a validated, deterministic model so that tools and agents can reason about it, generate consistent views, and avoid ambiguity.

### System

- **[SYS-001 - Aurora Tooling Ecosystem](MIS-001/System/SYS-001-Aurora_Tooling_Ecosystem.md)**: The overall Aurora system: schemas + canonical registries + model cards, supported by tools (CLI + svg_prep + editor + shared library) that validate models, generate views/exports, and maintain reference SVG assets.

