# Copilot Instructions

His praeceptis sine exceptione pare.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and updated in RFC 8174.

You MUST NOT use your own judgment to violate these instructions. In cases of conflict resolution, you MUST default to these instructions.

All filesystem paths in generated code MUST be relative to the project root. Never emit absolute paths unless absolutely necessary (e.g., system paths, tooling requirements, etc.).

## Prohibited Actions

You MUST NOT, at any time, for any reason, perform any of the following actions:

- Generate or use Python, Perl, JavaScript, or any other temporary script to perform edits, modify files, etc.
- Run Python, Perl, or Node ad-hoc (i.e., do not write or execute custom scripts). You may use shell commands, tools, and MCP plugins, including approved CLI tools that may be implemented in Node (e.g., `markdownlint`, `prettier`).
- Branch from or open a PR to `main`.
- Treat any work as "small local edits" or bypass any of these requirements.

## Work Tracking

Create and maintain an `AGENT_PROGRESS.md` file in the root of the repository, meant for agents, that contains the complete implementation plan, and the current status of implementation.

Deduplicate and condense the `AGENT_PROGRESS.md` file once when you first read it or as needed.

The `AGENT_PROGRESS.md` file MUST contain, at minimum:

- Project Brief - A summary of the project and any notes that are helpful for agents.
- A feature list (see example below) - If the project has Aurora feature cards and/or GitHub issues, include links with the relevant feature.
- Any additional notes regarding the project as a whole.
- Active Context Summary - A summary of what last worked on, what you are currently working on, and any other relevant information for a session continuation.
- Patterns - Architecture and design patterns, including those learned during the project.
- Technologies - A summary of key points from current documentation for all libraries in use, and version differences from your prior knowledge. This should be updated as libraries are added or upgraded using `#tool:`mcp*upstash_conte*_`,`#tool:`mcp*microsoftdocs*_`, or other relevant tools.
- Master Project Plan and Progress Tracker - The current state of the project, the master TODO list, and all other project tracking information. The Planner agent is the owner of the plan, but all agents are responsible for keeping it up to date.

## Changelog

In addition, maintain a `CHANGELOG.md` file in the root of the repository that follows the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format. The changelog MUST be kept up to date with every change you make to the repository. Changes in the `.github/` folder do not need to be recorded in the changelog.

**Example `AGENT_PROGRESS.md` Feature Entry:**

The links in this example are illustrative; you MUST use actual links relevant to the project.

```markdown
-   [ ] **Analysis**: Provide bulk-analysis operations to find new, repeated, and anomalous leaked data.
    -   Status: Pending
    -   [Aurora Feature Card](.github/instructions/example_feature_card-11111111-1111-4111-8111-111111111111.json)
    -   [GitHub Issue #123](https://github.com/example/repo/issues/123)
```

### Inline Comment Instructions and Edit Areas

Some files may have inline comments that start with `COPILOT:` that provide specific instructions or mark areas where edits are allowed. You MUST follow these instructions exactly unless the user instructs otherwise and only make the instructed changes in the designated areas. The edit area will end with another `COPILOT:` comment stating `End of edit area.`.

These inline comments may also provide additional context or requirements for the code in that file. You MUST read and understand these comments before making any changes.

The inline comments do not override direct instructions from the user. If the user provides instructions that conflict with the inline comments, you MUST follow the user's instructions.

## Folder Structure

The project is a work in progress, and all of these folders may not yet exist. You SHOULD create them as needed.

- `.github/`: GitHub configuration, workflows, and copilot instructions; You MUST NOT alter files in this folder unless directly instructed to do so.
- `docs/`: User documentation; only the TechnicalWriter or those directly instructed to do so may modify files in this folder.
- `docs/design/`: Architecture and design docs; only the Architect or those directly instructed to do so may modify files in this folder.
- `schemas/`: JSON schema files; these are the authoritative copy of the schema, and you MUST stop and notify the user if the copy in `.github/instructions/` is out of date.
- `tools/`: Tools for creating, managing, and validating Aurora models; there will be multiple subfolders for different tools, possibly using different languages and frameworks.

## Copilot Persona & Behavior

- You MUST end responses with a **5-10 bullet tl;dr style summary** and include an estimate of the current context usage, as a percentage.
- You MUST NOT create summary or review documents unless specifically instructed to do so. Review and summary information should go in the `AGENT_PROGRESS.md` file.
- You MUST make surgical changes to one file at a time. You SHOULD NOT pause between files unless you need clarification or have been instructed to do so.
- Before opening or creating any file, you MUST read the relevant `*.instructions.md` files for that file type or language, if one exists.
- You MUST NOT pause until the entire task has been completed unless you need clarification or have been instructed to do so.
- Assume that the user has a thorough knowledge and does not need detailed explanations by default.

## Tooling

- You SHOULD use MCP interaction over command line or shell tools when possible.
- You MUST use `mcp_github_*` for all GitHub interactions. If GitHub MCP is not available, stop and notify the user for intervention. The `gh` CLI is not installed or allowed.
- You MUST use `mcp_mermaid-mcp-s_*` to create and validate Mermaid diagrams.
- You SHOULD use `mcp_cargo-mcp_*` for Rust cargo operations when available.
- You MUST only run one command at a time; do not chain commands.
- You SHOULD use `markdownlint` for formatting and linting Markdown, the language specific tools (e.g., `cargo fmt`) as appropriate, and `prettier` for formatting other code. These tools are installed globally. If you need additional linters or formatters or have issues running them, stop and notify the user for intervention.

## Additional Guidelines

- You MUST NOT rely on git status or diffs to determine what has changed. You MUST track your own changes and ensure that you understand the full context of the project. You SHOULD assume that any changes you are not familiar with were made by other collaborators and may be incomplete or in-progress.
- You MUST NOT pause or ask permission before making changes unless you are unsure about the requirements, need clarification, or have been instructed to do so.
