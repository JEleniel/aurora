# Requirement: REQ-006 Audit Log Semantics

Each mission MUST have an append-only `AuditLog.ndjson` audit log where each line records one change event with timestamp, editor attribution, and a list of changed cards (including link changes when applicable). Multi-card operations (including batch edits) MUST be recorded as a single audit entry describing all affected cards and link changes. Audit entries MUST be appended by the editor/tooling layer that performs validation-gated writes, not by agents or UI code writing files directly.



## Attributes

_No attributes defined._

## Links

- requires [CAP-004](../Capability/CAP-004-Maintain_Audit_Trail.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | change |
