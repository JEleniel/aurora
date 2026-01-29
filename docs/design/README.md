# Architecture & Design

This folder contains the _rendered_ (human-readable) Aurora design artifacts produced from the source model stored under [`aurora/`](aurora/).

## Source of truth

- Source model home: [`aurora/`](aurora/)
- Per-model source cards live under `aurora/<MISSION_ID>/...`
- Rendered output is written under `docs/design/<MISSION_NAME>/...`
- View SVGs are emitted per root card under `docs/design/<MISSION_NAME>/Views/` using `<View>_View-<ROOT>.view.svg` naming (DOT sources live under `Views/source/`).

## Models

- **MIS-001 — Agent-Unified Representation of Requirements and Architecture (Aurora)**
    + Source mission card: [`aurora/MIS-001-AgentUnified_Representation_of_Requirements_and_Architecture_Aurora.jsjson`](aurora/MIS-001-AgentUnified_Representation_of_Requirements_and_Architecture_Aurora.jsjson)
    + Source cards folder: [`aurora/MIS-001/`](aurora/MIS-001/)
    + Rendered entrypoint: [`MIS-001-Agent-Unified_Representation_of_Requirements_and_Architecture__Aurora_/MIS-001-AgentUnified_Representation_of_Requirements_and_Architecture_Aurora.md`](MIS-001-Agent-Unified_Representation_of_Requirements_and_Architecture__Aurora_/MIS-001-AgentUnified_Representation_of_Requirements_and_Architecture_Aurora.md)

- **MIS-002 — Relationship Matrix Reference**
    + Source mission card: [`aurora/MIS-002-Relationship_Matrix_Reference.jsjson`](aurora/MIS-002-Relationship_Matrix_Reference.jsjson)
    + Source cards folder: [`aurora/MIS-002/`](aurora/MIS-002/)
    + Rendered entrypoint: [`MIS-002-Relationship_Matrix_Reference/MIS-002-Relationship_Matrix_Reference.md`](MIS-002-Relationship_Matrix_Reference/MIS-002-Relationship_Matrix_Reference.md)
    + Executive summary: [`MIS-002-Relationship_Matrix_Reference/MIS-002-Executive_Summary.md`](MIS-002-Relationship_Matrix_Reference/MIS-002-Executive_Summary.md)

- **MIS-003 — CLI Tooling**
    + Source mission card: [`aurora/MIS-003-CLI_Tooling.jsjson`](aurora/MIS-003-CLI_Tooling.jsjson)
    + Source cards folder: [`aurora/MIS-003/`](aurora/MIS-003/)
    + Rendered entrypoint: [`MIS-003-CLI_Tooling/MIS-003-CLI_Tooling.md`](MIS-003-CLI_Tooling/MIS-003-CLI_Tooling.md)
    + Executive summary: [`MIS-003-CLI_Tooling/MIS-003-Executive_Summary.md`](MIS-003-CLI_Tooling/MIS-003-Executive_Summary.md)

- **MIS-004 — Standalone Editor**
    + Source mission card: [`aurora/MIS-004-Standalone_Editor.jsjson`](aurora/MIS-004-Standalone_Editor.jsjson)
    + Source cards folder: [`aurora/MIS-004/`](aurora/MIS-004/)
    + Rendered entrypoint: [`MIS-004-Standalone_Editor/MIS-004-Standalone_Editor.md`](MIS-004-Standalone_Editor/MIS-004-Standalone_Editor.md)
    + Executive summary: [`MIS-004-Standalone_Editor/MIS-004-Executive_Summary.md`](MIS-004-Standalone_Editor/MIS-004-Executive_Summary.md)

## Regenerating outputs

Run from the repository root.

```text
cargo run -p aurora_cli -- -i docs/design/aurora/ validate
cargo run -p aurora_cli -- -i docs/design/aurora/ render-all -o docs/design/
```

Sample output (trimmed):

```text
Validation succeeded for 2 mission model(s) with 0 warning(s) and 0 info message(s).
Rendered 39 cards and 13 views into docs/design/MIS-002-Relationship_Matrix_Reference
Rendered 40 cards and 38 views into docs/design/MIS-003-CLI_Tooling
```
