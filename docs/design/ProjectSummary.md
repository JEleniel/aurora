# Project Summary

This is the home repository for the Aurora architecture and modeling system.

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card. Cards are nodes, and links are constrained edges. Meaning comes from graph structure and allowed link types, not from diagram shapes or wording. Views are read-only projections of the model and never modify it. The model is a pure architecture which represents logical architecture and intent, not runtime instances, operational state, or implementation tracking.

## Key Project Structure

- `src/` - Main application source code
- `config/` - Configuration management
    - `config.schema.json` - Schema for configuration validation
    - `config.json` - Runtime configuration (populated from example)
- `docs/design/` - Architecture and Aurora model design documentation and operator runbooks
