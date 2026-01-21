# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog],
and the project adheres to [Semantic Versioning].

## [Unreleased]

### Fixed

- Added the missing `DRI` → `Driver` entry to `CARD_PREFIXES` so it matches the documented Aurora card prefixes.
- Fixed view rendering to include configured root cards even when the root is the mission card.

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
