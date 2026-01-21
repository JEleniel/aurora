# Project Brief

## Mission Snapshot

- Mission IDs live under `docs/design/aurora/` and the generated `aurora/` model tree.
- All architectural changes must trace back to the active Mission cards (see Aurora model JSON files).

## Folder Ownership

| Area | Owner | Notes |
| --- | --- | --- |
| `.github/` | DevOps / instructions maintainers | Modify only when explicitly instructed, except to sync schemas. |
| `.agents/` | All agents | Shared coordination folder (see files below). |
| `docs/` | Technical Writer (except `docs/design/`) | End-user + operational docs. |
| `docs/design/` | Architect | Aurora cards and design views. |
| `tools/` | TestDeveloper, BackendDeveloper, UIDeveloper | Implementation of CLI/editor/lib tooling. |
| `src/` & language roots | BackendDeveloper, UIDeveloper, TestDeveloper | Implementation + tests follow architecture plans. |

## Required `.agents/` Files

- `PROJECT_BRIEF.md` (this file)
- `PROGRESS.md` (live project plan)
- `PATTERNS.md` (architecture/design patterns learned)
- `TECHNOLOGIES.md` (dependency knowledge)
- `CONTEXT.md` (session continuity notes)
- `REVIEW-*.md` files as reviews occur

## Quick Start Checklist

1. Read `.github/copilot-instructions.md` for global rules.
2. Confirm the latest Aurora mission files relevant to your work.
3. Update `.agents/CONTEXT.md` with your current focus at every summary.
4. Log plan updates inside `.agents/PROGRESS.md` following the plan contract.
5. Capture new technology caveats in `.agents/TECHNOLOGIES.md` when dependencies change.
