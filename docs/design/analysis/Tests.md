# Test Inventory

## `tools/aurora_shared/src/render/layout/graphviz.rs`

- `graphviz_input_emits_plain_tree_nodes_without_helper_root`: verifies Graphviz DOT emission for supported tree layouts writes ordinary node declarations and does not inject a helper root.
- `graphviz_input_uses_expected_spacing_and_rankdir`: verifies Graphviz DOT generation uses Aurora spacing and the expected `rankdir` mapping.
- `normalize_edge_endpoint_strips_ports`: verifies plain-output endpoint parsing removes Graphviz port suffixes.
- `parse_plain_output_scales_nodes_and_routes`: verifies Graphviz `plain` coordinates scale from inches into Aurora's 300 ppi SVG pixel space so route endpoints align with rendered symbols.
- `parse_plain_output_merges_wrapped_edge_records`: verifies wrapped Graphviz `plain` edge records are reassembled before parsing so continuation lines like `solid black` do not fail layout.

## `tools/aurora_shared/src/render/layout/graphviz_api.rs`

- `scoring_detects_crossing_routes`: verifies the Graphviz-family scorer penalizes crossing edge routes.
- `scoring_detects_bends`: verifies the scorer counts bends on multi-segment routes.
- `aspect_preference_prefers_ratio_below_target`: verifies best-family selection prefers the layout whose width:height ratio is closest to but still under $1.6$.
- `best_family_is_deterministic`: verifies best-family selection stays deterministic for fixed model input.

## `tools/aurora_shared/src/render/layout/api_tests.rs`

- `layout_model_positions_nodes_by_rank`: verifies top-down Graphviz layouts keep roots above descendants and emit explicit routes.
- `layout_model_is_deterministic`: verifies the default shared layout remains deterministic.
- `horizontal_layout_progresses_across_x_axis`: verifies left-right tree layouts progress along the X axis.

## `tools/aurora_shared/src/render/svg.rs`

- `render_includes_screen_background_and_no_text_stroke`: verifies shared SVG rendering still produces the expected Aurora layers and background when given a Graphviz-capable `Layout`.
- `materialize_layout_routes_falls_back_when_graphviz_route_is_missing`: verifies SVG rendering falls back to a simple center-to-center route when Graphviz omits a per-edge route entry.

## `tools/aurora_shared/src/render/svg/node.rs`

- `position_nodes_does_not_insert_empty_columns`: verifies grid layouts keep the focused-graph positioning contract.

## `tools/aurora_shared/src/registry/view_registry.rs`

- `parsed_view_definition_preserves_layout`: verifies view definitions preserve the configured per-view layout family from `Aurora.viewconfiguration.json`.

## `tools/aurora_shared/src/render.rs`

- `configured_view_layout_preserves_explicit_setting`: verifies configured views preserve the layout family loaded from `Aurora.viewconfiguration.json`.
- `configured_view_layout_can_be_absent`: verifies views may omit the layout field, allowing renderer best-family selection to apply.

## `tools/aurora_cli/tests/cli_acceptance.rs`

- `render_views_writes_requested_dot_files`: verifies `render-views --dot-output <DIR>` creates the requested folder structure and writes the Graphviz DOT used for the rendered SVG view.

### Gaps

- No golden SVG snapshot currently asserts exact Graphviz view output for canonical models.
- No dedicated automated test currently simulates Graphviz binaries being absent; runtime behavior is covered by structured error handling instead.
