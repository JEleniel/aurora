# Requirement: REQ-025 Single Instance Model Semantics

Multiple instances editing the same model are not supported. Aurora tooling (editor and MCP server) MUST prevent accidental concurrent editing using OS-level locking by holding an exclusive lock (write handle) on the mission audit log at `aurora/<MISSION_ID>/AuditLog.ndjson` for the full duration of an editing session. If the exclusive lock cannot be acquired because it is already held, the tool MUST refuse to open the model and MUST present a clear locked-model error. This relies on the OS to release locks on crash, minimizing stale-lock cleanup. Shared/networked models remain not officially supported.



## Attributes

_No attributes defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)
- requires [CAP-007](../Capability/CAP-007-Edit_Models_Interactively.md)
- requires [CAP-008](../Capability/CAP-008-Persist_And_Package_Models.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-18T13:55:00Z | Architect | create |
| 2026-02-21T00:00:00Z | Architect | change |
| 2026-02-21T18:10:00Z | Architect | change |
