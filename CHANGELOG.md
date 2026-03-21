# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Graphviz layout/parser coverage in `aurora_shared` for DOT emission, helper-root handling, plain-output scaling, and deterministic best-family selection.
- View-configuration coverage in `aurora_shared` for parsing and honoring explicit per-view layout families.

### Changed

- Replaced the shared in-repo view layout engine with Graphviz-backed layout and routing for SVG rendering.
- Renamed layout families to `TreeTopDown`, `TreeLeftRight`, `Radial`, `Radial1`, and `Circular`, mapped to `dot`, `dot` with `rankdir=LR`, `twopi`, `neato`, and `circo` respectively.
- Scaled Graphviz `plain` output into Aurora's existing 300 ppi SVG coordinate space so shared symbol rendering keeps its current visual units.
- Updated shared SVG rendering to consume Graphviz-provided edge routes when present while retaining the local router for grid-based focused graph layouts.
- Hardened Graphviz `plain` parsing to merge wrapped continuation lines before record parsing so long edge records do not fail on trailing style tokens.
- Updated shared SVG rendering to fall back to a simple center-to-center route when Graphviz omits a per-edge route record, avoiding full-view render failure on concentrated edges.
- Added an optional `layout` field to each view definition in the model's `Aurora.viewconfiguration.json`, using the new layout-family values and letting shared rendering fall back to best-family selection when the field is absent.
- Removed final-SVG domain frame output from shared rendering so configured domains affect layout/routing behavior without emitting visible domain boxes in exported views.
- Added optional `-D` / `--dot-output` support to SVG-producing CLI commands so Graphviz DOT inputs can be written alongside rendered views for debugging.
- Corrected Graphviz `plain` scaling to Aurora's 300 ppi SVG contract so routed edges and arrow tips align with rendered symbols again.
