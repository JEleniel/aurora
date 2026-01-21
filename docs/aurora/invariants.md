# Invariant rules

Aurora models follow a small set of invariant rules that make automated validation and deterministic rendering possible.

## Navigation

- [Aurora overview](README.md)
- [Concepts: cards, links, graph](concepts.md)
- [Views and rendering](views.md)
- [Design docs index](../design/README.md)

## Invariants (high level)

1. Every model starts with exactly one Mission card.
2. Links lead away from the Mission.
3. Every card must be reachable from Mission.
4. Local cycles are allowed (for example, state-machine loops), but no path can lead back to Mission.
5. All `links[].target` values must reference existing cards.

## Why these rules exist

These rules guarantee:

- There is a single, unambiguous entry point for tools.
- The model can be traversed deterministically.
- Renderers can generate consistent views without guessing.
- Validation can catch missing/typo’d links early.

## Example: a local cycle (allowed)

State machines often require loops.

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR
	STM-001["`**State Machine**<br />Session Lifecycle`"]
	STA-001["`**State**<br />Logged Out`"]
	STA-002["`**State**<br />Logged In`"]
	EVT-001["`**Event**<br />Login Requested`"]
	CON-001["`**Condition**<br />Credentials Valid`"]

	STM-001 -- starts_in --> STA-001
	STA-001 -- receives --> EVT-001
	EVT-001 -- triggers --> CON-001
	CON-001 -- triggers_true --> STA-002
	CON-001 -- triggers_false --> STA-001
```

The loop is contained within the state-machine subgraph; it does not point back to the Mission.

## Validating invariants

Use `aurora_cli validate` to validate schema and invariants.

```text
aurora_cli --input <MODEL_HOME> validate
```

Sample output:

```text
All models are valid.
```
