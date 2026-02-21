# Requirement: REQ-012 Model Home Local Schemas And References

Aurora tooling (editor and MCP server) MUST load and use the schemas and reference files included with the selected model home (not built-in or global defaults), allowing tooling to operate across multiple Aurora versions and customizations. At minimum this includes: using `schemas/*` for validation and using `reference/Aurora.modelconfiguration.json` (including its `version`) for canonical card registries, appearance/theming, and view definitions.



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
