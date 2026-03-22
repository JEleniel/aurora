# Requirement: REQ-017 Prevent Invalid Edits

Aurora tooling (editor and MCP server) MUST prevent edits that would break a model (schema, registry constraints, invariants). Validation is performed at write points; if a candidate edit would fail validation, the write MUST be blocked and errors MUST be presented. Warning-only checks (for example naming/relationship verb linting) MUST remain warnings. These rules apply equally to human UI edits and agent-assisted edits.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-007](../Capability/CAP-007-Edit_Models_Interactively.md)
- requires [CAP-001](../Capability/CAP-001-Validate_Aurora_Models.md)


## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-18T13:55:00Z | Architect | create |
| 2026-02-21T00:00:00Z | Architect | change |
| 2026-02-21T18:10:00Z | Architect | change |
