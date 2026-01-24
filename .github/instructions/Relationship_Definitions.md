# Relationship Definitions

This page defines the canonical set of Aurora relationship verbs. Each relationship is listed with its typical intent and the allowed source and target card types.

## Relationships

| Relationship | Category | Description | Source card types | Target card types | Notes |
| --- | --- | --- | --- | --- | --- |
| `establishes` | Motivation | Introduces a downstream motivation element. | Mission | Driver | Common: `Mission establishes Driver`. |
| `drives` | Motivation | Influences a downstream need. | Driver | Requirement; Capability; Feature; Constraint | Recommended: use `drives` for “why → what must be true”. |
| `necessitates` | Motivation | Higher-level goal requires an element. | Mission | System; Application; Capability; Feature; Process | Recommended: use for “must exist to fulfill the mission”. |
| `satisfies` | Realization | Fulfills a need. | Capability; Feature | Requirement | Recommended: “capability/feature satisfies requirement”. |
| `enables` | Realization | Makes another feasible. | Feature; System; Application; Component | Capability; Feature | Use for prerequisite capabilities (non-obligation links). |
| `implements` | Realization | Realizes another element. | Application; Component | Feature | Preferred for executable software realizing a feature. |
| `exposes` | Realization | Provides an interface. | System; Application; Component | Interface | Use when a deployable unit publishes a contract/boundary. |
| `generates` | Realization | Produces an artifact. | Feature; Application; Process | Artifact | Common: `Feature generates Artifact` (rendering views, docs). |
| `uses` | Realization | Consumes or depends on another element. | All card types | All card types | Intentionally broad; prefer more specific verbs when they fit. |
| `validates` | Realization | Verifies behavior. | Test | Requirement; Constraint; Feature; Capability; Control | Conformance/fitness functions are tests that validate constraints/requirements. |
| `contains` | Containment and Composition | Structural containment. | System; Application; Component; Boundary | System; Application; Component; Interface; Artifact; Data Store; Process; Activity | Use when the parent “has” the child as part of its structure. |
| `includes` | Containment and Composition | Membership without containment. | All card types | All card types | Use for grouping/membership when `contains` is too strong (e.g., parent includes a Boundary). |
| `integrates` | Containment and Composition | Pulls together sub-elements. | System; Application; Component | Application; Component; Interface; Data Store; Artifact | Use for “brings together” semantics across sub-elements. |
| `comprises` | Containment and Composition | Composition. | System; Application; Component; Boundary | Component; Interface; Artifact; Data Store | Use when the child is an integral part of the parent. |
| `participates_in` | Runtime and Topology | Involvement in a topology or deployment. | System; Application; Component; Data Store; Node; Node Instance | Deployment; Node | Use for “is part of” a deployment/topology slice. |
| `hosts` | Runtime and Topology | Runtime host relationship. | Node; Node Instance | Application; Component; Data Store | Use for “runs on”. |
| `instantiates` | Runtime and Topology | Creates a runtime instance. | Node | Node Instance | Use for mapping logical nodes to concrete instances. |
| `to_call` | Runtime and Topology | Invocation path. | Component; Application; Activity | Interface; Component; Application | Use to model call edges in comms/sequence views. |
| `persists_to` | Runtime and Topology | Persistence to storage. | Artifact; Asset; Application; Component; Feature | Data Store | Use for durable storage relationships (including generated artifacts). |
| `involves` | Behavior and Flow | Process or mission includes an actor. | Process; Mission | Actor | Use to relate participants to flows. |
| `desires` | Behavior and Flow | Actor has a story or goal. | Actor | Story | Used to avoid orphan stories and to tie goals to roles. |
| `performs` | Behavior and Flow | Actor executes an activity. | Actor | Activity | Used in process/activity views to show who does what. |
| `explains` | Behavior and Flow | Story elaborates on a requirement, capability, or feature. | Story | Requirement; Capability; Feature; Constraint | Recommended: stories should explain at least one requirement. |
| `implies` | Behavior and Flow | Story suggests a constraint. | Story | Constraint | Use when a narrative implies a policy/NFR/guardrail. |
| `starts_with` | Behavior and Flow | First event of a process. | Process | Event | Used to anchor process flows. |
| `receives` | Behavior and Flow | State receives an event. | State | Event | Used to model event-driven transitions. |
| `starts_in` | Behavior and Flow | Initial state of a state machine. | State Machine | State | Use exactly once per state machine when modeling an initial state. |
| `transitions_to` | Behavior and Flow | Transition edge. | State | State | Use alongside `receives`/`Condition`/`triggers_*` when needed. |
| `triggers` | Behavior and Flow | Event or activity triggers another element. | Event; Activity | Event; Activity; Condition; Control; State | Use for causal edges in process/state models. |
| `triggers_true` | Behavior and Flow | Conditional branching outcome (true). | Condition | Event; Activity; State | Pair with `triggers_false` for binary branching. |
| `triggers_false` | Behavior and Flow | Conditional branching outcome (false). | Condition | Event; Activity; State | Pair with `triggers_true` for binary branching. |
| `owns` | Ownership and Accountability | Responsible owner of an element (execution/doing). | Actor | All card types | Express RACI-style responsibility as explicit links. |
| `oversees` | Ownership and Accountability | Accountable owner of an element (answerable/outcome). | Actor | All card types | Express RACI-style accountability as explicit links. |
| `facilitates` | Ownership and Accountability | Involved/supporting role that helps execution without owning outcomes. | Actor | All card types | Express RACI-style support as explicit links. |
| `constrains` | Risk and Control | A constraint bounds another element (preferred when the source is a `Constraint`). | Constraint | Requirement; Capability; Feature; Process; Activity; Interface; Data Store; Asset; System; Application; Component; Test; Deployment; Node | Use this verb when modeling “Constraint applies to X”. |
| `limits` | Risk and Control | Bounds another element. | Constraint | All card types | Prefer `constrains` when the source is a `Constraint` and the intent is “applies to”. |
| `governs` | Risk and Control | A control applies to an element. | Control | Process; Activity; Interface; Deployment; System; Application; Component; Data Store; Asset | Use for enforcement points and gates. |
| `presents` | Risk and Control | Actor presents a threat. | Actor | Threat | Often used for threat actors. |
| `imposes` | Risk and Control | Threat introduces a risk. | Threat | Risk | Use for threat → risk linkage. |
| `impacts` | Risk and Control | Risk affects another element. | Risk | Mission; Requirement; Asset; System; Application; Component; Data Store; Deployment | Use for “what could be harmed”. |
| `mitigates` | Risk and Control | Feature/capability/control addresses a risk or threat. | Control; Feature; Capability | Risk; Threat | Use for mitigations (technical or procedural). |
| `is a` | Low Level Design | Inheritance / specialization relationship. | Class | Class | Example: `AuthenticationService is a Service`. |
| `aggregates` | Low Level Design | Holds a reference to another class. | Class | Class | Use for has-a (non-owning) references. |
| `composes` | Low Level Design | Is an integral part of (strong ownership). | Class | Class | Use for owned lifetimes/containment. |
| `depends on` | Low Level Design | Is dependent upon. | Class | Class | Use for dependency edges (often at compile-time or runtime). |
