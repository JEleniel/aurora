# Project Brief

Aurora (Agent-Unified Representation of Requirements and Architecture) is a rigorous, machine-readable architectural modeling framework built around a directed graph of JSON “cards” rooted at a single `Mission` card. The repo includes schemas, default canonical definitions, and Rust tooling to validate models and render views.

## Repo Contents

- Rust workspace
    + CLI tooling: `tools/aurora_cli/`
    + Shared library (model + rendering): `tools/aurora_shared/`
    + Desktop editor: `tools/aurora_editor/` (currently deleted/placeholder; a new editor will be started later)
- Design documentation: `docs/design/`
    + Aurora model home (source-of-truth cards): `docs/design/aurora/`

## Canonical Definitions

- Canonical card types + relationship verbs (source): `.github/agents/aurora/Aurora.canonical.definitions.json`
    + Mirrored copy for convenience: `schemas/Aurora.canonical.definitions.json`
- Human-facing reference (generated/checked-in): `docs/design/Aurora.canonical.definitions.md`
