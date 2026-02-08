# MIS-001: Model Aurora With Aurora

**[Mission Card](MIS-001-Model_Aurora_With_Aurora.md)**

Define Aurora (Agent-Unified Representation of Requirements and Architecture) using Aurora itself: a deterministic, machine-consumable architecture model rooted at a single Mission card, backed by schemas and canonical registries, and usable by tools and agents.

## Views

_No views available._

## Card Index

### ADR

### Activity

- **[ATV-005 - Edit Canonical Definitions Registry](MIS-001/Activity/ATV-005-Edit_Canonical_Definitions_Registry.md)**: Maintain the canonical registry of card types and allowed outgoing relationships.

- **[ATV-001 - Validate Model Home](MIS-001/Activity/ATV-001-Validate_Model_Home.md)**: Load cards from the model home and validate them against schemas, canonical registries, and graph invariants.

- **[ATV-006 - Edit View Definitions Registry](MIS-001/Activity/ATV-006-Edit_View_Definitions_Registry.md)**: Maintain the canonical registry of view definitions (roots, included types, and view intent).

- **[ATV-004 - Append Audit Log Entry](MIS-001/Activity/ATV-004-Append_Audit_Log_Entry.md)**: Append a create/change/delete entry to the mission audit log.

- **[ATV-002 - Render View Artifacts](MIS-001/Activity/ATV-002-Render_View_Artifacts.md)**: Select roots/included card types per view definitions, traverse the subgraph, and render the view into diagram artifacts.

- **[ATV-003 - Write Compact Export](MIS-001/Activity/ATV-003-Write_Compact_Export.md)**: Write the compact export JSON document for transport and agent consumption.

- **[ATV-007 - Maintain Aurora Schemas](MIS-001/Activity/ATV-007-Maintain_Aurora_Schemas.md)**: Maintain the Aurora JSON schemas used for cards, audit logs, and compact export.

### Actor

### Adversary

### Application

- **[APP-001 - Aurora CLI](MIS-001/Application/APP-001-Aurora_CLI.md)**: Command-line tooling for validating Aurora models, generating views, and exporting compact representations.

- **[APP-002 - Aurora Editor](MIS-001/Application/APP-002-Aurora_Editor.md)**: Interactive editor application for browsing, validating, rendering, and editing Aurora models with trust gating.

### Artifact

- **[ART-002 - View Diagram Artifacts](MIS-001/Artifact/ART-002-View_Diagram_Artifacts.md)**: Rendered view artifacts (for example SVG diagrams and DOT sources) generated from a model and view definitions.

- **[ART-003 - Compact Export](MIS-001/Artifact/ART-003-Compact_Export.md)**: The compact JSON export of the model cards and links for transport and agent consumption.

- **[ART-004 - Mission Audit Log](MIS-001/Artifact/ART-004-Mission_Audit_Log.md)**: The per-mission audit log JSON file that records create/change/delete events over time.

- **[ART-007 - Aurora Schemas](MIS-001/Artifact/ART-007-Aurora_Schemas.md)**: The Aurora JSON schemas used to validate cards, audit logs, and compact exports.

- **[ART-005 - Canonical Definitions Registry](MIS-001/Artifact/ART-005-Canonical_Definitions_Registry.md)**: The canonical registry JSON that defines card types, acronyms, and allowed outgoing relationships.

- **[ART-006 - View Definitions Registry](MIS-001/Artifact/ART-006-View_Definitions_Registry.md)**: The canonical registry JSON that defines views (roots, included card types, and view descriptions).

- **[ART-001 - Validation Report](MIS-001/Artifact/ART-001-Validation_Report.md)**: Diagnostics output describing schema, registry, and invariant validation results for a model home.

### Asset

### Capability

- **[CAP-002 - Generate Views](MIS-001/Capability/CAP-002-Generate_Views.md)**: Generate view artifacts from the model by selecting roots and included card types, traversing reachable subgraphs, and rendering diagrams without changing the underlying model.

- **[CAP-001 - Validate Aurora Models](MIS-001/Capability/CAP-001-Validate_Aurora_Models.md)**: Validate that an Aurora model conforms to schemas, canonical registries, and graph invariants (reachability, root direction, and no-orphan rules).

- **[CAP-005 - Maintain Canonical Registries](MIS-001/Capability/CAP-005-Maintain_Canonical_Registries.md)**: Maintain the canonical registries (card definitions and view definitions) that govern model validation and view generation.

