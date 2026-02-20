# Model Home Layout

Aurora tooling loads models from a **model home**: a directory named `aurora/` containing schemas, reference registries, and one or more models.

## Directory layout

A minimal model home looks like this:

```text
aurora/

    schemas/
        Aurora.audit.schema.json
        Aurora.card.schema.json
        Aurora.compact.schema.json
        Aurora.modelconfiguration.schema.json
    reference/
        Aurora.modelconfiguration.json
        SVGTemplate.svgz
    MIS-001-My_Mission.json
    MIS-001/
        AuditLog.ndjson
        <Other card-type folders...>
```

### What’s required

Aurora tooling requires these files to exist under the model home:

| Path                                                        |                     Required | Purpose                                                                                       |
| ----------------------------------------------------------- | ---------------------------: | --------------------------------------------------------------------------------------------- |
| `schemas/Aurora.card.schema.json`                           |                          Yes | Structure validation for card JSON files.                                                     |
| `schemas/Aurora.audit.schema.json`                          |                          Yes | Structure validation for `AuditLog.ndjson` lines.                                             |
| `schemas/Aurora.compact.schema.json`                        |                          Yes | Structure validation for compact exports (`Compact.json`).                                    |
| `schemas/Aurora.modelconfiguration.schema.json`             |                          Yes | Structure validation for the registry (`Aurora.modelconfiguration.json`).                     |
| `reference/Aurora.modelconfiguration.json`                  |                          Yes | Canonical card types, relationships, default appearance, and view definitions.                |
| `reference/SVGTemplate.svgz` or `reference/SVGTemplate.svg` | Required for rendering views | SVG defs template for shapes and icons. The tooling prefers `.svgz` and falls back to `.svg`. |

## Where models live

Inside the model home:

- The **mission card** is stored at the model home root as `MIS-<n>-<Name>.json`.
- All other cards are stored under a mission folder: `aurora/<MISSION_ID>/...`.
- Each mission folder must include an audit log: `aurora/<MISSION_ID>/AuditLog.ndjson`.

Card-type folder names are derived from the `card_type` by sanitizing:

- Spaces become underscores.
- Punctuation is removed.

Examples:

- `Data Store` becomes `Data_Store/`
- `State Machine` becomes `State_Machine/`

## `$schema` references

Aurora requires each card to include a `$schema` property that can be resolved relative to the card file.

Examples:

- Mission card at `aurora/MIS-002-Example.json`:

```json
{
    "$schema": "./schemas/Aurora.card.schema.json",
    "id": "MIS-002",
    "card_type": "Mission",
    "name": "...",
    "description": "...",
    "links": []
}
```

- Requirement card at `aurora/MIS-002/Requirement/REQ-900-Example.json`:

```json
{
    "$schema": "../../schemas/Aurora.card.schema.json",
    "id": "REQ-900",
    "card_type": "Requirement",
    "name": "...",
    "description": "...",
    "links": []
}
```

## SVG template expectations

View rendering requires a template file at:

- `reference/SVGTemplate.svgz` (preferred), or
- `reference/SVGTemplate.svg` (fallback)

The template must contain, in its `<defs>`:

- **Icons** as groups with ids `i-<icon_name>` (e.g., `i-wrench`)
- **Shapes** as groups with ids matching the `shape` values used in the model configuration (e.g., `rounded-rectangle`)

Notes:

- Icon ids are validated against the `available_icons` list in `reference/Aurora.modelconfiguration.json`.
- Shape ids are best-effort: if a configured shape id does not exist in the template, rendering falls back to `rectangle`.

## Reference sources in this repository

If you’re using this repository as-is:

- Example model home: `docs/design/aurora/`
- Specification artifacts: `Aurora/`

Aurora tooling uses the **local** files in the model home at runtime; `Aurora/` is the canonical source for the specification, but is not used directly by the CLI when loading models.
