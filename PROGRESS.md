# PROGRESS

## Project Brief

AURORA is a deterministic, JSON-based architectural modeling framework where architectural elements are Cards and relationships are directed Links away from a root `mission` card. The repo ships the canonical schema, a GitHub Pages site, a reference HTML viewer, and (for v2) reference tooling (editor, standalone viewer, CLI).

### Current Repo Deliverables

- Specification overview: [README.md](README.md)
- Machine agent guidance: [.github/instructions/AURORA.instructions.md](.github/instructions/AURORA.instructions.md)
- Canonical card schema: [schemas/AURORA.schema.json](schemas/AURORA.schema.json)
- GitHub Pages site (human docs): [docs/index.html](docs/index.html)
- HTML model viewer: [docs/viewer.html](docs/viewer.html)
- Example model (mobile ordering): [docs/AURORA](docs/AURORA)

### Feature List

- [ ] **Menu Browsing**: Allow customers to browse the restaurant menu, view prices, and see item availability.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/menu_browsing-d4c3b2a1-0f98-4b76-8a54-3210fedcba98.json)
- [ ] **Cart Customization**: Allow customers to customize menu items (add-ons, modifiers) and build a cart.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/cart_customization-b2a1908f-2d76-49f4-8c32-10fedcba9876.json)
- [ ] **Secure Checkout**: Collect payment authorization securely and confirm checkout.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/secure_checkout-39e0f1a2-91ab-4cde-8d34-8f9012345678.json)
- [ ] **Store Fulfillment Integration**: Integrate ordering backend with store fulfillment systems.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/store_fulfillment_integration-b5c6d7e8-5555-4fff-8012-234567890abc.json)
- [ ] **Order Status Updates**: Track order state transitions and publish updates.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/order_status_updates-a4b5c6d7-4444-4eee-8f12-1234567890ab.json)
- [ ] **Order Tracking UI**: Show order tracking status to customers.
    + Status: In Progress
    + [AURORA Feature Card](docs/AURORA/Feature/order_tracking_ui-c3b2a190-1e87-4a65-8b43-210fedcba987.json)
- [ ] **AURORA v2 Tooling + Docs Release**: Publish v2 documentation, viewer package, and initial reference tooling.
    + Status: Pending
    + Tracking: This plan (no AURORA meta-model cards yet)

## Active Context Summary

- The repo currently has a working GitHub Pages skeleton ([docs/index.html](docs/index.html)) and a JS-based viewer ([docs/viewer.html](docs/viewer.html) + [docs/assets/js/viewer.js](docs/assets/js/viewer.js)).
- The canonical schema is [schemas/AURORA.schema.json](schemas/AURORA.schema.json) (schema version 1.1.0, includes `state_machine`, and permits custom `card_type` values).
- Tooling source code is not yet implemented (no root-level `tools/` folder, no `Cargo.toml`, no `package.json`). The GitHub Pages placeholder folders exist under [docs/tools](docs/tools).
- Known spec inconsistencies exist across README, instructions, and ancillary docs (details tracked in Goal 1).

## Patterns

- **Directed graph from mission**: All `links[].target` relationships point away from `mission`; local cycles are allowed only for bounded flows.
- **One file per card**: Each Card is stored as a standalone JSON file named `{id}-{uuid}.json` (or `{card_subtype}-{id}-{uuid}.json`).
- **Type folders**: Cards are stored under type folders using Title Case (for example, `docs/AURORA/Requirement/`).
- **Schema is canonical**: The canonical schema lives in [schemas/AURORA.schema.json](schemas/AURORA.schema.json); a model may also carry a local schema copy for portability.
- **Views are projections**: Views are derived by traversing and filtering the same underlying directed graph.

## Technologies

- Static documentation: HTML/CSS/JS under [docs](docs)
- Viewer rendering: Mermaid (local `mermaid.min.js` if present; otherwise CDN fallback)
- Schema: JSON Schema under [schemas/AURORA.schema.json](schemas/AURORA.schema.json)
- Proposed tooling stacks (decision pending):
    + Option A: TypeScript (Node.js) for core + CLI; browser bundles for viewer/editor
    + Option B: Rust core + CLI, with WASM/JS bindings for viewer/editor

## Master Project Plan and Progress Tracker (v2)

> README.md is more authoritative than agent instructions, but neither is definitive. Any spec changes must be proposed, reviewed, and then implemented.

