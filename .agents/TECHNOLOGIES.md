# Technologies

## Tooling constraints (important)

- `aurora_cli` is runnable from source in this workspace via `cargo run -p aurora_cli -- <subcommand>`.
- Keep using source builds during validation so embedded template/routing changes are included in generated artifacts.

## Aurora Editor

- `tools/aurora_editor/` exists in this workspace.
- Do not rely on editor stack assumptions, command surfaces, or DTO contracts unless they are confirmed in the current code.

## SVG rendering notes

- `aurora_shared` SVG node rendering now fits symbols with axis-specific scaling in `render/svg/node.rs`.
- Node measurement in `render/svg/node.rs` no longer derives height from width-to-symbol scaling, reducing tall/narrow distortion.
- Text labels remain centered as a multiline block within symbol bounds, with icon placement anchored from symbol offsets.
- Edge routing in `render/svg/edge.rs` now uses deterministic orthogonal candidates first, then bounded grid A* fallback.
- Edge routing now includes obstacle-aware detour lanes and an emergency detour recovery pass that prevents straight fallback segments from crossing node bodies.
- Route obstacle publication for later edges excludes source/target node exclusion zones to keep endpoint entry/exit viable.
