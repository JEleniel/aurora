# "svg_prep" Tool Requirements

In `tools/svg_prep/` create a Rust project named "svg_prep" that is part of the workspace, and that performs the following functions:

## Create the Reference Icons File

1. Create an empty SVG file at `assets/references/Icons.svg` (output file) or a command line specified output location. It should include a `<defs></defs>` section.
2. Read each SVG file in `assets/icons/` (in case-insensitive alphabetical filename order) or a command line specified input location and extract all elements except `svg`, `defs`, `metadata`, `title`, `desc`, and `style` entries (when present).
3. Strip any `id` properties from the extracted elements.
4. Wrap the extracted elements in a `g` (group) with the `id` equal to `i-` plus the basename of the source file, e.g., `<g id="i-alarm">...</g>`.
5. Append the newly created group to the `defs` section of the output file.
6. Continue until all input files have been processed.
7. In the normal section of the output file (after `</defs>` and before the closing `</svg>` tag), create a grid of `<use ...>` entries to form a visual proof sheet of the defined icons.
8. The proof sheet should contain icons only (no labels), use no padding, and use a uniform cell size. If icon sizes differ, use the largest icon width and largest icon height as the cell size for all entries.

## Create the "SVGTemplate.svg" File

1. From the `Icons.svg` file created in the previous phase, extract the contents of the `defs` section.
2. From `assets/references/Shapes.svg` or a command line specified SVG file extract the contents of the `defs` section.
3. Combine both extracted sections and _replace_ the entire `defs` section of the `assets/references/SVGTemplate.svg` or a command line specified SVG template file.
4. Sort the entire combined `defs` section by `id` (case-insensitive) so the output is deterministic.

## Command-Line Options

Apply sane defaults so the tool works without any options, and allow explicit options so only the required paths can be overridden:

1. `--icons-in` (default: `assets/icons/`)
2. `--icons-out` (default: `assets/references/Icons.svg`)
3. `--shapes` (default: `assets/references/Shapes.svg`)
4. `--template` (default: `assets/references/SVGTemplate.svg`)

Assume icon filenames match `[a-z]*\.svg`.

**The final SVGTemplate should look like the following (truncated) example**:

```svg
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="720" height="450" viewBox="0 0 720 450" version="1.1" id="Shapes"
 xmlns="http://www.w3.org/2000/svg"
 xmlns:svg="http://www.w3.org/2000/svg" style="fill:none;stroke:#000000;font-size:16;line-height:1.2;font-family:'Noto Sans', Arial, Helvetica, sans-serif;stroke-width:2;stroke-linecap:round;stroke-linejoin:round;">
 <style>
   path, rect, circle, ellipse, line, polyline, polygon, use { vector-effect: non-scaling-stroke; }
 </style>
 <defs>
	<g id="i-alarm">
		<path d="M32.04,123.67 L32.04,123.67 C30.65,122.97 29.98,121.35 30.49,119.87 L34.51,108.1 L42.38,113.41 L36.02,122.65 C35.14,123.93 33.44,124.37 32.04,123.67 Z" fill="#82AEC0"/>
		<!-- Content Truncated -->
		<path d="M26.13,16.33 C26.78,17.68 27.51,18.99 28.32,20.25 C29.31,21.79 30.46,23.51 30.04,25.29 C29.55,27.33 27.29,28.31 25.3,28.99 C24.28,29.34 23.27,29.69 22.25,30.04 C21.71,30.23 21.14,30.42 20.57,30.39 C19.67,30.35 18.86,29.78 18.35,29.05 C17.83,28.32 17.57,27.44 17.37,26.57 C16.62,23.22 16.77,19.67 17.8,16.39 C18.4,14.51 19.34,12.53 21.47,12.38 C24.06,12.2 25.16,14.31 26.13,16.33 Z" fill="#FF4638"/>
	</g>
	<!-- Content Truncated -->
	<g id="trapezoid">
   		<path d="m 1,449 L 40,1 H 680 L 719,449 z" />
  	</g>
 </defs>
</svg>
```
