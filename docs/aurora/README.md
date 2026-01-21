# Aurora

Aurora is a deterministic, JSON-based approach to modeling requirements and architecture as a directed graph.

Instead of drawing diagrams directly, you author a set of small JSON “cards” and “links” between them. Tooling (for example `aurora_cli`) validates the model and renders diagrams and documentation.

## Navigation

- [Concepts: cards, links, graph](concepts.md)
- [Model and folder layout](model-layout.md)
- [Cards and fields](cards.md)
- [Links and relationships](links-and-relationships.md)
- [Invariant rules](invariants.md)
- [Views and rendering](views.md)
- [Mermaid rendering rules](mermaid-rules.md)
- [Getting started](getting-started.md)
- [Design docs index](../design/README.md)

## Source of truth

This documentation is a human-oriented guide. The normative (machine- and agent-oriented) guidance lives in:

- [Aurora.instructions.md](../../.github/instructions/Aurora.instructions.md)

## One idea to remember

Aurora is a directed graph rooted at exactly one Mission card.

- The Mission is the entry point.
- Every other card must be reachable from the Mission by following links.
- Loops are allowed locally (for example in state machines), but nothing can link back to Mission.
