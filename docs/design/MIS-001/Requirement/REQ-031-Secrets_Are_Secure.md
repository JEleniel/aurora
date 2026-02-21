# Requirement: REQ-031 Secrets Are Secure

Secrets (for example API keys and tokens) MUST NOT be stored in plain text. Prefer OS keychain or equivalent secure storage. Agent calls MUST be scrubbed of secrets. Logs MUST NOT contain secrets.



## Attributes

_No attributes defined._

## Links

- requires [CAP-012](../Capability/CAP-012-Agent_Assisted_Modeling.md)
- requires [CAP-010](../Capability/CAP-010-Editor_Observability.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
