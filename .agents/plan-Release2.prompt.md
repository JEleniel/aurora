# Project Plan

v2.0.0 release with comprehensive documentation, MIS-001 Architect session, cross-platform editor implementation, and in-code documentation for tools.

## Phase 1: Foundation & Design

- [ ] Architect works with user to design cross-platform editor.

## Phase 2: Tool Development

- [ ] Implement cross-platform editor per Architect design

## Phase 3: Documentation Writing

- [ ] Complete in-code documentation for svg_prep, aurora_shared, and aurora_cli
- [ ] Core Aurora documentation (concepts, card types, relationships, examples)
- [ ] aurora_cli documentation (commands, installation, workflows, troubleshooting)
- [ ] svg_prep documentation (icon ingestion, proof sheets, templates)
- [ ] Cross-platform editor documentation (features, workflows, UI guide)
- [ ] Developer/contributing guides (aurora_shared API reference, contributing to tools)

## Phase 4: Documentation Review & Refinement

- [ ] Review documentation for completeness, accuracy, clarity, consistency
- [ ] Validate documentation examples and links
- [ ] Organize docs/ hierarchy and finalize structure

## Phase 5: Integration & Release Prep

- [ ] Update CHANGELOG.md for v2.0.0
- [ ] Release v2.0.0

---

**Invariants** (durable, repo-wide):

- Keep `.agents/` docs accurate and non-redundant.
- Keep canonical schemas and reference registries authoritative under `.github/agents/aurora/`.
- Keep mission models under `docs/design/aurora/` passing `aurora_cli validate`.

Last updated: 2026-02-18
