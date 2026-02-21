# Requirement: REQ-035 Tool Call Progress And Cancellation

For long-running model tool operations (load/index/render/validate/batch edits), the editor and MCP server MUST remain responsive: every tool call MUST return a structured success or failure result, MUST report progress when work exceeds interactive latency budgets, and MUST support cancellation without leaving the model in an invalid state.



## Attributes

_No attributes defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)
- requires [CAP-012](../Capability/CAP-012-Agent_Assisted_Modeling.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T18:00:00Z | Architect | create |
