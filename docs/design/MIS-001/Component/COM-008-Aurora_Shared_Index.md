# Component (Module): COM-008 Aurora Shared Index

Indexing subsystem (shared by the editor and MCP server) that builds, persists, and queries the model index (for example using Tantivy) while keeping the host responsive and memory bounded. The index MUST be namespaced by model home identity and MUST track provenance so multiple model homes can coexist without collisions.



## Attributes

_No attributes defined._

## References

_No references defined._

## Links

- implements [FEA-012](../Feature/FEA-012-Index_And_Search.md)
- stores in [DST-001](../Data_Store/DST-001-Model_Home_Index_Store.md)
- produces [ART-015](../Artifact/ART-015-Index_Provenance_Manifest.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | Architect | create |
| 2026-02-21T18:00:00Z | Architect | change |
