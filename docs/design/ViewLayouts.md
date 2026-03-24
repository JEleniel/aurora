# View Layout Requirements

Aurora's current renderer supports only two layout families. Layout selection and routing are handled by `aurora_shared` through Graphviz-backed rendering.

## Supported Layout Families

- **TreeTopDown**: top-to-bottom tree layout, mapped to `dot` with `rankdir=TB`.
- **TreeLeftRight**: left-to-right tree layout, mapped to `dot` with `rankdir=LR`.

Legacy aliases such as `vertical` and `horizontal` remain accepted by the parser for compatibility.

## Rendering Contract

- Layout selection is deterministic for a fixed model and view definition.
- If a configured view does not specify a family, the shared renderer defaults to `TreeTopDown`.
- When the shared renderer evaluates families for a best-fit render, it only considers the two supported families above.
- No radial, circular, or other layout families are currently supported by the live renderer.

## Cross References

- Rendering ownership and boundaries: `docs/design/ViewRenderingArchitecture.md`
- Editor and MCP rendering requirements: `docs/design/AuroraEditor.md`, `docs/design/AuroraMCP.md`
