# Aurora Tooling (Tauri + Vite + React + TypeScript)

This folder contains the desktop UI and tooling used to author and visualize
AURORA cards and diagrams. It's implemented with Vite + React + TypeScript and
packaged with Tauri for native desktop support.

Quick start

1. Install Node (16+) and Rust toolchain (for Tauri). On Linux, install `libwebkit2gtk`.
2. From repo root (preferred: `pnpm`):

```bash
cd tool
pnpm install
pnpm run dev   # starts Vite dev server
# in another terminal (requires Rust + Tauri deps):
pnpm run tauri:dev
```

Build

```bash
cd tool
pnpm run build
pnpm run tauri:build
```

Notes

- The UI includes a Monaco JSON editor with AJV-based schema validation (schema files live at `schemas/`).
- To render diagrams, paste Mermaid into the editor or load a card with embedded Mermaid.
- Desktop packaging uses Tauri; building a distributable requires the Rust toolchain and Tauri prerequisites.
