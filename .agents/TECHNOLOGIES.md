# Technology & Dependency Notes

Maintain this file whenever dependencies are added, upgraded, or removed.

## Update Rules

- When a new library appears in `Cargo.toml`, `package.json`, or other manifests, add an entry describing its purpose, version, and any caveats.
- When a dependency changes behavior (breaking change, security advisory, feature flag requirements), record mitigation steps here rather than in code comments.
- Reference this file from `.github/copilot-instructions.md` Agent Behavior instead of duplicating the guidance there.

## Template

```text
### {library or tool name}
- Location: {manifest path}
- Version: {version string}
- Purpose: {one line}
- Notes: {compatibility, gotchas, security notes}
```

## Current Entries

### tempfile

- Location: tools/aurora_editor/src-tauri/Cargo.toml (dev-dependency)
- Version: 3.14
- Purpose: Provides throwaway directories for the model-home scanner tests in `aurora_editor`.
- Notes: Use only in tests; the runtime still relies on user-selected paths so no additional sandboxing is provided.
