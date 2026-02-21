# Layout and Edge Routing Requirements

## Overview

The goal of this set of changes is to replace the current ELK-based layout in `aurora_shared`, which produces very wide, shallow graphs, with something more robust that results in tighter layouts and more readable final diagrams. This involves a complete replacement of the layout and edge routing algorithms.

Terminology used in this document:

- A "node" is a graph node (an element of the rendered dependency graph).
- A "symbol" is the fixed-size graphic element drawn at the node's render location.
- The "template" is the SVG template used for rendering symbols and edges.

## Fundamental Changes

- Graphs with only one node should not be rendered.
- Graphs whose rendered graph contains only root nodes and no outgoing edges should not be rendered.
    - Note: This refers to the rendered graph after normalization (SCC collapse + transit element handling). Nodes may have edges in the underlying model that do not survive into the rendered graph.
- Adjust constants:
    - WRAP_COLS = 65.
    - DESCRIPTION_FONT_SIZE_PX = TEMPLATE_DEFAULT_FONT_SIZE_PX
    - ARROW_SIZE_PX = 40
    - These constants already exist in the code. The intent is that description text uses the template's default font size (it is currently larger), and that all sizes remain consistent with a 16px base grid.
- Remove the edge mask stroke; it's not working properly.

## Layout Algorithm

### 1. Graph Normalization

- Collapse all strongly connected components (SCCs) into single meta-nodes before layout.
- The resulting structure must be a true DAG to guarantee termination and stable ranking.
- Nodes with exactly one incoming and one outgoing edge must be treated as transit elements and excluded from placement.
    - Transit elements are restored during rendering by expanding the corresponding rendered edge into a sequence that includes the transit nodes.
    - Transit detection is performed on the normalized graph (after SCC collapse), and transit nodes that are part of an SCC are not eligible.

### 2. Spine-Based Layout

- Identify a deterministic primary path ("spine") using a longest-path calculation with stable tie-breaking.
    - When multiple longest paths exist, break ties using node IDs in lexicographic order.
- Place spine nodes in a single vertical column with fixed rank spacing.
- All non-spine structures are treated as side branches attached to the nearest spine ancestor.

### 3. Width Control

- Apply Coffman–Graham layering to non-spine nodes with a fixed maximum column count `k`.
- Choose `k` to target a printed-page-friendly aspect ratio by keeping the number of columns `k` approximately $0.75:1$ relative to the number of rows `r`.
    - `k` and `r` are counts of grid columns/rows used by the laid-out graph.
    - A simple deterministic starting point is to estimate `k` from the number of placed nodes $n$ using $k = \lceil\sqrt{0.75n}\rceil$, then compute $r = \lceil n / k \rceil$.
- Node ordering must be stable and deterministic.
    - Prefer ordering that minimizes edge crossings, with lexicographic node ID tie-breaking.
- The layout must never expand beyond the configured width bound.

### 4. Coordinate Assignment

- Use an integer grid model (row, column) rather than geometric optimization.
- Node size is fixed and applied only during rendering.
- Output coordinates must be identical across runs for identical input.

Grid model (render-time geometry):

- Symbol size is fixed at 720x450.
- Column pitch is 1080px.
- Row pitch is 810px.
- Pixel calculations for rendering are derived from the fixed symbol size to preserve consistent visual proportions.

## Edge Routing Algorithm

### 1. Channelized Edge Model

- Treat edges as routed “nets” between node columns, not independent splines.
- The horizontal space between adjacent node columns is a routing channel with a fixed number of lanes. The default is 16, based on the symbol spacing.
- Lanes are discrete, parallel tracks reserved for the full span of a net.

Lane geometry:

- The routing channel width is 160px.
- With 16 lanes, lane pitch is 10px.
- Stroke width is 2px, centered in the lane midpoint.
    - With a 10px lane, the stroke is centered 5px from the lane boundary; the 2px stroke occupies the center of the lane (visually aligning to the middle two pixels).
- Jump radius is 4px and aligned to the lane width.

### 2. Net Formation Rules

- All edges sharing the same source form a bundle and may share a lane until their divergence point.
    - The first edge should originate at or near the center of the relevant symbol edge.
    - Subsequent edges should be assigned to lanes one lane left or right of existing edges (deterministically).
    - Divergence occurs at the first required change in direction that differs between edges; for example, when one edge must turn to reach its target but the shared net does not.
- All edges sharing the same target may merge into a shared lane as early as possible.
- Edges with identical source and target must always occupy separate lanes.
- All other edges must never share a lane.

### 3. Lane Assignment

- Model each net as a horizontal interval from source boundary to target boundary.
- Assign lanes using a deterministic left-edge interval-coloring algorithm.
- A lane may be reused only when intervals do not overlap or sharing is explicitly allowed by bundling rules.
    - When ordering intervals for assignment, break ties deterministically using lexicographic ordering of (source node ID, target node ID), and then a stable per-edge index to distinguish multi-edges.

### 4. Edge Geometry

- Routes must be orthogonal with at most one vertical adjustment per column transition.
- Diagonal segments are allowed only as a simplification of a single jog and must not introduce ambiguity.
- Split points for bundled edges occur as late as possible; merge points occur as early as possible.

### 5. Crossing Treatment

- Layout should minimize crossings structurally; routing must not introduce new ones.
- When crossings remain, render a jump (semicircle) on one edge.
- Prefer jumps on horizontal segments; otherwise assign the jump to the edge whose source is farther right.
    - If still tied, use lexicographic ordering on (source node ID, target node ID) as a final tie-breaker.
- Jump placement must be deterministic.

### 6. Boundary Constraints

- Routing must remain inside the horizontal extent defined by the outermost node edges.
- No edge segment may extend into page margins or outside the symbol envelope.
- Lanes terminate exactly at the farthest participating connection.

## Rendering Changes

- The horizontal and vertical routing gap between symbols should be 320px.
- The outer 80px of each gap are reserved clear space to allow clean, orthogonal approaches in the final segment.
- The remaining 160px is divided into 16 lanes for edges, aligning with the routing.
- The remaining spacing between symbols (outside the 320px routing gap) is reserved as additional visual padding derived from symbol-relative pixel calculations.
- The margin around the page should be increased to 150px (1/2" at the drawing's default scale of 300ppi).
