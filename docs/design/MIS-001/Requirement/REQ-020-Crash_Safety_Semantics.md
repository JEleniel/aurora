# Requirement: REQ-020 Crash Safety Semantics

With autosave enabled, on crash the model may at worst contain an orphan card that needs to be linked; with autosave disabled, the saved model MUST always be valid and unsaved changes are lost on crash.



## Attributes

_No attributes defined._

## Links

- requires [CAP-008](../Capability/CAP-008-Persist_And_Package_Models.md)
- requires [CAP-004](../Capability/CAP-004-Maintain_Audit_Trail.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-18T13:55:00Z | Architect | create |
