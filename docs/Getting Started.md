# Getting Started

This guide gets you from “I have a model home” to “I can validate and render my model” using the current Aurora tooling (`aurora_cli` and `svg_prep`).

## Quick start (this repository)

This repository ships with an example model home at `docs/design/aurora/` containing:

- Schemas (`docs/design/aurora/schemas/`)
- Reference registries (`docs/design/aurora/reference/`)
- Example models (`MIS-001` and `MIS-002`)

### Validate the example models

Run:

```text
aurora_cli validate -i docs/design/aurora
```

Example output (captured from a successful run):

```text
2026-02-19T19:26:35.232950Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:26:35.234247Z  INFO Validating 2 model(s) from docs/design/aurora
2026-02-19T19:26:35.235097Z  INFO No warnings found.
2026-02-19T19:26:35.235100Z  INFO No validation errors found.
```

## Create (or locate) a model home

Aurora tooling loads models from a **model home**, which is a folder named `aurora/`.

The CLI input path (`-i/--input`) must be either:

- The model home itself (a directory named `aurora/`), or
- A directory that contains an `aurora/` child directory

Note:

- `aurora_cli --help` currently describes `--input` as accepting mission-card file paths as well.
- In the current build, passing a mission-card file path fails with an error that the path is not a valid model home.

For example:

```text
# Directly point at the model home
aurora_cli validate -i path/to/aurora

# Or point at a directory that contains ./aurora
aurora_cli validate -i path/to
```

What the model home must contain is described in **[Model Home Layout](./Model%20Home%20Layout.md)**.

## Render outputs

Aurora produces three kinds of outputs:

- **Views**: SVG diagrams generated from view definitions
- **Markdown**: a file per card plus an index per model
- **Compact exports**: a single-file, machine-friendly snapshot (`Compact.json`)

### Render views (SVG)

```text
aurora_cli render-views -i path/to/aurora -o tmp/aurora_cli_out
```

Example output (abbreviated):

```text
2026-02-19T19:39:25.835944Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:25.843365Z  INFO Rendering views for 2 model(s) into tmp/aurora_cli_out
2026-02-19T19:39:25.936176Z  INFO Skipping view 'Deployment' for MIS-001: no roots of required types present
2026-02-19T19:39:30.990963Z  INFO Rendered view 'Deployment' for MIS-002 to tmp/aurora_cli_out/MIS-002/Views/Deployment.svg
```

This writes files like:

- `<OUTPUT>/<MISSION_ID>/Views/<View_Name>.svg`

View filenames are “sanitized” (whitespace becomes underscores; other punctuation is removed).

### Render markdown

```text
aurora_cli render-aurora -i path/to/aurora -o tmp/aurora_cli_out
```

Example output:

```text
2026-02-19T19:39:35.999789Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:36.001243Z  INFO Rendering markdown for 2 model(s) into tmp/aurora_cli_out
2026-02-19T19:39:36.014041Z  INFO Wrote MIS-002: Example Canonical Coverage Model (81 cards)
```

This writes:

- `<OUTPUT>/README-<MISSION_ID>-<Mission_Name>.md` (per-model index)
- `<OUTPUT>/<MISSION_ID>-<Mission_Name>.md` (the mission card)
- `<OUTPUT>/<MISSION_ID>/<Card_Type>/<CARD_ID>-<Card_Name>.md` (every other card)

### Render everything

```text
aurora_cli render-all -i path/to/aurora -o tmp/aurora_cli_out
```

### Generate compact exports

```text
aurora_cli compact -i path/to/aurora -o tmp/aurora_cli_out
```

Example output:

```text
2026-02-19T19:39:38.429064Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:38.430551Z  INFO Writing compact exports for 2 model(s)
2026-02-19T19:39:38.430958Z  INFO Wrote compact MIS-002: Example Canonical Coverage Model (81 cards)
```

By default, compact exports are written into the model home as:

- `aurora/<MISSION_ID>/Compact.json`

Compact exports are intended for automation and analysis. They are not a rendering input. See **[Core Concepts](./Core%20Concepts.md)**.

## Next steps

- Learn Aurora’s invariants and file formats in **[Core Concepts](./Core%20Concepts.md)**.
- Explore the canonical vocabulary and default views in **[Canonical Set](./Canonical%20Set.md)**.
- Customize card types, relationships, appearance, and views in **[Configuration Guide](./Configuration%20Guide.md)**.
