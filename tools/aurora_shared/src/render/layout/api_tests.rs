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
