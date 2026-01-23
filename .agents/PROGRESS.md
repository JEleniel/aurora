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
  * Status: review
  * Owner: BackendDeveloper
  * Links: [docs/design/AGENT-MIS-002.json](../docs/design/AGENT-MIS-002.json)
  * Next Actions: Await review on the modularized Tauri backend; follow up with UI graph pane wiring after approval.

- [ ] **UIDeveloper** (FEA-003) **Graph navigation view**
  * Status: implementation
  * Owner: UIDeveloper
  * Links: [tools/aurora_editor/src/routes/+page.svelte](../tools/aurora_editor/src/routes/+page.svelte)
  * Next Actions: Run `pnpm tauri:dev`, launch the native folder picker, confirm the error banner’s copy action works, and re-verify the Graph Explorer (zoom slider + recenter) still passes AAA checks on a real mission.

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

- [ ] **BackendDeveloper** (BUG-003) **Editor selection stale after filesystem changes**
  * Status: review
  * Owner: BackendDeveloper
  * Links: [tools/aurora_editor/src/routes/+page.svelte](../tools/aurora_editor/src/routes/+page.svelte)
  * Next Actions: Re-run `pnpm tauri:dev` (now that watcher events emit via the supported Tauri API) and touch a card file to confirm the selection reload still fires end-to-end.

- [ ] **BackendDeveloper** (BUG-004) **Mission-specific compact respects input path**
  * Status: review
  * Owner: BackendDeveloper
  * Links: [tools/aurora_cli/src/aurora.rs](../tools/aurora_cli/src/aurora.rs), [tools/aurora_cli/src/aurora/model.rs](../tools/aurora_cli/src/aurora/model.rs)
  * Next Actions: Run `aurora_cli --input <Mission>.json compact` once more to confirm only that mission's AGENT file updates and share the results with the user.

## Documentation

- [x] **Architect** (ARCH-001) **Align MIS-002 mission metadata with MIS-001**
  * Status: verified
  * Owner: Architect
  * Links: [docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json](../docs/design/aurora/MIS-002-Enable_Aurora_Viewer_And_Editor.json), [docs/design/AGENT-MIS-002.json](../docs/design/AGENT-MIS-002.json)
  * Next Actions: Monitor for any follow-on parity updates needed in MIS-003.

- [x] **Architect** (ARCH-002) **Align MIS-003 mission metadata with MIS-001/002**
  * Status: verified
  * Owner: Architect
  * Links: [docs/design/aurora/MIS-003-Enable_VSCode_and_Copilot_Integration.json](../docs/design/aurora/MIS-003-Enable_VSCode_and_Copilot_Integration.json), [docs/design/AGENT-MIS-003.json](../docs/design/AGENT-MIS-003.json)
  * Next Actions: Confirm the VS Code extension code location/tool_path once the implementation is introduced.

## Reviews & Audits
