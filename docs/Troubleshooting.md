# Troubleshooting

This page lists common Aurora tool failures and what to check first.

## “The specified path is not a valid Aurora model home”

Example error:

```text
An Aurora error has occurred: The specified path is not a valid Aurora model home: <PATH>
```

What it usually means:

- You pointed `--input` at a path that does not resolve to a directory named `aurora/`.

Fix:

- Point at the model home directory:

```text
aurora_cli validate -i path/to/aurora
```

- Or point at a directory that contains an `aurora/` child directory:

```text
aurora_cli validate -i path/to
```

Note:

- `aurora_cli --help` currently suggests that mission-card file paths are accepted.
- In the current build, passing a mission-card file path fails with this error.

## “Missing required file in model home: …”

Example error class:

- Missing schemas (`schemas/*.json`)
- Missing registry (`reference/Aurora.modelconfiguration.json` or `reference/Aurora.viewconfiguration.json`)
- Missing template (`reference/SVGTemplate.svgz` or `reference/SVGTemplate.svg`)

Fix:

- Confirm your model home contains all required files listed in **[Model Home Layout](./Model%20Home%20Layout.md)**.

## View rendering fails because icons are missing or empty

Symptoms:

- Rendering views fails with a reference-validation error.
- Or the renderer reports that the template is missing required icon groups.

What it means:

- `reference/Aurora.viewconfiguration.json` → `available_icons` includes an icon id that does not exist in the template’s `<defs>` as a non-empty group with id `i-<icon_id>`.

Fix:

- Inspect the template at `reference/SVGTemplate.svgz` (preferred) or `reference/SVGTemplate.svg`.
- Ensure every icon id appears as a `<g id="i-...">...</g>` group.
- If you maintain the template in this repository, use **[SVG Prep](./SVG%20Prep.md)** to regenerate the merged template and icon list.

## A view is “skipped”

Example log line:

```text
INFO Skipping view 'Deployment' for MIS-001: no roots of required types present
```

What it means:

- The view definition’s `root_card_types` are not present in that model.

Fix:

- Either add at least one card whose acronym matches a required root type, or
- Adjust the view definition in `reference/Aurora.modelconfiguration.json`.

See **[Canonical Set](./Canonical%20Set.md)** for the default view definitions.

## Warnings about custom acronyms not included in any view

Example warning text:

```text
Custom acronym 'ABC' is present in model but not included in any view definition.
```

What it means:

- You created cards with ids like `ABC-001`, but none of your view definitions include `ABC` in either `root_card_types` or `included_card_types`.

Fix:

- Add your acronym to at least one view definition.

See **[Configuration Guide](./Configuration%20Guide.md)**.

## JSON parse errors

Example error class:

```text
A JSON parse error occurred: ...
```

Fix:

- Ensure every `.json` file is valid JSON.
- Ensure every NDJSON audit log line is its own valid JSON object.

See **[Core Concepts](./Core%20Concepts.md)** for the audit log structure.
