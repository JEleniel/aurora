# As Built

## 2026-03-20

- Shared view layout and routing now delegate to Graphviz `plain` output via `tools/aurora_shared/src/render/layout/graphviz.rs`.
- Layout families are now `TreeTopDown` (`dot` with `rankdir=TB`), `TreeLeftRight` (`dot` with `rankdir=LR`), `Radial` (`twopi`), `Radial1` (`neato`), and `Circular` (`circo`).
- Graphviz geometry is consumed in inches and scaled into Aurora's existing 300 ppi SVG coordinate space before symbol rendering.
- `tools/aurora_shared/src/render/svg.rs` now uses Graphviz-provided edge routes when available and keeps the in-repo edge router only for grid-based focused graph layouts.
- The Graphviz plain-output parser now merges wrapped continuation lines before record parsing, which prevents long edge records from failing on trailing style tokens such as `solid black`.
- Shared SVG rendering now falls back to a simple center-to-center edge when Graphviz omits a per-edge route record (for example when concentrated edges share a path) instead of skipping the entire view.
- The focused editor graph remains on explicit grid coordinates through `LayoutCoordinateSpace::Grid`; rendered views use `LayoutCoordinateSpace::Pixels`.
- View definitions in the model's `Aurora.viewconfiguration.json` can now carry an explicit `layout` value using the shared family enum; when absent, shared rendering falls back to best-family selection rather than hard-coded view-name inference.

## 2026-03-21

- `aurora_cli render-views` and `render-all` now accept `-D/--dot-output <DIR>` to write the Graphviz DOT used for each rendered view to `<DIR>/<MISSION_ID>/Views/<View_Name>.dot`, creating the last folder as needed.
- Radial Graphviz DOT input now marks the actual root cards directly and no longer injects a pseudo-root helper node.
- Graphviz node geometry in `tools/aurora_shared/src/render/layout/graphviz.rs` now matches Aurora's 720×450 symbol size at 300 ppi; this keeps rendered node boxes and Graphviz route endpoints aligned so arrow tips reach their symbols again.