- **[CAP-004 - Maintain Audit Trail](MIS-001/Capability/CAP-004-Maintain_Audit_Trail.md)**: Maintain an audit log per mission that records deterministic create/change/delete events for cards.

- **[CAP-003 - Export Compact Model](MIS-001/Capability/CAP-003-Export_Compact_Model.md)**: Export an agent-friendly compact model representation containing the cards and their directed links in a single JSON document.

### Component

- **[COM-004 - Editor Frontend UI](MIS-001/Component/COM-004-Editor_Frontend_UI.md)**: The editor’s frontend UI that calls the backend API, displays validation diagnostics, and provides interactive navigation and editing experiences.

- **[COM-001 - Aurora Shared Library](MIS-001/Component/COM-001-Aurora_Shared_Library.md)**: Shared Rust library providing registry-aware parsing, validation helpers, and rendering primitives used by Aurora tools.

- **[COM-003 - Tauri Rust Backend](MIS-001/Component/COM-003-Tauri_Rust_Backend.md)**: A trust-gated Rust backend exposing model discovery, snapshot loading, validation, rendering, exports, and card CRUD operations to the editor UI.

- **[COM-002 - Aurora CLI Binary](MIS-001/Component/COM-002-Aurora_CLI_Binary.md)**: The aurora_cli executable that exposes validate/render/compact commands to users and pipelines.

### Condition

### Constraint

- **[CNS-001 - Mission Has Outgoing Links Only](MIS-001/Constraint/CNS-001-Mission_Has_Outgoing_Links_Only.md)**: The Mission card serves as the root of the model graph and must only have outgoing links.

- **[CNS-002 - No Orphan Cards](MIS-001/Constraint/CNS-002-No_Orphan_Cards.md)**: Every non-Mission card must have one or more incoming links and be reachable from the Mission card.

- **[CNS-003 - Standard Model File Layout](MIS-001/Constraint/CNS-003-Standard_Model_File_Layout.md)**: Models must use the standard Aurora folder layout: Mission card at model home, mission-scoped cards under `<MISSION_ID>/<Card Type>/`, and an audit log at `<MISSION_ID>/AuditLog.json`.

### Control

### Data Source

### Data Store

### Deployment

### Driver

- **[DRI-002 - Canonical Vocabulary](MIS-001/Driver/DRI-002-Canonical_Vocabulary.md)**: Provide a single, normative registry for card types and allowed relationship targets/verbs so models can be validated and interpreted consistently across tools.

- **[DRI-001 - Deterministic Interpretation](MIS-001/Driver/DRI-001-Deterministic_Interpretation.md)**: Eliminate ambiguous architectural meaning by enforcing invariant rules and schema-backed cards so that any interpretation (views, traceability, impact analysis) is reproducible.

- **[DRI-003 - Automated View Generation](MIS-001/Driver/DRI-003-Automated_View_Generation.md)**: Enable tools to generate consistent diagrams and documentation from the model without manually drawing or maintaining multiple sources of truth.

### Event

### Feature

- **[FEA-002 - Render Views](MIS-001/Feature/FEA-002-Render_Views.md)**: Generate view diagrams and related artifacts from an Aurora model based on the view definitions registry.

- **[FEA-005 - Interactive Model Editing](MIS-001/Feature/FEA-005-Interactive_Model_Editing.md)**: Browse and edit Aurora cards interactively while preserving schema validity and model invariants.

- **[FEA-001 - Validate Model](MIS-001/Feature/FEA-001-Validate_Model.md)**: Validate an Aurora model against schemas, registries, and graph invariants, producing actionable diagnostics.

- **[FEA-004 - Editor Model Operations](MIS-001/Feature/FEA-004-Editor_Model_Operations.md)**: Provide model operations (discover/load/validate/render/export and CRUD) to the editor UI with trust gating.

- **[FEA-003 - Export Compact Model](MIS-001/Feature/FEA-003-Export_Compact_Model.md)**: Export an Aurora compact model representation suitable for agent consumption and transport.

### Interface

- **[INT-001 - Editor Backend API](MIS-001/Interface/INT-001-Editor_Backend_API.md)**: The interface contract between the editor UI and the Tauri backend for model operations (discover/load/validate/render/export and card CRUD).

### Mission

### Node

### Predicate

### Process

- **[PRO-001 - Validate Model](MIS-001/Process/PRO-001-Validate_Model.md)**: Validate a model home by loading cards, applying schema checks, validating canonical relationship constraints, and enforcing reachability and root-direction invariants.

