---
applyTo: '**/Cargo.toml'
---

# Cargo.toml Guide

Conventions for dependency management and Cargo metadata in this repository.

## Dependencies

- Prefer the latest stable crate versions, unless constrained by compatibility, MSRV, or security response.
- Add dependencies at the narrowest practical scope (package-level, not workspace-wide) unless multiple crates truly share them.
- Avoid new dependencies when the standard library or existing dependencies already solve the problem.

## Application Metadata

- Include a single reverse-DNS app ID in `Cargo.toml` as package/workspace metadata (not a top-level Cargo key).
- For package-level metadata:

```toml
[package.metadata.crystultima]
app_id = "org.crystultima.<package_name>"
```

- If the app ID is workspace-wide:

```toml
[workspace.metadata.crystultima]
app_id = "org.crystultima.<workspace_name>"
```

- Root domain: `crystultima.org` (owned by the maintainer).

## Cargo Operations

- Prefer `mcp_cargo-mcp_*` for Cargo operations when available.
- If unavailable, use the standard `cargo` CLI.

## Approved Libraries

The following libraries are approved for use. Sublibraries include crates that share the parent prefix or are designed as companions.

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
- `sha2`, and `hmac` for hashing
