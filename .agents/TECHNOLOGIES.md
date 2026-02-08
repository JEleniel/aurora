# Technologies

## Tooling constraints (important)

- `aurora_cli` is **not usable right now** in this workspace/environment.
    + Do not assume CLI validation (`validate`) or derived-asset generation (`render-all`, compact exports) can be run.
    + When model correctness matters, rely on schema/registry conformance and (when available) the editor backend/shared library validation surfaces.
    + CLI validation flow was updated in code to report errors cleanly and exit non-zero, but remains unverified here.

## Aurora Editor

- `tools/aurora_editor/` exists in this workspace.
- Do not rely on editor stack assumptions, command surfaces, or DTO contracts unless they are confirmed in the current code.
