# AGENT_PROGRESS

## Project Brief

AURORA is a deterministic, JSON-based architectural modeling format where Cards (JSON files) are connected by directed Links away from a single root `mission` card.

- Feature: Maintain authoritative schema and modeling guidance
    + Status: In Progress
    + Primary schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
    + Modeling guidance: [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md)

## Active Context Summary

- Branch: `v2.0.0`
- Current work: consolidate duplicated agent coding standards into `.github/instructions/Coding.instructions.md` and move Rust-specific standards into `.github/instructions/Rust.instructions.md`

### Agent Instruction Consistency Review (2026-01-12)

- Synced `.github/instructions/Aurora.schema.json` to be byte-identical with [schemas/Aurora.schema.json](schemas/Aurora.schema.json) (prevents drift/false “out of date” alerts).
- Fixed the example Aurora Feature Card link in [.github/copilot-instructions.md](.github/copilot-instructions.md) to point at the actual example card path.
- Clarified `.github/instructions/Rust.instructions.md` to avoid assuming a `rustfmt.toml` already exists in this repo.
- Follow-up recommendation: decide whether `model:` frontmatter in `.github/agents/*.agent.md` should be standardized (some agents use `GPT-5 mini` vs `GPT-5.2`).
- Follow-up recommendation: either rename `.github/agents/8-DocReviewer.agent.md` or align its `name:` (`DocumentationReviewer`) to avoid confusion for humans/tooling.

## Patterns

- Deterministic directed-graph model rooted at a single `mission` card
- Links are descriptive verbs; semantics emerge when rendering views, not from the link type alone

## Technologies

- JSON Schema: [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
- Static docs site under [docs/](docs/) (note: `docs/viewer.html` removed due to CORS issues and incompatibility with the current structure)
- Mermaid diagrams for examples

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

</memory>
