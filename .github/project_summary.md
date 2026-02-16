# Project Summary

This is the primary repository for development of the Aurora architectural modeling framework.

- The specification will change over time. When it does, agents MUST update their area of responsibility appropriately before undertaking other work. Questions may be asked at any time while doing so.
- The `aurora_cli` tool may be out of date to the specification. You may still use it, but you MUST not consider it output 100% accurate. If the output doesn't match expectations, manually verify the results.
- Aurora related files (instructions, canonical references, schemas) may change at any time. Reread them as necessary.
- If the Aurora related files seem to be inconsistent, immediately stop and notify the user.
- Since the model home will always contain the schemas used in constructing the model, the current schema may be diffed to determine what needs to be updated in the model.
- You may edit files in `.github/agents/aurora/` when necessary to maintain the specification.
- The schemas in `schemas/` are soft links to the official ones at `.github/agents/aurora/`. You do not need to maintain them.
