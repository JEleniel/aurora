# Component (Service): COM-010 Aurora MCP Server

Constrained model tool server that exposes the validation-gated model tool surface over MCP (structured JSON tool calls/results) without full model materialization. The server can run as a standalone stdio MCP process or be hosted in-process via a spawn-friendly entry point (thread friendly). It enforces schema/invariant validation, transactional writes, audit logging, backups, and exclusive-lock requirements by delegating core model operations to aurora_shared.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- composes [COM-001](COM-001-Aurora_Shared_Library.md)
- exposes [INT-002](../Interface/INT-002-Aurora_Model_Tool_MCP_Interface.md)
- exposes [INT-004](../Interface/INT-004-Aurora_Model_Tool_MCP_Query_Interface.md)
- exposes [INT-005](../Interface/INT-005-Aurora_Model_Tool_MCP_Edit_Interface.md)
- exposes [INT-006](../Interface/INT-006-Aurora_Model_Tool_MCP_Batch_Interface.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
| 2026-02-21T18:00:00Z | Architect | change |
| 2026-02-21T19:00:00Z | Architect | change |
| 2026-03-22T00:19:40Z | Copilot | change |
