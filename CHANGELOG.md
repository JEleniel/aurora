# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `aurora_shared::render::svg::Svg` for rendering an Aurora `Model` + `Layout` into a standalone SVG (with configurable spacing, font size, and edge style).

- Added a Dioxus-based `aurora_editor` crate with model loading, navigation tree, context view, and read-only inspector panes.
- Added clickable context and diagnostics panels in the Aurora editor to surface validation issues and jump to related cards.
- Added an SVG-based context graph in the Aurora editor that renders card shapes/icons and link lines centered on the focused card.
- Added public `CardDefinition` and icon glyph helpers in `aurora_shared` to support editor graph rendering.
- Introduced initial `aurora_cli` application wiring with validation, render, and compact subcommands.
- Added `assets/Symbols.md` with a Unicode color-named symbol table.
- Added validation requiring `Asset (Secret)` cards to have an incoming `owns` relationship (preferably from an `Actor`).
- Added validation test coverage for canonical Relationships Matrix verbs and source/target constraints.
- Added an `aurora_cli --instructions-root` override plus validation output that reports which instructions registry was used.
- Added file-system safe rendering helpers plus unit tests in `aurora_shared`.
- Added hierarchical layout computation for view graphs in `aurora_shared`.
- Added Everything View rendering support so the CLI view pipeline can include full-model diagrams.
- Added mission executive summary markdown output (`MIS-XXX-Executive_Summary.md`) to the render pipeline.
- Added DOT JSON → SVG rendering helpers in `aurora_shared` for Diagram-based SVG output.
- Registered reverse-DNS `app_id` metadata for each tool crate and seeded placeholder libraries for editor and VS Code hosts.
- Converted `aurora_editor_backend` into a Tauri project that exposes model discovery, load, validation, render, compact, and card-update commands for the forthcoming UI.
- Added Tauri editor backend commands for workspace configuration, model operations, and audit-aware card CRUD with trust gating.
- Added a Vite + Svelte + TypeScript frontend scaffold for the Aurora Editor with the initial futuristic blue/indigo UI shell.
- Added wired workspace controls and action buttons in the editor UI to invoke backend commands for validation, render, and export workflows.
- Implemented the canonical DOT → SVG → Markdown view pipeline in `aurora_shared::render_views`, including Graphviz invocation, instruction parsing, and regression tests for the CLI render path.
- Updated the view Markdown embeds to include a "Zoom & pan" note that links directly to the SVG so users can open it in VS Code's viewer without triggering sandbox warnings.

### Changed

- Aligned `aurora_shared` model loading/serialization with Aurora v2.0.0: v2 schema filenames, v2 card fields (`version`, `boundary`, `notes`, object-shaped `attributes`), and compact exports written to `MIS-XXX/Compact.json`.
- Failed model loading now returns validation errors instead of allowing invalid graphs.
- Filtered view root candidates that participate in cycles to honor the root safety rule during rendering.

- Refined the standalone Aurora editor layout with a labeled model selector, smaller navigation typography, tree lines, and a split editor/inspector pane.
- Adjusted the standalone editor pane widths to 25%/50%/25% and moved diagnostics into the inspector pane.
- Made the mindmap pane fill the center column height and added emoji-capable font fallbacks for graph icons.
- Added mindmap zoom controls with scroll-based panning and tightened layout sizing for the header and control rows.
- Fixed the editor CSS layout after an accidental corruption of the global styles.
- Embedded Aurora logo assets as data URLs in the standalone editor to avoid file-based loading.
- Refactored the standalone editor UI into smaller modules under `tools/aurora_editor/src/app/`.
- Added Aurora logo imagery to the standalone editor header and empty states.
- Reorganized the Aurora Editor project so the Tauri backend crate now lives under `tools/aurora_editor/src-tauri`, matching the standard Tauri folder layout and colocating static assets with the host project.
- Aligned the Aurora Editor frontend and Tauri build commands with the upgraded Svelte 5/Tauri v2 stack and pnpm workflow.
- Updated the Aurora Editor Tauri configuration to the v2 schema (`build.devUrl`, `build.frontendDist`, `app` root) and adjusted build hooks so `tauri:dev` validates.
- Aligned the Aurora Editor Tauri Rust crates and npm packages to v2.5.3 (latest crates.io release) to resolve CLI version mismatches.
- Wired the Aurora Editor bundle icons to the curated Aurora PNG assets under `assets/`.
- Updated aurora_shared validation fixtures to align with the canonical relationships matrix.
- Added a standalone `tools/aurora_shared/Cargo.lock` to support independent builds after splitting Rust projects.
- Updated aurora_cli help/logging to reference the canonical instruction registry files (card, matrix, relationships, views).
- Stabilized the Aurora Editor UI Tauri command contract with typed DTOs, centralized command names, and wrappers for the full backend command surface.
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
- View rendering now emits a single view per type using `<View>_View.view.svg`/`.view.dot` naming, skipping empty or single-card views except for the Entire Model view.
- Expanded aurora_shared validation fixtures to cover `Application implements Test`, `Component implements Class`, `Artifact persists to Data Store`, and `uses` relationships per the canonical matrix.
- Embedded the canonical instruction registries (card definitions, relationships, view definitions, styling guide) directly into `aurora_shared` so CLI validation/rendering no longer require external files at runtime.
- Reduced SVG shape sizes for State (50%) and Event/Condition/Actor/Mission (25%) to better balance text proportions.
- Centered SVG node icons vertically, shifted them right by 1rem, reduced icon/text spacing, widened State icons, moved octagon icons one icon-width to the right, and scaled oval/octagon widths; corrected Interface label color for legibility on light fills.
- Further refined SVG geometry by narrowing hexagons, making State nodes circular with a slight size increase, and moving diamond/hexagon icons further inward.
- Snapped edge paths to the scaled shape boundaries (eliminating arrow gaps) and tuned default layout spacing (nodesep/ranksep).
- Restored curved spline routing while keeping edge concentration for shared incoming/outgoing lines.

### Fixed

- Fixed the Rust workspace members list so Cargo no longer references the deleted `tools/aurora_editor` crate.
- Prevented extra `Mission` cards from loading inside mission folders.
- Sanitized card filenames by removing symbols and only keeping alphanumerics with underscores for whitespace.
- Made CLI validation report errors cleanly, exit non-zero on invalid models, and block render/compact on validation failures.

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
- Added validation warnings for relationships and card types that do not match the canonical relationships matrix without blocking validation.
- Ensured validation ignores instruction-root parameters and always uses embedded registries for matrix checks.
- Replaced the Class icon glyph to avoid Graphviz HTML-label parsing errors in view rendering.
- Reduced icon sizing in view layout/rendering to bring node shapes closer to text size.
- Ensured `aurora_cli` always prints completion summaries for validate/render/compact workflows and annotated CLI help with default paths.
- Updated the Everything View renderer to show only the most direct Mission paths as solid edges and dash alternate paths.
- Centered SVG node labels to remove excessive right-side whitespace in rendered shapes.
- Adjusted boundary rendering to avoid overlaps between disjoint boundaries and added overlap regression coverage.
- Restored `aurora_cli render-views`/`render-all` by implementing layout+SVG view rendering and writing SVGs under `aurora/<MISSION_ID>/Views/`.
- Made model Markdown view embedding deterministic by sorting embedded SVG filenames.
