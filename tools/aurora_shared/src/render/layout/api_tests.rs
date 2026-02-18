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

	let layout = layout_model(&model, &["MIS".to_string()], &["REQ".to_string()])
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
		layout.nodes.get("PRO-001").is_none(),
		"expected isolated PRO-001 to be pruned from the filtered view"
	);
}

#[test]
fn layout_model_centers_node_between_two_parents_across_long_edge() {
	// Build a graph where REQ-001 has two parents: PRO-001 (rank 2) and CAP-001 (rank 1).
	// The CAP-001 -> REQ-001 edge should span multiple ranks and must still influence centering.
	//
	// Important: edge classification builds a backbone spanning tree by discovery order. We want the
	// backbone path to reach REQ-001 via CAP-002 -> PRO-001 -> REQ-001, so CAP-002 must be explored
	// before CAP-001 (LIFO traversal with sorted neighbors makes that deterministic here).
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

	let cap_one = layout.nodes.get("CAP-001").expect("CAP-001 missing");
	let cap_two = layout.nodes.get("CAP-002").expect("CAP-002 missing");
	let process = layout.nodes.get("PRO-001").expect("PRO-001 missing");
	let requirement = layout.nodes.get("REQ-001").expect("REQ-001 missing");

	assert_eq!(cap_one.y, 1);
	assert_eq!(cap_two.y, 1);
	assert!(cap_one.x != cap_two.x);
	assert_eq!(process.y, 2);
	assert_eq!(requirement.y, 3);
	// CAP-001 -> REQ-001 should be a long edge (spanning at least one intermediate rank).
	assert_eq!(requirement.y - cap_one.y, 2);

	// Requirement should be centered under both parents, even though CAP-001 -> REQ-001 spans
	// multiple ranks.
	let expected = (cap_one.x + process.x) / 2;
	assert_eq!(requirement.x, expected);
}

#[test]
fn layout_model_can_center_a_node_between_two_parents() {
	// Two parents in the same layer should allow a child to land at the midpoint.
	//
	// This is one of the main reasons we use half-step x-coordinates (2 units per column):
	// parents at x=0 and x=2 can place a child at x=1.
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
	let process = layout.nodes.get("PRO-001").expect("PRO-001 missing");

	assert_eq!(cap_one.y, 1);
	assert_eq!(cap_two.y, 1);
	assert_eq!(process.y, 2);

	let left = cap_one.x.min(cap_two.x);
	let right = cap_one.x.max(cap_two.x);
	assert_eq!(right - left, 2);
	assert_eq!(process.x, left + 1);
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
