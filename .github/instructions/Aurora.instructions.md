---
applyTo: '**/*'
---
# Aurora Machine Agent Instruction

## Model Overview

**Version**: 2.0.0

Aurora is a deterministic architectural model where architectural elements are cards, the relationships between cards are represented as links, and the model forms a Directed Graph. The model is designed so that any interpretation, such as view diagrams, can be generated from the model; and for direct machine consumption by LLMs, agents, reasoners, and automated tools. The model invariants guarantee unambiguous interpretation and reasoning about the model.

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Models

The model is the central piece of the architecture and is a collection of cards that have links describing their relationships, starting from a `Mission`. Cards represent the elements of the design, described as nouns. Links represent how the elements interact, and are tagged with verbs (for human convenience).

Any pair of cards in the model can be described using simple sentences:

**Examples**:

```text
The mission "Drive Excellence" is "Drive excellence in operations by streamlining processes, integrating automation, and formalizing documentation".

The mission establishes the driver "Operational Friction Elimination" which is "Eliminate non-value-adding manual effort by enforcing end-to-end process automation, standardized workflows, and machine-verifiable documentation across all operational domains".

"Operational Friction Elimination" drives the requirement "Define a Deterministic Modeling Framework" which is "Design a framework for documenting deterministic process models with measurable latency and failure semantics".

The capability "Deterministic Process Authoring & Validation Workflow" which is "A machine-verifiable, executable representation of every operational workflow with deterministic guarantees" satisfies the requirement "Define a Deterministic Modeling Framework".

The feature "Deterministic Workflow Modeling Engine (DWME)" which is "Software tools to create, manage, and render machine-verifiable executable representations of operational workflows" satisfies the requirement "Define a Deterministic Modeling Framework" and enables the capability "Deterministic Process Authoring & Validation Workflow".

The mission necessitates the system "Workflow Model Tooling" which integrates the application "Workflow Modeler".

Workflow Modeler implements the feature "Deterministic Workflow Modeling Engine".
```

**These result in a model that looks like this**:

```mermaid
graph
	drive_excellence(("`Mission:<br />Drive Excellence`"))
	operational_friction_elimination(["`Driver:<br />Operational Friction Elimination`"])
	define_a_deterministic_modeling_framework(["`Requirement:<br />Define a Deterministic Modeling Framework`"])
	deterministic_process_authoring(["`Capability:<br />Deterministic Process Authoring & Validation Workflow`"])
	deterministic_workflow_modeling_engine(["`Feature:<br />Deterministic Workflow Modeling Engine (DWME)`"])
	workflow__model_tooling["`System:<br />Workflow Model Tooling`"]@{shape: div-rect}
	workflow_modeler["`Application:<br />Workflow Modeler`"]@{shape: lin-rect}

	drive_excellence -- establishes --> operational_friction_elimination
	operational_friction_elimination -- drives --> define_a_deterministic_modeling_framework
	deterministic_process_authoring -- satisfies --> define_a_deterministic_modeling_framework
	deterministic_workflow_modeling_engine -- satisfies --> define_a_deterministic_modeling_framework
	deterministic_workflow_modeling_engine -- enables --> deterministic_process_authoring
	drive_excellence -- necessitates --> workflow__model_tooling
	workflow__model_tooling -- integrates --> workflow_modeler
	workflow_modeler -- implements --> deterministic_workflow_modeling_engine
```

### Cards

