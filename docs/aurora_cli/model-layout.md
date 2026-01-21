# Model input layout

`aurora_cli` expects an Aurora “model home” directory containing schema files and one or more Mission cards.

## Model home contents

At the top level of the model home:

- `Aurora.schema.json`
- `Aurora.compact.schema.json`
- One or more Mission cards named like `MIS-001-Some_Name.json`
- A folder for each mission id, such as `MIS-001/`

You can pass `--input` as:

- the model home itself (for example `docs/design/aurora/`)
- a parent folder that contains `aurora/`
- or the parent of the model home, as long as the model home can be inferred

## Mission filename rule

Mission card filenames must follow:

```text
MIS-###-<Sanitized_Name>.json
```

Where `<Sanitized_Name>` is derived from the Mission card `name`:

- spaces and whitespace become `_`
- ASCII letters, digits, `_`, and `-` are preserved
- everything else becomes `_`

## Card layout inside a mission

Within each mission folder, cards are organized by `card_type` folder and `id` filename:

```text
<MISSION_ID>/<Card Type>/<CARD_ID>.json
```

Example:

```text
MIS-001/Requirement/REQ-001.json
MIS-001/Data Store/DTS-001.json
MIS-001/State Machine/STM-001.json
```

Notes:

- Card filenames must follow `XXX-###.json`.
- The file stem (for example `REQ-001`) must match the JSON card's `id`.
- The directory name must match the JSON card's `card_type` (including spaces and capitalization).

## Optional compact model file

If a compact file exists in the model home, it is loaded and available to the tool:

```text
AGENT-<MISSION_ID>.json
```
