# View Rendering Architecture

## Purpose

This document defines the ownership boundary for Aurora view rendering and the live Graphviz-backed flow used by all tooling.

## Architectural Placement

- The model remains a directed graph rooted at a `Mission` card.
- View definitions live in the model home configuration and are snapshotted per model home.
- `aurora_shared` owns traversal, layout selection, and SVG emission.
- The CLI, editor, and MCP server invoke the shared renderer and do not implement their own layout engine.

## Supported Layout Families

The live renderer supports only two families from `docs/design/ViewLayouts.md`:

- **TreeTopDown** mapped to `dot` with `rankdir=TB`.
- **TreeLeftRight** mapped to `dot` with `rankdir=LR`.

If a configured view omits a family, the shared renderer defaults to `TreeTopDown`.

## Rendering Flow

1. Resolve the view definition from the model home configuration.
2. Select viable roots and traverse the model to build the rendered subgraph.
3. Choose the configured family, or evaluate the two supported families when best-fit selection is needed.
4. Delegate layout and route generation to the Graphviz-backed renderer in `aurora_shared`.
5. Emit the SVG artifact for the caller.

## Determinism

Rendering is deterministic for fixed inputs. The shared renderer must produce the same result for the same model home, view definition, and renderer version.

## Cross References

- Layout family contract: `docs/design/ViewLayouts.md`
- Editor rendering requirements: `docs/design/AuroraEditor.md`
- MCP rendering requirements: `docs/design/AuroraMCP.md`
