# Requirement: REQ-037 Index And Record Provenance

When indexing and caching derived records for navigation/search, the editor and MCP server MUST track where each set of indices and derived records originates. Index and record stores MUST be namespaced by model home identity (and, where relevant, mission id) so multiple model homes can be opened and indexed concurrently without collisions. Provenance MUST include enough information to validate/invalidate cached data against the originating model home content (for example: model home path, schema/config versions, and content fingerprints).



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
| 2026-02-21T18:00:00Z | Architect | create |
