# Context

This file records short-lived, “current state” facts that affect agent work.

## Current status (2026-02-07)

- Aurora model home directory exists at `docs/design/aurora/` and is now populated (MIS-001 source cards + schemas + audit log).
- `aurora_cli` is currently not usable in this workspace/environment.
   	+ Plan items that depend on CLI validation/rendering should be treated as blocked.

## Sources of truth

- Canonical schemas/registries live in `.github/agents/aurora/` and are mirrored under `schemas/`.
- Human-facing canonical reference is in `docs/design/Aurora.canonical.definitions.md`.
