# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Graphviz layout/parser coverage in `aurora_shared` for DOT emission, plain-output scaling, and deterministic best-family selection.
- View-configuration coverage in `aurora_shared` for parsing and honoring explicit per-view layout families.

### Changed

- Replaced the shared in-repo view layout engine with Graphviz-backed layout and routing for SVG rendering.
- Reduced shared view rendering to the supported `TreeTopDown` and `TreeLeftRight` Graphviz families.
- Scaled Graphviz `plain` output into Aurora's existing 300 ppi SVG coordinate space so shared symbol rendering keeps its current visual units.
- Updated shared SVG rendering to consume Graphviz-provided edge routes when present while retaining the local router for grid-based focused graph layouts.
- Hardened Graphviz `plain` parsing to merge wrapped continuation lines before record parsing so long edge records do not fail on trailing style tokens.
- Updated shared SVG rendering to fall back to a simple center-to-center route when Graphviz omits a per-edge route record, avoiding full-view render failure on concentrated edges.
- Updated best-family selection to prefer the layout whose width:height ratio is closest to but still under $1.6$, using route-quality metrics only as deterministic tiebreakers.
- Removed final-SVG domain frame output from shared rendering so configured domains affect layout/routing behavior without emitting visible domain boxes in exported views.
- Added optional `-D` / `--dot-output` support to SVG-producing CLI commands so Graphviz DOT inputs can be written alongside rendered views for debugging.
- Corrected Graphviz view geometry to match Aurora's 720×450 symbol size at 300 ppi so routed edges and arrow tips align with rendered symbols again.
