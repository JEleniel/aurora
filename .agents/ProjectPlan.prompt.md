# Project Plan

v2.0.0 release with comprehensive documentation, MIS-001 Architect session, cross-platform editor implementation, and in-code documentation for tools.

## Phase 0: SVGZ support (template + tooling) (Complete)

Phase 0 is complete. `svg_prep` and `aurora_cli` are feature complete and refined to work from
Inkscape-authored masters under `assets/masters/`.

- [x] Update svg_prep to generate `SVGTemplate.svgz`.
- [x] Update aurora_shared to load and use `SVGTemplate.svgz`.
- [x] Update svg_prep to import and consolidate `<style>` elements from icons into a single `<style>` element in the output template, to reduce file size and improve maintainability.
- [x] Add a feature to svg_prep to strip Inkscape metadata and optimize SVG icon files for use in Aurora, to reduce file size and improve performance.
    - The original file should remain untouched.
    - The input folder is a CLI option (default: `assets/masters/icons/`).
    - The output folder should be a CLI option with a default of `assets/optimized/icons/`.
    - The proof file should be a CLI option with a default of `assets/proofs/Icons.svg`.
    - The optimization process should include:
        - Removing Inkscape-specific (`inkscape:*`) metadata and attributes that are not needed for rendering in Aurora.
        - Remove sodipodi attributes (`sodipodi:*`).
        - Remove comments and unnecessary whitespace to reduce file size.
        - Remove xmlns:inkscape, xmlns:sodipodi, and xmlns:xlink
        - Remove unnecessary id attributes that are not referenced by other elements in the file.
        - Rename the remaining id attributes to the format used by the current svg_prep template, to ensure compatibility with aurora_shared's template loading and card rendering logic.
        - Remove the "layer" elements.
        - Place the remaining icon elements directly under the root `<svg>` element, to simplify the structure and ensure compatibility with aurora_shared's template parsing logic.
        - Factor out common styles into a single `<style>` element to reduce file size and improve maintainability. Use class names based on the format used to rename id attributes.
    - [x] Update svg_prep to perform the same optimizations on a set of shape files so that the shapes no longer have to be maintained by hand. Each shape should be defined as a separate SVG file in the input folder (unlike the current single file) and end up as a separate output file. Like the icons, a proof file should be generated that combines all the shapes into a single SVG for easy review and reference.
    - The original file should remain untouched.
    - The input folder is a CLI option (default: `assets/masters/shapes/`).
    - The output folder should be a CLI option with a default of `assets/optimized/shapes/`.
    - The proof file should be a CLI option with a default of `assets/proofs/Shapes.svg`.
    * Additional optimizations for shapes:
        - Remove "style" attributes from group elements.
        - Remove "style" attributes from other elements with the exception of `fill-opacity:0;"` or `fill:#00000000;` which is used to define transparent elements of shapes. If one of these is present, the entire "style" attribute should be replaced with `style="fill-opacity:0;"` to ensure that the transparent parts of shapes are preserved while still removing any styles that would break the rendering.
- [x] Update svg_prep to read the shapes from `assets/optimized/shapes/` instead of the hand-edited `assets/proofs/Shapes.svg` file.
- [x] Update aurora_cli to use the svgz-based template flow.

## Phase 1: Foundation & Design

- [ ] Architect works with user to design cross-platform editor.

## Phase 2: Tool Development

- [ ] Implement cross-platform editor per Architect design

## Phase 3: Documentation Writing

- [ ] Complete in-code documentation for svg_prep, aurora_shared, and aurora_cli
- [ ] Core Aurora documentation (concepts, card types, relationships, examples)
- [ ] aurora_cli documentation (commands, installation, workflows, troubleshooting)
- [ ] svg_prep documentation (icon ingestion, proof sheets, templates)
- [ ] Cross-platform editor documentation (features, workflows, UI guide)
- [ ] Developer/contributing guides (aurora_shared API reference, contributing to tools)

## Phase 4: Documentation Review & Refinement

- [ ] Review documentation for completeness, accuracy, clarity, consistency
- [ ] Validate documentation examples and links
- [ ] Organize docs/ hierarchy and finalize structure

## Phase 5: Integration & Release Prep

- [ ] Update CHANGELOG.md for v2.0.0
- [ ] Release v2.0.0

## Parked for the next release:

- `aurora_cli` can cryptographically sign cards and models and verify signatures to ensure authenticity and integrity of card data.
- `AuditLog.json` extended to include an optional compressed record of the specific changes made to each card, including before and after states, to enable detailed change tracking and history.
- Review the implications of using a single JSON file for an entire model, and the effect on agent context windows and performance. Alternatives include a file per card type, or a simplified file format that allows streaming and partial loading of card data.
- Add a lightweight, standalone card viewer that can render cards from JSON files without needing the full aurora_cli toolchain, to enable easier sharing and viewing of cards outside of aurora-specific contexts.
- Implement a lightweight, standalone configuration editor that can manage card definitions, appearance options, relationships, and other model settings in a user-friendly way, without needing to edit JSON files directly.
- Design and build an MCP server that can parse and manage card data and enable agents to work with models in a more performant and scalable way, without needing to load entire models into memory or rely on file-based storage. This could include features like caching, indexing, and query capabilities to optimize agent interactions with card data.
- Look into the possibility of a delta feature for the CLI that can generate a diff of changes between two versions of a model, to help with tracking changes, code reviews, and understanding the evolution of card data. This definitely depends on having some kind of change tracking or versioning system in place, but could be a powerful tool for managing complex models and ensuring transparency in changes.
- Research other architectural functions and practices to allow Aurora to become a one-stop shop for all things agent architecture, including design, documentation, implementation, and best practices. This could include things like design patterns, architectural principles, case studies, and more to help architects and developers build better agents and models.

---

**Invariants** (durable, repo-wide):

- Keep `.agents/` docs accurate and non-redundant.
- Keep canonical schemas and reference registries authoritative under `.github/agents/aurora/`.
- Keep mission models under `docs/design/aurora/` passing `aurora_cli validate`.

Last updated: 2026-02-19
