# Agent-Unified Representation of Requirements and Architecture (Aurora)

 Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Status

v1.0.0 released
v2.0.0 in development

## Goals

- **Unified Semantics:** One representation equally usable by humans and machines.
- **Complete Architectural Coverage:** One model that includes operational, logical, physical, and behavioral modeling with complete traceability.
- **Tool-Agnostic Implementation:** Built on open standards, Aurora is simple to adopt, serialize, and integrate.
- **Open Data Formats:** Aurora uses JSON, NDJSON, and JSON Schemas for all data files and configuration. Output is Markdown and SVG.
- **Automated Reasoning Compatibility:** Strictly defined according to graph theory to enable reasoning directly from the model.
- **Human Readability:** Clear structure and terminology without sacrificing formal rigor.
- **Complete Provenance and Traceability:** Every action on a card is logged, providing a complete audit trail of who made changes. Delta are not tracked.

## Core Concepts

Aurora models are intentionally simple and explicit. Every architectural element is represented by a `card`, in graph terms a node, containing as little or as much detail as required. Interactions and relationships are `relationships`, in graph terms directed edges, between two cards. Beyond that, there are only three invariant rules that guarantee properties of the model that enable reasoning directly from the model:

### Invariant Rules

These invariant rules ensure that the model is a rooted directed graph with only local recurrence.

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links and serves as the root node of the directed graph.
2. **Direction (graph links)**: Traversal follows directed edges from `Mission` outward. Traversal algorithms MUST halt when they encounter either a leaf node (out-degree `0`) or a previously visited node.
3. **No orphans**: Other than the `Mission` card, all cards must have one or more incoming links and a path from the `Mission` card. All cards may have any number of outgoing links. All link targets must be valid cards in the model.

## Validation

- Every JSON and NDJSON file used in Aurora also has a schema.
- Each set of models (models can share a home) includes configuration files to define the available card types, relationships, appearance of rendered symbols, and view definitions.
- Between the invariant rules and the reference files a model can be easily validated, manually or with tools. Our tools all include validation built right in.

## Views

_Any_ view can be generated from the model simply by selecting what elements (card types) to show.

## The Canonical Set

While Aurora can use any set of elements and relationships to describe a model, we have included a complete, canonical set that covers everything describable in common architectural diagramming frameworks, like UML and SysML.

The canonical set includes:

- A complete set of card types and common subtypes.
- Appearance settings for every card, including fill, stroke, and text color; icon; and shape of the symbol.
- A complete set of relationships guaranteed to conform to the invariants.
- Definitions for a set of the most common architectural views (or diagrams).
- Over 128 SVG icons that can be included in any card.

## Tooling

Aurora includes a set of reference tooling including an editor, a standalone viewer, a utility to customize the rendering of diagrams, a command line tool, and a meta-model editor.

### Editor

We provide a cross-platform, full-featured editor for Aurora models. Everything you need to create, edit, and distribute your architecture is built-in.

#### Features

- **Interactive Exploration:** Navigate and understand complex architectures through centered element views with related links and dependencies displayed at a glance.
- **Built-in Views:** Can read view definitions and render any view.
- **Compressed Storage:** Can store the model in a single ZIP-compressed file.
- **Schema-Aware Editing:** Can read and enforce all Aurora schemas.
- **Live Validation & Linting:** Real-time model validation as well as style checking and linting.
- **Model Merge & Conflict Resolution:** Tools to merge divergent model branches with semantic conflict helpers.
- **Agent Integration & Automation:** Expose APIs for machine agents to read/write cards, run validations, and generate views.
- **Render Diagrams** Render the views into SVG diagrams.
- **Accessibility:** WCAG AA compliant accessibility.

#### Future Ideas

- **Access Control & RBAC:** Role-based permissions, read/write controls, and audit logging.
- **Extensible Plugin API:** Allow third-party extensions for importers, exporters, visualizations, and automation.
- **Collaborative Editing:** Real-time collaboration (multi-user) with conflict resolution and presence indicators.

### Viewer

We provide a completely stand-alone, cross-platform viewer that allows you to browse models using the same views as the editor, and any of the defined views. It also allows creating custom views just like the editor.

#### Features

- **Interactive Exploration:** Navigate and understand complex architectures through centered element views with related links and dependencies displayed at a glance.
- **Multi-View Support:** Provide all default and custom views with fast switching between perspectives.
- **Custom View Rendering:** Load and render user-defined views.
- **View Exporting:** Export any view as a SVG file.
- **Search, Filter & Highlight:** Full-text search, attribute filters, and path highlighting to trace relationships.
- **Offline & Portable:** Network not required.
- **High-Performance Rendering:** Handles large models multiple optimizations.
- **Accessibility:** WCAG AA compliant accessibility.

#### Future Ideas

- **Plugin Support:** Allow viewer extensions for custom rendering, analytics, or integrations.

### CLI Tool

We provide a scriptable command line tool supporting common operations.

#### Features

- **Validate Models:** Verify that a set of model files conforms with the specification.
- **Canon Check Models:** Check models against the canon definitions or your own custom set to identify deviations.
- **Generate Views:** Generate the entire set of defined views as SVG files with a single command.
- **Generate Markdown:** Generate Markdown of the entire model with a file per card, an index, and an executive summary.
- **Batch Operations:** Run operations across many models in a shared model home.

#### Future Ideas

- **Diff & Report Semantic Changes:** Show human-friendly diffs between two model snapshots.
- **Format & Pretty-Print:** Reformat JSON files to a canonical, stable layout for diffing and review.
- **Init New Model:** Scaffold a new `Aurora` folder with example `mission` and schema files.
- **Merge Models:** Assist with merging models and resolving conflicts with semantic awareness.
- **Sign & Encrypt Archives:** Support cryptographic signing and optional encryption of model archives.

## License

See [LICENSE.md](LICENSE.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## Support

See [SUPPORT.md](SUPPORT.md).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
