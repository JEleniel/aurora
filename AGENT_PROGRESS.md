# AGENT_PROGRESS

## Project Brief

AURORA is a deterministic, JSON-based architectural modeling format where Cards (JSON files) are connected by directed Links away from a single root `Mission` card.

- Feature: Maintain authoritative schema and modeling guidance
    + Status: In Progress
    + Primary schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
    + Compact schema: [schemas/Aurora.compact.schema.json](schemas/Aurora.compact.schema.json)
    + Modeling guidance: [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md)

## Active Context Summary

- Branch: `v2.0.0`
- Current work: implementing the `tools/aurora_cli` backend (model loader, validator, deterministic card/view renderers, CLI entrypoints) directly from the design model in `docs/design/aurora/`
- Backend update (2026-01-21): Mission loader now enforces sanitized `MIS-###-Name` filenames and reports schema violations with per-field line/column context during validation ([tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs)).
- Backend update (2026-01-21): Aurora root detection now keys off the presence of schema files, card discovery enforces `<Mission>/<CardType>/<ID>.json` layout with pointer-aware diagnostics, and compact exports serialize borrowed cards instead of cloning ([tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs), [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs), [tools/aurora_cli/src/aurora/model/compact_model.rs](tools/aurora_cli/src/aurora/model/compact_model.rs)).
- Backend update (2026-01-20): Improved `aurora_cli` model detection when invoked from subdirectories by resolving the default `--input` relative to the nearest git root and expanding Aurora home discovery to search upwards and include `docs/design/aurora/` ([tools/aurora_cli/src/lib.rs](tools/aurora_cli/src/lib.rs), [tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs)).
- Backend update (2026-01-20): Implemented boundary-aware Mermaid rendering in [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs), including subgraph generation, node/edge output, and class mapping normalization.
- Recent progress: rewired the CLI around new modules (`loader`, `validator`, `render`, `output`), added integration tests, fixed schema compilation + deployment view handling issues uncovered during `cargo test`, and added a `full` command that chains validation, render-all, and compact export
- Backend update (2026-01-19): Implemented the CLI renderer, bumper, and compactor flows in [tools/aurora_cli/src/renderer.rs](tools/aurora_cli/src/renderer.rs), [tools/aurora_cli/src/bumper.rs](tools/aurora_cli/src/bumper.rs), and [tools/aurora_cli/src/compactor.rs](tools/aurora_cli/src/compactor.rs) so each subcommand now produces deterministic Markdown/JSON output rather than panicking with `todo!()` placeholders.
- Backend update (2026-01-19): Follow-up fixes wired the renderer through dedicated card/model/view modules, added Card helper methods for writing/compacting/append-history, ensured compact exports validate against the local schema, and simplified all CLI command reports to short human-readable summaries.
- Example model work: created a complete sample Aurora model under [docs/example/aurora/](docs/example/aurora/) for an online ordering app (branding, legal constraints, and threat/risk/control evaluation), including missing `Feature` and `Process` cards to close dangling references
- Backend update (2026-01-19): Re-exported the `aurora::model` helpers and bump argument structs so the CLI subcommands compile without nested module errors.

### Rust Project State Review (2026-01-19)

- Scope: workspace `Cargo.toml`, `tools/aurora_cli/Cargo.toml`, Rust source layout under `tools/aurora_cli/src/`
- Findings: None observed in the Rust workspace configuration or crate layout.
- Notes:
	+ Workspace members: `tools/*` (currently only `tools/aurora_cli`)
	+ Toolchain: `rust-toolchain.toml` pins `stable`; `rustfmt.toml` enforces hard tabs + Unix newlines
    + Crate: `aurora_cli` uses Rust 2024 edition, reverse-DNS `app_id` in package metadata, and workspace-shared deps
- Residual risk: tests/lints not executed during this review; last recorded `cargo test` run is 2026-01-17 in this log.
- Required next action: None.

### aurora_cli design model (2026-01-18)

- Model root: [docs/design/aurora/](docs/design/aurora/)
- Mission entrypoint: [docs/design/aurora/MIS-001.json](docs/design/aurora/MIS-001.json)
- Design updates in-progress:
    + Multi-model semantics: a shared `aurora/` folder may contain multiple Missions; each is processed as a separate model.
    + Output layout: per-mission folders under [docs/design/](docs/design/) (`docs/design/{MissionId}/cards/` and `docs/design/{MissionId}/views/`) plus per-mission and aggregate indexes.
    + Mermaid: views require a standard init header (ELK) and the standard `classDef` palette + per-node `class` mapping.
    + Versioning: bump-major/minor/patch commands update `audit_trail.version` and append `audit_trail.history` entries with editor and timestamp.

### Test Report (Rust harness) - 2026-01-17

