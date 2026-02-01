# Agent Instructions

His praeceptis sine exceptione pare.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and updated in RFC 8174.

Instruction precedence (earlier entries override later ones):

1. System Instructions (including safety policies and tooling constraints)
2. User Instructions (nothing overrides user intent except System Instructions)
3. Inline comment instructions
4. Language-specific and applicable `.github/instructions/*.instructions.md`
5. Repo Instructions (this file)
6. Tool defaults and generated templates

If tooling limitations or system instructions prevent compliance, you MUST stop and notify the user of the conflict.

## Common Project Folders

- User documentation is at `docs/` and starts with `docs/README.md`
- Design documentation is at `docs/design/`
- Agent notes are at `.agents/`
- Working assets (styles, images) are at `assets/`
- Dot (`.`) folders should generally be ignored

## Coding Guidelines

- You MUST use relative paths for local files unless absolutely necessary (e.g., system paths, tooling requirements, etc.). Links in documentation MUST be relative to the document.
- You MUST conform to best practices for the language you are coding in. Language-specific configuration files (e.g., `rustfmt.toml`, `.markdownlint-cli2.jsonc`, and `.prettierrc.json`) are authoritative and override general style rules.
- You MUST use tabs whenever possible for indentation unless the formatter and associated configuration specify otherwise. Do not fight the formatter. If a file could use tabs but has spaces for indentation, keep the file consistent and report the exception to the user.
- You MUST organize code into logical modules that conform to the _single responsibility_ principle and the language-specific style. You SHOULD aim for a maximum of 20 lines per function, excluding boilerplate. You SHOULD aim for a maximum of ~200 lines per file.
    + Self contained objects with internally maintained state are preferred over scattered functions.
- You SHOULD aim for a maximum of approximately 200 lines per file. Modules SHOULD only contain a single primary structure and supporting elements _for that module only_. Shared supporting elements MUST be placed in separate files.
- You MUST use POSIX-style newlines (`\n`).
- You MUST use uppercase for hex literals. Other uses of hexadecimal should be consistent with idiomatic styles.
- Apply OWASP guidance, secure-by-design principles, and Twelve-Factor App principles.
- No global variables; global constants are allowed only in a dedicated constants file.
- Use descriptive names, full words, and verb-based function names (except standard getters/setters).
- Tests must prove behavior. Do not write null tests that only call functions without validation.
- You MUST NOT disable checks or tests (e.g., `// @ts-nocheck`, `#[allow(...)]`). Fix the underlying issue instead.
- Unimplemented paths must still fail fast and clearly communicate intent (`todo!`, `unimplemented!`, etc.).

### Logging

- Use structured, leveled logs where possible.
- `TRACE`, `DEBUG`, `INFO`, and `WARN` should be suitable for standard output; `ERROR` should go to standard error when the runtime supports it.
- Log at boundaries (CLI entry points, request handlers, job runners) and avoid noisy logs in tight loops.

## Prohibited Actions

- You MUST NOT write or execute custom scripts, or run Python, Perl, or Node ad-hoc.
    + Any multi-line terminal input (anything that contains a newline, including heredocs) is a script.
    + A series of commands joined using pipeline (`|`) is a single instruction for this purpose and is allowed.
    + Do not use shell backgrounding (`&`) or command chaining operators such as `&&` or `;`.
    + You may use approved tools (listed later in this file), shell commands, IDE tools, and MCP plugins.
- You MUST NOT use `true` (including patterns like `|| true`) to mask failures or override exit codes.
- You MUST NOT branch from or open a PR to `main`.
- You MUST NOT use the `gh` command line tool. It is not installed.
- You MUST NOT pause before beginning work unless you have specific questions. You MUST NOT pause once work has begun until all tasks are complete.

## Work Tracking

