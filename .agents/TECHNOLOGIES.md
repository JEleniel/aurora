# Technologies

## Languages and toolchain

- Rust (toolchain channel: `stable`; see `rust-toolchain.toml`).

## Repo tooling

- Formatting: `rustfmt` (config: `rustfmt.toml`).
- Markdown lint: `markdownlint-cli2` (config: `.markdownlint-cli2.jsonc`).
- JSON formatting: `prettier` (config: `.prettierrc.json`).

## Aurora tooling (Rust crates)

- `aurora_cli` (`tools/aurora_cli/`): validate, render, compact workflows.
- `aurora_shared` (`tools/aurora_shared/`): model + registries + renderers used by the CLI.
- `svg_prep` (`tools/svg_prep/`): prepares SVG templates and icon sheets.

## Notable libraries

- `.svgz` (gzipped SVG) support uses `flate2`.
- SVG parse/emit uses `xmltree` (crate `svg_prep` pins `xmltree = 0.11`).

## Checked-in outputs (examples)

- Rendered view SVGs: `docs/design/MIS-002/Views/`.

Last updated: 2026-02-19
