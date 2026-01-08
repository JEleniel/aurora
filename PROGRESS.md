# Progress

## Project Brief

AURORA (Agent-Unified Representation of Requirements and Architecture) is a deterministic, JSON-based architectural modeling framework designed for symmetric readability by humans and machine agents.

Feature list:

- **Specification and core rules**
    + Status: In Progress
    + Source: [README.md](README.md)

- **JSON Schema**
    + Status: Completed
    + Schema: [schemas/AURORA.schema.json](https://github.com/JEleniel/aurora/blob/main/schemas/AURORA.schema.json)
    + Download: [schemas/AURORA.schema.json](https://raw.githubusercontent.com/JEleniel/aurora/main/schemas/AURORA.schema.json)

- **Repository hygiene (standard docs)**
    + Status: Completed
    + Files: LICENSE.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, SUPPORT.md, CHANGELOG.md

## Active Context Summary

- Branch: `v1.0.0`
- Current focus: Align documentation with the updated AURORA rules (mission-rooted graph directed away from `mission`) and add standard GitHub repository docs.
- Next steps: Add LICENSE.md and the remaining repo docs, then run Markdown linting to ensure formatting is consistent.

## Patterns

- **Card model**: One Element = One Card stored as JSON.
- **Graph direction**: Links point away from `mission`; `mission` has only outgoing links.
- **View semantics**: `links[].relationship` is for human readability; interpretation is view/tool dependent.
- **File layout** (recommended): `AURORA/mission-{uuid}.json` and per-type folders under `AURORA/`.

## Technologies

| Area | Technology | Notes |
| --- | --- | --- |
| Schema | JSON Schema (draft-07) | Canonical schema in `schemas/` |
| Formatting | Prettier | Config in `.prettierrc.json` (tabs for Markdown) |
| Linting | markdownlint | Config in `.markdownlint.json` |
| Package manager | pnpm | Lockfile: `pnpm-lock.yaml` |

## Master Project Plan and Progress Tracker

- [x] Update README for updated AURORA rules
- [x] Create LICENSE.md (from former README Legal)
- [x] Add CONTRIBUTING.md
- [x] Add CODE_OF_CONDUCT.md
- [x] Add SECURITY.md
- [x] Add SUPPORT.md
- [x] Add CHANGELOG.md
- [ ] Run Markdown linting and address any findings

## Docs Site (GitHub Pages)

- **Action**: Create a GitHub Pages-ready site in `docs/` using a Midnight theme and sidebar.
    + Status: Completed
    + Files added: `docs/index.html`, `docs/assets/css/midnight.css`, `docs/assets/js/sidebar-toggle.js`
    + Notes: Start page content derived from root `README.md`. Publish the `docs/` folder via GitHub Pages (set source to `docs/` in repository Pages settings).
