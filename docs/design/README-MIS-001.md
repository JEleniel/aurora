# MIS-001: Enable Deterministic Aurora CLI Tooling

**Mission**: Provide a first-party command-line tool that validates Aurora models and deterministically generates human-friendly documentation and standard views so architects can model systems without hand-drawing diagrams or manually maintaining documentation.

## Views

- [Requirements](MIS-001/Requirements.view.md)
- [Deployment](MIS-001/Deployment.view.md)
- [Process](MIS-001/Process.view.md)
- [State Machine](MIS-001/State_Machine.view.md)
- [Threat Model](MIS-001/Threat_Model.view.md)
- [Component](MIS-001/Component.view.md)


## Cards

## Control

- [CTL-001: Deterministic Ordering](MIS-001/CTL/CTL-001.md)
## Condition

- [CON-001: Model Is Valid](MIS-001/CON/CON-001.md)
## Activity

- [ATV-002: Load Cards](MIS-001/ATV/ATV-002.md)
- [ATV-004: Validate Invariants](MIS-001/ATV/ATV-004.md)
## Interface

- [INT-002: View Rendering Contract](MIS-001/INT/INT-002.md)
## Requirement

- [REQ-005: Generate Card Catalog Indexes](MIS-001/REQ/REQ-005.md)
## Event

- [EVT-001: Command Invoked](MIS-001/EVT/EVT-001.md)
## Data Store

- [DTS-001: Aurora Model Folder](MIS-001/DTS/DTS-001.md)
## Capability

- [CAP-006: Maintain Audit Trails](MIS-001/CAP/CAP-006.md)
## State Machine

- [STM-001: Aurora CLI Run Lifecycle](MIS-001/STM/STM-001.md)
## Event

- [EVT-002: Model Loaded](MIS-001/EVT/EVT-002.md)
## Artifact

- [ART-001: Validation Report](MIS-001/ART/ART-001.md)
## Data Store

- [DTS-002: Documentation Output Folder](MIS-001/DTS/DTS-002.md)
## Activity

- [ATV-001: Resolve Model Root](MIS-001/ATV/ATV-001.md)
## Requirement

- [REQ-014: Determine Editor Identity](MIS-001/REQ/REQ-014.md)
## Capability

- [CAP-003: Render Markdown Card Catalog](MIS-001/CAP/CAP-003.md)
## Interface

- [INT-001: Command Line Interface](MIS-001/INT/INT-001.md)
## Component

- [COM-008: Filesystem Writer](MIS-001/COM/COM-008.md)
## Requirement

- [REQ-004: Render One Markdown File Per Card](MIS-001/REQ/REQ-004.md)
## Actor

- [ACT-001: Architect](MIS-001/ACT/ACT-001.md)
## Control

- [CTL-002: Output Path Sanitization](MIS-001/CTL/CTL-002.md)
## Actor

- [ACT-002: Tool User](MIS-001/ACT/ACT-002.md)
## Activity

- [ATV-006: Render Standard Views](MIS-001/ATV/ATV-006.md)
## Requirement

- [REQ-003: Resolve Model Root And Missions From Input](MIS-001/REQ/REQ-003.md)
- [REQ-012: Bump Card Semver Versions](MIS-001/REQ/REQ-012.md)
## Process

- [PRO-002: Documentation Generation](MIS-001/PRO/PRO-002.md)
## Requirement

- [REQ-008: Use ELK Layout And Per-Type Shapes](MIS-001/REQ/REQ-008.md)
## Application

- [APP-001: aurora_cli](MIS-001/APP/APP-001.md)
## Asset

- [AST-001: Aurora Model](MIS-001/AST/AST-001.md)
## Event

- [EVT-003: Validation Completed](MIS-001/EVT/EVT-003.md)
## Requirement

- [REQ-009: Support Configurable Output Roots](MIS-001/REQ/REQ-009.md)
## Driver

- [DRI-002: Trustworthy Model Validation](MIS-001/DRI/DRI-002.md)
## Feature

- [FEA-001: Validate Command](MIS-001/FEA/FEA-001.md)
## Constraint

- [CNS-002: Safe Filesystem Writes](MIS-001/CNS/CNS-002.md)
## Component

- [COM-005: Invariant Validator](MIS-001/COM/COM-005.md)
## Story

- [STR-001: Generate Documentation And Views From Model](MIS-001/STR/STR-001.md)
- [STR-002: Validate And Render Outputs For Sharing](MIS-001/STR/STR-002.md)
## State

