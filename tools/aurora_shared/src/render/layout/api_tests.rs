use std::collections::HashSet;

use crate::render::render_error::RenderError;

use super::api::{layout_model, layout_model_best_family, layout_model_with_family};
use super::test_support::{make_card, make_model};
use super::types::{Layout, LayoutCoordinateSpace, LayoutFamily};

#[test]
fn layout_model_positions_nodes_by_rank() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement_one = make_card("REQ-001", "Requirement", &["REQ-002"]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");

	assert_eq!(layout.coordinate_space, LayoutCoordinateSpace::Pixels);
	assert_eq!(layout.nodes.len(), 3);
	assert_eq!(layout.edges.len(), 2);
	assert_eq!(layout.routes.len(), layout.edges.len());

	let root_node = layout.nodes.get("MIS-001").expect("root node missing");
	let first_requirement = layout.nodes.get("REQ-001").expect("node missing");
	let second_requirement = layout.nodes.get("REQ-002").expect("node missing");
	assert!(root_node.y < first_requirement.y);
	assert!(first_requirement.y <= second_requirement.y);
}

#[test]
fn layout_model_rejects_missing_roots() {
	let root = make_card("MIS-001", "Mission", &[]);
	let model = make_model(root, vec![]);

	let result = layout_model(&model, &[], &["REQ".to_string()]);
	assert!(matches!(result, Err(RenderError::MissingRoots)));
}

#[test]
fn layout_model_rejects_unknown_targets() {
	let root = make_card("MIS-001", "Mission", &["REQ-404"]);
	let model = make_model(root, vec![]);

	let result = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()]);

	assert!(matches!(result, Err(RenderError::UnknownTarget(_, _))));
}

