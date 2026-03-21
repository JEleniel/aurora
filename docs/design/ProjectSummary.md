# Project Summary

This is the home repository for the Aurora architecture and modeling system.

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

## Key Project Structure

- `src/` - Main application source code
- `config/` - Configuration management
    - `config.schema.json` - Schema for configuration validation
    - `config.json` - Runtime configuration (populated from example)
- `docs/design/` - Architecture and Aurora model design documentation and operator runbooks

## Tooling Preference

When SymForge MCP is available, prefer its tools for repository and code
inspection before falling back to direct file reads.

Use SymForge first for:

- symbol discovery
- text/code search
- file outlines and context
- repository outlines
- targeted symbol/source retrieval
- surgical editing (symbol replacements, renames)
- impact analysis (what changed, what breaks)
- inspection of implementation code under `src/`, `tests/`, and similar
  code-bearing directories

Preferred tools for reading:

- `search_text` — full-text search with enclosing symbol context
- `search_symbols` — find symbols by name, kind, language, path
- `search_files` — ranked file path discovery, co-change coupling
- `get_file_context` — rich file summary with outline, imports, consumers
- `get_file_content` — read files with line ranges or around a symbol
- `get_repo_map` — repository overview at adjustable detail levels
- `get_symbol` — look up symbols by name, batch mode supported
- `get_symbol_context` — symbol body + callers + callees + type deps
- `find_references` — call sites, imports, type usages, implementations
- `find_dependents` — file-level dependency graph
- `inspect_match` — deep-dive a search match with full symbol context
- `analyze_file_impact` — re-read file, update index, report impact
- `what_changed` — files changed since timestamp, ref, or uncommitted
- `diff_symbols` — symbol-level diff between git refs
- `explore` — concept-driven exploration across the codebase

Preferred tools for editing:

- `replace_symbol_body` — replace a symbol's entire definition by name
- `edit_within_symbol` — scoped find-and-replace within a symbol's range
- `insert_symbol` — insert code before or after a named symbol
- `delete_symbol` — remove a symbol and its doc comments by name
- `batch_edit` — multiple symbol-addressed edits atomically across files
- `batch_rename` — rename a symbol and update all references project-wide
- `batch_insert` — insert code before/after multiple symbols across files

Default rule:

- use SymForge to narrow and target code inspection first
- use direct file reads only when exact full-file source or surrounding
  context is still required after tool-based narrowing
- use SymForge editing tools (`replace_symbol_body`, `batch_edit`,
  `edit_within_symbol`) over text-based find-and-replace whenever
  possible to ensure structural integrity and automatic re-indexing

Direct file reads are still appropriate for:

- exact document text in `docs/` or planning artifacts where literal
  wording matters
- configuration files where exact raw contents are the point of inspection

Do not default to broad raw file reads for source-code inspection when
SymForge can answer the question more directly.
