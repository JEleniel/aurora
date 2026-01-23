# MIS-002: Enable Aurora Viewer And Editor

**Mission**: Provide a cross-platform UI tool for viewing and editing Aurora models that enforces the Aurora schema and invariants, supports deterministic and conflict-safe edits, and stays consistent with the canonical aurora_cli outputs.

## Views

- [Component](MIS-002/Component.view.md)
- [Deployment](MIS-002/Deployment.view.md)
- [State Machine](MIS-002/State_Machine.view.md)
- [Process](MIS-002/Process.view.md)
- [Threat Model](MIS-002/Threat_Model.view.md)
- [Requirements](MIS-002/Requirements.view.md)


## Cards

## State

- [STA-001: Idle](MIS-002/STA/STA-001.md)
## Requirement

- [REQ-009: View Filters Match aurora_cli](MIS-002/REQ/REQ-009.md)
## Threat

- [THR-003: Resource Exhaustion](MIS-002/THR/THR-003.md)
## Component

- [COM-003: Tree View Controller](MIS-002/COM/COM-003.md)
- [COM-012: Audit And Hash Manager](MIS-002/COM/COM-012.md)
## Activity

- [ATV-011: Import Model Home](MIS-002/ATV/ATV-011.md)
## Data Store

- [DTS-001: Aurora Model Home Folder (Import/Export)](MIS-002/DTS/DTS-001.md)
## Event

- [EVT-006: Validation Failed](MIS-002/EVT/EVT-006.md)
## Requirement

- [REQ-007: Graph View Displays Local Topology](MIS-002/REQ/REQ-007.md)
## Driver

- [DRI-003: Canonical Output Parity](MIS-002/DRI/DRI-003.md)
## Threat

- [THR-001: Path Traversal And Unsafe File Writes](MIS-002/THR/THR-001.md)
## Requirement

- [REQ-019: MCP CRUD And Change Events](MIS-002/REQ/REQ-019.md)
- [REQ-035: Import Model Home Into Local Store](MIS-002/REQ/REQ-035.md)
## Component

- [COM-020: VS Code Host Adapter](MIS-002/COM/COM-020.md)
## Story

- [STR-002: Export Official Views To SVG](MIS-002/STR/STR-002.md)
## Requirement

- [REQ-013: Keep Derived Outputs In Sync](MIS-002/REQ/REQ-013.md)
## Component

- [COM-016: SVG Exporter](MIS-002/COM/COM-016.md)
## Control

- [CTL-002: Path And Filename Sanitization](MIS-002/CTL/CTL-002.md)
## Event

- [EVT-007: Output Generation Or Export Requested](MIS-002/EVT/EVT-007.md)
## Component

- [COM-004: Graph View Controller](MIS-002/COM/COM-004.md)
## Driver

- [DRI-002: Safe Invariant-Preserving Editing](MIS-002/DRI/DRI-002.md)
## Artifact

- [ART-005: Compact Model Snapshot](MIS-002/ART/ART-005.md)
## Feature

- [FEA-001: Multi-Model Loader](MIS-002/FEA/FEA-001.md)
## Capability

- [CAP-002: Browse Model Tree](MIS-002/CAP/CAP-002.md)
## Event

- [EVT-001: Model Home Selected](MIS-002/EVT/EVT-001.md)
## Feature

- [FEA-008: File Watching And Conflict Resolution](MIS-002/FEA/FEA-008.md)
- [FEA-013: View Browser](MIS-002/FEA/FEA-013.md)
- [FEA-004: View Filter Selector](MIS-002/FEA/FEA-004.md)
## Constraint

- [CNS-003: Treat Model Content As Untrusted](MIS-002/CNS/CNS-003.md)
## Capability

- [CAP-009: Maintain Audit Trail And Hash](MIS-002/CAP/CAP-009.md)
## Activity

- [ATV-005: Validate Edit](MIS-002/ATV/ATV-005.md)
## Process

- [PRO-002: Edit Card Safely](MIS-002/PRO/PRO-002.md)
## Requirement

- [REQ-026: Validate On Load Edit And Save](MIS-002/REQ/REQ-026.md)
## Application

- [APP-002: Aurora VS Code Extension](MIS-002/APP/APP-002.md)
## Component

- [COM-002: Model Graph Indexer](MIS-002/COM/COM-002.md)
## Process

- [PRO-003: Resolve Concurrent Changes](MIS-002/PRO/PRO-003.md)
## Component

- [COM-022: Model Import/Export Manager](MIS-002/COM/COM-022.md)
## Condition

- [CON-002: External Change Conflicts With Local Edit](MIS-002/CON/CON-002.md)
## Risk

- [RIS-002: Unauthorized Filesystem Access](MIS-002/RIS/RIS-002.md)
## Component

