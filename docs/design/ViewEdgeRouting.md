# View Rendering: Orthogonal Edge Routing with Bundles and Clearance

## 1. Purpose

Define a deterministic orthogonal (Manhattan) edge-routing algorithm for Aurora view rendering that:

- Maintains minimum spacing `s` between unrelated edges.
- Treats edges sharing a node as unrelated unless explicitly bundled.
- Merges edges from the same source until divergence is required.
- Merges edges entering the same target as early as possible.
- Keeps all routes at least `p` pixels away from non-incident nodes.
- Optionally enforces a single exit port (bottom center) and single entry port (top center) per node.

The algorithm uses obstacle-avoiding rectilinear Steiner arborescences and deterministic corridor track assignment.

---

## 2. Definitions

| Term                  | Meaning                                                                                                         |
| --------------------- | --------------------------------------------------------------------------------------------------------------- |
| Node obstacle         | Node bounding box expanded by clearance `p`.                                                                    |
| Routing corridor      | Maximal horizontal or vertical free space between obstacles.                                                    |
| Steiner point         | Artificial junction where bundled routes split or merge.                                                        |
| Arborescence          | Directed rectilinear Steiner tree rooted at a source or sink.                                                   |
| Track                 | A 1-D lane inside a corridor assigned to one routed bundle.                                                     |
| Top-left tie-break    | Lower `y` wins; if equal `y`, lower `x` wins; if still equal, Unicode case-insensitive lexical `id` wins.       |
| Unicode lexical order | Compare by Unicode case-folded, NFC-normalized strings; if equal, compare original strings by code point order. |

---

## 3. Inputs

```text
Nodes: rectangles with positions and sizes
Edges: (source, target) pairs with stable edge IDs
p: node clearance distance (pixels)
s: minimum spacing between unrelated edges (pixels)
single_port: true | false
```

### 3.1 Default Parameters

Use these defaults unless explicitly overridden by the caller:

| Parameter               | Default | Notes                                                 |
| ----------------------- | ------- | ----------------------------------------------------- |
| `p`                     | `32` px | Clearance around non-incident nodes.                  |
| `s`                     | `16` px | Minimum lane spacing between unrelated segments.      |
| `α`                     | `1.0`   | Length weight in route cost.                          |
| `β`                     | `16.0`  | Bend penalty weight in route cost.                    |
| `γ`                     | `64.0`  | Congestion penalty weight in route cost.              |
| `collector_bias`        | `1.0`   | Weight pulling incoming edges into target collectors. |
| `max_refinement_passes` | `4`     | Fixed pass count to preserve determinism.             |

Route cost is:

```text
cost = α * total_length + β * bend_count + γ * congestion_penalty
```

---

## 4. Output (Mapped to SVG Edges)

Routing output MUST map directly to view SVG edge primitives.

Per rendered edge, emit a route object:

```text
Route {
  edge_id: String,
  source_id: String,
  target_id: String,
  points: [(x0,y0), (x1,y1), ...],   // orthogonal polyline, integer pixels
  junction_ids: [String],            // optional explicit split/merge nodes
  track_id: String,                  // corridor+track identity for diagnostics
}
```

SVG mapping contract:

- One route object maps to one `<path>` in the edge layer.
- `points` are serialized as `M/L` commands in order.
- Arrowheads are generated from the terminal segment at `target_id`.
- Optional line-jumps are rendered as overlay paths at crossing points.

---

## 5. Determinism and Tie-Break Rules

Routing must be deterministic under fixed input.

When two candidates have equal score, choose in this order:

1. Candidate whose decisive segment/junction is top-left-most (lowest `y`, then lowest `x`).
2. Candidate with Unicode case-insensitive lexical lowest `edge_id`.
3. Candidate with Unicode case-insensitive lexical lowest `(source_id, target_id)` pair.

Use the same tie-break rule for corridor ordering and track assignment.

Definition of decisive segment/junction:

- Serialize each candidate route as ordered orthogonal vertices.
- Compare candidates vertex-by-vertex from the first point.
- The first differing vertex is the decisive point used for top-left tie-break.

---

## 6. Pipeline Overview

The routing pipeline has four phases:

1. Inflate nodes into routing obstacles.
2. Construct a rectilinear visibility graph.
3. Route bundled Steiner arborescences per source/target group.
4. Assign deterministic tracks inside corridors to enforce spacing.

No independent per-edge shortest-path routing is allowed.

---

## 7. Phase 1 — Obstacle Inflation and Legal Crossings

Each node is converted into a forbidden rectangle using a Minkowski sum:

```text
obstacle(node) = expand(node.bounds, p)
```

Hard rule:

- Inflated obstacle regions are forbidden for all routes,
- except for the incident edge segments used to exit a source or enter a target.

In other words, padding `p` may only be crossed at incident entry/exit transitions.

---

## 8. Phase 2 — Rectilinear Visibility Graph

Construct a geometry graph representing legal orthogonal motion.

### Steps

1. Project horizontal and vertical rays from every obstacle edge.
2. Stop rays when they hit another obstacle.
3. Each maximal free segment becomes a visibility edge.
4. Intersections become vertices.

