# Aurora User Documentation

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Start here

- **[Getting Started](./Getting%20Started.md)**
- **[Core Concepts](./Core%20Concepts.md)**
- **[Model Home Layout](./Model%20Home%20Layout.md)**
- **[Canonical Set](./Canonical%20Set.md)**
- **[Configuration Guide](./Configuration%20Guide.md)**
- **[Tools](./Tools.md)**
- **[MIS-002 Walkthrough](./MIS-002%20Walkthrough.md)**
- **[Troubleshooting](./Troubleshooting.md)**

## What Aurora is (in one page)

Aurora is a deterministic, typed, directed graph rooted at a single **Mission** card.

- **Cards** are nodes.
- **Links** are directed edges.
- **Meaning** comes from graph structure and allowed link types, not from diagram shapes or wording.
- **Views** are read-only projections and never modify the model.

Aurora is architecture and intent, not runtime state, operational instances, or implementation tracking.

## Where the “truth” lives

Aurora intentionally keeps its contracts simple:

- **Schemas** define structure and field constraints.
- **Reference registries** define the canonical card types, relationships, appearances, icons, and view definitions.
- **Models** are plain files (JSON and NDJSON) that are validated by schemas and checked against the registries.

In this repository, the authoritative specification artifacts live in `Aurora_Specs/`.

In a working model home (an `aurora/` folder), the tooling requires **local copies** of:

- `schemas/*.json` (the schemas)
- `reference/Aurora.modelconfiguration.json` (the canonical registry)
- `reference/SVGTemplate.svgz` or `reference/SVGTemplate.svg` (the SVG template)

See **[Model Home Layout](./Model%20Home%20Layout.md)** for details.
