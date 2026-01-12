---
applyTo: '*.md'
---

# Markdown Style Guide

This document defines formatting and style conventions for Markdown files.

The source of truth for enforcement is `.markdownlint.json`.

## Scope

- Follow these rules for all Markdown, with special attention to documentation under `docs/`.
- Repository-standard files are exempt from naming conventions when industry conventions conflict (e.g., `README.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, `LICENSE.md`, `CHANGELOG.md`, and `AGENT_PROGRESS.md`). Do not rename them.

## Tooling

- Use `markdownlint` to validate Markdown.
- Prefer fixing formatting by changing the Markdown rather than weakening lint rules.

## Headings

- Use ATX headings (`#`, `##`, …) and do not indent headings.
- Only use a single H1, and it must be the first line of the file.
- Increase heading levels one at a time; do not skip levels.
- Sibling headings must be unique.
- Do not use emphasis/strong as the entire heading text.

## Whitespace

- Do not hard-wrap lines.
- Do not use two spaces together.
- No trailing spaces.
- End files with exactly one trailing newline.
- Keep blank lines normalized (avoid multiple consecutive blank lines).

## Lists

- Use a consistent unordered list style per nesting level (markdownlint enforces a “sublist” style).
- Indent nested unordered lists to satisfy markdownlint’s configured indentation (4 spaces). Hard tabs are allowed by linting; if you use tabs, ensure the rendered indentation is equivalent to 4 spaces for nested list items.
- Ordered lists should use sequential numbers.

## Code Blocks

- Use fenced code blocks with backticks.
- Always include a language. Use `text` when no specific language applies.
- When documenting a command, include representative output where it improves clarity.

## Links

- Do not use bare URLs; use Markdown links.
- Avoid reversed links.
- Link fragments (anchors) are allowed. markdownlint is configured to validate that fragments are correct; this is not a limitation, it prevents broken intra-doc links.
- Do not create empty links.

## Images

- Provide alternative text for images.
- Inline HTML is allowed but should be avoided unless it is necessary.

## Tables

- Use leading and trailing pipe characters.
- Ensure consistent column counts in every row.
- Keep blank lines around tables.
