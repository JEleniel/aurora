# View Layout Requirements

While the current layout and routing produce a clean, grid-based layout, it is not always the best option for every view. In order to present more user-friendly diagrams, multiple layouts are required. This specification details the improved layout goals for each of the canonical views.

Unless otherwise specified, ordering is by `id` left to right, top to bottom.

## Global constraints and conventions

- **Input graphs:** Aurora guarantees any viable diagram can be decomposed into a DAG for layout purposes.
    - The source graph may only contain local cycles.
    - All edges are equal. The decomposition into a **layout DAG** is defined purely by the chosen depth assignment $d$:
        - An edge $(u \rightarrow v)$ is in the layout DAG iff $d(v) > d(u)$.
        - Otherwise, it is rendered as a **back-edge**.
- **Canonical view definitions:** Canonical view names and definitions live under `.github/aurora/**.*`.
- **Multi-root views:** Aurora views are multi-root.
    - Let $R$ be the set of roots for a view. Layout is performed per-root and then packed into a single diagram.
    - Root ordering defaults to `id` order.
- **Reachability invariant:** No view should ever have an unreachable node.
    - Nodes are selected by traversing from roots.
- **Shared nodes:** A node that is reachable from multiple parents is rendered once.
    - The node is owned by the lowest-`id` parent.
    - Edges from non-owning parents are still rendered (and may become back-edges under the chosen depth assignment).
- **Ordering default:** `id` ordering should naturally minimize crossings for most canonical views.
    - Crossing-minimization heuristics are permitted as a fine-tuning step.
    - `id` ordering is the tie-breaker for any heuristic.
- **Depth assignment:** Depth must be chosen to produce the best diagram.
    - Depth may differ per view (and per root-subgraph) based on diagram quality.
    - Prefer depth assignments that improve legibility and approach a target aspect ratio of **1.6:1** (W/H).
- **Scoring:** When choosing among candidate layouts (depth assignment, packing arrangement, and optional heuristic ordering), choose the layout with the best score.
    - Compute a candidate set by combining:
        - depth assignment strategies (shortest-path, longest-path, and hybrid)
        - packing strategies (single row, multiple rows)
        - ordering strategies (`id` only, and `id` plus a small fixed number of median/barycenter sweeps)
    - Score each candidate using:
        - $\Delta_{ar} = |\log((W/H) / 1.6)|$ (aspect ratio error)
        - $C$ crossings
        - $B$ bends (orthogonal turns)
        - $L$ total routed length
        - $S$ long-edge span penalty (sum over edges of $\max(0, \text{span} - s_0)$)
            - A reasonable default is $s_0 = 2$ ranks/columns.
    - A concrete default:
        - $\text{score} = 5\Delta_{ar} + 10C + 2B + 0.5L + 3S$
    - Tie-breakers (in order): fewer crossings, then fewer bends, then `id` ordering stability.
- **Edge routing (all layouts):** Edges must route orthogonally.
    - Edges must not lie on, touch, or cross the boundary of a node.
    - Use ports/attachment points on node sides with a fixed clearance margin.
    - Edges may share channels only under these rules:
        - Edges sharing an origin may be colinear until separation is required to reach distinct targets.
        - Edges sharing a target may merge and remain merged to the target.
        - Edges sharing both an origin and a target must not share a lane.
        - All other edges must be separate.
    - Where edge segments cross, render a **half-circle line jump**.
        - Prefer placing the jump on the **horizontal** segment at the crossing.
        - Otherwise, place it on the segment belonging to the edge whose **start point has the highest X coordinate** (tie-break by `id`).
    - Edges must not route outside the perimeter of the laid out nodes (treat that perimeter as the page margin).
        - If additional routing space is required (for example: dense back-edges), create it by increasing spacing between ranks/columns within the diagram.
    - Long edges (spanning many ranks/columns) should preferentially route along reserved internal highways (gutters between ranks/columns) and re-enter near the destination.
        - Use one or more internal highway lanes and allocate them deterministically.
        - Prefer bundling along highways until separation is required by the lane-sharing rules.

## Vertical Tree Layout

- **Goal:** Minimize edge crossings while enforcing strict parent→child layering.
- **Complexity:**
    - Traversal: `O(V + E)`
    - Crossing minimization (heuristic passes): typically `O(kE)`.
- **Encodes:** hierarchy, refinement, authority chain.
- **Group by**: Root

### Algorithm (layered / Sugiyama-style simplified)

1. Input: Directed graph plus roots $R$.
2. Generate candidates by combining:
    - depth assignment strategies (shortest-path, longest-path, hybrid)
    - ordering strategies (`id` only, and `id` plus a small fixed number of median/barycenter sweeps)
    - packing strategies (single row, multiple rows)
3. For each candidate:
    1. Produce the **layout DAG** from the candidate depth assignment $d$ (see global constraints).
    2. For each root $r \in R$:
        1. Group nodes by depth → these become horizontal ranks.
        2. Order nodes within each rank (default `id`, optional sweep fine-tuning; `id` tie-break).
        3. Assign Y by rank index; assign X by within-rank order with uniform spacing.
    3. Pack root-subgraphs into the overall diagram using the candidate packing strategy.
    4. Route layout-DAG edges monotonically downward using orthogonal segments.
    5. Route back-edges using internal gutters between ranks/columns (never outside the diagram perimeter).
    6. Compute the score.
