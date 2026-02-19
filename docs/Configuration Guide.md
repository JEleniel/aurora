# Configuration Guide

Aurora’s canonical vocabulary and rendering defaults are defined by a single JSON registry file:

- `reference/Aurora.modelconfiguration.json`

This file is validated by:

- `schemas/Aurora.modelconfiguration.schema.json`

The tooling uses the registry to:

- Check models for canonical consistency (warnings)
- Determine what views exist and what card types they include
- Provide rendering defaults (shape, colors, and default icon)

## File structure

`Aurora.modelconfiguration.json` has three top-level properties:

- `available_icons`: list of icon ids (strings)
- `cards`: list of card type definitions
- `views`: list of view definitions

The schema disallows additional top-level properties.

## `available_icons`

`available_icons` is the list of icon ids a card may reference.

- Each entry is a plain id such as `wrench`.
- In the SVG template, icons are stored as `<g id="i-<icon_id>">...</g>`.

Rendering and validation expectations:

- View rendering checks that every `available_icons` entry has a non-empty matching `i-...` group in `reference/SVGTemplate.svgz` (or `.svg`).
- Card `icon` overrides must reference an id that appears in `available_icons` (the renderer also tolerates `i-<id>` and `#i-<id>` forms).

## `cards` definitions

Each entry in `cards` defines a canonical card type and its defaults.

Key fields:

- `acronym` (required): three uppercase letters, e.g. `REQ`
- `card_type` (required): string used in card files, e.g. `Requirement`
- `description` (required): canonical meaning
- `shape` (required): a shape id that should exist in the SVG template defs
- `fill`, `stroke`, `text` (required): hex colors (`#RRGGBB`)
- `icon` (optional): default icon id
- `common_subtypes` (optional): list of strings (convenience vocabulary)
- `relationships` (optional): allowed relationship labels from this type to target acronyms

Example (simplified):

```json
{
	"acronym": "REQ",
	"card_type": "Requirement",
	"description": "Verifiable statement of need/obligation.",
	"shape": "rounded-rectangle",
	"fill": "#065f46",
	"stroke": "#000000",
	"text": "#FFFFFF",
	"icon": "check",
	"common_subtypes": ["Functional", "Non-Functional"],
	"relationships": [
		{ "target": "CAP", "relationship": "requires" },
		{ "target": "ADR", "relationship": "has" }
	]
}
```

### How relationships are validated

Registry relationship entries are declared using target **acronyms**.

When a model is checked:

- The target card’s acronym is extracted from the target id prefix (e.g. `CAP` from `CAP-001`).
- The acronym is resolved to a target card type using the registry.
- The `relationship` label string must match exactly.

Registry mismatches produce **warnings** (not invariant errors).

## `views` definitions

A view definition determines which portions of the model can be rendered.

Fields:

- `name` (required)
- `description` (required)
- `root_card_types` (required): list of acronyms
- `included_card_types` (required): list of acronyms

Example:

```json
{
	"name": "Requirements",
	"description": "Captures mission intent, motivation, and required capabilities.",
	"root_card_types": ["MIS"],
	"included_card_types": ["DRI", "STK", "CAP", "REQ", "ADR", "CNS"]
}
```

Rendering notes:

- A view is only rendered for a model if that model contains at least one card whose id prefix matches a `root_card_types` entry.
- Filenames for views are “sanitized” by keeping alphanumeric characters and converting whitespace to underscores.

## Customization recipes

This section shows how to extend Aurora beyond the canonical set without breaking the core invariants.

### Add a new card type

1. Choose a stable **three-letter acronym** (e.g. `TMI`).
2. Add a new card definition to `cards`.
3. Add the acronym to at least one view (otherwise the tools may warn that your acronym is unused).
4. Create card JSON files with `card_type` matching your definition.

Example definition:

```json
{
	"acronym": "TMI",
	"card_type": "Team",
	"description": "A team responsible for owning and operating a set of components.",
	"shape": "rounded-rectangle",
	"fill": "#0f172a",
	"stroke": "#000000",
	"text": "#FFFFFF",
	"icon": "group",
	"relationships": [
		{ "target": "COM", "relationship": "owns" }
	]
}
```

### Add a new relationship

To add a new relationship label, edit the source card definition and add a relationship entry.

Example (allow a `Team` to `owns` a `Component`):

```json
"relationships": [
	{ "target": "COM", "relationship": "owns" }
]
```

Then, in the model, use the same relationship string:

```json
{ "relationship": "owns", "target": "COM-123" }
```

### Customize appearance (shape and colors)

Appearance is defined per card type:

- `shape` chooses which `<g id="...">` is used from the SVG template.
- `fill`, `stroke`, and `text` control how the symbol and text are rendered.

If a `shape` id does not exist in the template, rendering falls back to `rectangle`.

### Add a new icon

Icons are a combination of:

- An entry in `available_icons`, and
- A `<g id="i-<icon_id>">...</g>` group in `reference/SVGTemplate.svgz` (or `.svg`)

To add an icon:

1. Add a master icon SVG under your icons source directory.
2. Run `svg_prep` to rebuild the template defs.
3. Ensure the registry’s `available_icons` includes the new icon id.

See **[SVG Prep](./SVG%20Prep.md)** for the build pipeline.

### Add a new view

To add a view:

1. Add an entry to `views`.
2. Choose `root_card_types` that exist in the models you want to render.
3. Include acronyms you want rendered via `included_card_types`.

Example:

```json
{
	"name": "Ownership",
	"description": "Shows ownership relationships from teams to components.",
	"root_card_types": ["TMI"],
	"included_card_types": ["TMI", "COM", "APP", "SYS"]
}
```

## Common pitfalls

- **Forgetting to update views:** custom acronyms not included in any view definition may trigger warnings.
- **Shape id mismatch:** if `shape` doesn’t exist in the SVG template, the diagram will render as rectangles.
- **Icon id mismatch:** if `available_icons` doesn’t match the template’s `i-...` groups, view rendering fails.