- You MUST maintain a `.agents/MAP.md` with details to help you find your way around the code, documentation, and models as you work.
- You MUST NOT worry about formatting or linting the files in `.agents/` as they are for agent use only.
- You MUST track your progress in the `.agents/PROGRESS.md` (Progress Plan).
- When performing a review, you MUST create a `.agents/REVIEW-{TYPE}.md` file with all findings, mitigation guidance, and references. Link the review file from `.agents/PROGRESS.md`.
- See `.agents/PROJECT_BRIEF.md` for the canonical description of required `.agents/` files (PROJECT_BRIEF, PROGRESS, PATTERNS, TECHNOLOGIES, CONTEXT, and any review files).

### Plan Format Contract (All Agents)

The Planner owns the plan structure, but all agents must follow the same format when updating Project Plan:

- Add new work items using a stable identifier, short title, and explicit **Status**.
- Every item must include **Owner**, **Links** (Aurora card when applicable), and **Next Action**.

**Example Project Plan Feature Entry:**

The links in this example are illustrative; you MUST use actual links relevant to the project and relative to the Project Plan.

```markdown
-   [ ] **{{owner}}** ({id}) **{name}**
    * Status: {status}
    * Next Actions: ...
    * [Aurora Feature Card](.github/instructions/example_feature_card-11111111-1111-4111-8111-111111111111.json)
    * [GitHub Issue #123](https://github.com/example/repo/issues/123)
```

### Changelog

You MUST maintain a `CHANGELOG.md` file in the root of the repository that follows the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. The changelog MUST be kept up to date with every change you make to the code. Documentation, the `.agents/` files, and the `.github/` folder MUST not be tracked in the `CHANGELOG.md`.

### Inline Comment Instructions and Edit Areas

Some files may have inline comments that start with `AGENT:` that provide specific instructions or mark areas where edits are allowed. You MUST follow these instructions in accordance with the instruction priorities. The edit area will end with another `AGENT:` comment stating `End of edit area.`. Remove the comments when you finish the edits.

These inline comments may also provide additional context or requirements for the code in that file. You MUST read and understand these comments before making any changes.

## Agent Behavior

- You MUST NOT modify `.github/` on any files in it unless explicitly instructed.
- Respect role ownership for `docs/`, `docs/design/`, `tools/`, and language-specific source trees; work inside those areas only when acting in that role.
- You MAY update `.agents/*` and `CHANGELOG.md` as required by these instructions.
- If a `docs/design/aurora/AGENT-*.json` file exists, read it to load the entire design. Except for the Architect, you do not need to read the entire model.
- When a new technology or dependency is added or an existing one is changed (including when detected from someone else's changes), you MUST read the current documentation for the correct version and annotate the `./agents/PROGRESS.md.md` with any notes needed to work safely and idiomatically.
- You MUST end final responses with a short summary paragraph, followed by a blank line, then **5-10 tl;dr bullets**. The last bullet MUST include an estimate of the current context usage as a percentage.
- You MUST make changes in small blocks, or use IDE or other approved tools for supported batch operations. You MUST NOT pause between files unless you need clarification or have been instructed to do so.
- Before opening or creating any file, you MUST read the relevant `*.instructions.md` files for that file type or language, if one exists.
- If you have any questions, ask before beginning work; otherwise go directly to work. Once started, continue until the work is complete unless absolutely necessary to stop.

## Tools

- You SHOULD use MCP interaction instead of command line or shell tools when possible.
- You MUST use the GitHub MCP for all GitHub interactions. If GitHub MCP is not available, stop and notify the user.
- You MUST use the Mermaid.js MCP to create and validate Mermaid diagrams.
- You MUST only run one command at a time; do not use shell backgrounding (`&`) or command chaining (e.g., `&&` or `;`). Pipelines (`|`) are allowed.
- You MUST use `markdownlint-cli2`, `prettier`, and language-specific tools for formatting and linting.

## Additional Guidelines

- You MUST NOT rely solely on git status or diffs to determine what has changed. You MUST track your own changes and ensure that you understand the full context of the project.
- Other agents and collaborators are also working on this project. Any changes you do not recognize were made by them. You MUST NOT revert changes you did not make.
- You MUST NOT pause or ask permission before making changes unless you are unsure about the requirements, need clarification, or have been instructed to do so.
