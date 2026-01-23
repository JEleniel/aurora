# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog],
and the project adheres to [Semantic Versioning].

## [Unreleased]

### Added

- Node, Svelte, and TypeScript ignore patterns in the root `.gitignore` to keep workspace noise out of source control.
- Aurora Editor now scans model homes, lists missions/cards, and loads card JSON through the shared Rust
 library to power the retro-futuristic editor view.
- Aurora Editor navigator now consumes the summary filters, with new filtering + graph commands that keep MCP surfaces aligned with aurora_cli views.
- Aurora Editor Graph Explorer visualizes upstream/downstream relationships with zoomable generations and click-to-recenter navigation backed by `graph_neighborhood`.
- Aurora Editor's model picker now uses the official Tauri dialog plugin so the folder chooser opens as a native window instead of overlapping the UI.
- Aurora Editor now provides a native folder picker for selecting the model home so users no longer need to paste paths manually.

### Changed

- Aurora Editor's Tauri backend now lives in a proper library with dedicated command/state modules, leaving `main.rs` as a thin bootstrapper for clearer factoring and easier testing.

### Fixed

- Added the missing `DRI` → `Driver` entry to `CARD_PREFIXES` so it matches the documented Aurora card prefixes.
- Fixed view rendering to include configured root cards even when the root is the mission card.
- Fixed view rendering to always include `Boundary` and `Note` cards when they are linked as children of other included cards.
- Aurora Editor now reloads (or clears) the active selection whenever filesystem watcher events detect external changes, so card details stay in sync with on-disk edits.
- Aurora Editor watcher events now use the supported Tauri `emit` API so the desktop shell compiles and runs again.
- Card parsing errors now include the offending file path, making it easier to locate malformed JSON or audit issues inside the model home.
- Model load failures now render inside a copyable alert with a one-click “Copy error” control so testers can share diagnostics quickly.
- `aurora_cli compact` now honors mission-card inputs by filtering to that mission only, ensuring a single AGENT file is rewritten when the user points the CLI at a specific JSON card.

## [2.0.0](https://github.com/JEleniel/aurora/releases/tag/v2.0.0) - 2026-01-21

### Added

- Aurora CLI features for model compaction, Markdown card/view rendering, validation, and automated version bumping.
- Aurora compact schema support plus a CLI workflow dedicated to schema validation.
- Initial JSON schema set, new state cards, and fast-food system definitions that flesh out the reference model.
- Aurora Viewer with Mermaid integration for interactive diagram exploration.
- Comprehensive documentation updates covering planner workflows, detailed view guides, and editor-tool design notes that reduce instruction token usage.
- `.gitignore` entries for Rust build artifacts and configuration files.
- Enhanced error reporting that includes detailed annotations for CLI consumers.

### Changed

- Refactored card models, rendering logic, and supporting documentation to simplify maintenance and drop deprecated JSON assets.
- Cleaned up logging by removing debug output, standardizing formats, and improving readability across the codebase.
- Updated schemas with clearer human-readable relationship descriptions and ensured stylistic consistency in supporting tests.
- Renamed progress tracking files and aligned planner documentation to the new coordination process.

### Fixed

- Corrected the `CARD_PREFIXES` array sizing bug and removed the unused prefix entry that caused validation noise.
- Fixed case-sensitive links inside the Architect agent documentation to restore reliable navigation.

### Removed

- Deprecated JSON cards and outdated schema artifacts that conflicted with the new compact modeling workflow.

[Unreleased]: https://github.com/JEleniel/aurora/compare/v2.0.0...HEAD
[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
