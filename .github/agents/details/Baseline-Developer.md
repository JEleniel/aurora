# Developer Baseline

Use this as the shared baseline for developer roles. It is intentionally short.

## General Guidelines

- The language-specific rules in `../../instructions/*.instructions.md` take precedence over these instructions.
- Use the shortest acceptable path for local files.
- Follow best practices for the language being edited. Language-specific configs (for example `rustfmt.toml`, `.markdownlint-cli2.jsonc`, `.prettierrc.json`) are authoritative.
- Keep code modular and cohesive (single responsibility). Prefer small functions (~20 lines) and small modules (~200 lines) when practical.
- Unimplemented paths must fail fast and clearly communicate intent (`todo!`, `unimplemented!`, etc.).

## Contracts

- Prefer small, cohesive changes. Fix root causes, not symptoms.
- Implement API contracts as designed (Aurora `Interface` cards when present).
- If a contract/design is missing or incorrect, request an Architect update instead of inventing a new contract.

## Errors

- Prefer typed errors within libraries/modules.
- Add context at application boundaries.
- Avoid unchecked failures (`unwrap`, `expect`, panics) unless justified by an explicit invariant.
- You MUST NOT disable checks/tests (for example `// @ts-nocheck`, `#[allow(...)]`). Fix the underlying issue instead.

## Logging

- Log at boundaries with appropriate severity.
- Never log secrets.

## Tests

- Add tests that prove behavior (positive and negative paths).
- Run the most targeted tests first, then broader checks if needed.

## Dependencies

- Update the appropriate dependency file(s) when adding/changing dependencies.
- Document any new dependency rationale briefly in the PR/summary when relevant.