- [STA-001: Initialized](MIS-001/STA/STA-001.md)
## Driver

- [DRI-001: Operational Friction Elimination](MIS-001/DRI/DRI-001.md)
## Capability

- [CAP-001: Resolve And Load Aurora Model](MIS-001/CAP/CAP-001.md)
## Component

- [COM-004: Schema Validator](MIS-001/COM/COM-004.md)
## Requirement

- [REQ-011: Discover Default Model Root](MIS-001/REQ/REQ-011.md)
## Feature

- [FEA-007: Compact Model Export Command](MIS-001/FEA/FEA-007.md)
## Component

- [COM-003: Model Loader](MIS-001/COM/COM-003.md)
- [COM-002: Model Root Resolver](MIS-001/COM/COM-002.md)
## Capability

- [CAP-004: Render Standard Views](MIS-001/CAP/CAP-004.md)
## Activity

- [ATV-003: Validate Schema](MIS-001/ATV/ATV-003.md)
## Requirement

- [REQ-007: Skip Empty Views](MIS-001/REQ/REQ-007.md)
## State

- [STA-004: Generated](MIS-001/STA/STA-004.md)
## Feature

- [FEA-004: All Command](MIS-001/FEA/FEA-004.md)
- [FEA-005: Standard View Set Generation](MIS-001/FEA/FEA-005.md)
## Requirement

- [REQ-001: Validate Cards Against Co-Located Schema](MIS-001/REQ/REQ-001.md)
## Capability

- [CAP-005: Write Outputs Deterministically](MIS-001/CAP/CAP-005.md)
## Requirement

- [REQ-015: Generate RFC3339 UTC Millisecond Timestamps](MIS-001/REQ/REQ-015.md)
## Asset

- [AST-002: Generated Documentation Set](MIS-001/AST/AST-002.md)
## State

- [STA-005: Failed](MIS-001/STA/STA-005.md)
## Feature

- [FEA-006: Bump Version Commands](MIS-001/FEA/FEA-006.md)
## Condition

- [CON-002: View Has Included Cards](MIS-001/CON/CON-002.md)
## Requirement

- [REQ-013: Append Audit History On Version Bumps](MIS-001/REQ/REQ-013.md)
## Feature

- [FEA-002: Render Cards Command](MIS-001/FEA/FEA-002.md)
## Artifact

- [ART-003: Card Index README](MIS-001/ART/ART-003.md)
## Component

- [COM-006: Markdown Card Generator](MIS-001/COM/COM-006.md)
## Activity

- [ATV-005: Render Markdown Cards](MIS-001/ATV/ATV-005.md)
## Requirement

- [REQ-016: Export Compact Model Snapshot](MIS-001/REQ/REQ-016.md)
## Component

- [COM-001: CLI Command Router](MIS-001/COM/COM-001.md)
## Capability

- [CAP-002: Validate Aurora Model](MIS-001/CAP/CAP-002.md)
## Constraint

- [CNS-001: Deterministic, Diff-Friendly Output](MIS-001/CNS/CNS-001.md)
## Component

- [COM-007: Standard View Generator](MIS-001/COM/COM-007.md)
## State

- [STA-002: Loaded](MIS-001/STA/STA-002.md)
## Artifact

- [ART-004: View Document](MIS-001/ART/ART-004.md)
## Feature

- [FEA-003: Render Views Command](MIS-001/FEA/FEA-003.md)
## Requirement

- [REQ-006: Generate Standard View Set](MIS-001/REQ/REQ-006.md)
## Capability

- [CAP-007: Compact Model Packaging](MIS-001/CAP/CAP-007.md)
## Process

- [PRO-001: Aurora CLI Execution](MIS-001/PRO/PRO-001.md)
## Requirement

- [REQ-010: Apply Standard Mermaid Palette And Classes](MIS-001/REQ/REQ-010.md)
- [REQ-002: Validate Model Invariants](MIS-001/REQ/REQ-002.md)
## Artifact

- [ART-002: Markdown Card File](MIS-001/ART/ART-002.md)
## System

- [SYS-001: Aurora Support Tooling](MIS-001/SYS/SYS-001.md)
## Activity

- [ATV-007: Write Outputs](MIS-001/ATV/ATV-007.md)
## Driver

- [DRI-003: Automate Architecture Communication](MIS-001/DRI/DRI-003.md)


