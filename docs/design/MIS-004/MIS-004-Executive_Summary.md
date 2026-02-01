# MIS-004 Executive Summary

This executive summary captures the mission intent plus 2 drivers and 7 requirements.

## Mission

- **ID:** `MIS-004`
- **Name:** Standalone Editor
- **Description:** Provide a standalone application capable of creating and manipulating Aurora models visually, reusing the aurora_shared library to facilitate development.

## Drivers

- `DRI-004` — Visual Model Authoring. Enable rapid comprehension and editing of an Aurora model through a visual, multi-pane editor that reduces cognitive load and navigation friction.
- `DRI-005` — Reuse Shared Aurora Semantics. Minimize duplicated logic and inconsistencies across tooling surfaces by reusing the aurora_shared library for model discovery, parsing, validation, and rendering.

## Requirements

- `REQ-009` — Three Pane Layout. The editor SHALL present a three-pane layout (similar to Visual Studio or Obsidian): left navigation tree, central graph view, and right inspector.
- `REQ-010` — Shortest Path Tree Navigation. The editor SHALL provide a left-pane tree view for navigation, where the tree is derived from a chosen shortest path from the mission root to each card.
- `REQ-011` — Brain Style Graph Navigation. The editor SHALL provide a central mind-map / TheBrain style view with the focused card centered. The view SHALL show the parent (from the chosen shortest-path chain), and MAY show siblings (same card_type) linked from that parent even if they are not linked to the focused card.
- `REQ-012` — Edit, Preview, and Audit Inspector. The editor SHALL provide a right-pane inspector: the top half edits the focused card; the bottom half renders a Markdown-style view of the card (including links). The bottom half SHALL provide a tab to reveal the card’s audit trail details.
- `REQ-013` — Dioxus UI. The standalone editor SHOULD be implemented using Dioxus as the primary UI library.
- `REQ-014` — Reuse aurora_shared. The editor SHALL reuse aurora_shared for model discovery/loading and validation/rendering semantics to stay consistent with other Aurora tooling.
- `REQ-015` — Safe Workspace Persistence. The editor SHALL persist edits to model files in a safe manner that avoids partial writes and preserves schema validity (e.g., atomic replace per card file).
