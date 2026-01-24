# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Introduced initial `aurora_cli` application wiring with validation, render, and compact subcommands.
- Added file-system safe rendering helpers plus unit tests in `aurora_shared`.
- Registered reverse-DNS `app_id` metadata for each tool crate and seeded placeholder libraries for editor and VS Code hosts.
- Converted `aurora_editor_backend` into a Tauri project that exposes model discovery, load, validation, render, compact, and card-update commands for the forthcoming UI.