- [COM-001: Model Home Scanner](MIS-002/COM/COM-001.md)
- [COM-017: MCP Server](MIS-002/COM/COM-017.md)
## Activity

- [ATV-008: Regenerate Derived Outputs](MIS-002/ATV/ATV-008.md)
## Event

- [EVT-003: Edit Initiated](MIS-002/EVT/EVT-003.md)
## Capability

- [CAP-010: Generate And Browse Derived Outputs](MIS-002/CAP/CAP-010.md)
## Process

- [PRO-005: Import And Export Model Home](MIS-002/PRO/PRO-005.md)
## Requirement

- [REQ-029: Link Creation Is Invariant Safe](MIS-002/REQ/REQ-029.md)
## Feature

- [FEA-007: Autosave And Undo](MIS-002/FEA/FEA-007.md)
## Requirement

- [REQ-036: Export Local Store To Model Home](MIS-002/REQ/REQ-036.md)
## Actor

- [ACT-001: Architect](MIS-002/ACT/ACT-001.md)
## Story

- [STR-003: Agent Updates Model Via MCP](MIS-002/STR/STR-003.md)
## Requirement

- [REQ-025: IDs Are Immutable](MIS-002/REQ/REQ-025.md)
## State

- [STA-006: Generating Outputs](MIS-002/STA/STA-006.md)
## Component

- [COM-018: Security Gate](MIS-002/COM/COM-018.md)
- [COM-007: Card Markdown Renderer](MIS-002/COM/COM-007.md)
## State

- [STA-007: Error](MIS-002/STA/STA-007.md)
## Component

- [COM-009: Filesystem Watcher](MIS-002/COM/COM-009.md)
## State

- [STA-004: Validating](MIS-002/STA/STA-004.md)
## Event

- [EVT-004: Save Requested](MIS-002/EVT/EVT-004.md)
## Condition

- [CON-001: Schema And Invariants Valid](MIS-002/CON/CON-001.md)
## Actor

- [ACT-002: Tool User](MIS-002/ACT/ACT-002.md)
## Interface

- [INT-002: Model Home Layout Contract](MIS-002/INT/INT-002.md)
## Feature

- [FEA-011: Safe Link Editor](MIS-002/FEA/FEA-011.md)
## Activity

- [ATV-010: Serve MCP Request](MIS-002/ATV/ATV-010.md)
## Requirement

- [REQ-003: Model Tabs Remember Context](MIS-002/REQ/REQ-003.md)
## Component

- [COM-021: IndraDB Store Adapter](MIS-002/COM/COM-021.md)
## Feature

- [FEA-019: IndraDB-Backed Model Storage](MIS-002/FEA/FEA-019.md)
## Event

- [EVT-008: MCP Request Received](MIS-002/EVT/EVT-008.md)
## Requirement

- [REQ-028: Compute Hash On Save](MIS-002/REQ/REQ-028.md)
## Application

- [APP-001: Aurora Viewer And Editor](MIS-002/APP/APP-001.md)
## Feature

- [FEA-010: Audit Trail And Hash Updates](MIS-002/FEA/FEA-010.md)
## State

- [STA-003: Editing](MIS-002/STA/STA-003.md)
## Capability

- [CAP-003: Navigate Model Graph](MIS-002/CAP/CAP-003.md)
## Component

- [COM-006: Card Editor Controller](MIS-002/COM/COM-006.md)
## Capability

- [CAP-007: Detect And Resolve Concurrent Changes](MIS-002/CAP/CAP-007.md)
## Feature

- [FEA-015: MCP Server And Change Events](MIS-002/FEA/FEA-015.md)
## Risk

- [RIS-001: Model Integrity Loss](MIS-002/RIS/RIS-001.md)
## Driver

- [DRI-005: Accessible Cross-Platform UX](MIS-002/DRI/DRI-005.md)
## Requirement

- [REQ-020: Keyboard Navigation Works End-To-End](MIS-002/REQ/REQ-020.md)
## Artifact

- [ART-003: Generated View Markdown File](MIS-002/ART/ART-003.md)
## Control

- [CTL-005: Accessibility Preference Enforcement](MIS-002/CTL/CTL-005.md)
## Capability

- [CAP-014: Import And Export Model Homes](MIS-002/CAP/CAP-014.md)
## Data Store

- [DTS-003: Undo History Store](MIS-002/DTS/DTS-003.md)
## Component

- [COM-010: Conflict Resolver](MIS-002/COM/COM-010.md)
## Activity

- [ATV-007: Resolve Concurrent Changes](MIS-002/ATV/ATV-007.md)
## Driver

- [DRI-001: Graph-Centric Model Navigation](MIS-002/DRI/DRI-001.md)
## Capability

