# Requirement: REQ-034 Modelconfiguration Is Versioned

`reference/Aurora.modelconfiguration.json` MUST include a `version` property so tooling can identify the exact canonical registry version. The modelconfiguration schema MUST require this field, and tooling that loads the registry MUST parse and surface it for diagnostics and compatibility decisions.



## Attributes

_No attributes defined._

## Links

- requires [CAP-005](../Capability/CAP-005-Maintain_Canonical_Registries.md)
- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)
- requires [CAP-001](../Capability/CAP-001-Validate_Aurora_Models.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
