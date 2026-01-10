# Views

Views are constructed by selecting a set of card types (and optionally subtypes) to include and creating a diagram showing those cards, and the links between them.

Our tools add a few enhancement to improve the look and readability of the diagrams:

- Mermaid shapes are used to help distinguish different types of cards.
- Boundaries are rendered as a dashed outline around the elements they contain

> **Boundaries**: `boundary` cards frequently "contain" `system`, `application`, `node`, `process`, `state_machine`, and similar higher level cards. They are linked from a parent of and to the contained card, e.g., parent --> boundary --> contained. They also include the attribute "recursive" which, if true, includes all of the children of the contained card, recursively, _that are in the current view_.

## Examples of Common Views

- [**Component View**](./Component.md) - A Component View shows the system's runtime and logical components, their public interfaces, and how they compose and depend on each other to realize features and services.

- [**Deployment View**](./Deployment.md) - A Deployment View shows the hosting topology—nodes, boundaries, and where components and applications are deployed.

- [**Everything View**](./Everything.md) - A view unique to Aurora, it is exactly what the name implies; it includes all cards and all relationships. For anything more than simple architectures, this can be quite large and complex.

- [**Requirements View**](./Requirements.md) - A Requirements View captures the system goals and constraints as testable, traceable requirements, showing their relationships to drivers, features, and tests.

- [**State Machine View**](./StateMachine.md) - A State Machine View models the lifecycle and valid transitions of a runtime element, showing its states, the conditions that guard transitions, events that trigger changes, and activities that occur within states.

## Unnecessary Views

- **Package View** - A Component View can easily use `boundary` cards, subtype `package`, to show the same thing this view would.

- **Class View** - The `component` card can be at any level, can have the subtype `class`, and the `"attributes"` property can contain a list of properties, methods, etc.

- **Sequence View** - State Machine View and Story View already capture the sequencing of events. Should it be necessary, a sequence could be generated from the model.

- **Communication View** - The State Machine View already captures this information. Should it be necessary, one could be generated from the model.
