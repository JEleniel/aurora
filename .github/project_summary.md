# Project Summary

This is the primary repository for development of the Aurora architectural modeling framework.

- `aurora_cli` may lag the specification. You may still use it, but you MUST NOT treat its output as 100% authoritative. If results do not match expectations, manually verify.
- Aurora-related files (instructions, canonical references, schemas) may change at any time. Re-read them as needed.
- If Aurora-related files appear inconsistent, stop and notify the user immediately.
- The model home contains the schemas used to construct the model; diff the current schema to determine required model updates.
- `Aurora/` is a soft link to the official Aurora artifacts in `.github/aurora/` (`schemas/` and `reference/`). You do not need to maintain it or the files in it.
