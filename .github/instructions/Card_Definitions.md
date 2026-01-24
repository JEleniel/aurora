# Card Definitions

This page defines the Aurora card palette and how each card is typically used, including the common subuses (subtypes) referenced in [View Definitions](View_Definitions.md). This is the canonical list.

## Cards

| Card type | Description (from instructions) | Common usage | Common subuses (subtypes) referenced in View Definitions | Common root usage (examples) |
| --- | --- | --- | --- | --- |
| Mission (MIS) | Root intent that spawns every downstream driver. | Root of the directed graph; primary “why” anchor; often used for roll-ups and summaries. | N/A | Executive Summary View; Cost Model / FinOps View |
| Driver (DRI) | Motivation explaining why requirements exist. | Links intent to concrete needs; used for traceability from mission to requirements/features. | N/A | Requirements View; Initiative/Epic Portfolio View |
| Requirement (REQ) | Verifiable statement of need/obligation. | What must be true; primary target for validation and traceability; commonly constrained and satisfied. | N/A | Requirements View; Test Strategy and Traceability View |
| Capability (CAP) | Implementation-independent ability that satisfies requirements. | “What we can do” regardless of implementation; map to features/systems; good for capability maps. | N/A | Capability Map and Heatmap |
| Feature (FEA) | Externally observable behavior realizing one or more requirements. | User-visible or operator-visible behaviors; often implemented by applications/components; generates artifacts. | N/A | (Typically rooted via Requirement/Driver; not commonly a primary root) |
| System (SYS) | Bounded collection of interacting applications that fulfill a mission. | Top-level structural boundary; used in component/context/security/dependency views. | N/A | Component View; Context View |
| Application (APP) | Deployable software system implementing features. | Deployable units; appear in structure, deployment, and interaction views. | N/A | Component View |
| Component (COM) | Modular unit with a single responsibility and explicit interfaces. | Primary building block inside apps; nested composition; participates in interactions and dependency graphs. | N/A | Component View; Communication View |
| Interface (INT) | Contract governing interaction across a boundary. | Defines call boundaries; used to model dependencies, message flows, and trust boundaries. | N/A | Communication View; Component View |
| ADR (ADR) | Architectural Decision Record capturing a significant decision, alternatives considered, rationale, trade-offs, and consequences. | Records decisions and ties them to impacted requirements/constraints/components. | N/A | Decision Log View |
| Artifact (ART) | Concrete work product produced or consumed by the system. | Durable outputs (rendered views, docs, schemas, examples); commonly persisted and generated; also used as references to external systems. | Object Snapshot; Data Entity; Data Element; Message; Metric; Initiative/Epic; Roadmap | Object View; Data Model View; Observability View |
| Asset (AST) | Valuable information/resources requiring protection. | Security/privacy classification; what threats/risks impact; what controls protect; can be persisted or handled by components. | Secret | Configuration and Secrets View; Privacy and Data Classification View |
| Data Store (DTS) | Persistent resource for durable storage/retrieval. | Where data/artifacts persist; appears in data model, deployment, and component views. | N/A | Data Model View |
| Deployment (DEP) | Defined environment or configuration where software executes. | Environment-level grouping (dev/test/prod); host relationships to nodes; drives environment views. | N/A | Deployment View; Environment View |
| Node (NOD) | Logical/physical execution environment hosting components or stores. | Hosts apps/components/stores in a deployment; used for topology. | N/A | Deployment View |
| Node Instance (NIN) | Concrete runtime realization of a node. | Concrete “where it runs” instance (hosts, clusters, specific machines). | N/A | Deployment View |
| Process (PRO) | Ordered sequence of activities/decisions enabling a capability. | High-level workflow; value streams, pipelines, runbooks are modeled as processes. | N/A | Process (Activity) View; Value Stream View |
| Activity (ATV) | Unit of behavior performed by an actor inside a process. | Steps within a process; used for pipelines/runbooks/flows; can call interfaces and trigger events. | N/A | Process (Activity) View; Sequence View |
| Actor (ACT) | External role interacting with or obligating the system. | Roles/users/systems-as-actors; threat actors; third parties; ownership (owns/oversees/facilitates). | N/A | Use Case View; Threat Model View |
| Story (STR) | Narrative expressing desired behavior/outcome. | Captures user goals and abuse/misuse narratives; explains requirements; roots use-case slices. | Abuse | Use Case View; Abuse/Misuse Case View |
| Event (EVT) | Discrete occurrence initiating a process or triggering behavior. | Triggers steps/transitions; used for time/timer modeling (ticks) and roadmap timing when needed. | N/A | Timing View; Roadmap View |
| State Machine (STM) | Behavioral model defining allowable states/transitions. | Lifecycles for assets/systems/accounts; “lifecycle view” is a state machine view. | N/A | State Machine View; Lifecycle/State View for Assets |
| State (STA) | Observable condition that persists until a transition. | Lifecycle states; used with events/conditions for transitions. | N/A | State Machine View |
| Condition (CON) | Binary predicate influencing behavior. | Branching, guards, timer expiry predicates; often paired with events and state transitions. | N/A | Timing View; Process (Activity) View |
| Control (CTL) | Mechanism governing or constraining a process. | Security and operational controls; gates in pipelines; protections mapped to threats/risks/assets. | N/A | Control Coverage View; Security Architecture View |
| Constraint (CNS) | Rule limiting allowable behavior or solutions. | Policies, standards, invariants, schema rules, and SLO/SLA targets; validated by tests/fitness functions. | Schema; SLO/SLA | Policy and Standards Compliance View; Resilience View |
| Risk (RIS) | Potential for loss/harm from threats/vulnerabilities. | Risk register entries; linked to impacted assets and mitigations/controls. | N/A | Risk Register View |
| Threat (THR) | Potential cause of an unwanted impact on the system/mission. | Threat modeling; linked to risks and mitigations; used with threat actors and attack steps. | N/A | Threat Model View; Abuse/Misuse Case View |
| Test (TES) | Procedure verifying that a feature satisfies requirements. | Validates requirements/constraints; conformance and fitness functions are tests with clear pass/fail criteria. | N/A | Test Strategy and Traceability View; Fitness Function View |
| Note (NOT) | Non-structural annotation attached to another element. | Reference/pattern notes; extra-model context; included automatically when linked from in-scope cards. | Pattern | Reference Architecture / Patterns Library View |
| Class (CLS) | Represents a programming class. | Implementation-level structure; supports modeling properties/methods/cardinality via `attributes`. | N/A | Class View |
| Boundary (BND) | Logical grouping of other cards. | Defines logical boundaries (packages, bounded contexts, trust zones); used to scope views and group subgraphs. | Package; Context | Package View; Domain and Bounded Context View |
