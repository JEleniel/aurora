# ADR-002: **ADR**

**Name:** Strict JSJSON Canonical Storage\

## Description

Decision: Aurora cards are stored as strict JSON using the .jsjson extension. Rationale: deterministic diffs, wide tool compatibility, and unambiguous machine parsing. Trade-off: no comments; humans must rely on explicit fields and Notes instead.

## Links

- `documents` → `REQ-016`
- `documents` → `REQ-017`
- `documents` → `REQ-027`
