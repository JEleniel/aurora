# Tools

Aurora’s reference tooling is currently focused on two command-line utilities:

- **`aurora_cli`**: validates models and renders Markdown, views (SVG), and compact exports.
- **`svg_prep`**: prepares the SVG template and reference icon/shape assets used by rendering.

## When to use which

| You want to… | Use | Output |
|---|---|---|
| Check graph invariants and canonical consistency | `aurora_cli validate` | Diagnostics (warnings + errors) |
| Generate card Markdown for reading/review | `aurora_cli render-aurora` | Markdown files per model/card |
| Generate diagram views from view definitions | `aurora_cli render-views` | SVG diagrams per view |
| Generate everything in one run | `aurora_cli render-all` | Markdown + SVG |
| Generate machine-friendly snapshots | `aurora_cli compact` | `Compact.json` per mission |
| Optimize icon/shape masters | `svg_prep optimize-icons` / `svg_prep optimize-shapes` | Optimized SVG + proof sheets |
| Merge defs into `SVGTemplate.svg` + write `.svgz` | `svg_prep` (no subcommand) | Updated template + proof sheets |

## Tool pages

- **[Aurora CLI](./Aurora%20CLI.md)**
- **[SVG Prep](./SVG%20Prep.md)**