- **[PRO-005 - Maintain Canonical Registries](MIS-001/Process/PRO-005-Maintain_Canonical_Registries.md)**: Maintain the canonical registries and schemas that define the Aurora vocabulary and view semantics.

- **[PRO-002 - Render Views](MIS-001/Process/PRO-002-Render_Views.md)**: Render view artifacts by selecting roots, traversing reachable subgraphs, and generating diagrams and related assets from the model.

- **[PRO-003 - Export Compact Model](MIS-001/Process/PRO-003-Export_Compact_Model.md)**: Export a compact representation of the model cards and links into a single JSON file.

- **[PRO-004 - Record Audit Log Entries](MIS-001/Process/PRO-004-Record_Audit_Log_Entries.md)**: Record create/change/delete events for mission cards into the mission audit log.

### Requirement

- **[REQ-005 - Standard Model Home Layout](MIS-001/Requirement/REQ-005-Standard_Model_Home_Layout.md)**: Model homes MUST contain the shared Aurora schemas, and missions MUST follow the standard folder layout for cards and audit logs.

- **[REQ-006 - Audit Log Semantics](MIS-001/Requirement/REQ-006-Audit_Log_Semantics.md)**: Each mission MUST have an audit log recording create/change/delete events with timestamp, editor, target id, and change type.

- **[REQ-010 - Rendering Semantics](MIS-001/Requirement/REQ-010-Rendering_Semantics.md)**: Rendering MUST treat canonical registry style fields (shape/icon/fill/color) as non-normative hints; model validity MUST depend on normative fields (types, ids, relationships), not styling.

- **[REQ-002 - Directed Graph Invariants](MIS-001/Requirement/REQ-002-Directed_Graph_Invariants.md)**: Starting from the Mission, all links MUST traverse away from the Mission; every card must be reachable; and traversal must terminate in a leaf or a previously seen card (local loop).

- **[REQ-003 - Canonical Definitions Registry](MIS-001/Requirement/REQ-003-Canonical_Definitions_Registry.md)**: A canonical registry MUST define the available card types, their id acronyms, and the allowed outgoing relationship targets and verbs for each card type.

- **[REQ-001 - Single Mission Root](MIS-001/Requirement/REQ-001-Single_Mission_Root.md)**: A model MUST include exactly one Mission card as the root intent, with only outgoing links, summarizing the high-level why of the model.

- **[REQ-008 - Compact Model Format](MIS-001/Requirement/REQ-008-Compact_Model_Format.md)**: A compact, single-file representation MUST be supported for transport and agent consumption, using the compact schema and retaining card ids, types, fields, and links.

- **[REQ-007 - View Definitions And Root Safety](MIS-001/Requirement/REQ-007-View_Definitions_And_Root_Safety.md)**: View definitions MUST specify root card types and included card types; view roots MUST exclude cards participating in cycles (root safety rule).

- **[REQ-004 - Schema-Backed Card Format](MIS-001/Requirement/REQ-004-SchemaBacked_Card_Format.md)**: Cards MUST conform to the Aurora card schema (required fields, identifier format, and link structure) and be stored as pretty-printed JSON.

- **[REQ-009 - Default Tooling Support](MIS-001/Requirement/REQ-009-Default_Tooling_Support.md)**: Default tooling SHOULD validate models, generate human-readable outputs, generate views, and export compact models from the source cards.

### Resource Owner

### Risk

### Stakeholder

- **[STK-001 - Architects And Agents](MIS-001/Stakeholder/STK-001-Architects_And_Agents.md)**: Humans and automation (LLMs, agents, validation/render tooling) that need a shared, unambiguous representation of architecture and requirements.

### State

### State Machine

### Story

- **[STR-001 - Deterministic Modeling Experience](MIS-001/Story/STR-001-Deterministic_Modeling_Experience.md)**: As an architect, I want to express architecture and requirements as a validated, deterministic model so that tools and agents can reason about it, generate consistent views, and avoid ambiguity.

### System

- **[SYS-001 - Aurora Tooling Ecosystem](MIS-001/System/SYS-001-Aurora_Tooling_Ecosystem.md)**: The overall Aurora system: schemas + canonical registries + model cards, supported by tools (CLI/editor/shared library) that validate models and generate views and exports.

### Test

### Threat Capability

### Threat Diamond

### Threat Model

### Trigger

### Victim
