# How to Talk to the Aurora Model

This repo’s architecture model is a directed graph of cards stored under `docs/design/aurora/`. Most day-to-day work should be expressed as precise edits to those source files so the CLI can validate and regenerate all derived artifacts.

This document is a template you can use when asking a human or an agent to make model changes.

## The golden rule

Prefer requests that describe changes in terms of:

- Card ids and fields (create/update)
- Links to add/remove (source, relationship, target)
- Expected view impacts

Avoid requests like “make the diagram look better” without specifying what should change in the model.

## Request template (copy/paste)

Fill in as much as you can. When in doubt, over-specify ids and relationships.

- Mission id(s):
    + Example: `MIS-001`
- Source-of-truth scope:
    + Files under: `docs/design/aurora/**`
- Cards to create (one entry per card):
    + `card_type`:
    + `card_subtype` (optional):
    + `name`:
    + `description`:
    + Proposed id (optional):
    + Intended folder (optional):
- Cards to update:
    + Card id:
    + Fields to change (name/description/status/attributes/...):
    + Any invariants to preserve (for example, id must not change):
- Links to add:
    + `source_id` → `relationship` → `target_id`
- Links to remove:
    + `source_id` → `relationship` → `target_id`
- Expected view impact:
    + View(s):
    + Root card(s) (if applicable):
    + What you expect to see appear/disappear:
- Constraints or conventions to follow:
    + Example: “Use `card_subtype: "struct"` for Rust constructs.”
    + Example: “Link from the Mission outward; do not create back-edges to the Mission.”
- Regeneration expectations:
    + Run: `aurora_cli -i docs/design/aurora validate`
    + Then: `aurora_cli -i docs/design/aurora render-all -o docs/design/`

## Common request examples

### Add a card subtype

Goal: add a subtype to make a card more precise (for example, modeling a Rust struct).

- Cards to update:
    + `CLS-012`: set `card_subtype` to `"struct"`
- Expected view impact:
	+ Class View should show `(struct)` after the existing line break in the node label for `CLS-012` (depending on view labeling rules).

### Add a Driver → Constraint link

Goal: connect a motivation driver to a constraint it drives.

- Links to add:
    + `DRI-002` → `drives` → `CNS-004`
- Expected view impact:
    + Requirements View should show `DRI-002` connected to `CNS-004`.

### Rename a card

Goal: change the human-readable name while preserving id stability.

- Cards to update:
    + `FEA-007`: update `name` from “Old Name” to “New Name”
- Constraints:
    + Do not change `id`.
    + Ensure any file name that includes the `name` is updated to match repo conventions.
- Expected view impact:
    + All views that include `FEA-007` should show the new label.

### Move an artifact (refactor the model layout)

Goal: relocate a card file to a different folder (for example, correcting a card_type folder) without changing the card id.

- Cards to update:
    + None (content unchanged)
- Files to move:
    + Move `ART-003-Old_File_Name.jsjson` to the correct `docs/design/aurora/<MISSION_ID>/Artifact/` folder and rename the file to match the card name.
- Constraints:
    + Preserve `id` and all inbound/outbound links.
    + After moving, validate to ensure no broken references.

### Adjust view roots

Goal: make a view render a specific slice (for example, root a Requirements View on a single requirement).

- Expected view impact:
    + Requirements View rooted on `REQ-010` should include only the neighborhood of `REQ-010` (plus linked Boundaries/Notes where applicable).
- Notes:
    + Root selection is typically a rendering parameter rather than a model change; specify whether you want the model changed or just want a different render invocation.

## Generated artifacts: editable when asked, but regeneratable

Rendered documentation and views under `docs/design/<Rendered_Model_Name>/` are derived outputs. They can be edited if explicitly requested (for example, to hotfix wording in a generated page), but the durable fix should usually be made in `docs/design/aurora/**` and then re-rendered.

## Related docs

- Model source of truth: [`docs/design/aurora/`](aurora/)
- Quickstart and CLI commands: [`docs/design/Model_Quickstart.md`](Model_Quickstart.md)
- DOT/SVG styling: [`.github/instructions/Graphviz_View_Styling_Guide.md`](../../.github/instructions/Graphviz_View_Styling_Guide.md)
