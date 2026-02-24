# Requirement: REQ-028 Agent Tool Surface Only

Aurora tooling MUST expose a constrained model tool surface that is the only way an agent can read or modify the model. Tool calls and tool results MUST be structured, machine-readable JSON (for example via MCP). Minimum non-mutating tools MUST include: find cards by id/name/type/subtype; fetch a card's normalized representation (attributes + links); fetch inbound/outbound adjacency and bounded neighborhood expansions; query model configuration; compute viable roots per the root safety rule; retrieve validation errors/warnings for a candidate edit; and list possible next target card types based on model configuration. Minimum write tools MUST include: create/update/delete cards (updates supply a complete replacement payload); create/update/delete links; and apply a batch edit as a single all-or-nothing operation with a clear failure report. All writes MUST be transactional and validation-gated, MUST require holding the exclusive model lock, and MUST append exactly one audit entry per successful operation.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-012](../Capability/CAP-012-Agent_Assisted_Modeling.md)
- imposes [CNS-007](../Constraint/CNS-007-Tool_Mediated_Agent_Actions.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
| 2026-02-21T18:10:00Z | Architect | change |
