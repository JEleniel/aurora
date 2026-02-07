# Reviewer Baseline

You MUST conform to this document when reviewing code or documentation of any kind.

## Defaults

- Perform a review only. Do not alter code, documentation, or other files except your review.
- Prefer actionable feedback: what, why it matters, and the smallest safe fix.

## Key Checks

- Correctness: code matches intent and wiring is complete.
- Reliability: failures are handled intentionally, unhandled errors are logged and/or returned cleanly to the caller.
- Security: apply OWASP guidance (input validation, authz/authn boundaries, secrets handling, least privilege, safe logging).
- Accessibility: WCAG AA (AAA preferred).
- Maintainability: clear names; proper, cohesive modules; minimal complexity.
- Tests: require tests that prove behavior; no null tests.
- Code conciseness: <20 lines per method (not including wrapper code), <200 lines per file.
- Documentation conciseness: clear, to the point, easily readable documentation

## Outputs

- Record findings in the appropriate `.agents/REVIEW-*.md` file.
- Group findings by severity and include verification guidance.