- Test crate: `tools/aurora_cli`
- Command: `cargo test`
- Tests executed: 3 (integration)
        + Previously executed: 3 (integration) — all passed as of 2026-01-17

#### Bump command integration tests (2026-01-18 update)

- Test file: `tools/aurora_cli/tests/bump_commands.rs`
- Command run: `cargo test --test bump_commands`
- Results: 3 tests run — 3 passed
    + `bump_patch_creates_history_when_empty`
    + `bump_minor_appends_edited_when_history_present`
    + `bump_with_editor_flag_sets_editor`

Notes: the CLI now exposes the bump subcommands end-to-end, so the history/version assertions in these tests all succeed. Keep these tests as guardrails whenever the audit trail logic is touched.

Highlights:

- `validation_passes_for_design_model` confirms schema/invariant checks succeed for `docs/design/aurora/`.
- `render_cards_produces_markdown_files` writes deterministic Markdown (includes `MIS-001.md` and README) to a temp directory.
- `render_views_creates_expected_files` now succeeds after adding anchor-aware skipping logic (deployment view omitted when no Deployment cards exist).
- `full_pipeline_renders_and_compacts` verifies the validate → render (views then cards) → compact pipeline used by the new `full` command.
- README generation now lists applicable views regardless of execution order, so the “Views” section is no longer blank after `render-all`.

Next steps:

- Extend integration coverage for render outputs (per-mission cards/views, README links, Mermaid styling) to lock in the current behavior.

### Test Report Update (2026-01-21)

