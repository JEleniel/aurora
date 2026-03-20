# As Built

## 2026-03-20

- Shared view layout and routing now delegate to Graphviz `plain` output via `tools/aurora_shared/src/render/layout/graphviz.rs`.
- Layout families are now `TreeTopDown` (`dot` with `rankdir=TB`), `TreeLeftRight` (`dot` with `rankdir=LR`), `Radial` (`twopi`), `Radial1` (`neato`), and `Circular` (`circo`).
- Graphviz geometry is consumed in inches and scaled into Aurora's existing 300 ppi SVG coordinate space before symbol rendering.
- `tools/aurora_shared/src/render/svg.rs` now uses Graphviz-provided edge routes when available and keeps the in-repo edge router only for grid-based focused graph layouts.
- The focused editor graph remains on explicit grid coordinates through `LayoutCoordinateSpace::Grid`; rendered views use `LayoutCoordinateSpace::Pixels`.
