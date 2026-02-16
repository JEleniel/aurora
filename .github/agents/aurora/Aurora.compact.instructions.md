---
applyTo: 'docs/design/aurora/*/Compact.json'
---

# Aurora Compact Model Instructions (Read-Only)

This instruction defines only what is needed to read and interpret Aurora compact model exports.

## Canonical split: schema vs instruction

- Structure is canonical in `Aurora.compact.schema.json`.
- This instruction is interpretation guidance only.

## What these files are

For agent work, compact exports are the preferred machine-readable model snapshot:

- `docs/design/aurora/<MISSION_ID>/Compact.json`

These files are derived artifacts and are not the editable source-of-truth representation of the model.

## Non-negotiable: do not modify

- You MUST NOT edit any `docs/design/aurora/<MISSION_ID>/Compact.json` compact export.

## Consumption rules

- Parse compact exports using the compact schema only.
- Compact exports MUST include the `$schema` property with a relative path to `Aurora.compact.schema.json`.
- Compact cards omit `description` and omit `version`.
- Compact cards require `attributes` to preserve implementation-specific metadata.
- Relationship semantics remain canonical to `Aurora.canonical.definitions.json`.

## Canonical definitions

When validating meaning (card types and relationships), use the canonical registry:

- Canonical cards and relationships: [Aurora.canonical.definitions.json](Aurora.canonical.definitions.json)

## Model interpretation (canonical invariants)

- Interpret a compact export using the same invariants as the full Aurora model.
- The model contains exactly one `Mission` card.
- The `Mission` card is the root vertex and has out-degree only.
- Every non-`Mission` card is reachable from `Mission` by at least one directed path.
- Traversal algorithms halt when they encounter either a sink (out-degree `0`) or a previously visited vertex.
- Relationship verbs are descriptive labels; validity comes from structural invariants and the canonical relationship registry.
