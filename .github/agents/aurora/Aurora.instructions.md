---
applyTo: '**/aurora/**/*'
---

# Aurora Machine Agent Instruction

## Model Overview

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges defined by a canonical relationship registry. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it.

Attributes capture properties of an element that are not already expressed by `card_type`, `card_subtype`, `name`, or `description`, while links capture relationships and interactions between elements. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

## Canonical split: schema vs instruction

- Schemas are canonical for structure and field constraints.
- Instructions are canonical for behavior and process rules.
- Instructions MUST NOT redefine schema structure.
- Examples are illustrative and non-canonical.
- `Aurora.canonical.definitions.json` is machine-first canonical data.

**One Goal**: Enable the Architect to focus on modeling the architecture instead of drawing diagrams and pictures.

## Models

A model is the central artifact in Aurora. It is a collection of cards that represent architectural elements, connected by links that describe their relationships, starting from a `Mission`.

Each model is identified by its `Mission` ID.

### Getting started

1. Create or locate the model home folder: `aurora/` (located at `docs/design/aurora/` by default).
2. Ensure `aurora/schemas/` contains `Aurora.card.schema.json`, `Aurora.audit.schema.json`, `Aurora.compact.schema.json`, and `Aurora.changed.cards.schema.json`. If missing, copy them from `.github/agents/aurora/`.
3. Create the `Mission` card in the model home (`aurora/`).
4. Add other cards under `{mission id}/{card type}/` and link them from existing cards.
5. Append to `{mission id}/AuditLog.ndjson` for every change event. One entry may include changes to multiple cards.
6. Regenerate `{mission id}/ChangedCards.json` as a **snapshot** after each appended audit entry.

At the end of making changes, validate the model(s), generate the Markdown, views, and compact model. **If this fails do not stop working.**

### How the Model Works

Any pair of cards in the model can be described using simple sentences:

**Examples**:

```text
The mission "Drive Excellence" is "Drive excellence in operations by streamlining processes, integrating automation, and formalizing documentation".

The mission establishes the driver "Operational Friction Elimination" which is "Eliminate non-value-adding manual effort by enforcing end-to-end process automation, standardized workflows, and machine-verifiable documentation across all operational domains".

"Operational Friction Elimination" drives the requirement "Define a Deterministic Modeling Framework" which is "Design a framework for documenting deterministic process models with measurable latency and failure semantics".
```

**These sentences produce a model that looks like this**:

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	drive_excellence(("`Mission:<br />Drive Excellence`"))
	operational_friction_elimination(["`Driver:<br />Operational Friction Elimination`"])
	define_a_deterministic_modeling_framework(["`Requirement:<br />Define a Deterministic Modeling Framework`"])

	drive_excellence -- establishes --> operational_friction_elimination
	operational_friction_elimination -- drives --> define_a_deterministic_modeling_framework
