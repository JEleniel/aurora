# AURORA Tool (Tauri + Vite + React + TypeScript)

The AURORA Tool is a desktop editor for creating, editing, validating, and
visualizing AURORA architecture models. It focuses on small, modular JSON "cards"
that represent actors, interfaces, requirements, drivers, constraints, and
logical components, and on generating the usual architecture diagrams from those
cards.

Key ideas

Single archive per architecture: the editor stores an architecture as a
   compressed archive containing all model JSON files. Use standard ZIP files
   (e.g. `my-architecture.zip`) that contain individual `.json` card files.

JSON-first: each architectural element is a JSON card validated against the
   repository `schemas/` (AJV). The editor provides a Monaco-based JSON editor
   with live validation to help keep cards correct.

Diagrams: generate diagrams (Mermaid-based, and other renderers) from the
   card set. Diagrams are derived artifacts — they are generated from the model
   and can be exported as images or embedded in documentation.

What this folder contains

The Tauri desktop application (Vite + React + TypeScript) used as the
   primary authoring environment for AURORA models.

Runtime and build scripts (`package.json`, Tauri configs) to run, develop,
   and build the app on supported platforms.

Integration with the repo `schemas/` for AJV validation and with the
   `tool/src` UI components (Monaco editor, card browser, diagram canvas).

Quick start (development)

1. Ensure prerequisites:

Node 16+ and `pnpm` for JavaScript tooling.

Rust toolchain (stable) for Tauri builds. Install from [rustup.rs](https://rustup.rs).

Platform dependencies for Tauri on Linux: install `libwebkit2gtk-4.0-dev` (package name differs by distro).

```bash
cd tool
pnpm install
pnpm run dev   # starts the Vite dev server (web UI)
```

To run the native desktop app (requires Rust & Tauri):

```bash
# once dependencies are installed and the dev server is running:
pnpm tauri:dev
```

Build (packaging)

```bash
cd tool
pnpm run build
pnpm run tauri:build
```

Usage notes

## Open and edit archives

The editor prefers a single ZIP archive that contains `.json` card files. Use
`Load Workspace` to open a ZIP — the app looks for `workspace.json` or will
assemble cards by scanning JSON files in the archive.

## Save behavior

`Save` writes the currently-open card back into the open archive (if an
archive is loaded) or prompts a save/download when running in the browser.
`Save All` writes `workspace.json` plus any modified card files into the
archive (useful to commit a set of edits at once).
`Autosave` (toggle) automatically saves the editor contents after a short
debounce (about 800ms). When an archive is open, autosave updates the
archive entry; otherwise it triggers a download/save dialog.

## File naming

Prefer descriptive paths inside ZIP archives such as `actors/user.json`,
`interfaces/public-api.json`, or a single `workspace.json` that contains
`cards` and `links` arrays. The app will accept either layout.

## Diagrams

Diagrams are generated from the card set (Mermaid). Use `Generate Diagrams`
to write Mermaid markdown to the repository `docs/`.

Developer notes

Schemas live at the repository root `schemas/` directory — keep them in sync
   with any card design changes. The Monaco editor is wired to the `card`
   schema by default and will show live validation.

The UI uses Monaco for JSON editing and AJV for validation; update
   dependencies carefully to avoid breaking validation behavior. See
   `tool/src/components/MonacoEditor.tsx` and `tool/src/lib/io.ts` for validation
   and file I/O helpers respectively.

Archive helpers: the app provides both `openArchive()` and `saveArchive()`
   helpers that use `jszip` (browser fallback) or Tauri FS APIs when running as
   a desktop app.

When packaging desktop builds, ensure the Rust toolchain and Tauri
   prerequisites are installed on the build machine. On Linux this typically
   requires `libwebkit2gtk` development packages and a working `cargo` toolchain.

Troubleshooting / tips

If `pnpm tauri:dev` fails with `tauri: not found`, make sure you installed
   the Tauri CLI (`cargo install tauri-cli`) and that `~/.cargo/bin` is on your
   PATH. You can also run the Tauri dev process via `npx tauri dev` if you
   prefer an npm-installed binary.

If Vite warns about `postcss.config.ts` module type, add `"type": "module"`
   to `tool/package.json` or rename to `postcss.config.cjs` to remove the
   warning.

Where to look next

+ `tool/src/` — the application source
+ `schemas/` — JSON Schema files used for validation
+ `docs/` — guidance and examples for AURORA cards and diagrams

If you'd like, I can:

add an example architecture archive to `examples/` demonstrating the ZIP + JSON layout,
add a short CLI helper script to create an empty archive skeleton,
or expand this README with screenshots and a walkthrough.
