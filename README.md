# Agent-Unified Representation of Requirements and Architecture (Aurora)

Aurora is a deterministic, typed, directed graph rooted at a single `Mission` card.

- **Cards** are nodes.
- **Links** are directed edges.
- **Meaning** comes from graph structure and allowed link types, not from diagram shapes or wording.
- **Views** are read-only projections of the model and never modify it.

Aurora models represent architecture and intent, not runtime instances, operational state, or implementation tracking.

## Documentation

User documentation starts at:

- **[`docs/README.md`](docs/README.md)**

That documentation covers:

- Model structure and invariants
- The canonical set (card types, relationships, and views)
- Tooling (`aurora_cli` and `svg_prep`)
- Configuration customization (new card types, relationships, appearance, and views)

## Quick start (example models)

This repository includes an example model home at `docs/design/aurora/`.

Validate the example models:

```text
aurora_cli validate -i docs/design/aurora
```

Example output:

```text
2026-02-19T19:26:35.232950Z  INFO Loaded 2 models from docs/design/aurora
2026-02-19T19:26:35.234247Z  INFO Validating 2 model(s) from docs/design/aurora
2026-02-19T19:26:35.235097Z  INFO No warnings found.
2026-02-19T19:26:35.235100Z  INFO No validation errors found.
```

Render Markdown and views into a temporary output directory:

```text
aurora_cli render-all -i docs/design/aurora -o tmp/aurora_cli_out
```

## Repository layout

Key folders:

- `Aurora_Specs/`: specification artifacts, including schemas and canonical registries.
- `docs/`: user documentation (start at `docs/README.md`).
- `docs/design/`: design documentation and example model home (`docs/design/aurora/`).
- `tools/aurora_cli/`: CLI for validation and rendering.
- `tools/svg_prep/`: SVG asset and template preparation.
- `assets/`: master and optimized SVG assets, plus proof sheets.

## Tooling

Currently supported tools:

- `aurora_cli` (validation, rendering, compact exports)
- `svg_prep` (SVG template/icon/shape preparation)

See **[Tools](docs/Tools.md)**.

## Editor and viewer (planned)

The repository contains design documentation for a future editor and standalone viewer, but those applications are not currently shipped as part of the reference tooling.

## License

See [LICENSE.md](LICENSE.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## Support

See [SUPPORT.md](SUPPORT.md).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
