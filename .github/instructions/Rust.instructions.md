---
description: 'Agent directives for Rust code structure, testing, and verification.'
applyTo: '**/*.rs'
---

# Rust Coding Guide

If present, the repository's Rust formatting config (`rustfmt.toml`) is the source of truth for formatting.

## Formatting Rules

- **Organization**: Organize code into cohesive modules; minimize top-level `*.rs` sprawl. Do not use `mod.rs`.
- **Formatting**: Use `cargo fmt`.
- **Indentation**: Use tabs; let `rustfmt` enforce indentation and alignment.
- **Line endings**: Use `\n`.
- **Comments and docs**:
    - Keep comments accurate and up to date.
    - Wrap comment text at 100 characters.
    - Use `//!` for module/crate docs and `///` for item docs.
    - Follow the [rustdoc book](https://doc.rust-lang.org/rustdoc/).
- **Imports**: Group standard/external/crate. Put `mod` declarations first (after module docs), then a blank line, then `use`.
- **Patterns**: Use `_` for single-item wildcards and `..` for rest patterns.
- **Initialization**: Use field init shorthand when possible.
- Baseline: [2024 Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/index.html).

## Coding Rules

- Apply these rules to Rust code you write or modify. Do not rewrite unrelated existing code solely for conformance.
- Do not use `unwrap`, `expect`, `panic`, or similar in non-test code unless explicitly instructed.
- Add documentation comments for new modules and new public items.
- Avoid `unsafe` unless a specific API requires it.
- When configuring logging, write `TRACE`, `DEBUG`, `INFO`, and `WARN` to stdout and `ERROR` to stderr. Optionally log to a structured file.
- Unit testing exercises the inside of a module. Integration testing exercises the outside through public APIs.
- Unit tests belong in a module-local `tests/` subfolder and may be brought in using the `path` directive.
- Integration tests belong in the crate-root `tests/` directory.

## Prohibitions

- You MUST NOT write functions that _only_ return a constant value.
- You MUST NOT `map_err`, wrap errors, flatten errors, or write functions that convert or wrap errors. Use `thiserror` `#[from]` directives instead along with the `?` operator.
- You MUST NOT expose the internals of a module for any reason.
- You MUST NOT let a source file exceed 500 lines or a single function exceed 50 lines. You may use the `./.github/violations.sh` script to verify compliance.
- You MUST NOT write a function that executes a simple calculation or call that is only used once. Just use the call or formula directly.
- You MUST NOT write multiple paths, functions, or other code that does the same thing. There MUST be one source of truth for any function or capability.
- You MUST NOT use any form of shared data across task or thread boundaries; all communication must use channels or similar. Use of `mutex` is a red flag for this.

## Error Handling

- You MUST NOT swallow errors. They MUST all be handled or logged at minimum.
- Any error that can be handled and recovered from should be.
- Library code SHOULD return typed errors (prefer `thiserror`).
- Executable entrypoints and true application boundaries—places where control leaves our code to an external runtime, caller, or user-facing shell—MUST use `anyhow`.
- All errors MUST be either handled or logged. The code should crash only if there is no choice.

## Acceptance Criteria

- For Rust work, relevant `cargo` checks pass (`fmt`, `clippy`, and targeted `test`).