A card contains the information about an element of the model and the links to other elements. The cards themselves and the links between them do not encode semantic meaning (see [Invariant Rules](#invariant-rules)). Cards also have an `attributes` property that allows additional, arbitrary information to be included. Card files should be "pretty printed" using `prettier`.

Each card is comprised of:

- `$schema` - the relative link to the Aurora schema file included with the model(s) in the model home.
- `id` - A unique identifier assigned to the card composed from a predefined prefix per `card_type` followed by a sequentially assigned integer per card type. If the model is extended, additional prefixes must also be issued for new card types. Once issued to a card the `id` MUST NOT change. If `card_type` changes, a new card should be issued and the original moved to the `status` "Deleted" with an appropriate audit history entry added. This enables traceability of the model over time.

**Example Names**:

- MIS-001
- DRI-001
- DRI-002
- REQ-001
- REQ-002

- `card_type` - The architectural element represented by the card in title case. See [Common Cards](#common-cards) for examples
- `card_subtype` - An optional refinement of the `card_type` in title case. See [Common Cards](#common-cards) for examples.
- `name` - A concise human-readable name for the card in title case
- `description` - Details regarding the element the card represents
- `status` - the status of an implementable element such as a `Feature`
	+ A common lifecycle is: "Proposed", "Backlog", "Design", "Implementation", "Review", "Pre-Release", "Released", "Deprecated", "Retired", "Deleted".
	+ Any other series of lifecycle states that make sense for the model may be used, as long as they are kept consistent per type across the model.
- `links` - pointers to other cards establishing relationships
	+ `target` - the destination card `id`
	+ `relationship` - verb describing the impact for human reference
- `audit_trail` - a record of the version and a history of the events (created/edited/deleted) the card has been through
	+ `version` - A semver version number for the card that is incremented when the card changes.
		* The major version is incremented for changes that alter the meaning or definition of the element, such as changing the `card_subtype`, updating the `name`, or adjusting the `description` in a way that changes meaning. The `status` moving to `Deleted` is also a major increment.
		* The minor version is incremented for changes that do not alter the element's definition, such as revising the phrasing of the `description` without changing meaning, or a change in `status` other than to "Deleted".
		* Insignificant changes, such as typographic or grammatical corrections, increase the patch version.
	+ `hash` - an optional SHA256 hash of the card with the hash temporarily set to `null` to calculate the hash
	+ `history` - an array of objects capturing the audit history of the card
		* `editor` - the identity of the entity making the change. Agents should use the name of their host (e.g. "Copilot") and, if acting as a particular role, a colon followed by a space and the agent role name, for example "Copilot: BackendDeveloper".
		* `timestamp` - the UTC time of the edit in RFC3339 format with millisecond resolution
		* `event` - the type of change event, one of: `created`, `edited`, `deleted`
- `attributes` - Arbitrary, optional key-value pairs providing additional data; the value can be any valid JSON value, including objects. See [Common Cards](#common-cards) for examples.

#### The Compressed Model

In order to make the model easier for implementing agents to process and save context space, a compressed model file may be generated named `{mission id}.agent.json` for agent reference in the model home that conforms to the `Aurora.compact.schema.json` in the model home.

The compressed file will have all of the cards in a top level array named `cards` and will have the `audit_trail` stripped from each. The compressed file may be "pretty printed" but that is not required.

#### Common Cards

The following list contains a set of common cards (enough for a complete model) that can be used as a starting point. It is a refined, internally consistent card list aligned to the Aurora semantics, BPMN-inclusive, and AI-friendly. The prefix for file names is in parentheses after the card type and colon.

- **Mission**: (MIS) The fundamental purpose that gives the model intent and meaning and from which all drivers originate.
	+ Examples: "Modernize operations through automation", "Protect sensitive information at scale", "Enable reliable digital service delivery", "Ensure regulatory compliance and trust", "Support equitable access to services"

- **Driver**: (DRI) A motivating force that explains why requirements exist.
	+ Examples: "Grow Responsibly", "Comply with Laws, Rules, and Regulations", "Prevent Data Breaches", "Reduce Operational Risk", "Improve User Trust"

- **Requirement**: (REQ) A verifiable statement of need or obligation that must be satisfied.
	+ Examples: "Users are authenticated", "Access to resources is controlled", "Security-relevant events are recorded", "Users can recover account access", "Transaction history is maintained"

- **Capability**: (CAP) A stable, implementation-independent ability required to satisfy one or more requirements.
	+ Examples: "Authenticate identities", "Authorize actions", "Manage configuration changes", "Monitor operational health", "Respond to security incidents"

- **Feature**: (FEA) An externally observable system behavior that realizes one or more requirements.
	+ Examples: "Single sign-on using an external identity provider", "Document upload and download", "Administrative approval workflow", "Audit log viewer", "Password reset via email link"

- **System**: (SYS) A bounded collection of interacting applications organized to achieve a mission.
	+ Examples: "Identity and Access System", "Payment Processing System", "Customer Management System", "Monitoring and Alerting System", "Content Delivery System"

- **Application**: (APP) A deployable software system that implements features.
	+ Examples: "Web Portal", "Mobile Application", "Background Service", "API Provider", "Desktop Console"

- **Component**: (COM) A modular unit with a single responsibility and explicit interfaces.
	+ Examples: "User Interface", "Authorization Service", "Notification Dispatcher", "Data Ingestion Pipeline", "Reporting Engine"

- **Interface**: (INT) A defined contract governing interaction across a boundary.
	+ Examples: "Authentication API", "GraphQL API", "Command Line Interface", "Webhook Endpoint", "Message Consumer Interface"

- **Artifact**: (ART) A concrete work product produced or consumed by the system.
	+ Examples: "Configuration File", "Executable Binary", "Deployment Package", "Generated Report", "Log File"

- **Asset**: (AST) Information or resources that have value and require protection or management.
	+ Examples: "User Credentials", "Customer Personal Data", "Transaction Records", "System Configuration Data", "Audit Logs"

- **Data Store**: (DTS) A persistent resource that robustly stores and recovers data for its intended lifetime.
	+ Examples: "Database", "File System", "Cache", "Message Queue", "Object Storage"

- **Deployment**: (DEP) A defined environment or configuration in which software is executed.
	+ Examples: "Development", "Integration Test", "User Acceptance Test", "Production", "Disaster Recovery"

- **Node**: (NOD) A logical or physical execution environment capable of hosting components or data stores.
	+ Examples: "Virtual Machine", "Docker Container", "Physical Server", "Edge Device", "Managed Cloud Runtime"

- **Node Instance**: (NIN) A concrete, identifiable runtime realization of a node.
	+ Examples: "app-server-03", "db-primary-us-east-1", "container-instance-7f9c8d", "edge-node-12", "worker-node-a"

- **Process**: (PRO) An ordered sequence of activities and decisions required to enable a capability.
	+ Examples: "Authentication Process", "Customer Onboarding Process", "Incident Response Process", "Configuration Update Process", "Metrics Collection Process"

- **Activity**: (ATV) A unit of behavior performed by an actor as part of a process.
	+ Examples: "Log In", "Submit Form", "Review Request", "Approve Change", "Reset Password"

- **Actor**: (ACT) An external role that interacts with the system or imposes obligations on it.
	+ Examples: "User", "Administrator", "Support Agent", "Threat Actor", "Department of the Treasury"

- **Story**: (STR) A concise narrative expressing a desired behavior or outcome from an actor’s perspective.
	+ Examples: "User logs in to access their account", "Administrator resets a user password", "Customer cancels an order", "Operator investigates a security alert", "Regulator reviews compliance evidence"

- **Event**: (EVT) A discrete occurrence that initiates a process or triggers behavior.
	+ Examples: "Login request received", "Password reset requested", "Configuration change submitted", "Alert generated", "Message received"

- **State Machine**: (STM) A behavioral model defining allowable states and transitions.
	+ Examples: "User Account Lifecycle", "Order Fulfillment Lifecycle", "Session Lifecycle", "Incident Lifecycle", "Deployment Lifecycle"

- **State**: (STA) A persistent, observable condition of an element until a transition occurs.
	+ Examples: "Logged Out", "Pending Approval", "Active", "Suspended", "Completed"

- **Condition**: (CON) A binary predicate that evaluates to true or false and influences behavior.
	+ Examples: "User is authenticated", "Input is valid", "Account is active", "Quota is exceeded", "Error is detected"

- **Control**: (CTL) A mechanism that governs or constrains a process.
	+ Examples: "Authorization Check", "Input Validation", "Rate Limiting", "Approval Gate", "Encryption Enforcement"

- **Constraint**: (CNS) A rule or limitation that restricts allowable behavior or solutions.
	+ Examples: "GDPR data minimization requirement", "PCI-DSS encryption requirement", "Retain audit logs for seven years", "Maximum transaction value limit", "Accessibility compliance obligation"

- **Risk**: (RIS) The potential for loss or harm arising from threats and vulnerabilities.
	+ Examples: "Unauthorized account access", "Sensitive data exposure", "Service outage during peak usage", "Privilege escalation by insider", "Loss of audit trail integrity"

- **Threat**: (THR) A potential cause of an unwanted impact on the system or mission.
	+ Examples: "Data exfiltration", "Credential theft", "Denial of service", "Unauthorized configuration change", "Malware injection"

- **Test**: (TES) A defined procedure used to verify that a feature satisfies its requirements.
	+ Examples: "Verify successful user login", "Validate access denial without authentication", "Confirm password reset email delivery", "Ensure audit log entry is created", "Measure API response time under load"

- **Note**: (NOT) A non-structural annotation attached to another element.

#### Common Relationships

Links are intentionally free-form; the `relationship` verb is descriptive for humans and does not encode semantic meaning by itself. The following relationships appear in the examples in this document and are recommended conventions for consistency:

- `contains` - used by `Boundary` to indicate containment (structural membership)
- `includes` - used to indicate membership without containment (e.g., view/topology constructs like `Deployment`, and associating a parent element with a `Boundary` in a view)
- `establishes` - introduces a downstream motivation element (e.g., `Mission` → `Driver`)
- `drives` - indicates a motivating element influences a downstream need (e.g., `Driver` → `Requirement`)
- `satisfies` - indicates a downstream element fulfills a need (e.g., `Capability`/`Feature` → `Requirement`)
- `enables` - indicates an element makes another feasible/possible (e.g., `Feature` → `Capability`)
- `necessitates` - indicates a higher-level goal requires an element to exist (e.g., `Mission` → `System`)
- `integrates` - indicates a container/system pulls together sub-elements (e.g., `System` → `Application`)
- `comprises` - indicates composition (e.g., `Application` → `Component`)
- `implements` - indicates an element realizes another (e.g., `Component` → `Feature`)
- `exposes` - indicates an element provides an interface (e.g., `Component` → `Interface`)
- `generates` - indicates an element produces an artifact (e.g., `Component` → `Artifact`)
- `uses` - indicates a dependency or consumption (e.g., `Component` → `Artifact`)
- `to_call` - indicates an invocation path (e.g., `Artifact` → `Interface`)
- `persists_to` - indicates persistence of an artifact to storage (e.g., `Artifact` → `Data Store`)
- `validates` - indicates a test verifies behavior (e.g., `Test` → `Component`/`Feature`)
- `participates_in` - indicates involvement in a topology (e.g., `Application` → `Deployment`)
- `hosts` - indicates a runtime host relationship (e.g., `Node`/`Node Instance` → `Component`/`Data Store`)
- `instantiates` - indicates a node creates a runtime instance (e.g., `Node` → `Node Instance`)
- `involves` - indicates a process or mission includes an actor (e.g., `Process`/`Mission` → `Actor`)
- `desires` - indicates an actor has a story/goal (e.g., `Actor` → `Story`)
- `performs` - indicates an actor executes an activity (e.g., `Actor` → `Activity`)
- `explains` - indicates a story elaborates on a capability/feature (e.g., `Story` → `Capability`)
- `implies` - indicates a story suggests a constraint (e.g., `Story` → `Constraint`)
- `starts_with` - indicates the first event of a process (e.g., `Process` → `Event`)
- `receives` - indicates a state receives an event (e.g., `State` → `Event`)
- `starts_in` - indicates the initial state of a state machine (e.g., `State Machine` → `State`)
- `transitions_to` - indicates a transition edge (e.g., `State`/`Activity` → `State`)
- `triggers` - indicates an event/activity triggers another element (e.g., `Event` → `State`)
- `triggers_true` / `triggers_false` - indicates conditional branching outcomes (e.g., `Condition` → `State`)
- `limits` - indicates a constraint bounds another element (e.g., `Constraint` → `Feature`)
- `governs` - indicates control/assurance applies to an element (e.g., `Control` → `Capability`)
- `presents` - indicates an actor presents a threat (e.g., `Actor` → `Threat`)
- `imposes` - indicates a threat introduces a risk (e.g., `Threat` → `Risk`)
- `impacts` - indicates a risk affects another element (e.g., `Risk` → `Feature`)
- `mitigates` - indicates that a feature or capability addresses a risk (e.g., `Feature` → `Risk`)

#### Special cards

##### The `Mission` card

The root card of the model is _always_ a `Mission` card. The `Mission` card captures the overarching purpose and goal of the model.

##### The `Boundary` card

The `Boundary` (BND) card represents a logical grouping of other cards, in other words, they contain them. For simplicity in the model, a parent card includes a `Boundary` and a boundary contains a link to a single target card; in order to represent complex boundaries, a `Boundary` card may include an attribute named `recursive` (boolean, default false). If true, the boundary includes all descendants of the contained card (the contains target) in the current view. Because the cards contained by a boundary may form local loops rendering engines have to be careful when traversing the descendants. `Boundary` cards frequently "contain" `System`, `Application`, `Node`, `Process`, `State Machine`, and similar higher-level cards, and optionally their descendants. They are linked between a parent and the contained card, e.g., parent -- includes --> `Boundary` -- contains --> target, always in parallel to another link.

When recursing, descendants are determined by link traversal and then filtered to only those cards included in the current view. Tooling should consider encountering a card previously traversed as a terminal point to prevent recursion.

**Example of a Boundary in a Model**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	a(A)
	boundary([Boundary])
	b(B)

	a -- verb --> b
	a -- includes --> boundary
	boundary -- contains --> b
```

**Example of a Boundary Rendered in a View**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	a(A)

	subgraph Boundary
		b(B)
	end

	a -- verb --> b

	classDef cls_boundary stroke-dasharray:5 5;
	class Boundary cls_boundary
```

##### The `Note` Card

The `Note` card is purely an annotation to the model. They only have an incoming link and are always leaf nodes. The `Note` card does not add any new elements to the model; it expands on the target card and is for additional information. When rendered, the link to a `Note` card is often drawn as a dotted or dashed line.

**Example**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	card[Card]
	note[Note]@{shape: card}

	card -.- note
```

#### Model Folders and Card Files

Models are stored in a folder named `aurora` (model home) with the root `Mission` card(s) in the model home and a subfolder named for the `id` of the mission containing all other cards (mission home). The mission home is divided further into a subfolder for each `card_type`. Multiple models may share a model home.

Each model starts with a single `Mission` card in the model home named `MIS-{number}-{name with spaces converted to underscores}.json`, for example `aurora/MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json`. Like all cards, the number is issued sequentially by card type.

All other cards are stored as a separate JSON files, named for the card's `id`, e.g., `REQ-001.json`. All cards starting from a shared model home must conform to the `Aurora.schema.json` schema in that model home. If the schema is not present in the model home when starting a model, the canonical `schemas/Aurora.schema.json`, if present, or the `.github/instructions/Aurora.schema.json` must be copied into the `aurora` folder before creating the first `Mission` card.

Compact model files must conform to the `Aurora.compact.schema.json` schema in the model home. If the schema is not present in the model home when starting a model, the canonical `schemas/Aurora.compact.schema.json`, if present, or the `.github/instructions/Aurora.compact.schema.json` must be copied into the `aurora` folder before creating the first `Mission` card.

**Example Folder and File Structure**:

```text
aurora
  ├─ Aurora.schema.json
  ├─ MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json
  ├─ MIS-002-Write_User_Documentation_for_Aurora.json
  ├─ MIS-001
  │    ├─ Driver
  │    │    ├─ DRI-001.json
  │    │    └─ DRI-002.json
  │	   └─ Requirement	 
  │	   	    └─ REQ-001.json
  ├─ MIS-002
  │    ├─ Driver
... etc
```

An entire model home may be stored in a ZIP-compressed file to allow for portability as long as the folder structure is preserved.

### Logical Structure

The logical structure of Aurora is designed to be easily extended to meet the needs of any architecture. While the structure and rules here are inviolate, they do not limit what is represented in the model and impose only necessary limitations on links.

By using `Boundary` cards and card subtypes almost any structure can be mapped onto a model.

### Invariant Rules

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links, and serves as the root of a directed graph.

2. **Direction (graph links)**: All links must lead away from the `Mission` card. There must be a route from `Mission` to every card. The graph is not acyclic; local loops can and often do exist, for example when a state model returns to the starting state. Traversing any path starting from `Mission` must have an increasing number of links and end either in a leaf card or a previously seen card (local loop).

3. **Hierarchy**: The model forms a directed graph rooted in the `Mission` card. Local cycles are allowed for bounded subgraphs such as state machines (for example: `State` → `Condition` → `State`) and event/action-driven transitions, as long as no link creates a path back to `Mission`. This ensures that all loops terminate locally.

4. **Semantics**: links have a `relationship` field that describes them but does not convey semantic meaning by itself. The relationship is meant to describe how one element impacts another, primarily for humans. Semantic meaning is derived from the link and relationship when interpreted for a purpose, such as generating a view, performing impact analysis, or producing traceability narratives.

5. **Every other card**: Other than the `Mission` card, all cards must have at least one incoming link. They must also have a path from the `Mission` card. These cards may have more than one incoming or any number of outgoing links.

6. **Validation**: All `links[].target` values must reference existing cards by `id`.

## Views

Views are generated by selecting a set of card types (and optionally subtypes) to include and rendering a diagram showing those cards, and the links between them. Views **do not change the model**; they change what part of and how the model is viewed. One model, many views.

In general, a view should display the `card_type`, `card_subtype`, and `name` fields as the text for each node.

## Default Tooling

### `aurora_cli`

- Validates models
- Generates human readable Markdown copies
- Generates Markdown files containing views
- Bumps the major, minor, and patch versions (for manual edits)
- Generates the compact model files

### Mermaid Rules

Every Mermaid diagram must start with the following line before the diagram type line that enables the "Elk" layout engine and makes subgraphs transparent (for boundaries):

```text
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
```

Mermaid diagrams should use the "graph LR" diagram type. The diagram text should be separated into ordered sections separated by a single blank line:

1. The node definitions, named for the `id` of the card.
2. The links between cards.
3. The `classDef` entries (specified below)
4. The `class` assignments (specified below)

Graph nodes should have their text wrapped in Mermaid style Markdown quoting, specifically a quotation mark, a grave, the text, another grave, and a closing quotation mark. This enables the use of bold and line breaks. Node text should include the `card_type` in bold, a line break (`<br />`) and the `name` of the card.

**Graph Node Format**:

```text
	{id}["`**{card_type}**<br />{name}`"]
```

**Example Graph Node**:

```text
	MIS-001(("`**Mission**<br />Enable_Deterministic_Aurora_CLI_Tooling`"))
```

Use tabs for indentation. Every line after the diagram type should be indented at least one tab. Elements in subgraphs should be indented an additional tab.

Every Mermaid diagram must include the appropriate `classDef` entries from the following list, along with `class` lines assigning cards to the appropriate entry. Use the single line form of `class`, e.g., `class REQ-001,REQ-002 cls_requirement;`:

```text
	classDef cls_boundary stroke-dasharray:5 5,stroke-width:4;
	classDef cls_mission fill:#022c22,color:#FFFFFF
	classDef cls_driver fill:#064e3b,color:#FFFFFF
	classDef cls_requirement fill:#065f46,color:#FFFFFF
	classDef cls_capability fill:#052e16,color:#FFFFFF
	classDef cls_feature fill:#14532d,color:#FFFFFF
	classDef cls_actor fill:#1a2e05,color:#FFFFFF
	classDef cls_story fill:#365314,color:#FFFFFF;
	classDef cls_condition fill:#422006,color:#FFFFFF
	classDef cls_control fill:#713f12,color:#FFFFFF
	classDef cls_constraint fill:#854d0e,color:#FFFFFF;
	classDef cls_system fill:#172554,color:#FFFFFF
	classDef cls_application fill:#1e3a8a,color:#FFFFFF
	classDef cls_component fill:#1e40af,color:#FFFFFF
	classDef cls_interface fill:#082f49,color:#FFFFFF
	classDef cls_artifact fill:#1e293b,color:#FFFFFF
	classDef cls_asset fill:#334155,color:#FFFFFF;
	classDef cls_data_store fill:#075985,color:#FFFFFF
	classDef cls_test fill:#022c22,color:#FFFFFF;
	classDef cls_deployment fill:#1e1b4b,color:#FFFFFF;
	classDef cls_node fill:#312e81,color:#FFFFFF;
	classDef cls_node_instance fill:#3730a3,color:#FFFFFF;
	classDef cls_process fill:#2e1065,color:#FFFFFF
	classDef cls_activity fill:#4c1d95,color:#FFFFFF
	classDef cls_event fill:#5b21b6,color:#FFFFFF
	classDef cls_state_machine fill:#4a044e,color:#FFFFFF
	classDef cls_state fill:#701a75,color:#FFFFFF
	classDef cls_risk fill:#881337,color:#FFFFFF;
	classDef cls_threat fill:#4c0519,color:#FFFFFF;
	classDef cls_note fill:#1f2937,color:#FFFFFF;
```
