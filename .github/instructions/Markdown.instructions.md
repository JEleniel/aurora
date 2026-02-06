---
applyTo: '*.md'
---

# Markdown Style Guide

The source of truth for enforcement is `.markdownlint-cli2.jsonc`.

## Guidelines

- You MUST use only ATX headings (`#`, `##`, …) and do not indent them.
- You MUST use a single H1.
- You MUST increase heading levels one at a time; do not skip levels.
    + Sibling headings must be unique.
- You MUST NOT use emphasis/strong as the entire heading text.
- You MUST NOT hard-wrap lines in paragraphs.
- You MUST NOT use extra spaces in prose (for example, multiple spaces between words). Tables are an exception where spacing inside cells may be used for readability or alignment.
- You MUST NOT manually align Markdown using spaces outside of tables. If you need alignment, use lists, tables, code blocks (tabs are fine in code), or omit the alignment.
- You MUST end files with exactly one trailing newline.
- You MUST avoid multiple blank lines together.
- Unordered lists MUST follow `.markdownlint-cli2.jsonc` (`ul-style: sublist`, `ul-indent: 4`): use `-` at depth 1, `+` at depth 2, `*` at depth 3, then repeat (`-`, `+`, `*`, ...). Indent each nested level by 4 spaces.

```markdown
- Level 1
    + Level 2
        * Level 3
```

- Ordered lists MUST use sequential numbers.
- You MUST use fenced code blocks with backticks.
- You MUST include a language for all code blocks. Use `text` when no specific language applies.
- When documenting a command, include sample output.
- Do not use bare URLs; use Markdown links. Avoid reversed links. Do not create empty links.
- You MAY use inline HTML but SHOULD avoid it unless it is necessary.
- You MUST use leading and trailing pipe characters for all table rows and ensure consistent column counts in every row.
- Keep blank lines around tables, code blocks, callouts, and headings.

## Frontmatter

- Markdown frontmatter MUST use `---` delimiters.
- Only include frontmatter when required by the consuming tool.
