# Agent Instructions

His praeceptis sine exceptione pare.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and updated in RFC 8174.

## Invariants

### Instruction Precedence

Instructions MUST be obeyed in the following order, earlier overriding later:

1. System Instructions (including safety policies and tooling constraints)
2. User Instructions (nothing overrides user intent except System Instructions)
3. Inline comment instructions
4. Language-specific and applicable `.github/instructions/*.instructions.md`
5. Repo Instructions (this file)
6. Tool defaults and generated templates

If tooling limitations or system instructions prevent compliance, you MUST stop and notify the user of the conflict.

### Work Tracking (Memory & .agents)

The `.agents/` folder is for agent use. You MUST create it, and the files in it, if they do not exist. You MUST NOT worry about formatting or linting the files in `.agents/`. Some tooling cannot open files above a fixed size limit (for example, ~50MB). Keep these files below that limit by de-duplicating and compressing as needed.

Minimum required files:

- `.agents/PROJECT_BRIEF.md` - A summary of the project and notes on changes to the scope
- `.agents/PROGRESS.md` - The Project Plan, written by the Planner and maintained by _all_ agents
- `.agents/MAP.md` - Notes on the layout of the source, locations of key functions, and other things to help agents navigate without searching

Review agents MUST create the appropriate review file, and other agents MUST act on the feedback:

- `.agents/REVIEW-CODE.md`
- `.agents/REVIEW-SECURITY.md`
- `.agents/REVIEW-DOCUMENTATION.md`
- `.agents/REVIEW-RELEASE.md`

### Changelog

Maintain `CHANGELOG.md` in Keep a Changelog format. Do not track changes to `.github/`, `docs/`, or `.agents/` in the changelog. Consolidate similar or related entries to keep the log concise.

### Other Invariants

- You MUST NOT modify `.github/**/*` unless the user asks.
- You MUST NOT rely solely on git status/diffs; track your own changes.
- You MUST NOT revert changes you did not make.
- If you are writing code, you MUST read and follow [Baseline-Developer.md](agents/details/Baseline-Developer.md).
- If you are writing documentation you MUST read and follow [Baseline-Documentation](agents/details/Baseline-Documentation.md)
- If you are reviewing code or documentation, you MUST read and follow [Baseline-Reviewer](agents/details/Baseline-Reviewer.md)

## Behavior

### Response Style

- Always be concise by default. Prefer one to two paragraphs or 5-10 bullets.
- Only use long explanations when the user asks for them or when correctness depends on it.
- Avoid repeating the prompt, restating plans, or narrating obvious steps.
- Prefer a 5-10 bullet summary format. Always end with an estimate of the current context usage as a percent.

### Tools

- Use MCP tools for GitHub interactions (do not use `gh`).
- Use the Mermaid.js MCP to render/validate Mermaid diagrams when creating diagrams.
- Terminal and scripting constraints are defined in `.github/instructions/IDE.instructions.md`, if present.

## Common Project Folders

- User documentation is at `docs/` and starts at `docs/README.md` (if present).
- Design documentation is at `docs/design/` (if present).
- Working assets (styles, images) are at `assets/`.
