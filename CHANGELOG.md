# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Introduced initial `aurora_cli` application wiring with validation, render, and compact subcommands.
- Added validation requiring `Asset (Secret)` cards to have an incoming `owns` relationship (preferably from an `Actor`).
- Added file-system safe rendering helpers plus unit tests in `aurora_shared`.
- Added Everything View rendering support so the CLI view pipeline can include full-model diagrams.
- Registered reverse-DNS `app_id` metadata for each tool crate and seeded placeholder libraries for editor and VS Code hosts.
- Converted `aurora_editor_backend` into a Tauri project that exposes model discovery, load, validation, render, compact, and card-update commands for the forthcoming UI.
- Added Tauri editor backend commands for workspace configuration, model operations, and audit-aware card CRUD with trust gating.
- Added a Vite + Svelte + TypeScript frontend scaffold for the Aurora Editor with the initial futuristic blue/indigo UI shell.
- Added wired workspace controls and action buttons in the editor UI to invoke backend commands for validation, render, and export workflows.
- Implemented the canonical DOT → SVG → Markdown view pipeline in `aurora_shared::render_views`, including Graphviz invocation, instruction parsing, and regression tests for the CLI render path.
- Updated the view Markdown embeds to include a "Zoom & pan" note that links directly to the SVG so users can open it in VS Code's viewer without triggering sandbox warnings.

### Changed

- Reorganized the Aurora Editor project so the Tauri backend crate now lives under `tools/aurora_editor/src-tauri`, matching the standard Tauri folder layout and colocating static assets with the host project.
- Aligned the Aurora Editor frontend and Tauri build commands with the upgraded Svelte 5/Tauri v2 stack and pnpm workflow.
- Updated the Aurora Editor Tauri configuration to the v2 schema (`build.devUrl`, `build.frontendDist`, `app` root) and adjusted build hooks so `tauri:dev` validates.
- Aligned the Aurora Editor Tauri Rust crates and npm packages to v2.5.3 (latest crates.io release) to resolve CLI version mismatches.
- Wired the Aurora Editor bundle icons to the curated Aurora PNG assets under `assets/`.
- Refined the Aurora Editor UI header layout with a 16px base font, a 4rem header bar, right-aligned status pills, and reduced top spacing above workspace controls.
- Added local `@tailwindcss/vite` and `tailwindcss` dev dependencies for the Aurora Editor so Vite plugin types resolve consistently.
- Switched Aurora Editor workspace selection to the Tauri folder dialog, auto-connecting and discovering model homes with the default output path set to `docs/design/`.
- Added the bundled Antonio and Noto fonts to the Aurora Editor CSS with reusable font stacks.
- Defaulted the Aurora Editor color scheme to dark while honoring user light mode preferences.
- Updated the `render_markdown`/`render_views` pipeline so derived card Markdown mirrors the source model folder structure and every DOT/SVG/Markdown view artifact is written under a `Views/` directory per mission.
- Taught the canonical view renderer to honor the icon column from `Card_Definitions.md`, translating the Lucide-style names into Unicode symbols that now prefix every node label in DOT/SVG/Markdown outputs.
- Expanded the Aurora Editor workspace path handling to trim whitespace, resolve `~/`, and accept absolute paths within the workspace root.
- View rendering now writes DOT files to `Views/source/` and omits view Markdown outputs, leaving SVGs as the primary artifacts.
- View rendering now emits HTML node labels that place card subtypes on their own parenthesized line and defaults to a white background with black edge lines/labels.
- View rendering now uses Graphviz for layout only (`dot -Tplain`) and generates Aurora-owned, theme-aware SVG output (CSS variables + `prefers-color-scheme`) instead of relying on Graphviz's SVG styling.
- View rendering now selects Graphviz layout engine per view: `osage` for the Requirements view, `dot` for all other views.
- View rendering now defaults to curved Graphviz splines for smoother edges.
- View rendering now converts Graphviz plain spline control points into cubic Bezier SVG paths for smoother edges.

### Fixed

- Ensured Aurora Editor UI actions dispatch reliably and surface backend availability, including Enter-to-connect support for workspace paths.
- Added a default Tauri capability for the main window so internal devtools toggling is permitted during development.
- Added interaction telemetry in the editor UI and allowed localhost IPC for the Vite dev server.
- Expanded dev IPC URL patterns to include all Vite asset paths and improved error formatting for failed invokes.
- Aligned editor command payload casing with Tauri backend expectations to unblock model validation and render actions.
- Normalized model discovery payloads to populate the model home selector reliably.
- Added model home selector sync and a debug list to verify detected model roots when dropdowns appear empty.
- Included linked `Note` and `Boundary` annotation cards in view filtering so their relationships render in generated diagrams.
- Added view connectivity validation to flag nodes that are not reachable from any root in a diagram.
- Ensured `Note` cards only render when linked from an in-view parent node.
- Updated view connectivity checks to honor per-view root card types, preventing false orphan errors in cyclic flows.
- Filtered view content to only include cards connected to the diagram roots, reducing unrelated nodes.
- Corrected canonical instruction registry paths so view rendering loads definitions from `.github/instructions/details/`.
- Ensured SVG rendering applies Card_Definitions shape variants (component, folder, note, tab, cylinder, box3d, cds, record) and icon prefixes in DOT/SVG labels.
- Rolled back editor card mutations that would introduce validation errors when the model was previously valid.
- Corrected Aurora Editor frontend invoke payload casing for validation/render/export commands and added path normalization hints for workspace-relative output.
- Updated aurora_cli to split model homes by mission so validation, rendering, and compact exports run per mission when multiple missions share a model home.
- Corrected boundary cluster DOT output to emit graph attribute statements inside subgraphs, preventing Graphviz syntax errors when boundaries are present.
- Updated rendered card Markdown headers to format as `ID: **Card Type**` and suppressed empty validation reports in the CLI.
- Adjusted SVG node labels to left-align large icons and include the card ID before the bold card type.
- Switched view layout selection to always use Graphviz `dot`.
- Increased icon-to-text spacing in rendered view labels to approximately 1rem.
- Fixed boundary `attributes.recursive` handling so boundaries include descendant nodes in view renders (with loop-safe traversal).
