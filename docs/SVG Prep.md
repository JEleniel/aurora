# SVG Prep

`svg_prep` prepares the SVG reference assets Aurora uses for rendering:

- Optimized **icons** (`assets/optimized/icons`)
- Optimized **shapes** (`assets/optimized/shapes`)
- Proof sheets for review (`assets/proofs/*.svg`)
- A merged **SVG template** with all icon + shape defs (`assets/templates/SVGTemplate.svg` and `.svgz`)

This tool is primarily for maintaining the **shared rendering template** and reference assets.

## Commands

| Command                    | What it does                                                                                                           |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `svg_prep`                 | Runs the default pipeline (optimize masters, generate proofs, merge defs into template, write `.svgz`, sync icon ids). |
| `svg_prep optimize-icons`  | Optimize master icons into Aurora’s normalized format and generate `Icons.svg`.                                        |
| `svg_prep optimize-shapes` | Optimize master shapes into Aurora’s normalized format and generate `Shapes.svg`.                                      |

## Default inputs and outputs

`svg_prep --help` documents the defaults:

- Master icon inputs: `assets/masters/icons/`
- Master shape inputs: `assets/masters/shapes/`
- Optimized icons: `assets/optimized/icons/`
- Optimized shapes: `assets/optimized/shapes/`
- Proof sheets:
    - `assets/proofs/Icons.svg`
    - `assets/proofs/Shapes.svg`
    - `assets/proofs/SVGTemplate.svg`
- Template input: `assets/masters/SVGTemplate.svg`
- Template outputs:
    - `assets/templates/SVGTemplate.svg`
    - `assets/templates/SVGTemplate.svgz`

## `optimize-icons`

Example:

```text
svg_prep optimize-icons
```

Sample output:

```text
2026-02-19T19:38:53.470467Z  INFO svg_prep starting
Optimize a set of source icons into Aurora's normalized icon format
```

Key options:

- `--input <DIR>` (default: `assets/masters/icons`)
- `--output <DIR>` (default: `assets/optimized/icons`)
- `--proof <FILE>` (default: `assets/proofs/Icons.svg`)

## `optimize-shapes`

Example:

```text
svg_prep optimize-shapes
```

Sample output:

```text
2026-02-19T19:38:55.472396Z  INFO svg_prep starting
Optimize a set of source shapes into Aurora's normalized shape format
```

Key options:

- `--input <DIR>` (default: `assets/masters/shapes`)
- `--output <DIR>` (default: `assets/optimized/shapes`)
- `--proof <FILE>` (default: `assets/proofs/Shapes.svg`)

## Full build (`svg_prep`)

Running `svg_prep` with no subcommand performs the full pipeline:

1. Optimize master icons (if `assets/masters/icons/` exists)
2. Optimize master shapes (if `assets/masters/shapes/` exists)
3. Generate proof sheets
4. Merge icon + shape defs into the template (`SVGTemplate.svg`)
5. Write a gzipped version (`SVGTemplate.svgz`)
6. Sync available icon ids based on template `<g id="i-...">` groups

### Master template safety

When `--template` points inside `assets/masters/`:

- The input is treated as **input-only**.
- The merged template is written to `assets/templates/` unless you set `--template-out`.

This prevents accidental overwrites of source masters.

### Proof-sheet note (blank template)

`assets/proofs/SVGTemplate.svg` is intended as a review artifact.

It may appear blank in many SVG viewers because it is primarily a `<defs>` container.

## Icon id synchronization

As part of the full build, `svg_prep` extracts all `<g id="i-<name>">` groups from the template `<defs>` and uses that list to rewrite the `available_icons` array in an Aurora view configuration file.

Resolution rules:

- If an `Aurora.viewconfiguration.json` file exists adjacent to the template path, it is updated.
- Otherwise, in this repository `svg_prep` looks for:
    - `.github/aurora/reference/Aurora.viewconfiguration.json`

This behavior is convenient for maintaining the repo’s canonical icon list, but it also means a full build can modify files outside `assets/`.

If you want to avoid updating configuration files, do not run the full build; use `optimize-icons` and `optimize-shapes` instead.

## See also

- **[Configuration Guide](./Configuration%20Guide.md)** for how `available_icons` relates to template `i-...` defs.
- **[Model Home Layout](./Model%20Home%20Layout.md)** for template expectations (`reference/SVGTemplate.svgz`).
