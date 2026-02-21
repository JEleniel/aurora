# Activity: ATV-023 Serve MCP Tool Calls

Accept MCP tool calls (stdio or in-process transport) and dispatch them to the Aurora model tool implementation. Read-only calls consult the index/working set; write calls are confirmation-gated by the hosting UI policy, acquire the exclusive model lock, validate, apply transactionally, append one audit entry, and schedule asynchronous index updates. Every tool call returns machine-readable JSON with success/failure plus progress/cancellation where applicable.



## Attributes

_No attributes defined._

## Links

- uses [COM-010](../Component/COM-010-Aurora_MCP_Server.md)
- produces [ART-014](../Artifact/ART-014-Model_Tool_Call_Result.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T18:00:00Z | Architect | create |
