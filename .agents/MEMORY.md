# Memory

- `tools/svg_prep`: `cargo check -p svg_prep` passes. `cargo clippy -p svg_prep -- -D warnings` is currently blocked by clap derive macros in `tools/svg_prep/src/cli.rs` conflicting with command-line `forbid` settings; the SVG nesting fixes in `tools/svg_prep/src/svg/*` are otherwise clean.