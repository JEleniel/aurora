# Project Brief

Aurora (Agent-Unified Representation of Requirements and Architecture) is a rigorous, machine-readable architectural modeling framework built around a directed graph of JSON “cards” rooted at a single `Mission` card. The repo includes schemas, default canonical definitions, and Rust tooling to validate models and render views.

## Repo Contents

- Rust workspace
   	+ CLI tooling: `tools/aurora_cli/`
   	+ Shared library (model + rendering): `tools/aurora_shared/`
   	+ Desktop editor: `tools/aurora_editor/`
- Design documentation: `docs/design/`

## Canonical Definitions

- Canonical card types + relationship verbs (input): `.github/agents/aurora/Aurora.canonical.definitions.json`
- Generated visualization (output): `docs/design/Canonical.md`
