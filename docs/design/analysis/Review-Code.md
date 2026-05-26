# Code review findings

Scope: `tools/aurora_cli`, `tools/aurora_editor`, and `tools/aurora_shared`.

## Findings

**Note**: The user has added their findings to this review. The number and nature of the issues found has made it clear that not only are you unable to follow instructiions, but every line of code you write has to be audited. Addressing all of the issues found will likely require a significant rework of the entire codebase. In addition, all instances are not listed here because there are too many.

## Red Flagged Code

- One line functions (`tools/aurora_cli/src/cli.rs:90`, `tools/aurora_editor/src/cli.rs:21`, `tools/aurora_editor/src/main.rs:7`, `tools/aurora_editor/src/navigation_sidebar.rs:351-411`, `tools/aurora_editor/src/app_settings.rs:428-454`, `tools/aurora_editor/src/app_shell.rs:435-439`, `tools/aurora_editor/src/bottom_panel.rs:210-264`, `tools/aurora_editor/src/inspector_sidebar.rs:476-545`, `tools/aurora_editor/src/graph_view.rs:360,434`).
    - e.g., `parse_args` in `cli.rs`
- Manually mapped errors (`tools/aurora_shared/src/render/layout/graphviz.rs:129-137`, `tools/aurora_shared/src/model_home_session.rs:44-58`, `tools/aurora_shared/src/aurora/model/card_persistence.rs:95-111`, `tools/aurora_cli/src/upgrade.rs:307-327`, `tools/aurora_editor/src/secrets.rs:84-102`, `tools/aurora_shared/src/background.rs:46-62`, `tools/aurora_shared/src/render/layout/graphviz.rs:144-151`).
- Errors containing only String. These are a sign of violation of the typed errors, and useless functions (`tools/aurora_shared/src/background.rs:57-62`, `tools/aurora_editor/src/app_settings.rs:10-22`, `tools/aurora_editor/src/graph_view.rs:151-172`, `tools/aurora_cli/src/upgrade.rs:457-490`, `tools/aurora_editor/src/inspector_model.rs:40-69`, `tools/aurora_editor/src/settings.rs:169-180`).
- Violations of One Source of Truth. There is duplicate code throughout the codebase (`tools/aurora_cli/src/upgrade.rs:81-93`, `tools/aurora_shared/src/model_home_session.rs:22-80`, `tools/aurora_shared/src/model_index.rs:31-62`, `tools/aurora_shared/src/backup.rs:76-111`, `tools/aurora_editor/src/inspector_model.rs:265-297`, `tools/aurora_editor/src/session.rs:184-195`, `tools/aurora_shared/src/aurora.rs:82-175`, `tools/aurora_editor/src/config.rs:84`, `tools/aurora_shared/src/model_index.rs:389-407`).
    - For example, I found at least three places where you find the model home.
- Spawned threads without tracking the handle or having a clean teardown path (`tools/aurora_shared/src/background.rs:11-29`, `tools/aurora_shared/src/backup.rs:76-100`, `tools/aurora_shared/src/backup.rs:265-343`, `tools/aurora_shared/src/render.rs:221`, `tools/aurora_shared/src/backup.rs:90-117`).
- Multiple typed error types in a single module (`tools/aurora_shared/src/aurora/model/card.rs:1-446`, `tools/aurora_shared/src/aurora/auditlog.rs:1-273`, `tools/aurora_shared/src/aurora/model/edit_history.rs:1-123`, `tools/aurora_shared/src/aurora.rs:608-752`, `tools/aurora_editor/src/session.rs:198-209`, `tools/aurora_editor/src/settings.rs:169-180`, `tools/aurora_shared/src/model_index.rs:451-470`).

### God Modules and Functions

The following functions exceed the limit:

