# Project Plan

Durable, repo-wide plan items. Use [.agents/PROGRESS.md](PROGRESS.md) for detailed status and per-owner checkpoints.

- [ ] Maintain `.agents/` docs (Brief / Plan / Map / Technologies / Context / Patterns)
- [ ] Keep canonical schemas/registries authoritative under `.github/agents/aurora/` and mirrored under `Aurora_Specs/`
- [ ] Maintain one or more Aurora mission models under `docs/design/aurora/` (source-of-truth cards)
- [ ] Keep mission models aligned to Aurora invariants (reachability, root safety, audit log semantics)
- [ ] Keep tooling components (`aurora_shared`, `aurora_cli`, editors) traceable to capabilities/features in the mission models
- [x] Implement `aurora_shared::render::svg::Svg` to render a `Model` + `Layout` into a standalone SVG export

Note: `tools/aurora_editor/` is currently deleted/placeholder in this workspace; a new editor will be started later.

Last updated: 2026-02-08 (added `aurora_shared` SVG renderer for layout-based graph export).
