# Contributing

## Code of Conduct

By participating, you agree to follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## What to Contribute

- Bug reports and clarifications for the specification
- Schema improvements and additional validation constraints
- Examples and reference models (Cards and Links)
- Documentation improvements (readability, consistency, cross-references)

## Repository Layout

- [README.md](README.md): Project overview and core concepts
- [schemas/](schemas/): JSON Schemas (canonical schema: [schemas/Aurora.schema.json](https://github.com/JEleniel/aurora/blob/main/schemas/Aurora.schema.json))

## Style and Quality

- Markdown should pass markdownlint using [.markdownlint.json](.markdownlint.json).
- Prefer reference-style links (defined at the bottom of the file).
- Keep changes focused and avoid unrelated reformatting.

## Proposing Specification or Schema Changes

- Keep the README aligned with the authoritative modeling rules in `.github/instructions/Aurora.instructions.md`.
- If you change the schema meaningfully, update the schema `version` field in [schemas/Aurora.schema.json](https://github.com/JEleniel/aurora/blob/main/schemas/Aurora.schema.json).
- Include rationale: _why_ the change is needed and what it enables.

## Submitting Changes

- Open a pull request against the default branch.
- Include a short summary of changes and any migration notes.
- If your change affects the schema, include a small example JSON snippet showing the intended usage.
