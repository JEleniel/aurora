---
applyTo: '*.rs'
---

# Rust Coding Guide

This document defines formatting and style conventions for all Rust source code.

## Formatting Rules

- **Edition (Required)**: Always use the Rust 2024 (or newer) edition, unless the `Cargo.toml` specifies an older version, in which case conform to the `Cargo.toml`. You MUST NOT use `mod.rs` files.
- **Organization**: Organize code into logical modules that conform to the _single responsibility_ principle. Minimize top-level `*.rs` files by using modules.
- **Indentation (Accessibility)**: Prefer hard tabs for indentation. `cargo fmt` is the source of truth; if this repo includes a `rustfmt.toml`, it MUST be treated as authoritative.
- **Line Endings**: You MUST use POSIX-style newlines (`\n`).
- **Comment Formatting**:
    + You MUST limit comments to 100 characters per line and wrap for readability - break at word boundaries and avoid hyphenation.
    + Format code in comments. Use `//!` for module/crate docs and `///` for item docs.
    + Ensure that existing comments are kept up to date and accurate to what they describe.
    + You MUST conform to the [rustdoc book](https://doc.rust-lang.org/rustdoc/).
- **Imports:** Group imports by standard, external, and crate. Use crate-level granularity and sort per the best style guide. Place `mod` commands first in the file, after the module documentation, followed by a blank line and then `use` commands.
- **Hexadecimal Literals:** Use uppercase for hex literals. Other uses of hexadecimal should be consistent with idiomatic styles.
- **Wildcards:** Use the underscore wildcard (`_`) for single items, and the rest pattern (`..`) for multiple items.
- **Field Initialization:** Use field initialization shorthand where possible (e.g., `property,` instead of `property: property,`).
- Follow the [2024 Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/index.html) as the baseline for all rules not covered here.

## Coding Rules

- You MUST follow these rules for all Rust code you write or modify. Existing code may have been written by other agents or users and you MUST NOT alter it simply for conformance.
- You MUST NOT use `unwrap`, `expect`, `panic`, or similar in non-test code unless explicitly instructed by the user.
- You MUST include documentation comments for all modules and public functions that you create.
- You SHOULD NOT use `unsafe` code unless a specific call or library requires it due to this being on an embedded platform.

## Error Handling

- Library code SHOULD return typed errors (prefer `thiserror`).
- Executables and application boundaries MAY use `anyhow` for ergonomic context (`anyhow::Context`) and `anyhow::Result`.
- Prefer `?` plus `#[from]` when mapping between error types.
- All errors MUST be either handled or logged. The code should crash only if there is no choice.

## Notes

- For services, configure TLS to use TLS 1.3 unless the user explicitly requires otherwise.
- For services, configure logging to write `TRACE`, `DEBUG`, `INFO`, and `WARN` to stdout and `ERROR` to stderr.

## Acceptance Criteria

- Relevant `cargo` checks for the change pass (`fmt`, `clippy`, and targeted `test`).
