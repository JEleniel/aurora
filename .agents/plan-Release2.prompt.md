# Project Plan

v2.0.0 release with comprehensive documentation, MIS-001 Architect session, cross-platform editor implementation, and in-code documentation for tools.

## Phase 1: Foundation & Design

- [ ] Architect works with user to design cross-platform editor.

## Phase 2: Tool Development

- [ ] Implement cross-platform editor per Architect design

## Phase 3: Documentation Writing

- [ ] Complete in-code documentation for svg_prep, aurora_shared, and aurora_cli
- [x] Core Aurora documentation (concepts, card types, relationships, examples)
- [x] aurora_cli documentation (commands, installation, workflows, troubleshooting)
- [x] svg_prep documentation (icon ingestion, proof sheets, templates)
- [ ] Cross-platform editor documentation (features, workflows, UI guide)
- [ ] Developer/contributing guides (aurora_shared API reference, contributing to tools)

## Phase 4: Documentation Review & Refinement

- [ ] Review documentation for completeness, accuracy, clarity, consistency
- [ ] Validate documentation examples and links
- [ ] Organize docs/ hierarchy and finalize structure

## Phase 5: Integration & Release Prep

- [ ] Update CHANGELOG.md for v2.0.0
- [ ] Release v2.0.0

## Parked for the next release:

- `aurora_cli` can cryptographically sign cards and models and verify signatures to ensure authenticity and integrity of card data.
- `AuditLog.json` extended to include an optional compressed record of the specific changes made to each card, including before and after states, to enable detailed change tracking and history.
- Review the implications of using a single JSON file for an entire model, and the effect on agent context windows and performance. Alternatives include a file per card type, or a simplified file format that allows streaming and partial loading of card data.
- Add a lightweight, standalone card viewer that can render cards from JSON files without needing the full aurora_cli toolchain, to enable easier sharing and viewing of cards outside of aurora-specific contexts.
- Implement a lightweight, standalone configuration editor that can manage card definitions, appearance options, relationships, and other model settings in a user-friendly way, without needing to edit JSON files directly.
- Design and build an MCP server that can parse and manage card data and enable agents to work with models in a more performant and scalable way, without needing to load entire models into memory or rely on file-based storage. This could include features like caching, indexing, and query capabilities to optimize agent interactions with card data.
- Look into the possibility of a delta feature for the CLI that can generate a diff of changes between two versions of a model, to help with tracking changes, code reviews, and understanding the evolution of card data over time. This definitely depends on having some kind of change tracking or versioning system in place, but could be a powerful tool for managing complex models and ensuring transparency in changes.
- Research other architectural functions and practices to allow Aurora to become a one-stop shop for all things agent architecture, including design, documentation, implementation, and best practices. This could include things like design patterns, architectural principles, case studies, and more to help architects and developers build better agents and models.

---

**Invariants** (durable, repo-wide):

- Keep `.agents/` docs accurate and non-redundant.
- Keep canonical schemas and reference registries authoritative under `.github/agents/aurora/`.
- Keep mission models under `docs/design/aurora/` passing `aurora_cli validate`.

Last updated: 2026-02-18
