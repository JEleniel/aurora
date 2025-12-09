# Tooling Notes

This page documents the minimal expectations for tooling that operates on AURORA artifacts.

- **Primary responsibility:** tooling is responsible for reading card JSON files and rendering them into diagrams, views, reports, and other consumable presentations. Example diagrams (under `docs/diagrams/`) show the intended views but are rendered by tooling in examples (they are not the canonical JSON).
- **Source of truth:** cards stored as JSON in `examples/cards/` are canonical; diagrams are views derived from those cards by tooling.
- **Validation:** tooling should validate cards against the JSON Schema files in `schemas/` (Draft-07). Validators should enforce `version` (semver), `status` (the enum), required `event_time` in audit entries, and allowed `type` values.
- **Serialization preferences:** the repository preference file `.aurora/preferences.json` specifies the preferred serialization (`json`) and other project-level flags (e.g., `pre_release`). Tooling should honor these preferences and reject YAML or other whitespace-delimited formats.
- **Diagram generation:** tooling may translate card graphs into Mermaid, SVG, or other diagram formats. The `docs/diagrams/drivers.md` file is a documentation example; the tooling should generate an equivalent view from the cards and links.
- **Links vs relations:** tooling should accept both inline `relations` arrays on cards and separate `link` artifacts (validated by `schemas/link.schema.json`). Prefer explicit link artifacts for richer metadata (rationale, strength) where needed.
- **Audit & provenance:** tooling must preserve `provenance` and `audit_history` when transforming or exporting artifacts and should provide human-friendly views of those records for review and compliance.

If you want, I can add a small validator script and an example `links/` directory with explicit link artifacts to demonstrate the full pipeline.
