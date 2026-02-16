---
applyTo: 'docs/design/aurora/*/Compact.json'
---

# Aurora Compact Model Instructions (Read-Only)

This instruction defines how to safely consume Aurora compact model exports.

## Canonical split: schema vs instruction

- Structure is canonical in `Aurora.compact.schema.json`.
- This instruction is behavioral guidance only.
- Examples are illustrative and non-canonical.

## What these files are

For agent work, prefer the compact exports when available:

- `docs/design/aurora/<MISSION_ID>/Compact.json`

These files are derived artifacts (typically produced by `aurora_cli`) optimized for machine consumption. They are not the editable source-of-truth representation of the model.

## Non-negotiable: do not modify

The Aurora model snapshot under `docs/design/aurora/` is read-only.

- You MUST NOT edit any `docs/design/aurora/<MISSION_ID>/Compact.json` compact export.
- You MUST NOT edit any model source cards under `docs/design/aurora/`.
- You MUST NOT add, delete, or renumber card ids.
- If you believe the model needs changes, describe the required change and ask the Architect (or a human maintainer) to apply it.

## Consumption rules

- Parse compact exports using the compact schema only.
- Compact cards omit `description` and omit `version`.
- Compact cards require `attributes` to preserve implementation-specific metadata.
- Relationship semantics remain canonical to `Aurora.canonical.definitions.json`.

## Canonical definitions

When validating meaning (card types and relationships) or generating consistent views, use the canonical registries referenced by this repository:

- Canonical cards and relationships: [Aurora.canonical.definitions.json](Aurora.canonical.definitions.json)
- View definitions: [View.Definitions.json](View.Definitions.json)

## Model interpretation (canonical invariants)

- Interpret a compact export using the same invariants as the full Aurora model.
- The model contains exactly one `Mission` card.
- The `Mission` card is the root of the directed graph and MUST only have outgoing links.
- When traversing starting from `Mission`, all links must lead away from `Mission`.
- There MUST be a route from `Mission` to every other card (no orphans).
- Traversing any path starting from `Mission` must end at a leaf card or a previously seen card, creating a local loop.
- Relationship verbs are descriptive only; meaning comes from the invariant rules and how a view projects the graph.

## Views

Views are projections over the model:

- A view selects root card types (and optional subtypes) and selects which card types to include during traversal.
- Views render only the selected cards and the links between them.
    + Root cards are always rendered.
    + Card types listed in `root_card_types` are implicitly eligible for rendering when encountered during traversal.
- Views do not change the model.
