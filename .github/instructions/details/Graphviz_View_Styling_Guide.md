# Graphviz View Styling Guide

Aurora views are rendered via a Graphviz pipeline:

1. The model is traversed to produce a view subgraph.
2. The view subgraph is emitted as Graphviz DOT.
3. Graphviz renders DOT to SVG.
4. The SVG is written to the rendered model output directory.

This document explains how styling works today and how to make it model-driven so humans and agents can change styling without patching renderer code.

## DOT → SVG: what gets generated

When you run `aurora_cli render-views`, the renderer writes two useful artifacts for each view:

- A rendered SVG: `docs/design/<Rendered_Model_Name>/Views/*.view.svg`
- Its DOT source: `docs/design/<Rendered_Model_Name>/Views/source/*.view.dot`

In some docs, these are described in shorthand as `docs/design/Views/*.view.svg` and `docs/design/Views/source/*.view.dot`; interpret that as relative to the rendered model output directory.

The `.view.dot` is the most direct way to debug styling. If an SVG looks unexpected, check whether the DOT contains the attributes you expected, then confirm Graphviz is honoring them.

## Model-driven styling via `CNS-005` (Constraint)

Styling rules should be encoded in the model so they are:

- Deterministic (same model → same styling).
- Reviewable (diffs show what changed).
- Agent-friendly (a single canonical place to read/write conventions).

The intended convention is a `Constraint` card named **“Graphviz DOT View Conventions”** (recommended id: `CNS-005`). This card can carry an `attributes.dot` object that the renderer reads when generating DOT.

If your model does not yet contain `CNS-005`, create it under the mission’s `Constraint/` directory in `docs/design/aurora/`.

### `attributes.dot` shape

The `attributes.dot` object is designed to carry DOT-level defaults and a small amount of per-card-type override mapping:

- `graph_defaults`: default DOT graph attributes (applied to the whole graph).
- `node_defaults`: default DOT node attributes (applied unless a node overrides).
- `edge_defaults`: default DOT edge attributes (applied unless an edge overrides).
- `card_type_symbols`: optional mapping from card type → DOT overrides.
    + Supported fields per card type: `shape`, `fillcolor`, `fontcolor`, `color`.
    + Note: today, `color` is treated as a fallback for `fontcolor`.
- `boundary_cluster`: optional defaults used when rendering `Boundary` cards as subgraphs/clusters.

This structure is intentionally “flat” and JSON-serializable so agents can edit it safely.

### Example `attributes.dot` snippet

This is an illustrative example (not a normative spec and not containing repo secrets):

```json
{
  "dot": {
    "graph_defaults": {
      "bgcolor": "#FFFFFF",
      "fontname": "Inter",
      "rankdir": "LR"
    },
    "node_defaults": {
      "shape": "box",
      "style": "filled",
      "fontname": "Inter",
      "fontsize": "12"
    },
    "edge_defaults": {
      "color": "#000000",
      "fontcolor": "#000000",
      "fontname": "Inter",
      "fontsize": "10"
    },
    "card_type_symbols": {
      "Mission": {
        "shape": "doubleoctagon",
        "fillcolor": "#022c22",
        "fontcolor": "#FFFFFF"
      },
      "Driver": {
        "shape": "box",
        "fontcolor": "#FFFFFF"
      }
    },
    "boundary_cluster": {
      "style": "dashed",
      "color": "#000000"
    }
  }
}
```

Precedence note: the canonical palette in [`View_Definitions.md`](View_Definitions.md) currently takes precedence for card-type fill and font colors.

## Current default styling expectations

Unless overridden by `attributes.dot`, the renderer currently assumes:

- Graph background color: `#FFFFFF`
- Edge line color: `#000000`
- Edge label font color: `#000000`

Node fill colors are driven by the canonical “Color by Card Type” palette in [`View_Definitions.md`](View_Definitions.md).

If you want to adjust node colors, update the palette at the canonical source and re-render.

## Notes on casing: `card_subtype` is user-defined

`card_subtype` values are treated as user-defined strings. The model and renderer do not enforce casing or a fixed vocabulary.

Conventions you should follow for consistency:

- Use Title Case for most `card_type` names (as required by the model).
- Use lower-case for Rust constructs when modeling code-level details.
    + Example: prefer `card_subtype: "struct"` (not `"Struct"`) when describing a Rust struct.

## Subtype rendering in view labels

`card_subtype` is displayed as additional context. When present, it should render as a parenthesized line break after the existing line break in the node label.

Target label snippet:

```text
...<br />
(struct)<br />
...
```

## Practical debugging checklist

- Confirm the view’s DOT exists: `docs/design/<Rendered_Model_Name>/Views/source/*.view.dot`.
- Look for `graph [...]`, `node [...]`, and `edge [...]` default blocks in DOT.
- Confirm per-node fill colors align with the canonical palette in `View_Definitions.md`.
- If a change doesn’t show up:
    + Validate the model.
    + Re-run `render-views`.
    + Ensure you edited the model (`docs/design/aurora/**`), not the rendered output.
