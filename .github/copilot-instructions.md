# Agent Instructions

His praeceptis sine exceptione pare.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and updated in RFC 8174.

Prioritize instructions in the following order, with higher numbers overriding lower:

1. Repo Instructions (this file)
2. Language Specific and applicable `*.instructions.md`
3. Inline comment instructions
4. User Instructions (the only thing that overrides user instructions is System Instructions)
5. System Instructions (including safety policies and tooling constraints)

If tooling limitations or system instructions prevent compliance, you MUST stop and notify the user of the conflict.

## General Coding Guidelines

- You MUST use relative paths for local files unless absolutely necessary (e.g., system paths, tooling requirements, etc.). Links in documentation MUST be relative to the document.
- You MUST conform to best practices for the language you are coding in. Language-specific configuration files (e.g., `rustfmt.toml`, `.markdownlint.json`, and `.prettierrc.json`) are authoritative and override general style rules.
- You MUST use tabs whenever possible for indentation unless the formatter and associated configuration specify otherwise. Do not fight the formatter. If a file could use tabs but has spaces for indentation, keep the file consistent and report the exception to the user.
- You MUST organize code into logical modules that conform to the _single responsibility_ principle and the language-specific style.
- You SHOULD aim for a maximum of approximately 200 lines per file. Modules SHOULD only contain a single primary structure and supporting elements _for that module only_. Shared supporting elements MUST be placed in separate files.
- You MUST use POSIX-style newlines (`\n`).
- You MUST use uppercase for hex literals. Other uses of hexadecimal should be consistent with idiomatic styles.
- Apply OWASP guidance and secure-by-design principles.
- Apply Twelve-Factor App principles.
- No global variables; global constants are allowed only in a dedicated constants file.
- Use descriptive names, full words, and verb-based function names (except standard getters/setters).
- Tests must prove behavior. Do not write null tests that only call functions without validation.
- This project follows Test Driven Development, so failing tests will be written by the TestDeveloper before the BackendDeveloper or UIDeveloper write the code.

## Security by Default

- Prefer secure defaults over optional hardening.
- Validate all external inputs at trust boundaries.
- Follow least privilege: minimize permissions, exposed surfaces, and sensitive data retention.
- Never log secrets or sensitive payloads.

## Logging

- Use structured, leveled logs where possible.
- `TRACE`, `DEBUG`, `INFO`, and `WARN` should be suitable for standard output; `ERROR` should go to standard error when the runtime supports it.
- Log at boundaries (CLI entry points, request handlers, job runners) and avoid noisy logs in tight loops.

## Prohibited Actions

- You MUST NOT write or execute custom scripts, or run Python, Perl, or Node ad-hoc.
    + Any multi-line terminal input (anything that contains a newline, including heredocs) is a script.
    + A series of commands joined using pipeline (`|`) is a single instruction for this purpose and is allowed.
    + Do not use command chaining operators such as `&&` or `;`.
    + You may use approved tools (listed later in this file), shell commands, IDE tools, and MCP plugins.
- You MUST NOT branch from or open a PR to `main`.
- When performing a review, you MUST NOT create summary or review documents unless specifically instructed to do so. Review and summary information MUST go in the `AGENT_PROGRESS.md` file.
- You MUST NOT use the `gh` command line tool. It is not installed.

## Work Tracking

- You MUST create and maintain an `AGENT_PROGRESS.md` file in the root of the repository meant for agents.
- You MUST de-duplicate and condense the `AGENT_PROGRESS.md` file once when you first read it or as needed (e.g., when it is over 500 lines).
- The `AGENT_PROGRESS.md` file MUST contain, at minimum:
    + Project Brief - A summary of the project. This MUST be kept in sync with the repo `README.md` and designs in `docs/design/`.
    + Master Project Plan - A step-by-step plan for implementation based on the designs, and all other project tracking information. The Planner agent is the owner of the plan, and _all_ agents are responsible for tracking progress against it.
    + Patterns - Architecture and design patterns, including those learned during the project.
    + Technologies - A list of technologies and dependencies extracted from the language-specific configuration. This list SHOULD include notes on differences from your knowledge to the current version of the technologies and dependencies. You MUST update this when adding or upgrading dependencies.
    + Active Context Summary - A summary of what you last worked on, what you are currently working on, and any other relevant information for a session continuation.

### Plan Format Contract (All Agents)

The Planner owns the plan structure, but all agents must follow the same format when updating `AGENT_PROGRESS.md`:

- Add new work items using a stable identifier, short title, and explicit **Status**.
- Update only what you are responsible for:
    + Implementation agents: status transitions, links to relevant code, and next actions.
    + Review agents: findings, severity, and required next actions.
    + Documentation agents: documentation issue lists and required updates.
