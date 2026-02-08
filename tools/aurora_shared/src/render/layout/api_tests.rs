use crate::render::render_error::RenderError;

use super::api::layout_model;
use super::test_support::{make_card, make_model};
use super::types::Layout;

#[test]
fn layout_model_positions_nodes_by_rank() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement_one = make_card("REQ-001", "Requirement", &["REQ-002"]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout = layout_model(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	)
	.expect("layout should succeed");

	assert_eq!(layout.nodes.len(), 3);
	assert_eq!(layout.edges.len(), 2);

	let root_node = layout.nodes.get("MIS-001").expect("root node missing");
	assert_eq!(root_node.y, 0);
	let first_requirement = layout.nodes.get("REQ-001").expect("node missing");
	assert_eq!(first_requirement.y, 1);
	let second_requirement = layout.nodes.get("REQ-002").expect("node missing");
	assert_eq!(second_requirement.y, 2);
}

#[test]
fn layout_model_rejects_missing_roots() {
	let root = make_card("MIS-001", "Mission", &[]);
	let model = make_model(root, vec![]);

	let result = layout_model(&model, &[], &["Requirement".to_string()]);
	assert!(matches!(result, Err(RenderError::MissingRoots)));
}

#[test]
fn layout_model_rejects_unknown_targets() {
	let root = make_card("MIS-001", "Mission", &["REQ-404"]);
	let model = make_model(root, vec![]);

	let result = layout_model(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	);

	assert!(matches!(result, Err(RenderError::UnknownTarget(_, _))));
}

#[test]
fn layout_model_is_deterministic() {
	let root = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002"]);
	let requirement_one = make_card("REQ-001", "Requirement", &[]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let layout_one = layout_model(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	)
	.expect("layout should succeed");
	let layout_two = layout_model(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	)
	.expect("layout should succeed");

	assert_eq!(sorted_nodes(&layout_one), sorted_nodes(&layout_two));
	assert_eq!(sorted_edges(&layout_one), sorted_edges(&layout_two));
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
