# Aurora CLI

`aurora_cli` is the primary command-line tool for:

- Validating one or more Aurora models
- Rendering **Markdown** (one file per card)
- Rendering **views** as **SVG** diagrams
- Generating **compact exports** (`Compact.json`) for automation

## Commands

| Command         | What it does                                                  |
| --------------- | ------------------------------------------------------------- |
| `validate`      | Validate models and print diagnostics (warnings + errors).    |
| `render-aurora` | Render Markdown for each card, plus a per-model index file.   |
| `render-views`  | Render SVG diagrams for each view definition in the registry. |
| `render-all`    | Run both Markdown and view rendering.                         |
| `compact`       | Write a compact snapshot export per model.                    |

## Global options

`aurora_cli` supports these options across commands:

- `-i, --input <PATH>`: input path (default: `docs/design/aurora`)
- `-l, --log <LEVEL>`: `trace|debug|info|warn|error` (default: `info`)

### Input path rules

In practice, the `--input` path must resolve to a **model home** directory named `aurora/`, or to a directory that contains an `aurora/` child directory.

Example:

```text
aurora_cli validate -i path/to/aurora
aurora_cli validate -i path/to   # where path/to/aurora exists
```

Note:

- `aurora_cli --help` currently describes `--input` as accepting mission card files as well.
- In the current build, passing a mission-card file path fails with an error like:

```text
An Aurora error has occurred: The specified path is not a valid Aurora model home: docs/design/aurora/MIS-001-Model_Aurora_With_Aurora.json
```

## `validate`

Validates model invariants and prints diagnostics.

Example:

```text
aurora_cli validate -i docs/design/aurora
```

Sample output:

```text
2026-02-19T19:26:35.232950Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:26:35.234247Z  INFO Validating 2 model(s) from docs/design/aurora
2026-02-19T19:26:35.235097Z  INFO No warnings found.
2026-02-19T19:26:35.235100Z  INFO No validation errors found.
```

## `render-aurora`

Renders Markdown for each model into an output directory.

Example:

```text
aurora_cli render-aurora -i docs/design/aurora -o tmp/aurora_cli_out
```

Sample output:

```text
2026-02-19T19:39:35.999789Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:36.001243Z  INFO Rendering markdown for 2 model(s) into tmp/aurora_cli_out
2026-02-19T19:39:36.008063Z  INFO Wrote MIS-001: Model Aurora With Aurora (105 cards)
2026-02-19T19:39:36.014041Z  INFO Wrote MIS-002: Example Canonical Coverage Model (81 cards)
```

### Markdown output layout

For each mission, you get:

- A per-model index:
    - `README-<MISSION_ID>-<Mission_Name>.md`
- A Markdown rendering of the mission card:
    - `<MISSION_ID>-<Mission_Name>.md`
- A file per non-mission card:
    - `<MISSION_ID>/<Card_Type>/<CARD_ID>-<Card_Name>.md`

Example (from a render run into `tmp/aurora_cli_out`):

```text
tmp/aurora_cli_out/README-MIS-002-Example_Canonical_Coverage_Model.md
tmp/aurora_cli_out/MIS-002-Example_Canonical_Coverage_Model.md
tmp/aurora_cli_out/MIS-002/Requirement/REQ-900-Exercise_All_Card_Types_and_Relationships.md
```

## `render-views`

Renders SVG diagrams for each view definition in the model configuration registry.

Example:

```text
aurora_cli render-views -i docs/design/aurora -o tmp/aurora_cli_out
```

Sample output (abbreviated):

```text
2026-02-19T19:39:25.835944Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:25.843365Z  INFO Rendering views for 2 model(s) into tmp/aurora_cli_out
2026-02-19T19:39:25.923643Z  INFO Rendered view 'Compliance Governance' for MIS-001 to tmp/aurora_cli_out/MIS-001/Views/Compliance_Governance.svg
2026-02-19T19:39:25.936176Z  INFO Skipping view 'Deployment' for MIS-001: no roots of required types present
2026-02-19T19:39:30.990963Z  INFO Rendered view 'Deployment' for MIS-002 to tmp/aurora_cli_out/MIS-002/Views/Deployment.svg
```

### Views output layout

Rendered view files are written to:

- `<OUTPUT>/<MISSION_ID>/Views/<Sanitized_View_Name>.svg`

View names are sanitized for filenames by converting whitespace to underscores and removing other punctuation.

## `render-all`

Runs both `render-aurora` and `render-views`.

Example:

```text
aurora_cli render-all -i docs/design/aurora -o tmp/aurora_cli_out
```

## `compact`

Writes a compact, machine-friendly snapshot export per mission.

Example:

```text
aurora_cli compact -i docs/design/aurora -o tmp/aurora_cli_out
```

Sample output:

```text
2026-02-19T19:39:38.429064Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:39:38.430551Z  INFO Writing compact exports for 2 model(s)
2026-02-19T19:39:38.430789Z  INFO Wrote compact MIS-001: Model Aurora With Aurora (105 cards)
2026-02-19T19:39:38.430958Z  INFO Wrote compact MIS-002: Example Canonical Coverage Model (81 cards)
```

### Compact output layout

Files are written to:

- `<OUTPUT>/<MISSION_ID>/Compact.json`

Example:

```text
tmp/aurora_cli_out/MIS-001/Compact.json
tmp/aurora_cli_out/MIS-002/Compact.json
```

## See also

- **[Getting Started](./Getting%20Started.md)** for end-to-end examples.
- **[Model Home Layout](./Model%20Home%20Layout.md)** for required files and folder structure.
- **[Configuration Guide](./Configuration%20Guide.md)** for card types, relationships, icons, and view definitions.