4. Select the best-scoring candidate.

### When to use

- Use when the view is intended to communicate hierarchy, refinement, or governance.
- Prefer when most edges naturally align with a single parent→child notion.

### Alternatives

- If the per-root layout DAG is a tree/forest: use a tidy tree layout (Reingold–Tilford / Buchheim) for cleaner symmetry and fewer bends.
- If the per-root layout DAG is a dense DAG with many long edges: use a full Sugiyama pipeline (network-simplex layering + Brandes–Köpf coordinate assignment) to reduce edge span and improve compactness.

### Diagrams

- Compliance Governance
- Requirements
- Security
- Traceability

## Horizontal Tree Layout

- **Goal:** Preserve layered structure but rotate the semantic axis to emphasize progression.
- **Complexity:** Same as vertical layout; geometric transform is `O(V)`.
- **Encodes:** temporal flow, lifecycle, or causality without changing graph semantics.
- **Group by**: Root
- **Loops**: Under the spine (increasing Y)

### Algorithm (axis-transposed layered layout)

1. Input: Directed graph plus roots $R$.
2. Generate candidates by combining:
    - depth assignment strategies (shortest-path, longest-path, hybrid)
    - ordering strategies (`id` only, and `id` plus a small fixed number of median/barycenter sweeps)
    - packing strategies (single row, multiple rows)
3. For each candidate:
    1. Produce the **layout DAG** from the candidate depth assignment $d$ (see global constraints).
    2. For each root $r \in R$:
        1. Treat **depth as X instead of Y**.
        2. Within each depth column, order nodes (default `id`, optional sweep fine-tuning; `id` tie-break).
        3. Assign X by depth; assign Y by intra-column ordering.
    3. Pack root-subgraphs into the overall diagram using the candidate packing strategy.
    4. Route layout-DAG edges monotonically left-to-right using orthogonal segments.
    5. Route back-edges **under the spine** within internal gutters:
        - The **spine** is the main band of ranks/rows containing the layout-DAG nodes.
        - **Under** means increasing Y in this horizontal view.
        - Back-edges run in reserved gutter lanes below the spine and between node rows, then rejoin at the destination column.
        - If additional back-edge lanes are required, increase vertical spacing within the diagram rather than routing outside the perimeter.
    6. Apply compaction to equalize column spacing, without violating node clearance or internal gutter reservations.
    7. Compute the score.
4. Select the best-scoring candidate.

### When to use

- Use when the view is intended to communicate progression, lifecycle, causality, or flow.
- Prefer when the majority of edges move forward in depth.

### Alternatives

- If the view is dominated by cycles (for example: state-machine-like): consider SCC condensation first, then lay out SCCs left-to-right and lay out SCC internals with a circular/ring strategy.
- If crossing reduction is more important than strict layering: consider a constrained force-directed layout with fixed root ordering.

### Diagrams

- Deployment
- Process
- Landscape
- State Machine

## Radial Subtree Layout, Roots Centered

- **Goal:** Eliminate directional bias by mapping depth to radius and siblings to angle.
- **Complexity:**
    - BFS + subtree sizing: `O(V + E)`
    - Placement: `O(V)`.
- **Encodes:** peer domains around a shared center; avoids implying sequence or dominance.
- **Group by**: Root, then subtree

### Algorithm (radial BFS embedding)

1. Input: Directed graph plus roots $R$.
2. Generate candidates by combining:
    - depth assignment strategies (shortest-path, longest-path, hybrid)
    - root-ring sizing/sector sizing strategies
3. For each candidate:
    1. Produce the **layout DAG** from the candidate depth assignment $d$ (see global constraints).
    2. Place roots as centers arranged in a circle-ish ring around the origin.
        - Root ordering defaults to `id` order.
        - Allocate each root a non-overlapping angular sector sized proportional to its subtree weight.
    3. For each root $r \in R$:
        1. Treat $r$ as the center of its own radial layout.
        2. Ensure the **first-order descendants** of $r$ form the base of visually separated subtrees by reserving distinct angular sub-sectors for each child of $r$.
        3. Map each depth to radius: $\text{radius} = k \cdot d(v)$ with constant band spacing.
        4. Place nodes by polar coordinate within the root's sector (subtree-weighted interval partitioning).
    4. Route edges orthogonally (not as arcs), using clearance rings between bands.
        - Cross-links within a root sector should prefer outer clearance bands to avoid clutter at smaller radii.
        - Cross-links between root sectors route through reserved internal clearance bands and shared buses between sectors, without leaving the perimeter of the laid out nodes.
    5. Optional relaxation step to equalize angular gaps while preserving sector separation.
    6. Compute the score.
4. Select the best-scoring candidate.

### When to use

- Use when the view is intended to communicate peer domains without implying a primary direction.
- Prefer when each root naturally owns a visually separated subtree.

### Alternatives

- If there are many cross-links: consider concentric-layer (radial Sugiyama) layouts for better crossing control.
- If strict orthogonal routing is causing excessive bends: consider allowing arcs for cross-links only while keeping primary tree edges orthogonal.

### Diagrams

- Component
- Entire Model (subgroup by first rank)
- Context
