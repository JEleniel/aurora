---
applyTo: "**/aurora/AGENT-MIS-*.jsjson"
---

# Aurora Compact Model Instructions (Read-Only)

This instruction exists to help agents consume Aurora models safely and efficiently.

## Architect? Read This First

If you are the Architect, read and follow [Aurora.instructions.md](Aurora.instructions.md) instead of this document.

## What to use

Before you perform and work, you MUST understand the compact Aurora Model rlevant to your work:

- `docs/design/aurora/AGENT-MIS-*.jsjson`

These compact files are intended for machine agents and are significantly smaller than loading the entire model home.

## Non-negotiable rule: do not modify the model

You are NOT permitted to modify Aurora models in this repository.

- You MUST NOT edit any `AGENT-MIS-*.jsjson` compact export.
- You MUST NOT edit any source cards under `docs/design/aurora/`.
- You MUST NOT add, delete, or renumber card ids.
- If you believe the model needs changes, you MUST describe the required change and ask the Architect (or the human maintainer) to apply it.

## How the compact model is structured

A compact export is a single JSON object with a top-level `cards` array.

- Each element of `cards` is a card object with (at minimum):
    + `id`
    + `card_type`
    + `name`
    + `description`
    + `links` (outgoing relationships)
    + `status` (optional)
    + `attributes` (optional; may be null)
- Compact cards omit `audit_trail` (the full model contains it).

## How to interpret it (Aurora semantics, summarized)

- The model is a directed graph rooted at a single `Mission` card.
- Links are directed away from the Mission; local cycles may exist (for example, within behavioral subgraphs), but no path may lead back to the Mission.
- Relationship verbs are descriptive labels; meaning is derived from the model invariants and the selected view/projection.
- Views are projections: a view selects card types (and optional subtypes) and renders only those cards and the links between them. Views do not change the model.

## Canonical registries

When validating meaning (card types, relationship verbs, and view definitions), you MUST use the canonical registries referenced by the repository:

- Cards: [`.github/instructions/details/1-Card_Definitions.md`](details/1-Card_Definitions.md)
- Relationships: [`.github/instructions/details/2-Relationship_Definitions.md`](details/2-Relationship_Definitions.md)
- Views: [`.github/instructions/details/3-View_Definitions.md`](details/3-View_Definitions.md)
