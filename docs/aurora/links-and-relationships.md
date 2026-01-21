# Links and relationships

Links connect cards into a directed graph.

## Navigation

- [Aurora overview](README.md)
- [Concepts: cards, links, graph](concepts.md)
- [Invariant rules](invariants.md)
- [Design docs index](../design/README.md)

## Link structure

A link is an object inside a card's `links` array.

Common fields:

| Field | Meaning |
| --- | --- |
| `target` | The id of another card in the same model. |
| `relationship` | A verb phrase describing the connection for humans. |

## Relationship verbs

The relationship label is descriptive. Aurora encourages consistent verbs for readability and to reduce ambiguity.

Common conventions include:

- `establishes` (Mission → Driver)
- `drives` (Driver → Requirement)
- `satisfies` (Capability/Feature → Requirement)
- `enables` (Feature → Capability)
- `necessitates` (Mission → System)
- `integrates` (System → Application)
- `comprises` (Application → Component)
- `exposes` (Component → Interface)
- `persists_to` (Artifact → Data Store)

## A relationship chain example

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	MIS-001(("`**Mission**<br />Enable Deterministic Tooling`"))
	DRI-001["`**Driver**<br />Reduce Operational Risk`"]
	REQ-001["`**Requirement**<br />Security Events Are Recorded`"]
	CAP-001["`**Capability**<br />Record Audit Events`"]
	FEA-001["`**Feature**<br />Audit Log Viewer`"]

	MIS-001 -- establishes --> DRI-001
	DRI-001 -- drives --> REQ-001
	CAP-001 -- satisfies --> REQ-001
	FEA-001 -- enables --> CAP-001
```

## Boundary and Note cards

Some cards affect how views are rendered:

- `Boundary` cards can group other cards when rendering views.
- `Note` cards are annotations (often rendered with dotted links).

The underlying model is still just cards and links.
