# MIS-004: Standalone Editor

**[Mission Card](MIS-004-Standalone Editor.md)**

Provide a standalone application capable of creating and manipulating Aurora models visually, reusing the aurora_shared library to facilitate development.

## Views
![Traceability.view.svg](MIS-004-views/Traceability.view.svg)

![Process.view.svg](MIS-004-views/Process.view.svg)

![Component.view.svg](MIS-004-views/Component.view.svg)

![Compliance_Governance.view.svg](MIS-004-views/Compliance_Governance.view.svg)

![Requirements.view.svg](MIS-004-views/Requirements.view.svg)

![Entire_Model.view.svg](MIS-004-views/Entire_Model.view.svg)

![Use_Case.view.svg](MIS-004-views/Use_Case.view.svg)
## Card Index

### ADR

### Activity

### Actor

- **[ACT-004 - Model Author](MIS-004/Actor/ACT-004.md)**: Human operator responsible for navigating, editing, and validating Aurora models within the standalone editor.

### Application

- **[APP-004 - aurora_editor](MIS-004/Application/APP-004.md)**: Standalone desktop application (built with Dioxus) for browsing and editing Aurora models, including live validation and Markdown-style previews.

### Artifact

- **[ART-005 - Aurora Model Source Files](MIS-004/Artifact/ART-005.md)**: Aurora model source-of-truth files stored as JSJSON: a mission card plus per-card files under <MISSION_ID>/<Card Type>/.

### Asset

### Boundary

- **[BND-003 - Local Workstation](MIS-004/Boundary/BND-003.md)**: Trust and storage boundary for local, standalone editor execution (application runtime + workspace file system access).

### Capability

- **[CAP-013 - Markdown Preview and Audit View](MIS-004/Capability/CAP-013.md)**: Ability to render the focused card as a readable Markdown-style document with link list, and to inspect audit trail history.

- **[CAP-011 - Context Graph Visualization](MIS-004/Capability/CAP-011.md)**: Ability to visualize a local subgraph around the focused card (including parent, siblings, and children) and navigate by clicking nodes.

- **[CAP-009 - Desktop UI Shell](MIS-004/Capability/CAP-009.md)**: Ability to present a desktop application UI with a stable three-pane layout and shared selection state.

- **[CAP-014 - aurora_shared Integration](MIS-004/Capability/CAP-014.md)**: Ability to reuse aurora_shared for model discovery/loading, validation, rendering, and file layout conventions to keep editor behavior consistent with CLI tooling.

- **[CAP-010 - Navigation and Focus Management](MIS-004/Capability/CAP-010.md)**: Ability to navigate the model by card id and keep the focused card synchronized across panes.

- **[CAP-012 - Interactive Card Editing](MIS-004/Capability/CAP-012.md)**: Ability to edit card fields and links interactively while maintaining model invariants and persisting changes safely.

### Class

### Component

- **[COM-009 - Brain Graph Pane](MIS-004/Component/COM-009.md)**: Central pane that renders a mind-map / TheBrain-style context graph with the focused card centered and related cards positioned by relationship role.

- **[COM-008 - Explorer Tree Pane](MIS-004/Component/COM-008.md)**: Left navigation pane that presents a tree view derived from shortest paths from the mission root, enabling fast card discovery and focus changes.

- **[COM-010 - Inspector Pane](MIS-004/Component/COM-010.md)**: Right pane that splits vertically: top half edits the focused card; bottom half shows a rendered Markdown-style preview (and links) with a tab to reveal audit details.

- **[COM-011 - Model Session and Persistence](MIS-004/Component/COM-011.md)**: In-memory model session that loads a workspace model home, computes shortest paths, validates changes, and persists updated card files using aurora_shared semantics.

- **[COM-007 - Dioxus App Shell](MIS-004/Component/COM-007.md)**: Top-level Dioxus component that owns the UI shell layout and routes focus/selection changes between panes.

### Condition

### Constraint

### Control

### Data Store

- **[DTS-003 - Workspace File System](MIS-004/Data Store/DTS-003.md)**: Filesystem-backed store holding Aurora model source files (Mission card plus per-card JSJSON files) within a user-selected workspace.

### Deployment

### Driver

- **[DRI-005 - Reuse Shared Aurora Semantics](MIS-004/Driver/DRI-005.md)**: Minimize duplicated logic and inconsistencies across tooling surfaces by reusing the aurora_shared library for model discovery, parsing, validation, and rendering.

