# Aurora Model Quickstart

This repository uses the Aurora model format to describe architecture as a deterministic directed graph of cards (nodes) connected by relationship links (edges). The goal is that humans can read it, tools can validate it, and agents can reason over it.

## Source of truth vs generated artifacts

- Source of truth: `docs/design/aurora/` (the model).
    + This directory contains the mission card(s) and the per-mission folders containing all other cards (Drivers, Requirements, Systems, etc.).
    + If you are changing the architecture model, you should edit files under `docs/design/aurora/**`.
- Generated artifacts: typically written under `docs/design/**` (rendered documentation and views).
    + These files are outputs of `aurora_cli` and are safe to regenerate.
    + If a rendered file looks “wrong”, fix the model and re-render rather than editing the generated output.
    + Generated artifacts can be edited if you are explicitly asked to, but treat them as ephemeral unless the change is also reflected in the model.

A common output layout (when rendering to `docs/design/`) looks like this:

- `docs/design/<Rendered_Model_Name>/...` (per-card rendered Markdown)
- `docs/design/<Rendered_Model_Name>/Views/*.view.svg` (rendered SVG views)
- `docs/design/<Rendered_Model_Name>/Views/source/*.view.dot` (Graphviz DOT sources used to produce the SVG)

You may also see these view paths written in shorthand as `docs/design/Views/*.view.svg` and `docs/design/Views/source/*.view.dot`; interpret that as “within the rendered model output directory”, which is typically `docs/design/<Rendered_Model_Name>/`.

## CLI usage: global options come first

`aurora_cli` follows a strict CLI shape where global options must appear before the subcommand.

For example, `-i/--input` is a global option:

```text
aurora_cli -i docs/design/aurora validate
```

Not:

```text
aurora_cli validate -i docs/design/aurora
```

If you omit `-i`, the default input is `docs/design/aurora`.

## Canonical commands

All commands below accept the global `-i/--input <PATH>` option. The `<PATH>` can point to a mission card, a model home, or an ancestor directory.

- Validate the model:

```text
aurora_cli -i docs/design/aurora validate
```

- Render everything (card Markdown and relationship views):

```text
aurora_cli -i docs/design/aurora render-all -o docs/design/
```

- Render just the views (DOT → SVG):

```text
aurora_cli -i docs/design/aurora render-views -o docs/design/
```

- Generate or refresh the compact “agent export”:

```text
aurora_cli -i docs/design/aurora compact
```

Optional: choose a specific output file for the compact export:

```text
aurora_cli -i docs/design/aurora compact -o docs/design/aurora/AGENT-MIS-001.jsjson
```

Notes:

- `render-all` and `render-views` write under a per-model output directory inside the `--output` directory.
- If you are working on a different mission, adjust the `AGENT-<MISSION_ID>.jsjson` file name accordingly.

## Where rendered views and DOT sources go

When you render views to `docs/design/`, you should expect them under the rendered model folder, not directly under `docs/design/Views/`.

Examples:

- SVG views:
    + `docs/design/<Rendered_Model_Name>/Views/*.view.svg`
- DOT sources:
    + `docs/design/<Rendered_Model_Name>/Views/source/*.view.dot`

If you need to inspect “why” a view looks the way it does, start with the `.view.dot` file in the `source/` directory.

## Quick workflow (edit → validate → render)

- Edit the model under `docs/design/aurora/**`.
- Validate:

```text
aurora_cli -i docs/design/aurora validate
```

- Render:

```text
aurora_cli -i docs/design/aurora render-all -o docs/design/
```

## Related docs

- See the canonical view registry and type color palette in [`.github/instructions/View_Definitions.md`](../../.github/instructions/View_Definitions.md).
- For Graphviz styling conventions and how to override defaults via the model, see [`.github/instructions/Graphviz_View_Styling_Guide.md`](../../.github/instructions/Graphviz_View_Styling_Guide.md).
