# Aurora CLI Contract

This document defines the operational contract for `aurora_cli` commands that interact with an Aurora model home.

## Shared command assumptions

- Input path (`--input`) may point to:
	+ a mission root card JSON file,
	+ the model home directory, or
	+ an ancestor directory that contains a valid model home.
- The model home must contain readable schema and reference artifacts:
	+ `schemas/Aurora.audit.schema.json`
	+ `schemas/Aurora.card.schema.json`
	+ `schemas/Aurora.compact.schema.json`
	+ `schemas/Aurora.modelconfiguration.schema.json`
	+ `reference/Aurora.modelconfiguration.json`
	+ `reference/SVGTemplate.svg`
- The CLI treats the model home as read-only input.
	+ It does not rewrite card JSON, audit NDJSON, schemas, or references.
	+ It writes only derived artifacts to output paths.

## Command behavior summary

| Command | Validation gate | Primary outputs |
| --- | --- | --- |
| `validate` | Always validates all discovered models | Diagnostics only (no files written) |
| `render-views` | Requires a valid model (`validate` hard errors block rendering) | `<output>/<MISSION_ID>/Views/*.svg` |
| `render-all` | Requires a valid model (`validate` hard errors block rendering) | View SVGs plus markdown docs |
| `compact` | Requires a valid model (`validate` hard errors block export) | `<output>/<MISSION_ID>/Compact.json` |

## `validate`

- Validates schema compliance, graph invariants, acronym consistency, and root safety.
- Emits warnings for non-fatal checks (for example unknown canonical registry items).
- Fails on hard errors (for example missing required files, missing `$schema` in card JSON, root-cycle safety violations).
- Output paths: none.

## `render-views`

- Runs full validation first.
- On success, renders diagram views using model-home references and view definitions.
- Outputs are written to `<output>/<MISSION_ID>/Views/`.

## `render-all`

- Runs the same validation gate as `render-views`.
- Produces:
	+ view SVGs under `<output>/<MISSION_ID>/Views/`
	+ mission/readme markdown and card markdown under `<output>/`

## `compact`

- Validates first, then exports compact model snapshots.
- Writes one compact file per mission to `<output>/<MISSION_ID>/Compact.json`.
- Compact output is derived from already-valid full model data.

## Exit and diagnostics policy

- Warning-only runs return exit code `0`.
- Any hard error returns a non-zero exit code.
- Severity model:
	+ validation errors: hard errors
	+ warnings: non-fatal
	+ load/render/export failures: hard errors