- Command: `cargo test`
- Result: Blocked by unresolved import `CompactModelBorrowed` in [tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs#L14); the existing module no longer defines that type, so the suite fails before exercising the updated loader. Fix or remove the stale import to restore regression coverage.

### Test Report Update (2026-01-21, later)

- Command: `cargo test`
- Result: Passed after introducing `CompactModelBorrowed` and the new card layout validation (warnings about unused CLI exit codes remain and are tracked under the “Improve render outputs and exit codes” task). The `runTests` helper could not discover the crate, so the check was executed via `cargo test` to ensure coverage.

### Code Review: tools/aurora_cli vs MIS-001 (2026-01-18)

Scope: review `tools/aurora_cli` against the mission intent in [docs/design/aurora/MIS-001.json](docs/design/aurora/MIS-001.json), with focus on determinism (CNS-001), safe filesystem writes (CNS-002), and trustworthy validation (DRI-002).

Strengths:

- Deterministic output structure: cards + views under a mission root; stable ordering in multiple places (file collection sort, grouped index ordering).
- Mostly safe write pattern: writes to a temp file then renames (best-effort atomic update) in [tools/aurora_cli/src/output.rs](tools/aurora_cli/src/output.rs#L112-L136).
- Validation is always run before render operations; validation failures produce a report with path context.
- Test suite currently passes (`cargo test --manifest-path tools/aurora_cli/Cargo.toml`) and covers validate + render + bump commands.

Findings (High):

- Output root derivation trusts `Mission.id` (and runs before schema validation): [tools/aurora_cli/src/lib.rs](tools/aurora_cli/src/lib.rs#L79-L80). With an untrusted model (and especially an untrusted `Aurora.schema.json` that permits arbitrary IDs), a crafted mission id containing path separators / `..` can:
    + Create the output directory outside `--output-root`.
    + Cause the validation report to be written outside the intended tree.
    + If validation passes and render runs, `OutputPaths::clean()` can delete unexpected directories via `remove_dir_all`.
    + Mitigation: reject mission ids containing path separators / `.` segments before using them in paths; alternatively, derive the output folder name from a sanitized slug and fail fast if sanitization changes the id. Consider deferring `OutputPaths::new()` until after a “trusted schema” validation step.

- Path containment checks are not robust against non-normalized paths or malicious path segments:
    + Card type is used directly as a directory segment ([tools/aurora_cli/src/render/cards.rs](tools/aurora_cli/src/render/cards.rs#L219-L229)), and containment relies on `diff_paths()` in [tools/aurora_cli/src/output.rs](tools/aurora_cli/src/output.rs#L166-L182).
    + If any segment contains `..` or separators, behavior depends on how `diff_paths()` normalizes; this is risky to rely on for a security boundary.
    + Mitigation: validate all path segments used for directory names (`Mission.id`, `card_type`, view filenames) as “single safe segments” (no separators, no `.`/`..`), and/or build paths using a safe-join utility that refuses traversal.

Findings (Medium):

- Loader silently skips JSON files missing `id` ([tools/aurora_cli/src/loader.rs](tools/aurora_cli/src/loader.rs#L46-L48)). This undermines DRI-002 (“high-confidence validation”) because malformed or incomplete cards can be ignored rather than reported.
    + Mitigation: treat “JSON file in model tree without `id`” as a validation error (or at minimum emit it directly in the CLI validation output).

- Invariant validation is incomplete vs the Aurora instructions:
    + Current logic checks missing link targets, “non-mission has inbound links”, and “reachable from mission” ([tools/aurora_cli/src/validator.rs](tools/aurora_cli/src/validator.rs#L99-L147)).
    + Missing invariants include: Mission must have no incoming links, and (depending on the spec you’re enforcing) additional direction/hierarchy constraints.
    + Mitigation: extend `validate_invariants` to codify the exact invariant rules from the Aurora instructions, especially around Mission inbound links.

- Design drift: MIS-001 attributes describe aggregate/per-mission README locations that don’t exactly match current renderer outputs (aggregate index at `docs/design/README.md` and per-mission at `docs/design/README-{MissionId}.md`). The implementation writes `README.md` and `README-{MissionId}.md` inside the mission root ([tools/aurora_cli/src/render/cards.rs](tools/aurora_cli/src/render/cards.rs#L62-L66)).
    + Mitigation: either update MIS-001 design attributes to reflect the implemented layout, or adjust the renderer to produce the aggregate/per-mission indexes as specified.

Findings (Low):

- Avoid `expect`/`unwrap` in library code:
    + `Model::mission()` uses `expect` ([tools/aurora_cli/src/model.rs](tools/aurora_cli/src/model.rs#L104-L108)).
    + `bump::sort_object` uses `unwrap` on a known-present key ([tools/aurora_cli/src/bump.rs](tools/aurora_cli/src/bump.rs#L25-L28)); safe but unnecessary.
    + Mitigation: return explicit errors (preferred) or use non-panicking alternatives.

- `cargo clippy` reports a small style warning in card rendering: [tools/aurora_cli/src/render/cards.rs](tools/aurora_cli/src/render/cards.rs#L164).

Doc hygiene note:

- AGENT_PROGRESS currently contains stale statements about bump command implementation state and dependency list (it mentions `anyhow`, while the crate uses `thiserror`). Consider updating those sections for accuracy.

### Agent Instruction Consistency Review (2026-01-12)

- Synced `.github/instructions/Aurora.schema.json` to be byte-identical with [schemas/Aurora.schema.json](schemas/Aurora.schema.json) (prevents drift/false “out of date” alerts).
- Fixed the example Aurora Feature Card link in [.github/copilot-instructions.md](.github/copilot-instructions.md) to point at the actual example card path.
- Clarified `.github/instructions/Rust.instructions.md` to avoid assuming a `rustfmt.toml` already exists in this repo.
- Follow-up recommendation: decide whether `model:` frontmatter in `.github/agents/*.agent.md` should be standardized (some agents use `GPT-5 mini` vs `GPT-5.2`).
- Follow-up recommendation: either rename `.github/agents/8-DocReviewer.agent.md` or align its `name:` (`DocumentationReviewer`) to avoid confusion for humans/tooling.

## Patterns

- Deterministic directed-graph model rooted at a single `Mission` card
- Links are descriptive verbs; semantics emerge when rendering views, not from the link type alone

## Technologies

- JSON Schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
- Static docs site under [docs/](docs/) (note: `docs/viewer.html` removed due to CORS issues and incompatibility with the current structure)
- Mermaid diagrams for examples
- Rust crates used by `tools/aurora_cli`: `chrono`, `clap`, `fern`, `indexmap`, `jsonschema`, `log`, `pathdiff`, `percent-encoding`, `regex`, `semver`, `serde`, `serde_json`, `thiserror`, `whoami`

## Master Project Plan and Progress Tracker

1. Keep schema, instructions, and examples aligned
    + Status: In Progress
2. Improve modeling guidance and consistency
    + Status: In Progress
3. Keep agent instructions consistent with source-of-truth constraints
    + Status: In Progress

<memory>

- 2026-01-11: Normalized `.github/agents/*.agent.md` for clarity and consistency, fixed typos, and aligned model strings.
- 2026-01-11: Added a Prettier override for `.github/agents/*.agent.md` in `.prettierrc.json`.
- 2026-01-11: Made “send back” handoffs non-automatic (`send: false`) to prevent loops; clarified that any agent may update `CHANGELOG.md`.
- 2026-01-12: Reviewed `.github/agents/*` + `.github/instructions/*`; synced `.github/instructions/Aurora.schema.json` with canonical schema; fixed an example link in `.github/copilot-instructions.md`; added `rustfmt.toml`.
- 2026-01-12: Consolidated repeated agent coding standards into `.github/instructions/Coding.instructions.md` and refactored developer agents to reference it.
- 2026-01-18: Updated [schemas/Aurora.schema.json](schemas/Aurora.schema.json) to match the v2 Aurora instructions (`audit_trail`, `MIS-001` ids, Title Case `card_type`, and link targets by card id).
- 2026-01-18: Created an example model under [docs/example/aurora/](docs/example/aurora/) for a simple online ordering system, including Features and Processes to maintain internal link consistency.
- 2026-01-20: Fixed `aurora_cli` input/model discovery so running from subdirectories still finds `docs/design/aurora/` via git-root resolution and upward Aurora-home search.

</memory>

### Rust Code Review Readout (2026-01-20)

- Scope: `tools/aurora_cli` plus Rust examples under `.github/instructions/rust_example/`
- Evidence:
    + `cargo test` passes, but warnings are present and will fail `clippy` under `-D warnings`.
    + No tests are currently detected/run (0 tests reported).

Findings (High):

- Schema loading logic currently treats schema text as a JSON string instead of parsing JSON, so schema validation/compilation is not trustworthy: [tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs#L163-L175)
    + Mitigation: parse via `serde_json::from_str::<serde_json::Value>(&data)` before `meta::is_valid`, and prefer taking `&Path` instead of `&PathBuf` in the public API.

- Card discovery in `load_cards` appears functionally broken because it checks `path.file_name()` (the root folder name) rather than each `entry_path.file_name()`, so the regex gate will skip every file: [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L559-L596)
    + Mitigation: switch the filename check to `entry_path.file_name()` and ensure card validation/load runs for matching `XXX-000.json` files.

- View definitions include embedded trailing newlines in view names and card type names (e.g., `"Driver\n"`, `"System\n"`), which will not match `card.card_type` values loaded from JSON, resulting in empty/incomplete views: [tools/aurora_cli/src/aurora/model/view_definitions.rs](tools/aurora_cli/src/aurora/model/view_definitions.rs#L3-L65)
    + Mitigation: remove the embedded newlines and enforce exact Title Case card type strings.

- Card Markdown template placeholders do not match the renderer replacements (`{{audit_trail}}` exists in the template, but the renderer writes `{{history}}` / `{{version}}`): [tools/aurora_cli/src/aurora/model/card/template.rs](tools/aurora_cli/src/aurora/model/card/template.rs#L1-L21), [tools/aurora_cli/src/aurora/model/card.rs](tools/aurora_cli/src/aurora/model/card.rs#L112-L178)
    + Mitigation: rename placeholders to one consistent contract and add a small unit/integration test that asserts rendered Markdown contains the audit trail fields.

Findings (Medium):

- Determinism gaps due to `HashMap` iteration order:
    + Card index generation iterates `self.cards.values()` without sorting: [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L397-L437)
    + Card writes iterate `self.cards.values()` without sorting: [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L385-L395)
    + Mitigation: collect/sort card ids first, then iterate deterministically.

- Broken/incorrect links in the generated card index: the “no subtype” branch uses `prefixes::get_prefix(card_type)` as a directory component, but cards are written under the `card_type` directory name: [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L420-L433)

- Mermaid wrapper strings likely contain quoting/escaping mistakes (notably `Interface`, `System`, `Risk`, and several shapes), risking invalid Mermaid output: [tools/aurora_cli/src/aurora/model/view_definitions.rs](tools/aurora_cli/src/aurora/model/view_definitions.rs#L67-L99)

- `main.rs` defines multiple exit codes but maps most errors to `99`, and it treats any `ModelError` as “bump failed” even when the operation is render/compact: [tools/aurora_cli/src/main.rs](tools/aurora_cli/src/main.rs#L1-L16)

- CLI arg parsing for `log::Level` may not be robust/portable across clap versions without an explicit value parser/value enum: [tools/aurora_cli/src/cli.rs](tools/aurora_cli/src/cli.rs#L15-L22)

Findings (Low):

- `clippy` cleanups:
    + Nested `format!` (prefer direct format args): [tools/aurora_cli/src/aurora/model/card.rs](tools/aurora_cli/src/aurora/model/card.rs#L139-L148)
    + Unused import and unused fields warnings: [tools/aurora_cli/src/aurora.rs](tools/aurora_cli/src/aurora.rs#L3-L25)
    + Unused function: [tools/aurora_cli/src/aurora/model/view_definitions.rs](tools/aurora_cli/src/aurora/model/view_definitions.rs#L102-L107)

- Boundary-related prefix handling is inconsistent (`BND` vs `BOU`): [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L190-L196), [tools/aurora_cli/src/aurora/model.rs](tools/aurora_cli/src/aurora/model.rs#L359-L373)

Next actions:

- Fix the three correctness blockers (schema parse, card discovery, view-definition string normalization), then rerun `cargo test` and `cargo clippy -- -D warnings`.
- Reintroduce/restore test coverage (current harness reports 0 tests).
