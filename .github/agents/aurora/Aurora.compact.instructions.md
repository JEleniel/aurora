---
applyTo: 'docs/design/aurora/AGENT-MIS-*.json'
---

# Aurora Compact Model Instructions (Read-Only)

This instruction defines how to safely consume Aurora compact model exports. It is consistent with the canonical Aurora semantics (cards + directed links, rooted at a single Mission) while focusing on the compact export format.

## What these files are

For agent work, prefer the compact exports when available:

- `docs/design/aurora/AGENT-MIS-*.json`

These files are derived artifacts (typically produced by `aurora_cli`) optimized for machine consumption. They are not the editable source-of-truth representation of the model.

## Non-negotiable: do not modify

The Aurora model snapshot under `docs/design/aurora/` is read-only.

- You MUST NOT edit any `docs/design/aurora/AGENT-MIS-*.json` compact export.
- You MUST NOT edit any model source cards under `docs/design/aurora/`.
- You MUST NOT add, delete, or renumber card ids.
- If you believe the model needs changes, describe the required change and ask the Architect (or a human maintainer) to apply it.

## Compact export structure

An `AGENT-MIS-*.json` compact export is a single JSON object with:

- `cards`: an array of card objects.

Compact exports may also include `$schema`. If present, it MUST be a valid relative path (from the compact export file) to the compact schema in the model home.

Each card object typically contains:

- `id`: a stable identifier such as `MIS-001`, `DRI-001`.
- `card_type`: the architectural element type in title case (for example, `Mission`, `Driver`, `Requirement`).
- `card_subtype`: optional refinement of `card_type`.
- `name`: a concise human-readable name in title case.
- `description`: details about the element represented by the card.
- `links`: outgoing relationships.
    + Each link has:
        * `target`: the destination card id.
        * `relationship`: a human-readable verb.
- `status`: optional lifecycle state.
- `version`: may be present; interpret as semantic versioning of the card meaning.
- `attributes`: optional arbitrary JSON; may be `null`.

Compact exports are derived artifacts and commonly omit per-card audit metadata even if the full-format model includes it.

## Model interpretation (canonical invariants)

Interpret a compact export using the same invariants as the full Aurora model:

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

## Canonical definitions

When validating meaning (card types and relationships) or generating consistent views, use the canonical registries referenced by this repository:

- Canonical cards and relationships: [../agents/aurora/Aurora.canonical.definitions.json](../agents/aurora/Aurora.canonical.definitions.json)
- View definitions: [../agents/aurora/View.Definitions.json](../agents/aurora/View.Definitions.json)
