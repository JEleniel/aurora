---
applyTo: '**/Cargo.toml'
---

# Cargo.toml Guide

This document defines conventions for Rust dependency management and Cargo metadata in this repository.

## Dependencies

- Prefer the latest stable version of a crate.
- Prefer adding dependencies at the narrowest scope practical (package-level, not workspace-wide), unless multiple crates truly share the dependency.
- Avoid introducing new dependencies when the standard library or existing dependencies already solve the problem.

## Application Metadata

- Include a single reverse-DNS app ID in `Cargo.toml` as a package/workspace metadata key (not a top-level Cargo key).
- For package-level metadata, use:

```toml
[package.metadata.crystultima]
app_id = "org.crystultima.<package_name>"
```

- If the app ID is truly workspace-wide, use:

```toml
[workspace.metadata.crystultima]
app_id = "org.crystultima.<workspace_name>"
```

- The root domain is `crystultima.org` (owned by the maintainer).

## Cargo Operations

- Use `mcp_cargo-mcp_*` for Cargo operations when available. If it is not available, use the standard `cargo` CLI.

## Approved Libraries

The following libraries are approved for use. Sublibraries are those that start with the name of the parent or are designed to work as children of the parent.

- `anyhow`, `thiserror` for error handling
- `axum` (and sublibraries), `tower` (and sublibraries), `hyper` (and sublibraries) for web servers
- `base64`, `hex`, `num-traits`, `regex`, `unicode-normalization`, `uuid` for utilities
- `chrono` for time and date handling
- `clap` for CLI interfaces
- `config` for configuration file handling
- `ctrlc` for signal handling
- `dirs` (preferred) or `directories` for standard config/data/cache directories
- `fern` (preferred) or `tracing` (and sublibraries) for logging
- `log` for logging API
- `ollama-rs` for Ollama access
- `openssl` or `rustls` (and sublibraries) for TLS
- `r2d2`, `r2d2_sqlite`, `rusqlite` for SQLite (use `rusqlite` with the `bundled` feature)
- `reqwest` for HTTP client calls
- `serde` (and sublibraries), `serde_json` for serialization
- `tokio` (and sublibraries) for async runtime
- `url`, `urlencoding` for URL handling
