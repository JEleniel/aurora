# Relationship Definitions

This page defines the canonical relationship registry used by Aurora tooling.

## Relationships

| Relationship | Description | Source card types | Target card types |
| --- | --- | --- | --- |
| `invokes` | Invokes an interface by contract. | `Component` | `Interface` |
| `composes` | Defines a composition relationship. | `Application` | `Component` |
| `contains` | Defines boundary containment. | `Boundary` | `!Mission` |
| `provisions` | Provisions runtime infrastructure. | `Deployment` | `Node` |
| `desires` | Expresses a goal or story. | `Actor` | `Story` |
| `documents` | Records a decision tied to a requirement. | `ADR` | `Requirement` |
| `drives` | Influences a downstream requirement. | `Driver` | `Requirement` |
| `enables` | Makes a capability feasible. | `Feature` | `Capability` |
| `establishes` | Introduces a downstream driver. | `Mission` | `Driver` |
| `exposes` | Publishes an interface. | `Data Store` | `Interface` |
| `generates` | Produces an artifact. | `Component` | `Artifact` |
| `instantiates` | Creates a runtime instance. | `Node` | `Node Instance` |
| `hosts` | Hosts a runtime component. | `Node` | `Component` |
| `hosts` | Hosts a runtime data store. | `Node` | `Data Store` |
| `implements` | Implements a feature. | `Component` | `Feature` |
| `realizes` | Realizes a class definition. | `Component` | `Class` |
| `fulfills` | Fulfills a test. | `Component` | `Test` |
| `enforces` | Applies a control. | `Component`, `System` | `Control` |
| `implies` | Implies a constraint. | `Story` | `Constraint` |
| `includes` | Scopes an element within a boundary. | `*` | `Boundary` |
| `integrates` | Integrates applications into a system. | `System` | `Application` |
| `involves` | Includes an actor in a mission. | `Mission` | `Actor` |
| `is` | Classifies an artifact as an asset. | `Artifact` | `Asset` |
| `mitigates` | Reduces a risk. | `Control` | `Risk` |
| `requires` | Requires a system or application. | `Mission` | `System`, `Application` |
| `owns` | Defines ownership responsibility. | `Actor` | `Asset` |
| `performs` | Executes an activity. | `Actor` | `Activity` |
| `persists` | Persists an artifact to storage. | `Artifact` | `Data Store` |
| `presents` | Introduces a threat. | `Actor` | `Threat` |
| `raises` | Raises a risk. | `Threat` | `Risk` |
| `safeguards` | Protects an asset. | `Control` | `Asset` |
| `necessitates` | Necessitates a process. | `Capability` | `Process` |
| `executes` | Executes and owns a state machine. | `Component` | `State Machine` |
| `satisfies` | Satisfies a requirement. | `Capability` | `Requirement` |
| `starts in` | Defines the initial state. | `State Machine` | `State` |
| `transitions to` | Transitions to a next state. | `State`, `Predicate`, `Event` | `State` |
| `evaluates` | Evaluates a condition. | `Activity` | `Condition` |
| `emits` | Emits an event. | `Activity` | `Event` |
| `leads` | Leads to another activity. | `Activity` | `Activity` |
