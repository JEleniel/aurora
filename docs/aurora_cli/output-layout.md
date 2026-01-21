# Output layout

`aurora_cli` writes generated Markdown under a single output root.

## Example output tree

A typical `render-all` output (single mission) looks like:

```text
<OUTPUT_ROOT>/
  README.md
  README-MIS-001.md
  AGENT-MIS-001.json                # only when running `compact`
  MIS-001/
    Requirements.view.md
    Process.view.md
    Component.view.md
    Deployment.view.md
    State_Machine.view.md
    Threat_Model.view.md

    Requirement/
      REQ-001.md
      ...

    Feature/
      FEA-001.md
      ...
```

## What gets written

- `README.md` (output root): an index of Missions.
- `README-<MISSION_ID>.md` (output root): a per-mission README that links to cards and views.
- `<MISSION_ID>/<Card Type>/<CARD_ID>.md`: a Markdown file per card.
- `<MISSION_ID>/*.view.md`: Markdown files containing Mermaid diagrams for each default view.

## Clearing behavior

When clearing is enabled, the tool removes existing `.md` files under the output root before writing new content.

Known limitation: if you point `--output` at a directory that does not exist, the clear step can fail with an `os error 2`. Work around this by creating the output directory first.

## Known issues

- The generated root `README.md` currently links to `README-<MISSION_ID>-<Sanitized_Name>.md`, but the per-mission README file is written as `README-<MISSION_ID>.md`.
