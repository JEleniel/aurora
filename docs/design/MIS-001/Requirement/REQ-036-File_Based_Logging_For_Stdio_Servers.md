# Requirement: REQ-036 File Based Logging For Stdio Servers

When running as an stdio MCP server, logging MUST be file-based (and MUST NOT corrupt the protocol stream). The server SHOULD still support optional console logging when not using stdio for protocol transport.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-010](../Capability/CAP-010-Editor_Observability.md)
- imposes [CNS-006](../Constraint/CNS-006-Fern_Logging.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T18:00:00Z | Architect | create |
