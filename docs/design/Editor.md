# APP-002: Aurora Editor Design

## Definitions

- Files: Model files may be stored in either `*.json` or `*.jsjson` files. A single model MUST only use one or the other across the entire model. Anywhere a `*.json` file is referenced in this document it also refers to the `*.jsjson` equivalent.
- `*.jsjson` files are functionally identical to `*.json` files.
- Model Home: Models MUST be located in a folder named `aurora/`. User input may point to the parent of a Model Home, in which case the real Model Home should be discovered and used.
- Card Schema: `Aurora.schema.json` in the Model Home.
- Compact Model: optional file `AGENT-<MISSION_ID>.jsjson` in the Model Home.
- Compact Model Schema: `Aurora.compact.schema.json` in the Model Home.

## Capabilities

The editor is designed to load and manipulate a single model at a time.

- Provide an "Explorer" tree view of the model in the left pane. Where multiple paths exist, the separate subtrees should include the duplicates.
- Provide a TheBrain / mind-map-like browser for the model in the center pane. Parents above, elder siblings (same type) pointing in to the left, younger siblings (same type, pointed at) to the right, and children (other cards pointed to) below.
- Provide an editor for the current card and a "Markdown" view of it in the right pane, split 50/50 by default.
- Allow the user to add, edit, and delete cards, maintaining a full audit trail
- Support WCAG Accessibility
- Support drag and drop card/relationship creation (e.g., drag a plus icon down to add a child)
- Click on a card to bring it to the center (in focus)
- If possible, animate nicely when changing cards.
- Zoom to show from 1-3 generations at once, screen permitting
- Work cross platform

It shares these capabilities with the other tools:

- Validate Models
	+ Confirm that all cards conform to the Card Schema.
	+ Confirm that the compact model, if present, conforms to the Compact Model Schema.
	+ Verify that the model conforms to the invariants.
	+ Verify that the cards and relationships from the canonical set are used properly. Unknown cards and relationships should be ignored.
	+ Verify that renderings (Markdown, Views, etc.) are current and consistent with the model.
- Render a human friendly, fully linked markdown version of models
	+ A set of SVG icons will be included in the application and used to mark the files more clearly.
- Render a set of common architectural views, with appropriate icons.
	+ A set of SVG icons will be included in the application and used per common diagram standards.
- Generate a "Compact Model" which is a single file with all cards in an array named "cards" and stripped of their audit history and "$schema" properties, conforming to the Compact Model Schema. This copy is intended to facilitate use by agents by reducing the token usage.
- Package a model in a ZIP file for easy transport. It should be structured such that `unzip models.zip` creates the `aurora/` folder and all contained substructures.
- Rename models between `*.json` and `*.jsjson`

These commands are best implemented in the Shared Library, since the Editor and Extension will both need the same capabilities.

## User flows (happy paths)

- Open model
	+ Choose a workspace folder or a model home (`aurora/`).
	+ If a parent folder is selected, discover `aurora/` (see “Model discovery” below).
	+ Load cards into memory, then show explorer + focused graph + inspector.
	+ Show a persistent “Model Health” status (valid/invalid + counts).
- Browse and focus
	+ Tree: click a card ID/name to focus.
	+ Graph: click a card to focus and animate into place.
	+ Keyboard: search box to jump to card by ID/name.
- Edit
	+ Edit card fields (name/description/status/subtype/attributes) and manage outgoing links.
	+ Add a new card (choose card type, assign next available ID, scaffold required fields).
	+ Delete a card (soft-delete recommended: set status to Deleted and keep tombstone).
	+ Every edit appends an audit history entry.
- Validate
	+ Run validation on demand and on save; optionally in background with debounce.
	+ Errors are navigable: clicking an error takes you to the offending card and field.
- Render
	+ Render “human-friendly” markdown and view diagrams to a chosen output folder.
	+ Render should be deterministic (same input → same output).
- Export and share
	+ Export compact model file.
	+ Package model home into a ZIP (safe zip creation rules apply).

