# Design Documentation

This folder contains the generated Aurora documentation for the `Enable Deterministic Aurora CLI Tooling` mission.

## Model

- Aurora model root: /home/jeleniel/repos/aurora/docs/design/aurora
- Mission entrypoint: /home/jeleniel/repos/aurora/docs/design/aurora/MIS-001.json

## Cards

Cards are rendered under `cards` and grouped below by card type. Link text is the card name.

### Mission

- [Enable Deterministic Aurora CLI Tooling](cards/MIS-001.md)

### Driver

- [Operational Friction Elimination](cards/Driver/DRI-001.md)
- [Trustworthy Model Validation](cards/Driver/DRI-002.md)
- [Automate Architecture Communication](cards/Driver/DRI-003.md)

### Constraint

- [Deterministic, Diff-Friendly Output](cards/Constraint/CNS-001.md)
- [Safe Filesystem Writes](cards/Constraint/CNS-002.md)

### Requirement

- [Validate Aurora Schema](cards/Requirement/REQ-001.md)
- [Validate Aurora Graph Invariants](cards/Requirement/REQ-002.md)
- [Accept Model Root Or Mission Path](cards/Requirement/REQ-003.md)
- [Generate Human-Friendly Markdown Cards](cards/Requirement/REQ-004.md)
- [Generate Card Index README](cards/Requirement/REQ-005.md)
- [Generate Standard Views](cards/Requirement/REQ-006.md)
- [Skip Empty Views](cards/Requirement/REQ-007.md)
- [Use ELK Layout And Type Shapes](cards/Requirement/REQ-008.md)
- [Configurable Output Folders](cards/Requirement/REQ-009.md)
- [Standard Mermaid Styling](cards/Requirement/REQ-010.md)
- [Discover Default Input Model](cards/Requirement/REQ-011.md)
- [Bump Audit Trail Version](cards/Requirement/REQ-012.md)
- [Append Audit Trail History Entry](cards/Requirement/REQ-013.md)
- [Derive Editor Identity](cards/Requirement/REQ-014.md)
- [Generate Audit Timestamps](cards/Requirement/REQ-015.md)

### Capability

- [Resolve And Load Aurora Model](cards/Capability/CAP-001.md)
- [Validate Aurora Model](cards/Capability/CAP-002.md)
- [Render Markdown Card Catalog](cards/Capability/CAP-003.md)
- [Render Standard Views](cards/Capability/CAP-004.md)
- [Write Outputs Deterministically](cards/Capability/CAP-005.md)
- [Maintain Audit Trails](cards/Capability/CAP-006.md)

### Feature

- [Validate Command](cards/Feature/FEA-001.md)
- [Render Cards Command](cards/Feature/FEA-002.md)
- [Render Views Command](cards/Feature/FEA-003.md)
- [All Command](cards/Feature/FEA-004.md)
- [Standard View Set Generation](cards/Feature/FEA-005.md)
- [Bump Version Commands](cards/Feature/FEA-006.md)

### System

- [Aurora Support Tooling](cards/System/SYS-001.md)

### Application

- [aurora_cli](cards/Application/APP-001.md)

### Component

- [CLI Command Router](cards/Component/COM-001.md)
- [Model Root Resolver](cards/Component/COM-002.md)
- [Model Loader](cards/Component/COM-003.md)
- [Schema Validator](cards/Component/COM-004.md)
- [Invariant Validator](cards/Component/COM-005.md)
- [Markdown Card Generator](cards/Component/COM-006.md)
- [Standard View Generator](cards/Component/COM-007.md)
- [Filesystem Writer](cards/Component/COM-008.md)

### Interface

- [Command Line Interface](cards/Interface/INT-001.md)
- [View Rendering Contract](cards/Interface/INT-002.md)

### Artifact

- [Validation Report](cards/Artifact/ART-001.md)
- [Markdown Card File](cards/Artifact/ART-002.md)
- [Card Index README](cards/Artifact/ART-003.md)
- [View Document](cards/Artifact/ART-004.md)

### Asset

- [Aurora Model](cards/Asset/AST-001.md)
- [Generated Documentation Set](cards/Asset/AST-002.md)

### Data Store

- [Aurora Model Folder](cards/Data%20Store/DTS-001.md)
- [Documentation Output Folder](cards/Data%20Store/DTS-002.md)

### Process

- [Aurora CLI Execution](cards/Process/PRO-001.md)
- [Documentation Generation](cards/Process/PRO-002.md)

### Story

- [Architect Generates Documentation From Model](cards/Story/STR-001.md)
- [Tool User Validates And Renders Outputs](cards/Story/STR-002.md)

### Actor

- [Architect](cards/Actor/ACT-001.md)
- [Tool User](cards/Actor/ACT-002.md)

### Event

- [Command Invoked](cards/Event/EVT-001.md)
- [Model Loaded](cards/Event/EVT-002.md)
- [Validation Completed](cards/Event/EVT-003.md)

### Activity

- [Resolve Model Root](cards/Activity/ATV-001.md)
- [Load Cards](cards/Activity/ATV-002.md)
- [Validate Schema](cards/Activity/ATV-003.md)
- [Validate Invariants](cards/Activity/ATV-004.md)
- [Render Markdown Cards](cards/Activity/ATV-005.md)
- [Render Standard Views](cards/Activity/ATV-006.md)
- [Write Outputs](cards/Activity/ATV-007.md)

### Condition

- [Model Is Valid](cards/Condition/CON-001.md)
- [View Has Included Cards](cards/Condition/CON-002.md)

### Control

- [Deterministic Ordering](cards/Control/CTL-001.md)
- [Output Path Sanitization](cards/Control/CTL-002.md)

### State Machine

- [Aurora CLI Run Lifecycle](cards/State%20Machine/STM-001.md)

### State

- [Initialized](cards/State/STA-001.md)
- [Loaded](cards/State/STA-002.md)
- [Generated](cards/State/STA-004.md)
- [Failed](cards/State/STA-005.md)

## Views

Standard views are rendered under `views`.

<!-- Rendered view files live in ./views/ -->
<!-- Views are emitted by aurora_cli render-views. -->
<!-- This section is intentionally minimal. -->