- [CAP-005: Edit Cards Safely](MIS-002/CAP/CAP-005.md)
## Requirement

- [REQ-010: Filtering Affects Visibility Only](MIS-002/REQ/REQ-010.md)
## Control

- [CTL-003: Deterministic Conflict Resolution](MIS-002/CTL/CTL-003.md)
## Feature

- [FEA-017: Accessible And Styled UI](MIS-002/FEA/FEA-017.md)
## Requirement

- [REQ-015: Export SVG Of Official Diagrams](MIS-002/REQ/REQ-015.md)
## Component

- [COM-019: Accessibility And Theme Manager](MIS-002/COM/COM-019.md)
## Capability

- [CAP-013: Accessible And Polished UI](MIS-002/CAP/CAP-013.md)
- [CAP-011: Export Official Diagrams To SVG](MIS-002/CAP/CAP-011.md)
## Interface

- [INT-004: Mermaid SVG Rendering Contract](MIS-002/INT/INT-004.md)
## Capability

- [CAP-001: Load And Index Local Store](MIS-002/CAP/CAP-001.md)
## Data Store

- [DTS-004: Local IndraDB Store](MIS-002/DTS/DTS-004.md)
## Component

- [COM-015: Output Browser](MIS-002/COM/COM-015.md)
## Process

- [PRO-004: Generate Outputs And Export SVG](MIS-002/PRO/PRO-004.md)
## Constraint

- [CNS-004: No tmp Directory Inside Model Homes](MIS-002/CNS/CNS-004.md)
## Interface

- [INT-001: MCP API Contract](MIS-002/INT/INT-001.md)
## Activity

- [ATV-002: Build Graph Index](MIS-002/ATV/ATV-002.md)
## Constraint

- [CNS-001: Cross-Platform Desktop Application](MIS-002/CNS/CNS-001.md)
## Artifact

- [ART-004: SVG Diagram Export](MIS-002/ART/ART-004.md)
## Requirement

- [REQ-021: Respect Reduced Motion Preferences](MIS-002/REQ/REQ-021.md)
## Activity

- [ATV-006: Persist Card Change](MIS-002/ATV/ATV-006.md)
## Feature

- [FEA-009: Validation On Load Edit And Save](MIS-002/FEA/FEA-009.md)
## Constraint

- [CNS-002: WCAG AA Conformance](MIS-002/CNS/CNS-002.md)
## Asset

- [AST-002: Audit Trail Metadata](MIS-002/AST/AST-002.md)
## Component

- [COM-008: Autosave And Undo Manager](MIS-002/COM/COM-008.md)
## Process

- [PRO-001: Open And Index Model Home](MIS-002/PRO/PRO-001.md)
## Requirement

- [REQ-006: Tree Never Shows tmp](MIS-002/REQ/REQ-006.md)
## Activity

- [ATV-003: Render Navigation Panes](MIS-002/ATV/ATV-003.md)
## Capability

- [CAP-006: Persist Model Changes](MIS-002/CAP/CAP-006.md)
## Data Store

- [DTS-002: Derived Output Folder](MIS-002/DTS/DTS-002.md)
## Requirement

- [REQ-023: Editor Pane Has JSON And Markdown](MIS-002/REQ/REQ-023.md)
## Control

- [CTL-001: Schema And Invariant Validation Gate](MIS-002/CTL/CTL-001.md)
## Requirement

- [REQ-002: Default To Lowest Mission](MIS-002/REQ/REQ-002.md)
## Feature

- [FEA-002: Model Tree View](MIS-002/FEA/FEA-002.md)
## Activity

- [ATV-004: Edit Card Fields](MIS-002/ATV/ATV-004.md)
## Requirement

- [REQ-027: Audit Trail And Version Bump](MIS-002/REQ/REQ-027.md)
- [REQ-014: Browse Views And Generated Markdown](MIS-002/REQ/REQ-014.md)
- [REQ-017: First Writer Wins For Conflicts](MIS-002/REQ/REQ-017.md)
## Feature

- [FEA-018: VS Code Extension UI](MIS-002/FEA/FEA-018.md)
## Threat

- [THR-002: Model Corruption Via Invalid Edits](MIS-002/THR/THR-002.md)
## System

- [SYS-001: Aurora Viewer And Editor Tooling](MIS-002/SYS/SYS-001.md)
## Component

- [COM-011: Model Validator](MIS-002/COM/COM-011.md)
## Actor

- [ACT-004: Threat Actor](MIS-002/ACT/ACT-004.md)
## Component

- [COM-014: aurora_cli Invoker](MIS-002/COM/COM-014.md)
## Requirement

- [REQ-016: Watch Model Folders](MIS-002/REQ/REQ-016.md)
- [REQ-024: Undo Queue Is Available](MIS-002/REQ/REQ-024.md)
## Asset

