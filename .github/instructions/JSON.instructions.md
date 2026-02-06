---
description: 'Agent directives for JSON formatting and linting.'
applyTo: '*.json'
---

# Agent Directives — JSON Formatting & Linting

The repository's Prettier config, if present, (`.prettierrc.json`) is definitive for formatting.

## Formatting & Content Rules

- **Include `$schema` when available**: If a JSON Schema exists for the file being edited or created, include the `$schema` property as the first key in the file with the appropriate URL. If no specific schema is known, prefer widely used schemastore URLs (e.g., `https://json.schemastore.org/prettierrc` for Prettier config).
- **Validate against schema (best-effort)**: When `$schema` is present or a known schema exists for a filename (e.g., `package.json`, `tsconfig.json`, `.prettierrc.json`, `.markdownlint-cli2.jsonc`), attempt to validate the JSON against that schema. Validation is best-effort: do not block or fail a proposed change solely because validation could not be performed (for example, if the schema is unreachable or private). Annotate the summary with the validation outcome if not successful.
- **No comments**: Do not add JavaScript-style comments to `.json` files. JSON must be valid JSON.
- **Double quotes only**: Use double quotes for all keys and string values (JSON standard). Do not use single quotes.
- **Correct primitive types**: Preserve types — booleans and numbers must be JSON booleans/numbers, not strings.
- **Encoding**: Save JSON files as UTF-8 without BOM.
- **Trailing newline**: Ensure a single trailing newline at EOF.
- **Do not edit generated or lock files**: Avoid manual edits to machine-generated files (lockfiles, package manager caches). If you must, document why and test the change.
