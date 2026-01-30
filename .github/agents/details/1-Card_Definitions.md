# Card Definitions

This page defines the Aurora card palette and how each card is typically used.

## Cards

| Card Type     | Acronym | Shape            | Icon           | Description                                                                                                                       | Fill    | Color   |
| ------------- | ------- | ---------------- | -------------- | --------------------------------------------------------------------------------------------------------------------------------- | ------- | ------- |
| Actor         | ACT     | ellipse          | user           | External role interacting with or obligating the system.                                                                          | #1a2e05 | #FFFFFF |
| Trigger       | TRG     | hexagon          | alarm          | Event or condition initiating a process or action.                                                                                | #78350f | #FFFFFF |
| ADR           | ADR     | note             | file-text      | Architectural Decision Record capturing a significant decision, alternatives considered, rationale, trade-offs, and consequences. | #1f2937 | #FFFFFF |
| Application   | APP     | component        | app-window     | Deployable software system implementing features.                                                                                 | #1e3a8a | #FFFFFF |
| Artifact      | ART     | tab              | file           | Concrete work product produced or consumed by the system.                                                                         | #1e293b | #FFFFFF |
| Asset         | AST     | box              | shield-key     | Valuable information/resources requiring protection.                                                                              | #334155 | #FFFFFF |
| Boundary      | BND     | cluster (dashed) | square-dashed  | Logical grouping of other cards; `End` subtype acts as a recursion limit to end a boundary explicitly.                            | #FFFFFF | #FFFFFF |
| Capability    | CAP     | rounded box      | spark          | Implementation-independent ability that satisfies requirements.                                                                   | #052e16 | #FFFFFF |
| Class         | CLS     | record           | braces         | Represents a programming class.                                                                                                   | #0f172a | #FFFFFF |
| Component     | COM     | box3d            | cube           | Modular unit with a single responsibility and explicit interfaces.                                                                | #1e40af | #FFFFFF |
| Condition     | CON     | diamond          | split          | Binary predicate influencing behavior.                                                                                            | #422006 | #FFFFFF |
| Constraint    | CNS     | box              | ruler          | Rule limiting allowable behavior or solutions.                                                                                    | #082f49 | #FFFFFF |
| Control       | CTL     | octagon          | lock           | Mechanism governing or constraining a process.                                                                                    | #713f12 | #FFFFFF |
| Data Store    | DTS     | cylinder         | database       | Persistent resource for durable storage/retrieval.                                                                                | #075985 | #FFFFFF |
| Deployment    | DEP     | box              | cloud          | Defined environment or configuration where software executes.                                                                     | #1e1b4b | #FFFFFF |
| Driver        | DRI     | rounded box      | compass        | Motivation explaining why requirements exist.                                                                                     | #064e3b | #FFFFFF |
| Event         | EVT     | ellipse          | bolt           | Discrete occurrence initiating a process or triggering behavior.                                                                  | #5b21b6 | #FFFFFF |
| Feature       | FEA     | box              | star           | Externally observable behavior realizing one or more requirements.                                                                | #14532d | #FFFFFF |
| Interface     | INT     | note             | plug           | Contract governing interaction across a boundary.                                                                                 | #102080 | #000000 |
| Mission       | MIS     | double-octagon   | target         | Root intent that spawns every downstream driver.                                                                                  | #022c22 | #FFFFFF |
| Node          | NOD     | box              | server         | Logical/physical execution environment hosting components or stores.                                                              | #312e81 | #FFFFFF |
| Node Instance | NIN     | box              | server-cog     | Concrete runtime realization of a node.                                                                                           | #3730a3 | #FFFFFF |
| Note          | NOT     | note             | note-sticky    | Non-structural annotation attached to another element without a verb.                                                             | #1f2937 | #FFFFFF |
| Process       | PRO     | box              | workflow       | Ordered sequence of activities/decisions enabling a capability.                                                                   | #2e1065 | #FFFFFF |
| Predicate     | PRD     | diamond          | function       | Logical statement evaluating to true/false in a state machine.                                                                    | #581c87 | #FFFFFF |
| Activity      | ATV     | rounded box      | play-circle    | Discrete step within a process.                                                                                                   | #4c1d95 | #FFFFFF |
| Requirement   | REQ     | rounded box      | checklist      | Verifiable statement of need/obligation.                                                                                          | #065f46 | #FFFFFF |
| Risk          | RIS     | octagon          | alert-triangle | Potential for loss/harm from threats/vulnerabilities.                                                                             | #881337 | #FFFFFF |
| State         | STA     | circle           | dot            | Observable condition that persists until a transition.                                                                            | #701a75 | #FFFFFF |
| State Machine | STM     | octagon          | timeline       | Behavioral model defining allowable states/transitions.                                                                           | #4a044e | #FFFFFF |
| Story         | STR     | cds              | book           | Narrative expressing desired behavior/outcome; should follow the “`As an ___ I want/need ___.`” format.                           | #365314 | #FFFFFF |
| System        | SYS     | folder           | layers         | Bounded collection of interacting applications that fulfill a mission.                                                            | #172554 | #FFFFFF |
| Test          | TES     | hexagon          | beaker         | Procedure verifying that a feature satisfies requirements.                                                                        | #022c22 | #FFFFFF |
| Threat        | THR     | octagon          | bug            | Potential cause of an unwanted impact on the system/mission.                                                                      | #4c0519 | #FFFFFF |

### Special cards

Each special card follows the standard card fields plus the exceptions noted below:

#### `Mission`

- Purpose: Root of the model that captures the overarching purpose.
- Required fields: Standard card fields.
- Allowed links: Outgoing only; no incoming links.

#### `Boundary` (BND)

- Purpose: Logical grouping of other cards.
- Optional subtype "End" indicated the limit of recursion when rendering a boundary of the same name.
- Required fields: Standard card fields; optional `attributes.recursive` boolean (default false).
- Allowed links: Parent includes the `Boundary`; the `Boundary` contains one or more target cards, in parallel to another link between parent and target.
- Exception: When `recursive` is true, the boundary includes all descendants of the target in the current view; traversal must avoid loops.

#### `Note` (NOT)

- Purpose: Annotation on another card; does not add new model elements.
- Required fields: Standard card fields.
- Allowed links: Incoming only; always a leaf node.
