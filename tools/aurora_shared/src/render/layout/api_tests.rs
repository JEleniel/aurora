use std::collections::HashSet;

use crate::render::render_error::RenderError;

use super::api::{layout_model, layout_model_best_family, layout_model_with_family};
use super::test_support::{make_card, make_model};
use super::types::{Layout, LayoutFamily};

#[test]
fn layout_model_positions_nodes_by_rank() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement_one = make_card("REQ-001", "Requirement", &["REQ-002"]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("layout should succeed");

	// REQ-001 is a normalized transit node (1 incoming, 1 outgoing), so it is removed.
	assert_eq!(layout.nodes.len(), 2);
	assert_eq!(layout.edges.len(), 1);

	let root_node = layout.nodes.get("MIS-001").expect("root node missing");
	assert_eq!(root_node.y, 0);
	let second_requirement = layout.nodes.get("REQ-002").expect("node missing");
	assert_eq!(second_requirement.y, 1);
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

	assert!(
		!layout.nodes.contains_key("CAP-001"),
		"CAP-001 should be removed as a transit node"
	);
	assert!(
		!layout.nodes.contains_key("CAP-002"),
		"CAP-002 should be removed as a transit node"
	);
	assert!(
		!layout.nodes.contains_key("PRO-001"),
		"PRO-001 should be removed as a transit node"
	);
	let requirement = layout.nodes.get("REQ-001").expect("REQ-001 missing");

	assert_eq!(layout.nodes.get("MIS-001").expect("MIS-001 missing").x, 0);
	assert_eq!(requirement.x, 0);
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

	let non_spine_max_col = layout.nodes.values().map(|node| node.x).max().unwrap_or(0);
	// n=1 non-spine node -> k=ceil(sqrt(0.75*1))=1
	assert!(non_spine_max_col <= 1);
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
		LayoutFamily::HorizontalTree,
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
		LayoutFamily::RadialSubtree,
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
		LayoutFamily::RadialSubtree,
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
		LayoutFamily::RadialSubtree,
	)
	.expect("radial layout should succeed");

	let root_one = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let root_two = layout.nodes.get("MIS-002").expect("MIS-002 missing");
	let root_three = layout.nodes.get("MIS-003").expect("MIS-003 missing");
	let root_four = layout.nodes.get("MIS-004").expect("MIS-004 missing");

	assert_eq!((root_one.x, root_one.y), (0, -2));
	assert_eq!((root_two.x, root_two.y), (2, 0));
	assert_eq!((root_three.x, root_three.y), (0, 2));
	assert_eq!((root_four.x, root_four.y), (-2, 0));
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
		LayoutFamily::RadialSubtree,
	)
	.expect("radial layout should succeed");

	let root = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	for node_id in ["REQ-001", "REQ-002", "REQ-003"] {
		let child = layout.nodes.get(node_id).expect("child missing");
		let dx = (child.x - root.x).abs();
		let dy = (child.y - root.y).abs();
		assert!(
			dx <= 2 && dy <= 2,
			"direct radial child {node_id} should remain within the compact first two rings; got Δx={dx}, Δy={dy}"
		);
	}
}

#[test]
fn radial_layout_allows_two_rows_for_first_descendants() {
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
		LayoutFamily::RadialSubtree,
	)
	.expect("radial layout should succeed");

	assert_eq!(layout.family, Some(LayoutFamily::RadialSubtree));

	let root = layout.nodes.get("MIS-001").expect("MIS-001 missing");
	let mut rings: HashSet<i32> = HashSet::new();
	for node_id in &links {
		let child = layout.nodes.get(node_id).expect("child missing");
		let dx = (child.x - root.x).abs();
		let dy = (child.y - root.y).abs();
		rings.insert(dx.max(dy));
	}

	assert!(
		rings.len() >= 2,
		"expected first descendants to span two rows"
	);
	assert!(
		rings.iter().copied().max().unwrap_or(0) <= 2,
		"expected first descendants to remain compact within two rows"
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
		LayoutFamily::RadialSubtree,
	)
	.expect("radial layout should succeed");

	let max_abs_x = layout
		.nodes
		.values()
		.map(|node| node.x.abs())
		.max()
		.unwrap_or(0);
	assert!(
		max_abs_x <= 5,
		"expected multi-root radial spread to stay compact; max |x| was {max_abs_x}"
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