- **[DRI-004 - Visual Model Authoring](MIS-004/Driver/DRI-004.md)**: Enable rapid comprehension and editing of an Aurora model through a visual, multi-pane editor that reduces cognitive load and navigation friction.

### Event

### Feature

- **[FEA-011 - Compute Shortest Path Tree](MIS-004/Feature/FEA-011.md)**: Compute a stable shortest-path parent for each card from the mission root and present it as a tree for navigation (one chosen path per node).

- **[FEA-013 - Card Inspector Editing](MIS-004/Feature/FEA-013.md)**: Edit the focused card’s core fields (name, description, type/subtype, status, attributes, and links) with immediate validation feedback.

- **[FEA-014 - Preview and Audit Panel](MIS-004/Feature/FEA-014.md)**: Render a Markdown-style card preview (including outgoing link list) and provide a tab to display audit trail details for the focused card.

- **[FEA-010 - Three Pane Shell](MIS-004/Feature/FEA-010.md)**: Render the editor UI as three primary panes: left navigation tree, central context graph, and right inspector split into edit + preview/audit.

- **[FEA-015 - Load, Validate, and Save Models](MIS-004/Feature/FEA-015.md)**: Load Aurora models from a chosen workspace, validate invariants on every change, and persist edits back to JSJSON card files using shared aurora_shared semantics.

- **[FEA-012 - Brain Style Graph View](MIS-004/Feature/FEA-012.md)**: Render a TheBrain-like local context view: focused card centered; show parent from the chosen shortest-path chain; show siblings (same card_type) as adjacent nodes linked from the parent; show focused card’s outgoing links as navigable children.

### Interface

- **[INT-004 - aurora_shared Domain API](MIS-004/Interface/INT-004.md)**: Public domain interface (Rust API) for discovering model homes, loading cards, validating invariants, and rendering Markdown-style representations.

- **[INT-003 - Dioxus UI Framework API](MIS-004/Interface/INT-003.md)**: UI framework interface used to declare layouts, render widgets, and process user events for the standalone editor.

- **[INT-005 - Local Filesystem API](MIS-004/Interface/INT-005.md)**: Contract for reading and writing Aurora model files on the local filesystem (open, read, write, atomic replace, and directory enumeration).

### Mission

### Node

### Node Instance

### Note

### Predicate

### Process

### Requirement

- **[REQ-009 - Three Pane Layout](MIS-004/Requirement/REQ-009.md)**: The editor SHALL present a three-pane layout (similar to Visual Studio or Obsidian): left navigation tree, central graph view, and right inspector.

- **[REQ-014 - Reuse aurora_shared](MIS-004/Requirement/REQ-014.md)**: The editor SHALL reuse aurora_shared for model discovery/loading and validation/rendering semantics to stay consistent with other Aurora tooling.

- **[REQ-015 - Safe Workspace Persistence](MIS-004/Requirement/REQ-015.md)**: The editor SHALL persist edits to model files in a safe manner that avoids partial writes and preserves schema validity (e.g., atomic replace per card file).

- **[REQ-012 - Edit, Preview, and Audit Inspector](MIS-004/Requirement/REQ-012.md)**: The editor SHALL provide a right-pane inspector: the top half edits the focused card; the bottom half renders a Markdown-style view of the card (including links). The bottom half SHALL provide a tab to reveal the card’s audit trail details.

- **[REQ-013 - Dioxus UI](MIS-004/Requirement/REQ-013.md)**: The standalone editor SHOULD be implemented using Dioxus as the primary UI library.

- **[REQ-010 - Shortest Path Tree Navigation](MIS-004/Requirement/REQ-010.md)**: The editor SHALL provide a left-pane tree view for navigation, where the tree is derived from a chosen shortest path from the mission root to each card.

- **[REQ-011 - Brain Style Graph Navigation](MIS-004/Requirement/REQ-011.md)**: The editor SHALL provide a central mind-map / TheBrain style view with the focused card centered. The view SHALL show the parent (from the chosen shortest-path chain), and MAY show siblings (same card_type) linked from that parent even if they are not linked to the focused card.

### Risk

### State

### State Machine

### Story

### System

- **[SYS-003 - Aurora Standalone Editor System](MIS-004/System/SYS-003.md)**: Bounded system that provides a desktop UI for exploring and editing Aurora models with a three-pane workflow (navigation, graph context, inspector).

### Test

### Threat

### Trigger
