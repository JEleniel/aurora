# PROGRESS

## Project Brief

Aurora is a deterministic, JSON-based architectural modeling framework where architectural elements are Cards and relationships are directed Links away from a root `mission` card. This repository ships the canonical JSON Schema, a GitHub Pages site, a reference HTML viewer, and an example model.

### Current Repo Deliverables

- Specification overview: [README.md](README.md)
- Machine agent guidance: [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md)
- Canonical card schema (v1.2.0): [schemas/Aurora.schema.json](schemas/Aurora.schema.json)
- GitHub Pages site (human docs): [docs/index.html](docs/index.html)
- HTML model viewer: [docs/viewer.html](docs/viewer.html)
- View documentation: [docs/views/Views.md](docs/views/Views.md)
- Example model (mobile ordering): [docs/aurora](docs/aurora)
- Tooling documentation placeholders: [docs/tools](docs/tools)

### Example Model Feature Cards

These are feature cards in the reference model under [docs/aurora](docs/aurora). They are not the repository deliverables; they exist to demonstrate modeling, traceability, and view generation.

- [ ] **Menu Browsing**: Allow customers to browse the restaurant menu, view prices, and see item availability. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/menu_browsing-d4c3b2a1-0f98-4b76-8a54-3210fedcba98.json))
- [ ] **Cart Customization**: Allow customers to customize menu items (add-ons, modifiers) and build a cart. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/cart_customization-b2a1908f-2d76-49f4-8c32-10fedcba9876.json))
- [ ] **Secure Checkout**: Collect payment authorization securely and confirm checkout. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/secure_checkout-39e0f1a2-91ab-4cde-8d34-8f9012345678.json))
- [ ] **Store Fulfillment Integration**: Integrate ordering backend with store fulfillment systems. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/store_fulfillment_integration-b5c6d7e8-5555-4fff-8012-234567890abc.json))
- [ ] **Order Status Updates**: Track order state transitions and publish updates. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/order_status_updates-a4b5c6d7-4444-4eee-8f12-1234567890ab.json))
- [ ] **Order Tracking UI**: Show order tracking status to customers. (Status: In Progress; [Aurora Feature Card](docs/aurora/Feature/order_tracking_ui-c3b2a190-1e87-4a65-8b43-210fedcba987.json))
- [ ] **Aurora v2 Tooling + Docs Release**: Publish v2 documentation, viewer package, and initial reference tooling. (Status: In Progress; Tracking: This plan, tooling implementation is not started in this repository)

## Active Context Summary

- Branch `v2.0.0` contains the documentation viewer ([docs/viewer.html](docs/viewer.html)) and the default view docs under [docs/views](docs/views).
- The canonical schema is [schemas/Aurora.schema.json](schemas/Aurora.schema.json) (schema version 1.2.0).
- The example model lives under [docs/aurora](docs/aurora) (lowercase folder name for case-sensitive filesystems).
- The example model cards have been bumped to card version `2.0.0` with an `edited` audit entry dated `2026-01-10T00:00:00Z` to align with the Aurora v2.0.0 model guidance.
- Tooling source code is not implemented in this repository (no root-level `tools/`, no `Cargo.toml`, no `package.json`); placeholder documentation folders exist under [docs/tools](docs/tools).

## Patterns

- **Directed graph from mission**: All non-`note` `links[].target` relationships point away from `mission`; local cycles are allowed only for bounded flows.
- **One file per card**: Each Card is stored as a standalone JSON file named `{id}-{uuid}.json` (or `{card_subtype}-{id}-{uuid}.json`).
- **Type folders**: Cards are stored under type folders using Title Case (for example, `docs/aurora/Requirement/`).
- **Views are projections**: Views are derived by traversing and filtering the same underlying directed graph.

## Technologies

- Static documentation site: HTML/CSS/JS under [docs](docs)
- Viewer diagram rendering: Mermaid (integrated into the HTML viewer assets)
- Schema: JSON Schema under [schemas/Aurora.schema.json](schemas/Aurora.schema.json)

## Master Project Plan and Progress Tracker

- [ ] **Goal 1: Align README ↔ Agent Instructions ↔ Schema ↔ Site** (Status: In Progress; Focus: terminology, paths/casing, and the release story across [README.md](README.md), [CHANGELOG.md](CHANGELOG.md), [.github/instructions/Aurora.instructions.md](.github/instructions/Aurora.instructions.md), and [schemas/Aurora.schema.json](schemas/Aurora.schema.json).)
- [ ] **Goal 2: GitHub Pages-ready documentation set (docs/)** (Status: In Progress; Focus: keep [docs/index.html](docs/index.html), [docs/viewer.html](docs/viewer.html), and [docs/views](docs/views) synchronized with the schema and instructions.)
- [ ] **Goal 3: Package the viewer for offline use** (Status: Pending; Focus: define the artifact contents and a reproducible packaging process for [docs/viewer.html](docs/viewer.html) + [docs/assets](docs/assets).)
- [ ] **Goal 4: Implement the tooling described in README** (Status: Pending; Focus: decide Rust vs TypeScript and implement a real `tools/` workspace at repo root; current state is placeholders only under [docs/tools](docs/tools).)
- [ ] **Goal 5: Tooling documentation** (Status: Pending; Focus: populate [docs/tools](docs/tools) with installation/usage/troubleshooting once tooling exists.)
- [ ] **Goal 6: Release readiness gates** (Status: Pending; Focus: CI validation for schema + graph invariants and integrity checks for the example model under [docs/aurora](docs/aurora).)