- [ ] **Goal 1: Reconcile README ↔ Agent Instructions ↔ Schema ↔ Site**
    + Status: In Progress
    + Owners: Architect (primary), Technical Writer (support), Security Reviewer (spot-check)
    + Deliverables:
        * A written “alignment report” enumerating differences and proposed resolutions
        * A small set of approved edits (ready for implementation)
    + Work items:
        * Confirm release-status narrative consistency across README vs [CHANGELOG.md](CHANGELOG.md) (README says v1.0.0 released; changelog says pre-release)
        * Fix path assumptions in instructions (currently references `docs/design/AURORA/cards/*.json`, but this repo stores the example model under [docs/AURORA](docs/AURORA))
        * Confirm and document how `state_machine` and custom `card_type` values are treated (schema supports both; instructions and README should reflect this)
        * Confirm default view definitions are consistent across README, instructions, and [docs/assets/js/viewer.js](docs/assets/js/viewer.js) (example: “Communication Diagram” card set differs)
        * Define a change-control process for spec text (issue template + review gates) and add it to this tracker (implementation will require repo policy work)

- [ ] **Goal 2: GitHub Pages-ready human documentation (docs/)**
    + Status: Pending
    + Owners: Technical Writer (primary), Architect (technical review)
    + Deliverables:
        * Updated [docs/index.html](docs/index.html) aligning terminology and framing with README
        * A minimal, navigable documentation set under [docs](docs) covering: schema, card types, link rules, views, file layout, and examples
        * Links from the site to schema, viewer, and downloads
    + Work items:
        * Update the “Overview / Goals / Core Concepts” narrative to match README wording (no new marketing claims)
        * Add sections for “Card Schema”, “Link Rules”, “Default Views”, and “Model Packaging”
        * Add a “Tooling” section that clearly marks features as planned/pending until implemented
        * Ensure the Schema link strategy is correct for GitHub Pages (raw GitHub link vs publishing a copy under docs)

- [ ] **Goal 3: Package viewer.html and ancillary files for download**
    + Status: Pending
    + Owners: UI Developer (primary), Release Reviewer (check packaging), Technical Writer (download instructions)
    + Deliverables:
        * A reproducible packaging process producing a single zip containing the HTML viewer and required assets
        * A clear “offline use” story (include `mermaid.min.js` in the package, or provide explicit fetch steps)
    + Work items:
        * Define the package contents (viewer.html, assets, optional schema copy, optional sample model)
        * Decide where artifacts live (release assets vs committed under [docs/tools/viewer](docs/tools/viewer))
        * Add license/attribution handling for bundled third-party assets (Mermaid, fonts, etc.)

- [ ] **Goal 4: Design and implement tooling mentioned in README (tools/)**
    + Status: Pending
    + Owners: Architect (architecture), Backend Developer (CLI/core), UI Developer (viewer/editor), Test Developer (tests)
    + Deliverables:
        * A root-level `tools/` workspace implementing `aurora-core`, `aurora-cli`, and a defined MVP for editor/standalone viewer
        * Tests covering model validation and edge cases (invalid UUIDs, orphan cards, inverted links, cycles)
    + Work items:
        * Make a stack decision (Option A TypeScript vs Option B Rust) and record rationale in this tracker
        * Define MVP vs post-v2 features for Editor and Standalone Viewer (README currently describes a large scope)
        * Implement `aurora-core` parsing + indexing (load cards, validate schema, validate graph invariants)
        * Implement `aurora-cli` MVP commands (`validate`, `init`, `format`, `export-views`, `serve-viewer`)
        * Add plugin hooks design (keep stable surface area small for v2)

- [ ] **Goal 5: Add user documentation for the tooling**
    + Status: Pending
    + Owners: Technical Writer (primary), Engineering owners for review
    + Deliverables:
        * Usage documentation under [docs/tools](docs/tools) for CLI, editor, and viewer
        * Installation instructions (per-platform, including “no build tools required” path if shipping binaries)
        * Examples using the reference model under [docs/AURORA](docs/AURORA)
    + Work items:
        * Populate [docs/tools/cli](docs/tools/cli), [docs/tools/editor](docs/tools/editor), and [docs/tools/viewer](docs/tools/viewer)
        * Add troubleshooting (common validation errors, broken links, schema mismatch)

- [ ] **Goal 6: Review and finalize everything for v2 release**
    + Status: Pending
    + Owners: Release Reviewer (primary), Security Reviewer (security), Architect (final sign-off)
    + Deliverables:
        * Release checklist executed (docs, schema, example model integrity, tooling build/repro)
        * Updated release notes and version story aligned across README, changelog, tags, and GitHub releases
    + Work items:
        * Add CI checks for schema validation (all cards) and graph integrity (broken targets, unreachable cards, forbidden links)
        * Validate the example model integrity and remove placeholder UUID targets where present
        * Ensure all promised downloadable artifacts are present (viewer zip; CLI binaries or install path)
        * Security review of crypto/sign/encrypt features (if included in v2 scope)

## Documentation Reviews

- README.md review (2026-01-09): TBD (to be authored under docs/reviews/)
    + Status: In Progress
    + Open items tracked under Goal 1 and Goal 6
    + Notes:
        * Editorial cleanup suggested for the “HTML Page Viewer Features” wording
        * CI validation for `card_type` values and extension handling is still pending
