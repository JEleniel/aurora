# Requirement: REQ-013 Fast Streamed Loading

Models of any size SHOULD load almost instantly; the editor MUST become interactive with visible progress feedback within 2 seconds on the minimum target platform (8GiB RAM baseline) for a typical model home (~2000 cards, ~3500 links). If reading model files exceeds this budget, the editor MUST still open quickly and continue loading with progress feedback. Operations MAY take longer on large models, but the editor MUST remain responsive (no runaway memory growth) and MUST NOT load the entire model into memory at once.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)
- requires [CAP-001](../Capability/CAP-001-Validate_Aurora_Models.md)


## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-18T13:55:00Z | Architect | create |
| 2026-02-21T00:00:00Z | Architect | change |
