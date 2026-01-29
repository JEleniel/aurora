# Architecture & Design

This folder contains the _rendered_ (human-readable) Aurora design artifacts produced from the source model stored under [`aurora/`](aurora/).

## Source of truth

- Source model home: [`aurora/`](aurora/)
- Per-model source cards live under `aurora/<MISSION_ID>/...`
- Rendered output is written under `docs/design/<MISSION_NAME>/...`

## Models

- **MIS-002 — Relationship Matrix Reference**
    + Source mission card: [`aurora/MIS-002-Relationship_Matrix_Reference.jsjson`](aurora/MIS-002-Relationship_Matrix_Reference.jsjson)
    + Source cards folder: [`aurora/MIS-002/`](aurora/MIS-002/)
    + Rendered entrypoint: [`MIS-002-Relationship_Matrix_Reference/MIS-002-Relationship_Matrix_Reference.md`](MIS-002-Relationship_Matrix_Reference/MIS-002-Relationship_Matrix_Reference.md)

## Regenerating outputs

Run from the repository root.

```text
cargo run -p aurora_cli -- -i docs/design/aurora/ validate
cargo run -p aurora_cli -- -i docs/design/aurora/ render-aurora -o docs/design/
```

Sample output (trimmed):

```text
Validation succeeded for 1 mission model(s) with 61 warning(s) and 0 info message(s).
Rendered 37 cards and 0 views into docs/design/MIS-002-Relationship_Matrix_Reference
```

Note: `render-all` currently fails on this workspace due to a Graphviz HTML-label formatting error triggered by some card icons in the embedded registry (observed while attempting to render the “Entire Model” view). Use `render-aurora` until the renderer is adjusted.
