# Requirement: REQ-014 Backup Zip On Load

At load time, Aurora tooling (editor and MCP server) MUST begin creating a timestamped backup ZIP of the entire model home under `aurora/backups/` (for example `MIS-001-20260210T061800Z.zip`). Backup creation MUST be asynchronous and MUST NOT block UI interactivity or tool-call responsiveness. If backup creation fails, the tool MUST warn the user but continue loading. A configurable number of ZIPs will be retained (default 5), with older backups automatically deleted.



## Attributes

_No attributes defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-18T13:55:00Z | Architect | create |
| 2026-02-21T00:00:00Z | Architect | change |
| 2026-02-21T18:10:00Z | Architect | change |
