# Model and folder layout

Aurora models are stored as JSON files in a predictable folder layout.

## Navigation

- [Aurora overview](README.md)
- [Cards and fields](cards.md)
- [Getting started](getting-started.md)
- [Design docs index](../design/README.md)

## Model home

A model home directory contains:

- `Aurora.schema.json` (card schema)
- `Aurora.compact.schema.json` (compact export schema)
- One or more Mission card JSON files
- One folder per Mission id

Example:

```text
aurora/
	Aurora.schema.json
	Aurora.compact.schema.json
	MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json
	MIS-001/
		Driver/
			DRI-001.json
		Requirement/
			REQ-001.json
```

## Mission card filename

Mission files are named:

```text
MIS-###-<Sanitized_Name>.json
```

Where `<Sanitized_Name>` is derived from the Mission card `name` (whitespace becomes `_`, unsafe characters become `_`).

## Cards under a mission

Inside each mission folder, cards are grouped by `card_type` directory:

```text
<MISSION_ID>/<Card Type>/<CARD_ID>.json
```

Examples:

```text
MIS-001/Requirement/REQ-001.json
MIS-001/Data Store/DTS-001.json
MIS-001/State Machine/STM-001.json
```

## Why the layout matters

Tools can rely on this structure to:

- find all cards deterministically
- validate that filenames, ids, and card types agree
- render documentation in a stable output tree
