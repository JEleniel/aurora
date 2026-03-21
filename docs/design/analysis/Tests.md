# Test Inventory

## `tools/aurora_shared/src/render/layout/graphviz.rs`

- `helper_root_is_used_for_multi_root_radial_layouts`: verifies multi-root radial layouts inject a deterministic helper root for Graphviz.
- `graphviz_input_uses_expected_spacing_and_rankdir`: verifies Graphviz DOT generation uses Aurora spacing and the expected `rankdir` mapping.
- `normalize_edge_endpoint_strips_ports`: verifies plain-output endpoint parsing removes Graphviz port suffixes.
- `parse_plain_output_scales_nodes_and_routes`: verifies Graphviz `plain` coordinates scale from inches into Aurora's SVG pixel space.
- `parse_plain_output_merges_wrapped_edge_records`: verifies wrapped Graphviz `plain` edge records are reassembled before parsing so continuation lines like `solid black` do not fail layout.

## `tools/aurora_shared/src/render/layout/graphviz_api.rs`

- `scoring_detects_crossing_routes`: verifies the Graphviz-family scorer penalizes crossing edge routes.
- `scoring_detects_bends`: verifies the scorer counts bends on multi-segment routes.
- `best_family_is_deterministic`: verifies best-family selection stays deterministic for fixed model input.

## `tools/aurora_shared/src/render/layout/api_tests.rs`

- `layout_model_positions_nodes_by_rank`: verifies top-down Graphviz layouts keep roots above descendants and emit explicit routes.
- `layout_model_is_deterministic`: verifies the default shared layout remains deterministic.
- `horizontal_layout_progresses_across_x_axis`: verifies left-right tree layouts progress along the X axis.
- `radial_layout_spreads_children_around_root`: verifies radial layouts place distinct descendants at distinct coordinates.
- `radial_layout_places_single_root_at_center`: verifies single-root radial layouts center the root.
- `radial_layout_places_multi_roots_north_then_equal_angles`: verifies multi-root radial layouts keep roots on a consistent ring.
- `radial_layout_keeps_root_clusters_separate`: verifies radial cluster groups stay separated.

## `tools/aurora_shared/src/render/svg.rs`

- `render_includes_screen_background_and_no_text_stroke`: verifies shared SVG rendering still produces the expected Aurora layers and background when given a Graphviz-capable `Layout`.
- `materialize_layout_routes_falls_back_when_graphviz_route_is_missing`: verifies SVG rendering falls back to a simple center-to-center route when Graphviz omits a per-edge route entry.

## `tools/aurora_shared/src/render/svg/node.rs`

- `position_nodes_does_not_insert_empty_columns`: verifies grid layouts keep the focused-graph positioning contract.
- `position_nodes_uses_compact_pitch_for_radial_layouts`: verifies legacy grid rendering still uses compact radial spacing for focused graphs.

## `tools/aurora_shared/src/registry/view_registry.rs`

- `parsed_view_definition_preserves_layout`: verifies view definitions preserve the configured per-view layout family from `Aurora.viewconfiguration.json`.

## `tools/aurora_shared/src/render.rs`

- `configured_view_layout_preserves_explicit_setting`: verifies configured views preserve the layout family loaded from `Aurora.viewconfiguration.json`.
- `configured_view_layout_can_be_absent`: verifies views may omit the layout field, allowing renderer best-family selection to apply.

### Gaps

- No golden SVG snapshot currently asserts exact Graphviz view output for canonical models.
- No dedicated automated test currently simulates Graphviz binaries being absent; runtime behavior is covered by structured error handling instead.
