# Quickstart

This quickstart assumes you are working inside this repository.

## Build

Build the CLI (release):

```text
cargo build --release --manifest-path tools/aurora_cli/Cargo.toml
```

The binary will be at `target/release/aurora_cli`.

## Validate a model

Validate the design model shipped in this repo:

```text
./target/release/aurora_cli --input docs/design/aurora validate
```

Sample output:

```text
All models are valid.
```

## Render cards and views

Choose an output directory and create it before rendering (see [Troubleshooting](troubleshooting.md) for why).

```text
mkdir -p tmp/aurora_cli_docs
./target/release/aurora_cli --input docs/design/aurora render-all --output tmp/aurora_cli_docs
```

Sample output:

```text
Rendered views for model MIS-001.
Rendered cards for 1 models.
```

## Export a compact model

```text
./target/release/aurora_cli --input docs/design/aurora compact --output tmp/aurora_cli_docs
```

Sample output:

```text
Wrote compact model for MIS-001 to tmp/aurora_cli_docs/AGENT-MIS-001.json.
```

## Get help

```text
./target/release/aurora_cli --help
./target/release/aurora_cli render-all --help
```