- [AST-001: Aurora Model](MIS-002/AST/AST-001.md)
## Requirement

- [REQ-008: Click To Recenter](MIS-002/REQ/REQ-008.md)
- [REQ-012: Save-As-You-Go With Conflict Handling](MIS-002/REQ/REQ-012.md)
- [REQ-022: Graph View Supports Zoom](MIS-002/REQ/REQ-022.md)
## Capability

- [CAP-004: Filter Views](MIS-002/CAP/CAP-004.md)
## Interface

- [INT-003: aurora_cli Invocation Contract](MIS-002/INT/INT-003.md)
## Feature

- [FEA-006: Rendered Markdown Preview](MIS-002/FEA/FEA-006.md)
- [FEA-014: SVG Export](MIS-002/FEA/FEA-014.md)
- [FEA-020: Import And Export Aurora Model Homes](MIS-002/FEA/FEA-020.md)
## Event

- [EVT-002: Filesystem Change Detected](MIS-002/EVT/EVT-002.md)
## Requirement

- [REQ-031: Never Execute Model Content](MIS-002/REQ/REQ-031.md)
## Artifact

- [ART-001: Card JSON File](MIS-002/ART/ART-001.md)
## State

- [STA-002: Model Loaded](MIS-002/STA/STA-002.md)
## Constraint

- [CNS-005: Prefer aurora_cli For Derived Outputs](MIS-002/CNS/CNS-005.md)
## State Machine

- [STM-001: Editing Session Lifecycle](MIS-002/STM/STM-001.md)
## Driver

- [DRI-004: Live Modeling With Agents](MIS-002/DRI/DRI-004.md)
## Feature

- [FEA-016: Secure File Handling](MIS-002/FEA/FEA-016.md)
## Capability

- [CAP-008: Validate Schema And Invariants](MIS-002/CAP/CAP-008.md)
## Activity

- [ATV-009: Export SVG](MIS-002/ATV/ATV-009.md)
## Risk

- [RIS-003: Tool Unavailability Or Lag](MIS-002/RIS/RIS-003.md)
## Feature

- [FEA-003: Graph Navigation View](MIS-002/FEA/FEA-003.md)
## Requirement

- [REQ-032: Retro-Futuristic Control-Panel Aesthetic](MIS-002/REQ/REQ-032.md)
## Condition

- [CON-003: Persistent Store Write Is Safe](MIS-002/CON/CON-003.md)
## Event

- [EVT-005: Validation Passed](MIS-002/EVT/EVT-005.md)
## Requirement

- [REQ-018: Expose MCP Endpoint While Running](MIS-002/REQ/REQ-018.md)
- [REQ-030: Paths And Filenames Are Safe](MIS-002/REQ/REQ-030.md)
## Component

- [COM-013: Link Safety Engine](MIS-002/COM/COM-013.md)
## Story

- [STR-001: Edit Cards Safely With Feedback](MIS-002/STR/STR-001.md)
## Requirement

- [REQ-005: Tree Shows Only Card JSON Files](MIS-002/REQ/REQ-005.md)
- [REQ-034: VS Code Extension Reuses Core Logic](MIS-002/REQ/REQ-034.md)
## Component

- [COM-005: View Filter Engine](MIS-002/COM/COM-005.md)
## State

- [STA-005: Resolving Conflict](MIS-002/STA/STA-005.md)
## Artifact

- [ART-002: Rendered Card Preview](MIS-002/ART/ART-002.md)
## Activity

- [ATV-001: Scan Model Home](MIS-002/ATV/ATV-001.md)
## Control

- [CTL-004: Invariant-Safe Link Targeting](MIS-002/CTL/CTL-004.md)
## Requirement

- [REQ-004: Model Tree Mirrors On-Disk Layout](MIS-002/REQ/REQ-004.md)
## Feature

- [FEA-012: Output Synchronization Via aurora_cli](MIS-002/FEA/FEA-012.md)
- [FEA-005: Card JSON Editor](MIS-002/FEA/FEA-005.md)
## Actor

- [ACT-003: Agent](MIS-002/ACT/ACT-003.md)
## Requirement

- [REQ-011: Edit Allowed Fields](MIS-002/REQ/REQ-011.md)
## Activity

- [ATV-012: Export Model Home](MIS-002/ATV/ATV-012.md)
## Requirement

- [REQ-001: Load Models Into Local Store](MIS-002/REQ/REQ-001.md)
## Capability

- [CAP-012: Expose MCP CRUD And Events](MIS-002/CAP/CAP-012.md)
## Requirement

- [REQ-033: Store Cards In Local IndraDB](MIS-002/REQ/REQ-033.md)


