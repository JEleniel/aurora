# Requirement: REQ-027 Index Cache In User Cache Directory

Index persistence MUST use the user cache directory (OS cache folder) rather than the model home. Cached indices are disposable cache artifacts: they SHOULD be safe to delete, MUST be validated/invalidated against the current model home content, and MUST be rebuilt when stale or incompatible.



## Attributes

_No attributes defined._

## Links

- requires [CAP-011](../Capability/CAP-011-Index_And_Search_Model_Homes.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
