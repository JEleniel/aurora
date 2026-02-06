---
applyTo: 'docs/design/aurora/AGENT-MIS-*.json'
---

# Aurora Compact Model Instructions (Read-Only)

This instruction defines how to safely consume compact Aurora model exports. It is intentionally consistent with the canonical Aurora model semantics (cards + directed links, rooted at a Mission) while adding repository-specific guardrails.

## What to use

For agent work, prefer the compact exports when available:

- `docs/design/aurora/AGENT-MIS-*.json`

These files are optimized for machine consumption and are much smaller than traversing the full model snapshot.

## Non-negotiable rule: do not modify the model

The Aurora model snapshot in this repository is read-only.

- You MUST NOT edit any `docs/design/aurora/AGENT-MIS-*.json` compact export.
- You MUST NOT edit any model source cards under `docs/design/aurora/`.
- You MUST NOT add, delete, or renumber card ids.
- If you believe the model needs changes, you MUST describe the required change and ask the Architect (or a human maintainer) to apply it.

## Compact export structure

An `AGENT-MIS-*.json` compact export is a single JSON object with a top-level `cards` array.

- Each element of `cards` is a card object with (at minimum):
    + `id`
    + `card_type`
    + `name`
    + `description`
    + `links` (outgoing relationships; each link has a `target` card id and a human-readable `relationship` verb)
    + `status` (optional)
    + `attributes` (optional; may be null)
- Compact cards typically omit `audit_trail` even when the full-format cards include it.

## Model interpretation (canonical Aurora semantics)

Interpret the compact export using the same invariants as the full Aurora model.

- Cards represent architectural elements (nouns) with attributes and outbound links.
- The model is a traversable directed graph rooted at exactly one `Mission` card.
- All links are directed away from the `Mission` card; traversing from the `Mission` must be able to reach every card (no orphans).
- Traversal ends at a leaf card or a previously seen card (local loops are allowed), but paths must not lead back to the `Mission`.
- Relationship verbs are descriptive only; meaning comes from the invariant rules and how a view projects the graph.

## Views

Views are projections over the model.

- A view selects root card types (and optional subtypes), selects which card types to include during traversal, and renders only those cards and the links between them.
- Views do not change the model.

## Canonical registries

When validating meaning (allowed card types, relationship verbs, and view definitions), use the canonical registries referenced by the repository:

- Cards: [`../agents/details/1-Card_Definitions.json`](../agents/details/1-Card_Definitions.json)
- Relationships: [`../agents/details/2-Relationship_Definitions.json`](../agents/details/2-Relationship_Definitions.json)
- Views: [`../agents/details/3-View_Definitions.json`](../agents/details/3-View_Definitions.json)
