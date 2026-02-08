---
name: coding
description: Guidelines for writing code of any kind.
---

1. General Guidelines

- The language-specific rules in `../../instructions/*.instructions.md` take precedence over these instructions.
- Follow best practices for the language being edited. Language-specific configs (for example `rustfmt.toml`, `.markdownlint-cli2.jsonc`, `.prettierrc.json`) are authoritative.
- Keep code modular and cohesive (single responsibility). Prefer small functions (~20 lines) and small modules (~200 lines) when practical.
- Unimplemented paths must fail fast and clearly communicate intent (`todo!`, `unimplemented!`, etc.).
- Prefer small, cohesive changes. Fix root causes, not symptoms.
- Use the shortest acceptable path for local files.
- You MUST NOT create a PR unless instructed.
- Prefer mature, well supported dependencies with GPL, MIT, or Apache-2.0 licenses. Dependencies not updated after 2024 are generally discouraged and require explicit justification.

2. Errors and Logging

- Prefer typed errors within libraries/modules.
- Add context at application boundaries.
- Avoid unchecked failures (`unwrap`, `expect`, panics) unless justified by an explicit invariant.
- You MUST NOT disable checks/tests (for example `// @ts-nocheck`, `#[allow(...)]`). Fix the underlying issue instead.
- Include helpful TRACE and DEBUG logging where appropriate for troubleshooting.
- Log at boundaries with appropriate severity.
- Never log secrets at any level.

3. Tests

For all added code:

- Add tests that prove behavior (positive and negative paths).
- Add tests that prove deterministic behavior, when appropriate (for example, fixed seeds for randomized tests).
- Add tests that prove secure behavior, e.g., malformed input, out of range values, etc.

4. Deliverables

- Source code is modular, clean, readable, idiomatic, and aligned with project conventions.
- Appropriate unit and integration tests are added, passing, and maintain at least 90% coverage on all functional code.
- Notes added for the documentation writer explaining changes, new features, and other relevant information for the project documentation.
- Linting, formatting, and static analysis checks are passing.
