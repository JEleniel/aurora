# Views: Orthogonal Edge Routing with Bundling and Clearance

## 1. Purpose

The current view edge routing is not working; it demonstrates a number of common edge cases and artifacts. The purpose of this is to define an orthogonal (Manhattan) routing algorithm for directed graphs that:

- Maintains minimum spacing `s` between unrelated edges.
- Treats edges sharing a node as unrelated unless explicitly bundled.
- Merges edges from the same source until divergence is required.
- Merges edges entering the same target as early as possible.
- Keeps all routes at least `p` pixels away from non‑incident nodes.
- Optionally enforces a single exit port (bottom center) and single entry port (top center) per node.

This is achieved using obstacle‑avoiding rectilinear Steiner arborescences plus corridor track assignment.

---

## 2. Definitions

| Term                | Meaning                                                       |
| ------------------- | ------------------------------------------------------------- |
| Node obstacle       | Node bounding box expanded by clearance `p`.                  |
| Visibility corridor | Maximal horizontal or vertical free space between obstacles.  |
| Steiner point       | Artificial junction where bundled routes split or merge.      |
| Arborescence        | Directed rectilinear Steiner tree rooted at a source or sink. |
| Track               | A 1‑D lane inside a corridor assigned to one routed bundle.   |

---

## 3. Inputs

```
Nodes: rectangles with positions and sizes
Edges: (source, target) pairs
p: node clearance distance (pixels)
s: minimum spacing between unrelated edges (pixels)
```

Optional constraint:

```
single_port = true | false
```

---

## 4. Output

A set of orthogonal polylines consisting only of horizontal/vertical segments with explicit junctions where bundles split or merge.

---

## 5. Algorithm Overview

The routing pipeline has four independent phases:

1. Inflate nodes into routing obstacles.
2. Construct a rectilinear visibility graph.
3. Route bundled Steiner arborescences per source/target.
4. Assign tracks inside corridors to enforce spacing.

No per‑edge pathfinding is performed.

---

## 6. Phase 1 — Obstacle Inflation

Each node is converted into a forbidden rectangle using a Minkowski sum.

```
obstacle(node) = expand(node.bounds, p)
```

All routing must remain outside these obstacles.

This guarantees clearance requirement automatically.

---

## 7. Phase 2 — Rectilinear Visibility Graph

Construct a geometry graph representing legal orthogonal motion.

### Steps

1. Project horizontal and vertical rays from every obstacle edge.
2. Stop rays when they hit another obstacle.
3. Each maximal free segment becomes a visibility edge.
4. Intersections become vertices.

This produces a sparse orthogonal navigation graph without imposing a grid.

---

## 8. Phase 3 — Steiner Arborescence Routing

Edges are routed in bundled trees, not individually.

### 8.1 Group by Source

For each source node `S`:

```
T = {targets reachable from S}
```

Create terminals:

```
root = bottom_center(S)
leaves = entry_ports(T)
```

### 8.2 Compute Rectilinear Steiner Tree

Use a heuristic Steiner solver to connect all terminals simultaneously.

Recommended heuristic:

- Batched Iterated 1‑Steiner (BI1S)
- FLUTE‑style refinement

Cost function:

```
cost = α * total_length
     + β * bend_count
     + γ * congestion_penalty
```

This naturally produces shared trunks that later branch.

### 8.3 Enforce Early Merging at Targets

For each target `D`, create an attracting vertical collector segment above it.

Incoming trees are biased toward this segment using congestion weighting.

This forces merges before final descent.

---

## 9. Phase 4 — Corridor Track Assignment

Within each visibility corridor, bundles must be separated by at least `s`.

This is solved as interval graph coloring.

### 9.1 Build Usage Intervals

Each routed segment occupying a corridor defines an interval:

```
interval = (start_position, end_position)
```

### 9.2 Assign Tracks

Greedy coloring is sufficient because interval graphs are perfect.

```
sort intervals by start
for each interval:
    assign lowest track not violating spacing s
```

Track coordinate:

```
position = base + track_index * s
```

Bundles share a track until a Steiner split.

---

## 10. Optional Single‑Port Discipline

If enabled, normalize edges before routing:

```
all outgoing edges -> virtual port at bottom center
all incoming edges -> virtual port at top center
```

The Steiner solver then guarantees a single trunk per node.

---

## 11. Geometry Post‑Processing

After routing:

1. Convert Steiner nodes into explicit junction objects.
2. Collapse collinear segments.
3. Snap coordinates to integer pixels.
4. Extend final segment orthogonally into node boundary.

---

## 12. Data Structures

Recommended representations:

```
Obstacle: axis-aligned rectangle
VisibilityGraph: adjacency list of orthogonal segments
SteinerNode: {x, y, degree}
Corridor: {axis, span, capacity}
TrackAssignment: integer index per segment
```

---

## 13. Complexity Characteristics

| Phase              | Complexity              |
| ------------------ | ----------------------- |
| Obstacle inflation | O(n)                    |
| Visibility graph   | O(n log n)              |
| Steiner routing    | Near-linear (heuristic) |
| Track assignment   | O(k log k) per corridor |

Scales to large graphs because routing is tree‑based rather than edge‑based.

---

## 14. Implementation Constraints

Do NOT:

- Use grid A\* search.
- Route edges independently.
- Allow arbitrary diagonal visibility.

These destroy bundling guarantees and spacing invariants.

---

## 15. Validation Checklist

Implementation is correct if:

- No segment enters an inflated obstacle.
- Parallel unrelated segments are ≥ `s` apart.
- Edges from a source share a trunk before diverging.
- Incoming edges visibly merge before reaching a node.
- All final approaches are orthogonal.

---

## 16. Minimal Pseudocode Skeleton

```
build_obstacles(nodes, p)
V = build_visibility_graph(obstacles)

for each source S:
    terminals = collect_targets(S)
    tree = steiner_route(V, S, terminals)
    embed(tree)

corridors = extract_corridors(V)
assign_tracks(corridors, s)

polylines = materialize_geometry()
return polylines
```

---

## 17. Rationale

Transforming edge routing into Steiner tree embedding converts a combinatorial pairwise conflict problem into a geometric flow problem with local lane assignment, which is tractable and stable for interactive layouts.