#[test]
fn layout_model_is_deterministic() {
	let root = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002"]);
	let requirement_one = make_card("REQ-001", "Requirement", &[]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout_one = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");
	let layout_two = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");

	assert_eq!(sorted_nodes(&layout_one), sorted_nodes(&layout_two));
	assert_eq!(sorted_edges(&layout_one), sorted_edges(&layout_two));
}

#[test]
fn layout_model_succeeds_when_filtering_orphanizes_nodes() {
	let root = make_card("MIS-001", "Mission", &["CAP-001"]);
	let capability = make_card("CAP-001", "Capability", &["PRO-001"]);
	let process = make_card("PRO-001", "Process", &[]);
	let model = make_model(root, vec![capability, process]);

	let layout = layout_model(&model, &["MIS".to_string()], &["PRO".to_string()])
		.expect("layout should succeed");

	let mission = layout.nodes.get("MIS-001").expect("mission missing");
	assert_eq!(mission.y, 0);
	assert!(
		!layout.nodes.contains_key("PRO-001"),
		"expected isolated PRO-001 to be pruned from the filtered view"
	);
}

#[test]
fn layout_model_places_longest_path_on_spine_column() {
	let root = make_card("MIS-001", "Mission", &["CAP-001", "CAP-002"]);
	let cap_one = make_card("CAP-001", "Capability", &["REQ-001"]);
	let cap_two = make_card("CAP-002", "Capability", &["PRO-001"]);
	let process = make_card("PRO-001", "Process", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![cap_one, cap_two, process, requirement]);

	let layout = layout_model(
		&model,
		&["MIS".to_string()],
		&["CAP".to_string(), "PRO".to_string(), "REQ".to_string()],
	)
	.expect("layout should succeed");

	let mission = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let capability = layout.nodes.get("CAP-001").expect("CAP-001 missing");
	let requirement = layout.nodes.get("REQ-001").expect("REQ-001 missing");

	assert!(mission.y < capability.y);
	assert!(capability.y < requirement.y);
}

#[test]
fn layout_model_bounds_non_spine_width() {
	let root = make_card("MIS-001", "Mission", &["CAP-001", "CAP-002"]);
	let cap_one = make_card("CAP-001", "Capability", &["PRO-001"]);
	let cap_two = make_card("CAP-002", "Capability", &["PRO-001"]);
	let process = make_card("PRO-001", "Process", &[]);
	let model = make_model(root, vec![cap_one, cap_two, process]);

	let layout = layout_model(
		&model,
		&["MIS".to_string()],
		&["CAP".to_string(), "PRO".to_string()],
	)
	.expect("layout should succeed");

	let cap_one = layout.nodes.get("CAP-001").expect("CAP-001 missing");
	let cap_two = layout.nodes.get("CAP-002").expect("CAP-002 missing");
	assert_ne!(
		cap_one.x, cap_two.x,
		"sibling branches should occupy distinct columns"
	);
}

#[test]
fn layout_model_places_roots_above_descendants() {
	let root_one = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002"]);
	let root_two = make_card("MIS-002", "Mission", &["REQ-003"]);
	let req_one = make_card("REQ-001", "Requirement", &["REQ-004"]);
	let req_two = make_card("REQ-002", "Requirement", &[]);
	let req_three = make_card("REQ-003", "Requirement", &[]);
	let req_four = make_card("REQ-004", "Requirement", &[]);
	let model = make_model(
		root_one,
		vec![root_two, req_one, req_two, req_three, req_four],
	);

	let layout = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");

	let mission = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	for edge in &layout.edges {
		if edge.a == "MIS-001" {
			let target = layout.nodes.get(edge.b.as_str()).expect("target missing");
			assert!(
				target.y > mission.y,
				"root should be above descendants: {} -> {}",
				edge.a,
				edge.b
			);
		}
	}
}

#[test]
fn layout_model_does_not_overlap_node_coordinates() {
	let root_one = make_card("MIS-001", "Mission", &["REQ-001"]);
	let root_two = make_card("MIS-002", "Mission", &[]);
	let root_three = make_card("MIS-003", "Mission", &[]);
	let root_four = make_card("MIS-004", "Mission", &[]);
	let root_five = make_card("MIS-005", "Mission", &[]);
	let root_six = make_card("MIS-006", "Mission", &[]);
	let req_one = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(
		root_one,
		vec![
			root_two, root_three, root_four, root_five, root_six, req_one,
		],
	);

	let layout = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");

	let mut used = HashSet::new();
	for node in layout.nodes.values() {
		assert!(
			used.insert((node.x, node.y)),
			"node '{}' overlaps at ({}, {})",
			node.id,
			node.x,
			node.y
		);
	}
}

#[test]
fn horizontal_layout_progresses_across_x_axis() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement_one = make_card("REQ-001", "Requirement", &["REQ-002"]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["REQ".to_string()],
		LayoutFamily::TreeLeftRight,
	)
	.expect("horizontal layout should succeed");

	let mission = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let req = layout.nodes.get("REQ-002").expect("REQ-002 missing");
	assert!(
		req.x > mission.x,
		"expected horizontal progression on X axis"
	);
}

