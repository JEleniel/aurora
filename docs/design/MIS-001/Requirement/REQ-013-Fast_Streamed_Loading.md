# Requirement: REQ-013 Fast Streamed Loading

Models of any size SHOULD load almost instantly; the load method MUST traverse and validate in the time it takes to read files, and the editor MUST NOT load the entire model into memory at once.

## Attributes

_No attributes defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)
- requires [CAP-001](../Capability/CAP-001-Validate_Aurora_Models.md)

## Version

## Audit Log

| Timestamp            | Editor    | Change |
| -------------------- | --------- | ------ |
| 2026-02-18T13:55:00Z | Architect | create |
