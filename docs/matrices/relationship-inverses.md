# Relationship Names and Inverses

This document defines relationship names, their human-readable display form, and an inverse wording suitable for UI and reports.

The machine-readable source for these mappings is `schemas/relationship-inverses.json`.

| Link token | Forward (source → target) | Inverse (target → source) | Symmetric |
| --- | --- | --- | --- |
| `derives-from` | derives from | drives | no |
| `satisfies` | satisfies | is satisfied by | no |
| `refines` | refines | is refined by | no |
| `depends-on` | depends on | is depended on by | no |
| `verified-by` | verified by | verifies | no |
| `related-to` | related to | related to | yes |
| `is-composed-of` | is composed of | composes | no |
| `aggregates` | aggregates | is aggregated by | no |
| `extends` | extends (is a) | is extended by | no |
| `implements` | implements | is implemented by | no |
| `realizes` | realizes | is realized by | no |
| `serves` | serves | is served by | no |
| `data-flows` | data flows to | receives data from | no |

Notes:

- Use the JSON mapping in `schemas/relationship-inverses.json` as the canonical source for rendering labels and determining whether a relation is symmetric.
- The generator and UI should prefer the `type` property on link artifacts; older examples may use `link_type` — both are supported by tooling, but `type` is canonical in schema.
