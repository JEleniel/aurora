# Progress Plan

## Usage

- Treat this file as the single source of truth for feature, bug, and documentation work.
- Follow the Plan Format Contract in `.github/copilot-instructions.md`.
- Use checkboxes for completion state and keep statuses in sync with the instructions (proposed → pending → implementation → review → verified → deprecated → retired).

## Template

```text
- [ ] **{owner}** ({work-id}) **{summary}**
  * Status: {status}
  * Owner: {role responsible for next action}
  * Links: [.agents/PROJECT_BRIEF.md](../.agents/PROJECT_BRIEF.md) (plus Aurora card, GitHub issue, etc.)
  * Next Actions: {single concrete step}
```

Add new entries under the headings below.

## Features

- [ ] **BackendDeveloper** (FEA-002) **Aurora Editor loads model homes**
  * Status: implementation
  * Owner: BackendDeveloper
  * Links: [docs/design/AGENT-MIS-002.json](../docs/design/AGENT-MIS-002.json)
  * Next Actions: Expand the navigator with filtering plus hook the graph + MCP surfaces to the new summary API.

## Bugs

- [x] **BackendDeveloper** (BUG-001) **Schema & tooling sample fixes**
  * Status: verified
  * Owner: BackendDeveloper (monitor)
  * Links: [schemas/Aurora.schema.json](../schemas/Aurora.schema.json), [schemas/Aurora.compact.schema.json](../schemas/Aurora.compact.schema.json), [.github/instructions/rust_example/model.rs](../.github/instructions/rust_example/model.rs)
  * Next Actions: Keep schemas and instructional samples aligned with future mission updates.

- [x] **BackendDeveloper** (BUG-002) **Include Boundary/Note cards in views**
  * Status: verified
  * Owner: BackendDeveloper (monitor)
  * Links: [tools/aurora_cli/src/aurora/model.rs](../tools/aurora_cli/src/aurora/model.rs), [tools/aurora_cli/src/aurora.rs](../tools/aurora_cli/src/aurora.rs)
  * Next Actions: Keep the view renderer aligned with Aurora special-card semantics.

## Documentation

## Reviews & Audits