- Every item must include **Owner**, **Links** (Aurora card when applicable), and **Next Action**.

**Example `AGENT_PROGRESS.md` Feature Entry:**

The links in this example are illustrative; you MUST use actual links relevant to the project and relative to the `AGENT_PROGRESS.md`.

```markdown
-   [ ] **{{owner}}** ({id}) **{name}**
    * Status: {status}
    * Next Actions: ...
    * [Aurora Feature Card](.github/instructions/example_feature_card-11111111-1111-4111-8111-111111111111.json)
    * [GitHub Issue #123](https://github.com/example/repo/issues/123)
```

### Changelog

You MUST maintain a `CHANGELOG.md` file in the root of the repository that follows the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. The changelog MUST be kept up to date with every change you make to the code. Documentation and the `.github/` folder MUST not be tracked in the `CHANGELOG.md`.

### Inline Comment Instructions and Edit Areas

Some files may have inline comments that start with `AGENT:` that provide specific instructions or mark areas where edits are allowed. You MUST follow these instructions exactly unless the user instructs otherwise and only make the instructed changes in the designated areas. The edit area will end with another `AGENT:` comment stating `End of edit area.`. Remove the contents when you finish the edits.

These inline comments may also provide additional context or requirements for the code in that file. You MUST read and understand these comments before making any changes.

The inline comments do not override direct instructions from the user. If the user provides instructions that conflict with the inline comments, you MUST follow the user's instructions.

## Project Structure

This project is a work in progress, and all of these folders and files may not yet exist.

You MAY only create or modify files within an owned area when you are explicitly acting in the role of that owning agent. You SHOULD NOT stop and ask permission before doing so if it is required to finish your assigned task.

- `.github/`: GitHub configuration, workflows, and copilot instructions; You MUST NOT alter files in this folder unless explicitly instructed by the user, except for synchronizing the schema copy in `.github/instructions/`.
- `docs/`: User documentation. The TechnicalWriter owns the files in this folder except for `docs/design/`.
- `docs/design/`: Architecture and design docs. The Architect owns these files.
- `schemas/`: JSON schema files. These are the authoritative schema(s). If there is an `Aurora.schema.json` in this folder that does not match the one in `.github/instructions/` then you MUST update the copy in instructions.
- `tools/`: Tools created to assist with creating and maintaining the project. There will be multiple subfolders for different tools, possibly using different languages and frameworks. These are all approved tools. The TestDeveloper, BackendDeveloper, and UIDeveloper own the tool code.
- Other dot (`.`) folders: tooling specific folders. You MUST ignore them.
- Source code is located in language appropriate locations (e.g., `src/` for Rust). The TestDeveloper, BackendDeveloper, and UIDeveloper own the code.

**Exceptions**:

- You MAY update `AGENT_PROGRESS.md` and `CHANGELOG.md` as required by these instructions.
- You MAY update the `.github/instructions/` schema copy to match `schemas/Aurora.schema.json`.

## Agent Behavior

- If an `docs/design/aurora/AGENT-*.json` file exists, read it to load the entire design.
- When a new technology or dependency is added or an existing one is changed (including when detected from someone else's changes), you MUST read the current documentation for the correct version and annotate the `AGENT_PROGRESS.md` with any notes needed to work safely and idiomatically.
- You MUST end final responses with a short summary paragraph, followed by a blank line, then **5-10 tl;dr bullets**. The last bullet MUST include an estimate of the current context usage as a percentage.
- You MUST make changes in small blocks, or use IDE or other approved tools for supported batch operations. You MUST NOT pause between files unless you need clarification or have been instructed to do so.
- Before opening or creating any file, you MUST read the relevant `*.instructions.md` files for that file type or language, if one exists.
- You MUST ask any questions you have before you begin work. Once you start, you MUST NOT pause until your work is complete unless absolutely necessary.

## Tools

- You SHOULD use MCP interaction instead of command line or shell tools when possible.
- You MUST use the GitHub MCP for all GitHub interactions. If GitHub MCP is not available, stop and notify the user.
- You MUST use the Mermaid.js MCP to create and validate Mermaid diagrams.
- You MUST only run one command at a time; do not chain commands (e.g., `&&` or `;`).
- You MUST use `markdownlint` for formatting and linting Markdown.
- You MUST use the language-specific tools to lint and format code.
- You MUST use `prettier` for JSON and other supported files not covered by other tooling.

## Additional Guidelines

- You MUST NOT rely solely on git status or diffs to determine what has changed. You MUST track your own changes and ensure that you understand the full context of the project.
- Other agents and collaborators are also working on this project. Any changes you do not recognize (e.g., not tracked in the `AGENT_PROGRESS.md`) were made by them.
- You MUST NOT pause or ask permission before making changes unless you are unsure about the requirements, need clarification, or have been instructed to do so.
