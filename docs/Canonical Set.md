# Canonical Set

Aurora includes a canonical vocabulary of card types, relationships, icon ids, and view definitions.

This canonical set is defined in the model configuration registry:

- `reference/Aurora.modelconfiguration.json`

…and validated by:

- `schemas/Aurora.modelconfiguration.schema.json`

## What the canonical set provides

- A **broad set of card types** covering common architectural domains (requirements, structure, process, behavior, security, data, deployment).
- A set of **allowed relationships** (per card type) intended to keep models consistent and machine-checkable.
- Default **appearance settings** per card type (shape, fill/stroke/text colors, default icon).
- A set of **views** (diagram definitions) that can be rendered from any model using the canonical card types.

Note:

- This page is a navigational overview.
- The authoritative names, descriptions, and constraints live in `reference/Aurora.modelconfiguration.json`.

## Canonical card types (overview)

The canonical set is intentionally broad; the included example model `MIS-002` exercises all of these types.

### Intent and motivation

| Acronym | Card type | Description |
|---|---|---|
| `MIS` | Mission | Root intent that spawns every downstream driver. |
| `DRI` | Driver | Motivation explaining why requirements exist. |
| `STK` | Stakeholder | Person, group, or role with an interest in or affected by the system. |
| `STR` | Story | Narrative expressing desired behavior/outcome; should follow the “`As an ___ I want/need ___.`” format. |
| `ADR` | ADR | Architectural Decision Record capturing a significant decision, alternatives considered, rationale, trade-offs, and consequences. |

### Requirements and capability

| Acronym | Card type | Description |
|---|---|---|
| `REQ` | Requirement | Verifiable statement of need/obligation. |
| `CAP` | Capability | Implementation-independent ability that satisfies requirements. |
| `CNS` | Constraint | Rule limiting allowable behavior or solutions. |

### Structure and interfaces

| Acronym | Card type | Description |
|---|---|---|
| `SYS` | System | Top-level software system composed of applications and components. |
| `APP` | Application | Deployable application within a system, composed of components. |
| `COM` | Component | Modular unit with a single responsibility and explicit interfaces. |
| `INT` | Interface | Contract governing interaction across a boundary. |
| `VND` | Vendor | Third party providing products/services integrated into the system. |

### Data and artifacts

| Acronym | Card type | Description |
|---|---|---|
| `ART` | Artifact | Concrete work product produced or consumed by the system. |
| `DSR` | Data Source | Origin of artifacts and data entering the system, used to model provenance and lineage. |
| `DST` | Data Store | Persistent storage system that holds artifacts and data for retrieval and lineage. |
| `AST` | Asset | Valuable information/resources requiring protection. |

### Process modeling

| Acronym | Card type | Description |
|---|---|---|
| `PRO` | Process | Ordered sequence of activities/decisions enabling a capability. |
| `ATV` | Activity | Discrete step within a process. |
| `TRG` | Trigger | Process-domain trigger that initiates activities/conditions; distinct from state-machine events. |
| `CON` | Condition | Process-domain branching condition used by activities/processes; not used for state-machine branching. |
| `ACT` | Actor | External role interacting with or obligating the system. |

### Deployment

| Acronym | Card type | Description |
|---|---|---|
| `DEP` | Deployment | Defined environment or configuration where software executes. |
| `NOD` | Node | Logical/physical execution environment hosting components or stores. |

### Behavioral modeling

| Acronym | Card type | Description |
|---|---|---|
| `STM` | State Machine | Behavioral model defining allowable states/transitions. |
| `STA` | State | Observable condition that persists until a transition. |
| `EVT` | Event | State-machine event that causes transitions or predicate evaluation; distinct from process-domain triggers. |
| `PRD` | Predicate | State-machine branching predicate that evaluates to true/false; distinct from process-domain conditions. |

### Verification

| Acronym | Card type | Description |
|---|---|---|
| `TES` | Test | Procedure verifying that a feature satisfies requirements. |
| `FEA` | Feature | Externally observable behavior realizing one or more requirements. |

### Security and threat modeling

| Acronym | Card type | Description |
|---|---|---|
| `THM` | Threat Model | Structured analysis of assets, adversaries, threats, and mitigations. |
| `THD` | Threat Diamond | Threat rendered as a diamond (diagramming variant). |
| `ADV` | Adversary | Threat source: actor with intent and capability to cause harm. |
| `THC` | Threat Capability | Skills, access, tooling, or resources enabling an adversary to execute threats. |
| `VIC` | Victim | Entity that suffers impact when a threat is realized. |
| `ROW` | Resource Owner | Role accountable for an asset/resource and its protection. |
| `CTL` | Control | Mechanism governing or constraining a process. |
| `RIS` | Risk | Potential for loss/harm from threats/vulnerabilities. |

## Canonical views

Views are defined in `reference/Aurora.modelconfiguration.json` under the `views` list.

Each view defines:

- `root_card_types`: which acronyms may act as roots for that view
- `included_card_types`: which acronyms are included when traversing links

Default view set (names and intent):

| View name | Root card types | What it’s for |
|---|---|---|
| Compliance Governance | `MIS` | Compliance narrative from drivers/requirements through controls and tests. |
| Component | `SYS`, `APP`, `VND` | Internal implementation structure and integration points. |
| Deployment | `DEP`, `VND` | Where software executes and what runs where. |
| Entire Model | `MIS` | Broadest coverage view across the canonical set. |
| Process | `CAP` | How a capability is realized via processes, activities, triggers, and conditions. |
| Requirements | `MIS` | Mission intent, motivation, requirements, and decisions. |
| Context | `MIS` | System context: stakeholders, actors, vendors, and boundary interfaces. |
| Landscape | `SYS`, `APP`, `VND` | High-level landscape of major components and runtime elements. |
| Security | `THM` | Threat model coverage: assets, adversaries, threats, controls, and risks. |
| State Machine | `STM` | Behavioral model via state machines, states, predicates, and events. |
| Traceability | `MIS` | End-to-end traceability from requirements to implementation and verification. |
| Use Case | `STK`, `ACT` | Externally visible behavior via stories and actor activities. |

## Relationship constraints (how to think about them)

Aurora’s registry declares allowable relationship labels per source card type.

Example “spine”:

```mermaid
flowchart LR
	MIS["Mission"] -->|establishes| DRI["Driver"]
	DRI -->|drives| REQ["Requirement"]
	REQ -->|requires| CAP["Capability"]
```

These constraints are intended to keep models consistent.

Important nuance:

- Aurora’s **invariants** are validation errors.
- Registry deviations are typically treated as **warnings** (to support controlled extension beyond the canonical set).

See **[Configuration Guide](./Configuration%20Guide.md)** for adding new card types and relationships.