- 85 pub fn try_new_from_structs (`tools/aurora_shared/src/registry/card_registry.rs:26-96`).
- 87 fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> { (`tools/aurora_shared/src/model_home_session.rs:224-260`).
- **100** fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> { (`tools/aurora_editor/src/inspector_model.rs:265-310`).
- 59 fn make_model(model_home: &Path, description: &str, root_target: &str) -> Model { (`tools/aurora_shared/src/render/layout/test_support.rs:35-70`, `tools/aurora_shared/src/aurora/model/edit_history_tests.rs:38-80`).
- 59 fn build_graphviz_input(graph: &LayoutGraph, spec: EngineSpec, family: LayoutFamily) -> String { (`tools/aurora_shared/src/render/layout/graphviz.rs:77-124`).
- 71 pub fn write_markdown(&self, path: &Path) -> Result<(), ModelError> { (`tools/aurora_shared/src/aurora/model/model_write_support.rs:82-120`).
- 79 fn collapse_strongly_connected_components( (`tools/aurora_shared/src/render/layout/api.rs:147-229`).
- 73 fn remove_transit_nodes(graph: CollapsedGraph) -> Result<NormalizedGraph, RenderError> { (`tools/aurora_shared/src/render/layout/api.rs:280-365`).
- 72 fn assign_spine_and_branches( (`tools/aurora_shared/src/render/layout/api.rs:450-590`).
- 53 fn score_layout(graph: &NormalizedGraph, layout: &Layout) -> LayoutScore { (`tools/aurora_shared/src/render/layout/api.rs:941-1005`).
- 51 fn materialize_layout_routes( (`tools/aurora_shared/src/render/svg.rs:252-355`).
- 81 fn render_note_callouts( (`tools/aurora_shared/src/render/svg.rs:505-650`).
- 53 fn span_contains_referenced_id(span: &str, referenced_ids: &HashSet<String>) -> bool { (`tools/aurora_shared/src/render/svg.rs:1040-1085`).
- 59 pub fn get_compact(&self) -> Value { (`tools/aurora_shared/src/aurora/model/card.rs:229-286`, `tools/aurora_shared/src/aurora/model.rs:252-310`).
- **143** fn try_load_with_mode(path: &Path, load_mode: AuroraLoadMode) -> Result<Self, AuroraError> { (`tools/aurora_shared/src/aurora.rs:82-175`).
- **355** fn collect_lines( (`tools/aurora_shared/src/render/svg/edge_router.rs:232-302`).
- 58 fn synthetic*layout(routes: Vec<SyntheticRoute<'*>>) -> Layout { (`tools/aurora_shared/src/render/layout/graphviz_api.rs:380-420`, `tools/aurora_shared/src/render/layout/graphviz_api.rs:393-431`).
- **115** pub fn render_node( (`tools/aurora_shared/src/render/svg/node.rs:149-230`).

The following are far past the file-size limit:

585 tools/aurora_shared/src/aurora/model/model_tests.rs (`tools/aurora_shared/src/aurora/model/model_tests.rs`)
502 tools/aurora_shared/src/aurora/aurora_tests.rs (`tools/aurora_shared/src/aurora/aurora_tests.rs`)
771 tools/aurora_shared/src/model_index.rs (`tools/aurora_shared/src/model_index.rs`)
514 tools/aurora_shared/src/render/svg/edge_router_graph.rs (`tools/aurora_shared/src/render/svg/edge_router_graph.rs`)
616 tools/aurora_shared/src/render/svg/edge_router.rs (`tools/aurora_shared/src/render/svg/edge_router.rs`)
683 tools/aurora_shared/src/render/svg/node.rs (`tools/aurora_shared/src/render/svg/node.rs`)
1574 tools/aurora_shared/src/render/svg.rs (`tools/aurora_shared/src/render/svg.rs`)
701 tools/aurora_shared/src/render/layout/ordering.rs (`tools/aurora_shared/src/render/layout/ordering.rs`)
1140 tools/aurora_shared/src/render/layout/api.rs (`tools/aurora_shared/src/render/layout/api.rs`)
766 tools/aurora_shared/src/aurora.rs (`tools/aurora_shared/src/aurora.rs`)
682 tools/svg_prep/src/svg/tests.rs (`tools/svg_prep/src/svg/tests.rs`)
546 tools/svg_prep/src/svg/shapes.rs (`tools/svg_prep/src/svg/shapes.rs`)
649 tools/aurora_editor/src/inspector_sidebar.rs (`tools/aurora_editor/src/inspector_sidebar.rs`)
505 tools/aurora_editor/src/app_settings.rs (`tools/aurora_editor/src/app_settings.rs`)

### Use of `map_err`, `unwrap`, `expect`, `panic`

There are 468 instances of this violation spread across 65 files. This is every single source file in the repository. This is completely unaccpetable (`tools/aurora_shared/src/aurora.rs:139-175`, `tools/aurora_shared/src/backup.rs:132-200`, `tools/aurora_shared/src/model_index.rs:84-220`, `tools/aurora_shared/src/aurora/model/card.rs:56-229`).

### Improper Typing

- The `log` crate supports serde, so log levels should not be handled as strings (`tools/aurora_cli/src/cli.rs:31-39`, `tools/aurora_editor/src/cli.rs:13-17`, `tools/aurora_cli/src/lib.rs:162-163`).

## Inproper Error Handling

The following fail to use properly typed error enums and from constructs:

- `tools/aurora_editor/src/app_settings.rs` (`tools/aurora_editor/src/app_settings.rs:1-260`, `tools/aurora_editor/src/app_settings.rs:377-406`)
- `tools/aurora_shared/src/aurora/model/card.rs` (`tools/aurora_shared/src/aurora/model/card.rs:1-446`)
- `tools/aurora_editor/src/graph_view.rs` (`tools/aurora_editor/src/graph_view.rs:1-260`, `tools/aurora_editor/src/graph_view.rs:243-299`)
- `tools/aurora_shared/src/background.rs` (`tools/aurora_shared/src/background.rs:1-200`)
- `tools/aurora_shared/src/background.rs` (`tools/aurora_shared/src/background.rs:1-200`)
- `tools/aurora_shared/src/backup.rs` (`tools/aurora_shared/src/backup.rs:1-420`)
- `tools/aurora_shared/src/model_index.rs` (`tools/aurora_shared/src/model_index.rs:1-260`)

## Other Issues

- The upgrade and supporting functions should be in aurora_shared because it is also needed for the editor and MCP (`tools/aurora_cli/src/upgrade.rs:1-380`, `tools/aurora_cli/src/upgrade.rs:430-520`, `tools/aurora_editor/src/session.rs:1-160`, `tools/aurora_shared/src/backup.rs:1-200`, `tools/aurora_editor/src/app_settings.rs:1-60`).
- Hardcoded paths, filenames, and keys (`tools/aurora_cli/src/upgrade.rs:10-55`, `tools/aurora_cli/src/constants.rs:4-7`, `tools/aurora_shared/src/aurora.rs:93-155`, `tools/aurora_shared/src/model_home_session.rs:27-38`, `tools/aurora_editor/src/session.rs:160-189`, `tools/aurora_editor/src/config.rs:11`, `tools/aurora_cli/src/upgrade.rs:16-44`).
- Manual parsing and transformation of JSON. This is not forbidden, just not recommended. There are a number of libraries that make this safer (`tools/aurora_cli/src/upgrade.rs:150-380`, `tools/aurora_cli/src/upgrade.rs:430-520`, `tools/aurora_shared/src/aurora/model/card.rs:53-229`, `tools/aurora_shared/src/model_index.rs:190-220`, `tools/aurora_shared/src/aurora/auditlog.rs:31-70`, `tools/aurora_cli/src/upgrade.rs:199-380`).
- Test modules belong at the end of the file (`tools/aurora_shared/src/aurora.rs:9`, `tools/aurora_shared/src/render/svg.rs:1077-1352`, `tools/aurora_shared/src/model_index.rs:477`).
- Verbose, missing, and uninformative doc comments. Comments should explain why, not what. They definitely should not repeat the code (`tools/aurora_shared/src/aurora.rs:7-35`, `tools/aurora_cli/src/cli.rs:7-90`, `tools/aurora_editor/src/app_settings.rs:1-60`, `tools/aurora_editor/src/inspector_sidebar.rs:1-60`, `tools/aurora_editor/src/navigation_sidebar.rs:1-60`).
- Using `fs::read_to_string` on files that may exceed the memory capacity of a string (`tools/aurora_shared/src/aurora.rs:139-154`, `tools/aurora_shared/src/model_home_session.rs:38`, `tools/aurora_shared/src/model_index.rs:212`, `tools/aurora_shared/src/aurora/model/card.rs:56-68`, `tools/aurora_shared/src/aurora/auditlog.rs:39`, `tools/aurora_cli/src/upgrade.rs:218-225`).
- Allocation of multiple variables where one will do (`try_load_with_mode` is a major example) (`tools/aurora_shared/src/aurora.rs:82-175`).
- There is significant bleed through of what should be clean module boundaries (`tools/aurora_editor/src/session.rs:1-160`, `tools/aurora_shared/src/model_home_session.rs:1-120`, `tools/aurora_shared/src/model_index.rs:1-220`, `tools/aurora_shared/src/backup.rs:1-200`).
- Typographical errors (`tools/aurora_shared/src/aurora.rs:33,315`).
- The validation functions traverse the cards multiple times without even building an in memory index or adjacency list (`tools/aurora_shared/src/aurora.rs:316-353`, `tools/aurora_shared/src/aurora/model.rs:145-210`, `tools/aurora_shared/src/aurora/model/card.rs:60-111`, `tools/aurora_shared/src/registry/card_registry.rs:26-96`).
- Tests directly in the source file (`tools/aurora_shared/src/aurora.rs:9,752`, `tools/aurora_shared/src/render/svg.rs:1077-1352`, `tools/aurora_shared/src/render/layout/graphviz.rs:312-360`).
- Different code paths for the editor and MCP. This is a violation of One Source of Truth (`tools/aurora_shared/src/backup.rs:12-58`, `tools/aurora_editor/src/session.rs:20-106`, `tools/aurora_shared/src/model_home_session.rs:22-80`, `tools/aurora_cli/src/upgrade.rs:1-380`, `tools/aurora_editor/src/inspector_sidebar.rs:1-60`).

### Dead Code and Compatability Shims

- `tools/aurora_cli/src/upgrade.rs` still carries a dead compatibility shim and an empty constant (`tools/aurora_cli/src/upgrade.rs:10-55`, `tools/aurora_cli/src/upgrade.rs:260-380`, `tools/aurora_cli/src/upgrade.rs:430-520`, `tools/aurora_cli/src/compact.rs:1-85`).
- `tools/aurora_cli/src/compact.rs` appears to be orphaned dead code (`tools/aurora_cli/src/compact.rs:1-85`).
