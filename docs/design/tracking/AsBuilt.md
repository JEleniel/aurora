# As Built

## 2026-03-20

- Shared view layout and routing now delegate to Graphviz `plain` output via `tools/aurora_shared/src/render/layout/graphviz.rs`.
- Supported view layout families are now `TreeTopDown` (`dot` with `rankdir=TB`) and `TreeLeftRight` (`dot` with `rankdir=LR`).
- Graphviz geometry is consumed in inches and scaled into Aurora's existing 300 ppi SVG coordinate space before symbol rendering.
- `tools/aurora_shared/src/render/svg.rs` now uses Graphviz-provided edge routes when available and keeps the in-repo edge router only for grid-based focused graph layouts.
- The Graphviz plain-output parser now merges wrapped continuation lines before record parsing, which prevents long edge records from failing on trailing style tokens such as `solid black`.
- Shared SVG rendering now falls back to a simple center-to-center edge when Graphviz omits a per-edge route record (for example when concentrated edges share a path) instead of skipping the entire view.
- The focused editor graph remains on explicit grid coordinates through `LayoutCoordinateSpace::Grid`; rendered views use `LayoutCoordinateSpace::Pixels`.
- When a view omits `layout`, shared rendering now picks the family whose width:height ratio is closest to but still under $1.6$ before falling back to route-quality tiebreakers.

## 2026-03-21

- `aurora_cli render-views` and `render-all` now accept `-D/--dot-output <DIR>` to write the Graphviz DOT used for each rendered view to `<DIR>/<MISSION_ID>/Views/<View_Name>.dot`, creating the last folder as needed.
- Graphviz node geometry in `tools/aurora_shared/src/render/layout/graphviz.rs` now matches Aurora's 720×450 symbol size at 300 ppi; this keeps rendered node boxes and Graphviz route endpoints aligned so arrow tips reach their symbols again.
- The shipped Aurora view-configuration references no longer pin a per-view `layout`; renderer fallback now chooses between the two supported tree families.