#[test]
fn radial_layout_spreads_children_around_root() {
	let root = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002", "REQ-003"]);
	let req_one = make_card("REQ-001", "Requirement", &[]);
	let req_two = make_card("REQ-002", "Requirement", &[]);
	let req_three = make_card("REQ-003", "Requirement", &[]);
	let model = make_model(root, vec![req_one, req_two, req_three]);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["REQ".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let root = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let req_one = layout.nodes.get("REQ-001").expect("REQ-001 missing");
	let req_two = layout.nodes.get("REQ-002").expect("REQ-002 missing");

	assert_ne!((root.x, root.y), (req_one.x, req_one.y));
	assert_ne!((req_one.x, req_one.y), (req_two.x, req_two.y));
}

#[test]
fn radial_layout_places_single_root_at_center() {
	let root = make_card("MIS-001", "Mission", &[]);
	let model = make_model(root, vec![]);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["MIS".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let root = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	assert_eq!((root.x, root.y), (0, 0));
}

#[test]
fn radial_layout_places_multi_roots_north_then_equal_angles() {
	let root_one = make_card("MIS-001", "Mission", &[]);
	let root_two = make_card("MIS-002", "Mission", &[]);
	let root_three = make_card("MIS-003", "Mission", &[]);
	let root_four = make_card("MIS-004", "Mission", &[]);
	let model = make_model(root_one, vec![root_two, root_three, root_four]);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["MIS".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let root_one = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let root_two = layout.nodes.get("MIS-002").expect("MIS-002 missing");
	let root_three = layout.nodes.get("MIS-003").expect("MIS-003 missing");
	let root_four = layout.nodes.get("MIS-004").expect("MIS-004 missing");

	let center_x = (root_one.x + root_two.x + root_three.x + root_four.x) as f32 / 4.0;
	let center_y = (root_one.y + root_two.y + root_three.y + root_four.y) as f32 / 4.0;
	let distances = [root_one, root_two, root_three, root_four].map(|node| {
		((node.x as f32 - center_x).powi(2) + (node.y as f32 - center_y).powi(2)).sqrt()
	});
	let min_distance = distances.into_iter().fold(f32::INFINITY, f32::min);
	let max_distance = distances.into_iter().fold(f32::NEG_INFINITY, f32::max);
	assert!(
		max_distance - min_distance < 250.0,
		"roots should stay on a consistent ring"
	);
}

#[test]
fn radial_layout_keeps_first_ring_compact() {
	let root = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002", "REQ-003"]);
	let req_one = make_card("REQ-001", "Requirement", &[]);
	let req_two = make_card("REQ-002", "Requirement", &[]);
	let req_three = make_card("REQ-003", "Requirement", &[]);
	let model = make_model(root, vec![req_one, req_two, req_three]);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["REQ".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let root = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	for node_id in ["REQ-001", "REQ-002", "REQ-003"] {
		let child = layout.nodes.get(node_id).expect("child missing");
		let dx = (child.x - root.x).abs();
		let dy = (child.y - root.y).abs();
		assert!(
			dx <= 4000 && dy <= 4000,
			"clustered tree child {node_id} should remain within the first radial band; got Δx={dx}, Δy={dy}"
		);
	}
}

#[test]
fn radial_layout_stacks_wide_first_level_into_multiple_tree_rows() {
	let links: Vec<String> = (1..=10).map(|index| format!("REQ-{:03}", index)).collect();
	let link_refs: Vec<&str> = links.iter().map(String::as_str).collect();
	let root = make_card("MIS-001", "Mission", link_refs.as_slice());
	let requirements = links
		.iter()
		.map(|id| make_card(id.as_str(), "Requirement", &[]))
		.collect::<Vec<_>>();
	let model = make_model(root, requirements);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["REQ".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	assert_eq!(layout.family, Some(LayoutFamily::Radial));

	let mut rows: HashSet<i32> = HashSet::new();
	let mut columns: HashSet<i32> = HashSet::new();
	for node_id in &links {
		let child = layout.nodes.get(node_id).expect("child missing");
		rows.insert(child.y);
		columns.insert(child.x);
	}

	assert!(
		rows.len() >= 2 || columns.len() >= 2,
		"expected a wide first level to spread across multiple radial lanes"
	);
}

#[test]
fn radial_layout_keeps_multi_root_groups_reasonably_compact() {
	let root_one = make_card("MIS-001", "Mission", &["REQ-001"]);
	let root_two = make_card("MIS-002", "Mission", &["REQ-002"]);
	let root_three = make_card("MIS-003", "Mission", &["REQ-003"]);
	let root_four = make_card("MIS-004", "Mission", &["REQ-004"]);
	let req_one = make_card("REQ-001", "Requirement", &[]);
	let req_two = make_card("REQ-002", "Requirement", &[]);
	let req_three = make_card("REQ-003", "Requirement", &[]);
	let req_four = make_card("REQ-004", "Requirement", &[]);
	let model = make_model(
		root_one,
		vec![
			root_two, root_three, root_four, req_one, req_two, req_three, req_four,
		],
	);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["MIS".to_string(), "REQ".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let (min_x, max_x, min_y, max_y) = bounds_for(
		&layout,
		&[
			"MIS-001", "MIS-002", "MIS-003", "MIS-004", "REQ-001", "REQ-002", "REQ-003", "REQ-004",
		],
	);
	assert!(
		(max_x - min_x) <= 12000 && (max_y - min_y) <= 12000,
		"expected clustered multi-root spread to stay within a bounded canvas; bounds=({min_x}, {max_x}, {min_y}, {max_y})"
	);
}

#[test]
fn radial_layout_keeps_root_clusters_separate() {
	let root_one = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002"]);
	let root_two = make_card("MIS-002", "Mission", &["REQ-003", "REQ-004"]);
	let req_one = make_card("REQ-001", "Requirement", &["REQ-005", "REQ-006"]);
	let req_two = make_card("REQ-002", "Requirement", &[]);
	let req_three = make_card("REQ-003", "Requirement", &["REQ-007", "REQ-008"]);
	let req_four = make_card("REQ-004", "Requirement", &[]);
	let req_five = make_card("REQ-005", "Requirement", &[]);
	let req_six = make_card("REQ-006", "Requirement", &[]);
	let req_seven = make_card("REQ-007", "Requirement", &[]);
	let req_eight = make_card("REQ-008", "Requirement", &[]);
	let model = make_model(
		root_one,
		vec![
			root_two, req_one, req_two, req_three, req_four, req_five, req_six, req_seven,
			req_eight,
		],
	);

	let layout = layout_model_with_family(
		&model,
		&["MIS".to_string()],
		&["MIS".to_string(), "REQ".to_string()],
		LayoutFamily::Radial,
	)
	.expect("radial layout should succeed");

	let left = bounds_for(
		&layout,
		&["MIS-001", "REQ-001", "REQ-002", "REQ-005", "REQ-006"],
	);
	let right = bounds_for(
		&layout,
		&["MIS-002", "REQ-003", "REQ-004", "REQ-007", "REQ-008"],
	);

	assert!(
		left.1 < right.0 || right.1 < left.0 || left.3 < right.2 || right.3 < left.2,
		"expected packed root clusters to remain separate: left={left:?}, right={right:?}"
	);
}

#[test]
fn best_family_layout_is_deterministic() {
	let root = make_card("MIS-001", "Mission", &["CAP-001", "CAP-002"]);
	let cap_one = make_card("CAP-001", "Capability", &["REQ-001"]);
	let cap_two = make_card("CAP-002", "Capability", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![cap_one, cap_two, requirement]);

	let first = layout_model_best_family(
		&model,
		&["MIS".to_string()],
		&["CAP".to_string(), "REQ".to_string()],
	)
	.expect("best-family layout should succeed");
	let second = layout_model_best_family(
		&model,
		&["MIS".to_string()],
		&["CAP".to_string(), "REQ".to_string()],
	)
	.expect("best-family layout should succeed");

	assert_eq!(sorted_nodes(&first), sorted_nodes(&second));
	assert_eq!(sorted_edges(&first), sorted_edges(&second));
}

fn sorted_nodes(layout: &Layout) -> Vec<(String, i32, i32)> {
	let mut nodes: Vec<(String, i32, i32)> = layout
		.nodes
		.iter()
		.map(|(id, node)| (id.clone(), node.x, node.y))
		.collect();
	nodes.sort_by(|left, right| left.0.cmp(&right.0));
	nodes
}

fn sorted_edges(layout: &Layout) -> Vec<(String, String)> {
	let mut edges: Vec<(String, String)> = layout
		.edges
		.iter()
		.map(|edge| (edge.a.clone(), edge.b.clone()))
		.collect();
	edges.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
	edges
}

fn bounds_for(layout: &Layout, node_ids: &[&str]) -> (i32, i32, i32, i32) {
	let mut min_x = i32::MAX;
	let mut max_x = i32::MIN;
	let mut min_y = i32::MAX;
	let mut max_y = i32::MIN;
	for node_id in node_ids {
		let node = layout.nodes.get(*node_id).expect("node missing");
		min_x = min_x.min(node.x);
		max_x = max_x.max(node.x);
		min_y = min_y.min(node.y);
		max_y = max_y.max(node.y);
	}
	(min_x, max_x, min_y, max_y)
}
