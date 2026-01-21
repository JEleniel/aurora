# Mermaid rendering rules

This page describes the recommended Mermaid conventions used when rendering Aurora views.

## Navigation

- [Aurora overview](README.md)
- [Views and rendering](views.md)
- [Design docs index](../design/README.md)

## Required init header

Each Mermaid diagram should start with the following line to enable ELK layout and transparent boundary subgraphs:

```text
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
```

## Diagram structure

For consistency, renderers should emit sections in this order, with a single blank line between sections:

1. Node definitions
2. Links (edges)
3. `classDef` entries
4. `class` assignments

## Node label format

Nodes are typically labeled using the card type and the card name.

Example:

```text
\tREQ-001["`**Requirement**<br />Users Are Authenticated`"]
```

## Class palette

Rendered views usually include a palette of `classDef` entries (one per card type) and assign each card to the appropriate class.

Example snippet:

```text
\tclassDef cls_mission fill:#022c22,color:#FFFFFF
\tclassDef cls_driver fill:#064e3b,color:#FFFFFF
\tclassDef cls_requirement fill:#065f46,color:#FFFFFF

\tclass MIS-001 cls_mission;
\tclass DRI-001 cls_driver;
\tclass REQ-001 cls_requirement;
```

## A small styled example

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	MIS-001(("`**Mission**<br />Enable Deterministic Tooling`"))
	DRI-001["`**Driver**<br />Reduce Operational Risk`"]
	REQ-001["`**Requirement**<br />Security Events Are Recorded`"]

	MIS-001 -- establishes --> DRI-001
	DRI-001 -- drives --> REQ-001

	classDef cls_mission fill:#022c22,color:#FFFFFF
	classDef cls_driver fill:#064e3b,color:#FFFFFF
	classDef cls_requirement fill:#065f46,color:#FFFFFF

	class MIS-001 cls_mission;
	class DRI-001 cls_driver;
	class REQ-001 cls_requirement;
```
