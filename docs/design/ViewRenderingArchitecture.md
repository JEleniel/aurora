# View Rendering Architecture

## Purpose

This document integrates `ViewLayouts.md` into the Aurora project architecture by defining where view layout and edge routing requirements live, which components own them, and how all Aurora tooling consumes them.

Aurora’s model invariants define which graphs exist and how they can be traversed. View rendering defines how selected subgraphs are turned into deterministic, readable diagrams.

## Architectural placement

In Aurora terms:

- The model is a directed graph rooted at a `Mission`, with local cycles only.
- Views are interpretations of that model:
    - View definitions (roots + included card types) live in `.github/aurora/**.*` and are snapshotted into each model home.
    - View traversal selects the rendered subgraph by starting at each viable root and traversing until a leaf or local loop.
- Rendering turns the selected subgraph into diagram artifacts (for example SVG) without changing the model.

### Normative vs rendering requirements

- **Normative (model validity):** schemas, registries, and invariants (root safety, no orphans, local cycles only).
- **Rendering (diagram quality):** layout selection, depth assignment, packing, and routing constraints.

`docs/design/ViewLayouts.md` is the design specification for rendering behavior (layout, routing, scoring), not for model validity.

## Component responsibilities

Aurora’s rendering behavior is shared across tooling. The intended ownership boundaries are:

- `aurora_shared` (library)
    - Owns deterministic view traversal and view rendering primitives.
    - Implements layout selection, scoring, and routing according to `ViewLayouts.md`.
- CLI / Editor / MCP server
    - Invoke `aurora_shared` rendering primitives.
    - Provide UX / API surfaces (progress, cancellation, configuration) but do not re-implement layout/routing.

This keeps view output consistent across environments and prevents “same model, different diagram” drift.

## Rendering pipeline (logical)

1. **Resolve view definition** (roots + included card types) from the model home’s configuration.
2. **Select viable roots** per the root safety rule.
3. **Traverse from roots** to select the subgraph to be rendered.
4. **Lay out the graph** (choose depth assignment, ordering strategy, and packing) using the scoring rules defined in `ViewLayouts.md`.
5. **Route edges orthogonally** inside the node perimeter using the channel sharing rules, internal gutters/highways, and line-jump rules defined in `ViewLayouts.md`.
6. **Emit artifacts** (SVG and any auxiliary debug artifacts as needed).

## Determinism and stability

Aurora is deterministic by design. View rendering MUST be deterministic under fixed inputs:

- Same model home content + same view definition + same renderer version MUST produce identical diagrams.
- `id` ordering is the default stabilizer.
- Heuristics may be used for fine tuning but MUST remain deterministic (fixed pass count, stable tie-breakers).

## Cross references

- Rendering/layout/routing requirements: `docs/design/ViewLayouts.md`
- View definitions registry: `.github/aurora/**.*`
- Editor requirements (including view root safety): `docs/design/AuroraEditor.md`
- MCP server requirements: `docs/design/AuroraMCP.md`
