# APP-001: Aurora CLI Tool Design

## Definitions

- Files: Model files may be stored in either `*.json` or `*.jsjson` files. A single model MUST only use one or the other across the entire model. Anywhere a `*.json` file is referenced in this document it also refers to the `*.jsjson` equivalent.
- `*.jsjson` files are functionally identical to `*.json` files.
- Model Home: Models MUST be located in a folder named `aurora/`. User input may point to the parent of a Model Home, in which case the real Model Home should be discovered and used.
- Card Schema: `Aurora.schema.json` in the Model Home.
- Compact Model: optional file `AGENT-<MISSION_ID>.jsjson` in the Model Home.
- Compact Model Schema: `Aurora.compact.schema.json` in the Model Home.

## Scope and intent

APP-001 is a command-line surface for validating and rendering Aurora models. It is optimized for deterministic, automatable workflows (CI, local scripting, and repeatable documentation generation).

The CLI is intentionally “thin”:

- It should delegate core model semantics (parsing, invariants, rendering logic) to the shared library.
- It should focus on:
	+ Discovering inputs safely.
	+ Selecting subcommands.
	+ Surfacing diagnostics and exit codes.
	+ Writing outputs deterministically.

## Command surface (current)

The tool exposes the following subcommands and options (captured from `aurora_cli --help`):

```text
Aurora model validation and rendering toolkit

Usage: aurora_cli [OPTIONS] <COMMAND>

Commands:
  validate
  render-aurora
  render-views
  render-all
  compact
  bump-patch
  bump-minor
  bump-major
  help           Print this message or the help of the given subcommand(s)

Options:
  -i, --input <INPUT_PATH>  [default: docs/design/aurora/]
  -l, --log <LOG_LEVEL>     [default: INFO]
  -h, --help                Print help
  -V, --version             Print version
```

## Capabilities

All capabilities can be pointed at either:

- A mission card file, to process a single model.
- A model home folder (`aurora/`), to process that model.
- A parent folder containing one or more model homes, to process a set of models.

### Validate models

- Confirm cards conform to the card schema.
- Confirm the compact model (if present) conforms to the compact schema.
- Verify model invariants.
- Verify canonical vocabulary usage.
	+ Unknown cards and relationships should be ignored (warning-level), unless strict mode is enabled.
- Optionally verify derived renderings (Markdown, views, compact export) are current and consistent with the model.

### Render human-friendly documentation

- Render a fully linked Markdown representation of the model.
- Render common architectural views.
- Rendering must be deterministic (same input produces stable output).

### Generate compact model export

- Generate a single-file compact export conforming to the compact schema.
- The compact file should:
	+ Contain a top-level `cards` array.
	+ Remove `$schema` and `audit_trail` from the exported cards.
	+ Preserve card `id`, `card_type`, and all semantic fields and links.

### Package a model

- Package a model home into a ZIP for transport.
- ZIP layout must preserve the model home root folder so `unzip model.zip` produces an `aurora/` folder.

### Rename models (`*.json` ↔ `*.jsjson`)

- Provide a conversion workflow that renames files and updates internal references.
- Enforce the invariant that a model uses a single extension consistently.

### Maintain semver audit trail

- Bump patch/minor/major on demand.
- The bump commands should be purely mechanical:
	+ Do not change model semantics.
	+ Update only the audit trail version and audit history (policy described in “Open questions”).

## User flows (happy paths)

### Validate a model

- User points the CLI at either a mission card, a model home, or a parent folder.
- CLI discovers the model home.
- CLI loads cards and runs validation.
- CLI prints a concise summary and returns an exit code.

### Render documentation

- User selects an output folder.
- CLI validates first.
- CLI renders markdown and/or views to the output folder.
- CLI prints:
	+ Output paths written.
	+ Counts of generated artifacts.
	+ Any warnings.

### Generate a compact export

- User runs the compact command.
- CLI produces (or updates) `AGENT-<MISSION_ID>.jsjson` in the model home.
- CLI optionally re-validates the compact file against the compact schema.

## Model discovery (input handling)

Given an input path:

- If the input path is a file:
	+ If it is a mission card (by `card_type: Mission`), treat its containing model home as the target model.
	+ Otherwise, show an actionable error.
- If the input path is a directory:
	+ If it contains `Aurora.schema.json` (or `Aurora.schema.jsjson`), it is a model home.
	+ Else if it contains a child `aurora/` folder that contains the schema, treat that child as the model home.
	+ Else, search for one or more `aurora/` folders beneath it and run in multi-model mode.
- If multiple model homes are found:
	+ Process each independently and report per-model summaries.

## Diagnostics, exit codes, and logging

- Diagnostics
	+ Provide severity (error/warn/info), stable codes, and the card ID + field path when relevant.
	+ Output should be human-friendly by default and machine-friendly when requested (open question: `--format json`).
- Exit codes
	+ `0` for success.
	+ Non-zero for failure.
	+ Note: `render-all` currently returns exit code `99` on failure in this workspace; treat that as an error until a canonical mapping is defined.
- Logging
	+ Support structured logs when possible.
	+ Never log secrets or sensitive payloads.
	+ Respect `--log` level.

## Safety and security

- Treat model files as untrusted data.
- Never execute any content from model files.
- Constrain writes:
	+ Model edits (e.g., bump commands) write inside model home only.
	+ Renders write only to the explicitly selected output folder.
	+ ZIP creation must prevent path traversal.

## Performance considerations

- Large models:
	+ Validation should stream or batch where practical.
	+ Avoid quadratic graph traversals by maintaining indexes.
- Multi-model mode:
	+ Process models independently.
	+ Offer parallelism as an implementation detail only when deterministic outputs are preserved.

## Extensibility

- Unknown card types and relationships
	+ Ignore by default and warn.
	+ Provide an optional strict mode that fails validation.
- Output formats
	+ Consider a stable machine-readable diagnostics output (`--format json`).
	+ Consider a dry-run mode for render and package operations.

## Open questions

- What is the canonical mapping from validation failures to exit codes?
- Should validation default to “strict” regarding unknown vocabulary, or default to warning-only?
- Should render commands fail if derived outputs are stale, or should that be a separate “check” mode?
- What is the canonical policy for `audit_trail.version` bumps in automation?

These commands are best implemented in the shared library, since the editor and VS Code extension will need the same behaviors.
