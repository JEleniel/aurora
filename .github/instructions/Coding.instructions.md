---
applyTo: '**/*'
---

# Coding Instructions

These instructions apply to all work in this repository.

- Instructions specific to a language or file supersede these.
- Never disable checks or tests (e.g. `// @ts-nocheck`, `#[allow(...)]`). Fix code, not checks.
- Apply OWASP guidance and secure-by-design principles.
- Apply Twelve-Factor App principles.
- Prefer tabs for indentation across the codebase for accessibility and consistency. Language-specific requirements and existing file style supersede this; if a file is predominantly spaces, keep spaces and note it in the summary.
- No global variables; global constants are allowed only in a dedicated constants file.
- Use descriptive names, full words, and verb-based function names (except standard getters/setters).
- Tests must prove behavior. Do not write null tests that only call functions without validation.
- Do not declare code “Production Ready”.
- Ensure code is actually wired and behaves as intended. Unimplemented paths must fail fast and be explicit; use the language-specific guidance for the appropriate mechanism.
