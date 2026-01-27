# Relationships Matrix

This matrix is derived from the canonical card palette in `.github/instructions/details/Card_Definitions.md` and the relationship registry in `.github/instructions/details/Relationship_Definitions.md`. Use it as a quick reference for allowed relationships, including the expected source and target card types. Relationship verbs are lowercase, active phrases; notes attach without a verb and are not listed here.

| Relationship | Category | Description | Source card types | Target card types | Notes |
| --- | --- | --- | --- | --- | --- |
| `aggregates` | Low Level Design | Holds a reference to another class. | Class | Class | Use for has-a (non-owning) references. |
| `calls` | Runtime and Topology | Invokes a contract. | Component | Interface | Components call interfaces (no passive form). |
| `composes` | Low Level Design | Strong ownership composition. | Class; Component | Class | Classes compose classes; components compose classes. |
| `comprises` | Realization | Composition from application to component. | Application | Component | Applications comprise components. |
| `contains` | Containment and Composition | Structural containment. | Boundary | All card types | Only boundaries contain other elements; a boundary must contain a target. |
| `desires` | Behavior and Flow | Actor has a story or goal. | Actor | Story | Used to avoid orphan stories and to tie goals to roles. |
| `drives` | Motivation | Influences a downstream need. | Driver | Requirement | Drivers only drive requirements. |
| `enables` | Realization | Makes another feasible. | Feature | Capability | Features enable capabilities. |
| `enforces` | Risk and Control | Enforces a constraint. | Control | Constraint | Controls enforce constraints. |
| `establishes` | Motivation | Introduces a downstream motivation element. | Mission | Driver | Common: `Mission establishes Driver`. |
| `explains` | Behavior and Flow | Story elaborates a requirement. | Story | Requirement | Stories explain requirements. |
| `exposes` | Realization | Provides an interface. | Application; Component | Interface | Use when a deployable unit publishes a contract. |
| `facilitates` | Ownership and Accountability | Supporting role. | Actor | Process; Activity | Express RACI-style support. |
| `generates` | Realization | Produces an artifact. | Component; Activity | Artifact | Components and activities generate artifacts. |
| `governs` | Risk and Control | Applies a control to a process. | Control | Process | Use for enforcement points and gates. |
| `hosts` | Runtime and Topology | Runtime host relationship. | Node; Node Instance | Component; Application; Data Store | Nodes and node instances host runtime elements. |
| `impacts` | Risk and Control | Risk affects another element. | Risk | Requirement; Capability; Feature; System; Application; Component; Asset | Use for “what could be harmed”. |
| `implements` | Realization | Realizes a feature. | Component | Feature | Components implement features. |
| `implies` | Behavior and Flow | Story suggests a constraint. | Story | Constraint | Use when a narrative implies a policy/NFR. |
| `imposes` | Risk and Control | Threat introduces a risk. | Threat | Risk | Use for threat → risk linkage. |
| `includes` | Containment and Composition | Membership without containment. | All card types | All card types | Use for grouping; nodes include boundaries; with the exception of Boundary cards, avoid pairing with more specific verbs between the same card types. |
| `instantiates` | Runtime and Topology | Creates a runtime instance. | Node | Node Instance | Used for logical-to-concrete mapping. |
| `integrates` | Realization | Pulls together sub-elements. | Application | Application; Component | Applications integrate components and other applications. |
| `involves` | Behavior and Flow | Includes an actor in a flow. | Mission; Process; Deployment; System; Application | Actor | Missions, processes, deployments, systems, and apps involve actors. |
| `is a` | Low Level Design | Inheritance / specialization relationship. | Class | Class | Example: `AuthenticationService is a Service`. |
| `limits` | Risk and Control | Bounds allowable behavior. | Constraint | Capability; Feature; Process; Application | Constraints limit capabilities, features, processes, and applications. |
| `mitigates` | Risk and Control | Reduces risk. | Control; Feature; Capability | Risk | Controls, features, and capabilities mitigate risks. |
| `necessitates` | Motivation | Higher-level goal requires an element. | Mission | System; Application; Process | Mission may necessitate systems, applications, or processes. |
| `oversees` | Ownership and Accountability | Accountable owner (outcome). | Actor | Process; Activity | Express RACI-style accountability. |
| `owns` | Ownership and Accountability | Responsible owner (execution). | Actor | Process; Activity | Express RACI-style responsibility. |
| `performs` | Behavior and Flow | Executes an activity. | Actor; System; Component; Application | Activity | Actors, systems, components, and apps perform activities. |
| `persists to` | Realization | Stores an artifact in a data store. | Artifact | Data Store | Use for durable storage of artifacts or data entities. |
| `presents` | Risk and Control | Actor presents a threat. | Actor | Threat | Often used for threat actors. |
| `provides` | Runtime and Topology | Supplies nodes for a deployment environment. | Deployment | Node | Deployments provide the nodes they include. |
| `receives` | Behavior and Flow | State receives an event. | State | Event | Used to model event-driven transitions. |
| `reverse proxies` | Runtime and Topology | Accepts traffic on behalf of a downstream service. | Application; Component | Application; Component | Use for ingress, edge, or internal routing layers. |
| `satisfies` | Realization | Fulfills a need. | Capability; Feature | Requirement | Features may satisfy requirements directly. |
| `starts with` | Behavior and Flow | Trigger for a process. | Process | Activity; Event | Processes start with an activity or event. |
| `transitions on false` | Behavior and Flow | Conditional transition when false. | Condition | Condition; State | Use for false-path transitions. |
| `transitions on true` | Behavior and Flow | Conditional transition when true. | Condition | Condition; State | Use for true-path transitions. |
| `triggers` | Behavior and Flow | Event triggers a condition. | Event | Condition | Events trigger conditions. |
| `uses` | Realization | Loose coupling or dependency. | All card types except Mission, Driver | All card types | Use for optional dependencies or weak coupling; avoid pairing with more specific verbs between the same card types. |
| `validates` | Risk and Control | Verifies behavior. | Test; Control | Feature; Capability; Control (test only) | Tests validate features, controls, and capabilities; controls validate features and capabilities. |
