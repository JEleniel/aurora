# View Definitions

This page defines the canonical view registry used by Aurora tooling.

## Views

| View | Description | Root card types | Included card types | Optional card types |
| --- | --- | --- | --- | --- |
| Requirements | Captures mission intent, motivation, and required capabilities, including architectural decisions that justify scope. | `Mission` | `Mission`, `Driver`, `Capability`, `Requirement`, `ADR`, `Boundary`, `Note` | `Constraint`, `Risk`, `Story` |
| Use Case | Describes externally visible behavior from the perspective of actors using narrative stories and constraints. | `Actor` | `Actor`, `Story`, `Requirement`, `Constraint`, `Boundary`, `Note` | `Capability`, `Feature`, `Risk` |
| Process | Details how a capability is realized through ordered processes and activities with explicit triggering conditions. | `Capability` | `Capability`, `Process`, `Activity`, `Condition`, `Trigger`, `Boundary`, `Note` | `Control`, `Predicate` |
| Behavioral | Models system behavior using state machines, states, predicates, and events without process contamination. | `State Machine` | `State Machine`, `State`, `Predicate`, `Event`, `Boundary`, `Note` | `Condition`, `Control` |
| System Structure | Shows the structural decomposition of the system into applications and components and their provided interfaces. | `System` | `System`, `Application`, `Component`, `Interface`, `Data Store`, `Artifact`, `Feature`, `Test`, `Boundary`, `Note` | `Class`, `Capability` |
| Class | Explains internal implementation structure, focusing on how applications and components are realized in code. | `Application` | `Application`, `Component`, `Class`, `Interface`, `Data Store`, `Artifact`, `Feature`, `Test`, `Boundary`, `Note` | `Constraint` |
| Deployment | Defines where software executes, including environments, nodes, and relevant threats to deployed assets. | `Deployment` | `Deployment`, `Node`, `Application`, `Threat`, `Boundary`, `Note` | `Node Instance`, `Component`, `Data Store` |
| Runtime | Describes concrete runtime realizations of nodes and the components executing within them. | `Node Instance` | `Node Instance`, `Node`, `Component`, `Boundary`, `Note` | `Application`, `Control` |
| Security | Analyzes protection of assets against threats, including actors, constraints, controls, and resulting risks. | `Asset` | `Asset`, `Threat`, `Risk`, `Control`, `Constraint`, `Actor`, `Boundary`, `Note` | `Component`, `Interface` |
| Compliance Governance | Demonstrates adherence to constraints through controls, risk treatment, and verifiable tests. | `Requirement` | `Requirement`, `Constraint`, `Control`, `Risk`, `Test`, `Boundary`, `Note` | `ADR`, `Process` |
| Traceability | Provides end-to-end impact and coverage analysis from requirements through implementation and verification. | `Requirement` | `Requirement`, `Capability`, `Feature`, `Component`, `Interface`, `Test`, `Constraint`, `Control`, `Risk`, `Boundary`, `Note` | `Mission`, `Driver`, `Application`, `Data Store`, `Artifact`, `Asset`, `Threat`, `Process`, `Activity`, `ADR`, `Class` |
| Entire Model | Displays the complete architectural model, highlighting shortest paths from the mission and alternate relationships. | `Mission` | `Mission`, `Driver`, `Capability`, `Requirement`, `Feature`, `System`, `Application`, `Component`, `Interface`, `Process`, `Activity`, `Condition`, `Trigger`, `State Machine`, `State`, `Predicate`, `Event`, `Deployment`, `Node`, `Node Instance`, `Data Store`, `Class`, `Story`, `Actor`, `Constraint`, `Control`, `Risk`, `Threat`, `Asset`, `Artifact`, `Test`, `ADR`, `Boundary`, `Note` | None |
