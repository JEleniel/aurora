# Requirement: REQ-026 Searchable Index

The editor MUST build and maintain a searchable index so users and agents can navigate and search while keeping memory bounded. Full indexing SHOULD be available almost instantly for typical model homes. Minimum index coverage MUST include: card type, card subtype, card ID, card name, outbound link adjacency, and attribute property names. Indexing MUST update on save; index updates are asynchronous, expected to be inexpensive, and SHOULD not block the UI thread. Full-text search is optional.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- requires [CAP-011](../Capability/CAP-011-Index_And_Search_Model_Homes.md)
- requires [CAP-006](../Capability/CAP-006-Load_Model_Homes.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