## Model discovery (folder selection)

- If user selects a folder:
	+ If it contains `Aurora.schema.json` and looks like a model home, treat it as the model home.
	+ Else if it contains a child folder named `aurora/` that contains `Aurora.schema.json`, treat that as the model home.
	+ Else, show a clear error with “How to fix” and a “Create model home” action (optional).
- If multiple `aurora/` candidates exist beneath the chosen folder:
	+ Ask the user which model to open (show mission cards found).

## Data model & state (internal)

- In-memory representation
	+ Parse each card into a strongly typed structure (preserve unknown fields under a map so round-tripping is lossless).
	+ Maintain indexes:
			- `by_id`: card ID → card
			- `outgoing`: card ID → outgoing links
			- `incoming`: card ID → incoming links (derived)
			- `by_type`: card_type → ids
			- `search`: tokens for id/name/description
	+ Track “dirty” state per card and per file.
- Graph navigation helpers
	+ Parents: incoming links.
	+ Children: outgoing links.
	+ Siblings: by card_type among parents/children context.
	+ Boundaries/Notes:
			- Notes should be treated as incoming-only leaf annotations.
			- Boundaries should group but not change semantics; rendering can optionally cluster.

## Persistence & audit trail

- Save strategy
	+ Write changes atomically: write temp file then rename.
	+ Preserve file ordering/formatting as much as feasible; prefer stable pretty-print.
	+ Never write outside the chosen model home unless explicitly asked for render outputs.
- Audit trail
	+ On every edit, bump `audit_trail.version` (patch/minor/major policy is an open question).
	+ Append audit event with editor identity and RFC3339 timestamp.
	+ Avoid duplicate audit entries when a single user action touches multiple fields.
- Undo/redo
	+ Recommended: command-stack of semantic edits (add link, edit description, etc.) not raw text diffs.
	+ Undo should also revert audit entries or mark compensating audit events (decide and document).

## Validation & diagnostics

- Validation modes
	+ Fast (incremental): schema shape + link target existence + basic invariants.
	+ Full: invariants + relationship registry checks + “stale render output” checks.
- Diagnostics UX
	+ Show severity (error/warn/info).
	+ Include: card ID, field path, message, and suggested remediation.
	+ Provide “quick fix” actions where safe (e.g., rename invalid verb → `uses`).

## Security & trust boundaries

- Treat model input as untrusted.
	+ Never execute content from model files.
	+ Sanitized rendering for Markdown previews (no script execution; no remote fetch by default).
- File safety
	+ Prevent path traversal when rendering/packaging.
	+ Constrain all writes to either the model home (card edits) or an explicit output folder (renders).
	+ ZIP creation must preserve the `aurora/` root folder.

## Performance considerations

- Large models
	+ Tree view should virtualize large lists.
	+ Search should be indexed.
	+ Graph view should render a bounded neighborhood (1–3 generations) by default.
- Incremental updates
	+ When a file watcher notices changes, re-parse only the impacted card file(s) and re-validate impacted neighborhoods.
	+ Avoid full reload unless invariants are broken.

## Accessibility (WCAG)

- Full keyboard navigation for tree, graph focus, and inspector.
- Screen reader labels for nodes/edges and all interactive controls.
- High-contrast themes and respect OS font scaling.

## Extensibility

- Unknown card types/relationships
	+ Display them generically (do not drop data), and treat unknown relationships as non-fatal.
	+ Allow filtering and grouping by card_type even when not in the canonical palette.
- View definitions
	+ Consider loading view presets from a configuration file so new views don’t require UI code changes.

## Open questions

- What is the canonical policy for `audit_trail.version` bumps in interactive tools?
- Should “delete card” be a tombstone (`status: Deleted`) or a physical delete (file removal) with a recorded event elsewhere?
- Should renders be stored under `docs/design/` by convention, or always user-chosen?
- Do we support multi-mission workspaces in the editor, or strictly one model home per window?
