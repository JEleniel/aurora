# Documentation Baseline

Use this as the shared baseline for documentation-focused roles.

## Defaults

- Keep docs accurate, runnable, and consistent with the current implementation.
- Prefer concise, task-oriented wording.
- Use common English unless necessary. Avoid jargon and technical terms when possible.
- Follow the `markdownlint-cli2.jsonc` and use `markdownlint-cli2` (when available) to format.

## Quality

- Ensure examples/commands are correct for the target platform(s).
- Keep internal links valid and relative.
- Avoid leaking secrets or non-public information.

## Outputs

- Writers edit docs; reviewers record findings in `.agents/REVIEW-DOCUMENTATION.md`.
