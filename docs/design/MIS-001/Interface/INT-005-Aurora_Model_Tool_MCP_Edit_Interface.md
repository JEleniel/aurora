# Interface: INT-005 Aurora Model Tool MCP Edit Interface

Mutating MCP tool subset for editing Aurora model homes via validation-gated, transactional operations (no direct file writes). Covers the minimum write capabilities in REQ-028 (create/update/delete cards; create/update/delete links) and enforces confirmation gating by hosting policy (REQ-029), exclusive lock semantics (REQ-025), and validation on write (REQ-017).



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- accepts [ART-013](../Artifact/ART-013-Model_Tool_Call_Request.md)
- returns [ART-014](../Artifact/ART-014-Model_Tool_Call_Result.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T19:00:00Z | Architect | create |