```

### Cards

Cards represent architectural elements (nouns). A card contains properties of the element and links to other elements. Use attributes for properties that are not already represented by `card_type`, `card_subtype`, `name`, or `description`. Use links for relationships and interactions between elements. Card files should be "pretty printed" using `prettier` or a similar tool.

Card field structure is defined exclusively in `Aurora.card.schema.json`.

**Example `id`s**:

- MIS-001
- DRI-001
- DRI-002

#### Canonical Cards

A canonical set of cards and relationships is included. The canonical set is designed to cover all normal architectural elements and ensure that the relationships conform to the invariants. The canonical set also includes common subtypes for convenience. Aurora is designed to be easily extended, so models are not limited to the canonical set.

- [Aurora Canonical Definitions](Aurora.canonical.definitions.json)
- [Aurora Canonical Definitions Schema](Aurora.canonical.definitions.schema.json)

#### Non-Canonical Attributes

The following is a list of commonly used attributes. It is not canonical:

- Global (All Cards)
   	+ assumptions: array of strings
- Mission (MIS)
   	+ in_scope: array of strings
   	+ out_of_scope: array of strings
- System (SYS)
   	+ in_scope: array of strings
   	+ out_of_scope: array of strings
- Application (APP)
   	+ in_scope: array of strings
   	+ out_of_scope: array of strings
- Capability (CAP)
   	+ in_scope: array of strings
   	+ out_of_scope: array of strings
- Threat Model (THM)
   	+ in_scope: array of strings
   	+ out_of_scope: array of strings
- Artifact (ART)
   	+ format: string
   	+ data_classification: string

#### Appearance Definitions

To aid rendering, Aurora includes a set of shapes, icons, backgrounds, and foregrounds for each card type in the canonical set. Icons can be any valid Unicode character.

- [Aurora Appearance Configuration](Aurora.appearance.json)
- [Aurora Appearance Configuration Schema](Aurora.appearance.schema.json)

### File and Folder Structure

Models live in a folder named `aurora/` (the model home). If an `aurora/` folder already exists at the target location, use it. Never create an `aurora` folder inside another `aurora` folder. Multiple models may share one model home. The `Mission` card is stored in the model home, and all other cards are stored in mission- and card-type-specific folders: `{mission id}/{card type}/`.

A model may include five kinds of files:

1. Schemas: `Aurora.card.schema.json`, `Aurora.audit.schema.json`, `Aurora.compact.schema.json`, and `Aurora.changed.cards.schema.json` files in `aurora/schemas/`.
    + These schemas are shared by all models, audit logs, and compact models in the same model home.
    + If these files are not present, or when creating a new model home, copy only those schemas from `.github/agents/aurora/` into `aurora/schemas/` before creating any other files.
    + All model JSON document files MUST have the `$schema` attribute with the relative path from that file to the appropriate schema. Entries in `AuditLog.ndjson` are line-delimited JSON objects and do not include `$schema`.
2. Cards: the central component of the model, each card is stored in a separate JSON file named `{id}-{name}.json` where name has had all symbols removed and spaces replaced with underscores. All cards must conform to the `Aurora.card.schema.json` in the model home.
3. Audit Log: an append-only, line-delimited JSON history of change events over time, stored at `{mission id}/AuditLog.ndjson`. Each line MUST conform to `Aurora.audit.schema.json` and may capture multiple changed cards in one entry. An audit log entry MUST be appended for every change event.
4. Changed Cards Snapshot: a mission-local snapshot at `{mission id}/ChangedCards.json`, conforming to `Aurora.changed.cards.schema.json`. It MUST be regenerated after each appended audit entry.
5. Compact Model: an optional compact, single file version of the model at `{mission id}/Compact.json` and conforming to the `Aurora.compact.schema.json` in the model home.

The model home may be stored as a ZIP file for transport if the folder structure is preserved.

**Example Folder and File Structure**:

```text
aurora
  ├─ MIS-001
  │    ├─ Driver
  │    │    ├─ DRI-001-Some_Reason.json
  │    │    └─ DRI-002-Another_Reason.json
  │    ├─ Requirement
  │        └─ REQ-001-Something_Has_To_Happen.json
  │    ├─ AuditLog.ndjson
  │    ├─ ChangedCards.json
  │    └─ Compact.json
  ├─ MIS-002
  │    ├─ Driver
  │    └─ ...
  ├─ schemas
  │    ├─ Aurora.audit.schema.json
  │    ├─ Aurora.changed.cards.schema.json
  │    ├─ Aurora.card.schema.json
  │    └─ Aurora.compact.schema.json
  ├─ MIS-001-Enable_Deterministic_Aurora_CLI_Tooling.json
  └─ MIS-002-Write_User_Documentation_for_Aurora.json
... etc
```

### Invariant Rules

These invariant rules ensure that the model is a traversable, directed graph with local cycles only (cycles that do not reach a view root).

1. **The `Mission` Card**: All models must start with and include a single `Mission` card that summarizes the high-level "why" of the project. The `Mission` card must only have outgoing links and serves as the root of the directed graph.
2. **Direction (graph links)**: When traversing from `Mission`, all links must lead away from the `Mission` card. Any traversal path that starts from `Mission` must end either in a leaf card or a previously seen card, creating a local loop.
3. **No orphans**: Other than the `Mission` card, all cards must have one or more incoming links and a path from the `Mission` card. All cards may have any number of outgoing links. All link targets must be valid cards in the model.

## Views

Views are generated by selecting a set of card types as roots for local graphs, selecting a set of other card types to include when traversing from the roots, and rendering a diagram showing those cards and the links between them. Views **do not change the model**; they change what part of and how the model is viewed. One model, many views.

The following is a set of common view definitions based on the canonical set:

- [View Definitions](View.Definitions.json)
- [View Definitions Schema](View.Definitions.schema.json)

### View roots and traversal

- Traversal always starts from each root and (by the invariants) continues away until either reaching a leaf or closing a local loop.
- Links are not filtered for general traversal.
- When rendering a view, root cards are always rendered.
- During traversal, a non-root card is eligible to be rendered if its `card_type` is in `included_card_types`.
    + Nothing outside the rendered cards (and the links between them) is rendered.