This yields a sparse orthogonal navigation graph without imposing a fixed grid.

---

## 9. Phase 3 — Bundled Steiner Routing

Edges are routed in bundled trees, not independently.

### 9.1 Group by source

For each source node `S`:

```text
T = {targets of rendered edges where source_id == S}
```

Create terminals:

```text
root = bottom_center(S)
leaves = entry_ports(T)
```

### 9.2 Compute rectilinear Steiner tree

Use a deterministic heuristic solver (for example BI1S / FLUTE-style refinement) with fixed pass count.

### 9.3 Enforce early target merging

For each target `D`, create an attracting vertical collector segment above it.

Incoming trees are biased toward this collector using `collector_bias`.

---

## 10. Phase 4 — Corridor Track Assignment

Within each routing corridor, bundles must be separated by at least `s`.

### 10.1 Build usage intervals

Each routed segment occupying a corridor defines:

```text
interval = (start_position, end_position)
```

### 10.2 Assign tracks

Use deterministic interval coloring:

```text
sort intervals by (start_position, top-left tie-break, edge_id)
for each interval:
    assign lowest track that preserves spacing s
```

Track coordinate:

```text
position = base + track_index * s
```

Bundles may share tracks until Steiner divergence.

---

## 11. Optional Single-Port Discipline

If `single_port = true`, normalize ports before routing:

```text
all outgoing edges -> virtual port at bottom center
all incoming edges -> virtual port at top center
```

This yields a single visible trunk per node side before divergence/merge.

---

## 12. Geometry Post-Processing and SVG Emission

After routing:

1. Convert Steiner nodes into explicit junction objects.
2. Collapse collinear segments.
3. Snap coordinates to integer pixels.
4. Extend final segment orthogonally into the incident node boundary.
5. Emit SVG edge `<path>` elements from ordered route points.

---

## 13. Data Structures

Recommended representations:

```text
Obstacle: axis-aligned rectangle
VisibilityGraph: adjacency list of orthogonal segments
SteinerNode: {x, y, degree}
Corridor: {axis, span, capacity}
TrackAssignment: integer index per segment
Route: {edge_id, source_id, target_id, points, junction_ids, track_id}
```

---

## 14. Implementation Constraints

Do NOT:

- Use grid A\* search.
- Route edges independently.
- Allow arbitrary diagonal visibility.

These violate bundling guarantees and spacing invariants.

---

## 15. Validation Checklist

Implementation is correct if:

- No non-incident segment enters an inflated obstacle.
- Padding `p` is crossed only at incident source exit / target entry.
- Parallel unrelated segments are `>= s` apart.
- Edges from a source share a trunk before diverging.
- Incoming edges merge before final target descent.
- All final approaches are orthogonal.
- Every output route object maps to exactly one SVG edge path.

---

## 16. Minimal Pseudocode Skeleton

```text
build_obstacles(nodes, p)
V = build_visibility_graph(obstacles)

for each source S in deterministic order:
    terminals = collect_targets(S)
    tree = steiner_route(V, S, terminals, α, β, γ)
    embed(tree)

corridors = extract_corridors(V)
assign_tracks(corridors, s)

routes = materialize_routes()
emit_svg_edge_paths(routes)
return routes
```

---

## 17. Worked Example

Given nodes:

```text
S at (100,100), A at (100,260), B at (260,260), T at (180,420)
```

Edges:

```text
e1: S -> A
e2: S -> B
e3: A -> T
e4: B -> T
```

With defaults (`p=32`, `s=16`):

1. Inflate all node boxes by 32 px.
2. Build visibility graph around inflated boxes.
3. Source bundle from `S` creates one trunk downward, then splits toward `A` and `B`.
4. Target collector above `T` pulls `e3` and `e4` together before final descent.
5. Track assignment keeps unrelated parallel segments 16 px apart.

Resulting route points (illustrative):

```text
e1: (140,180) -> (140,240) -> (120,240) -> (120,260)
e2: (140,180) -> (140,240) -> (280,240) -> (280,260)
e3: (120,300) -> (120,360) -> (180,360) -> (180,420)
e4: (280,300) -> (280,360) -> (180,360) -> (180,420)
```

Mapped SVG edge paths:

```text
e1 -> <path d="M 140 180 L 140 240 L 120 240 L 120 260" ... />
e2 -> <path d="M 140 180 L 140 240 L 280 240 L 280 260" ... />
e3 -> <path d="M 120 300 L 120 360 L 180 360 L 180 420" ... />
e4 -> <path d="M 280 300 L 280 360 L 180 360 L 180 420" ... />
```

This worked example is explanatory and non-normative.

---

## 18. Model Validity and Routability Contract

Within Aurora view rendering, unroutable edges are treated as an input validity failure.

Normative contract:

- A valid view graph must be routable under this specification.
- If routing cannot produce a legal path for any edge, the input model/layout state is invalid and validation must fail.
- On invalid model/layout input, abort view rendering for the model (do not emit partial outputs), because other views may also be incorrect.

---

## 19. Rationale

Steiner-bundle routing plus deterministic lane assignment turns pairwise edge conflicts into structured geometric flow decisions. This provides stable, legible view rendering output and direct SVG edge emission.
