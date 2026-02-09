use super::graph::{build_graph, classify_edges, validate_graph};
use super::test_support::{make_card, make_model};

#[test]
fn build_graph_filters_cards_by_type() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let driver = make_card("DRI-001", "Driver", &["MIS-001"]);
	let model = make_model(root, vec![requirement, driver]);

	let graph = build_graph(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	)
	.expect("graph should build");

	assert!(graph.allowed_nodes.contains("MIS-001"));
	assert!(graph.allowed_nodes.contains("REQ-001"));
	assert!(!graph.allowed_nodes.contains("DRI-001"));
	assert_eq!(graph.edges.len(), 1);
}

#[test]
fn build_graph_prunes_unreachable_components() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let driver_one = make_card("DRI-001", "Driver", &["DRI-002"]);
	let driver_two = make_card("DRI-002", "Driver", &["DRI-001"]);
	let model = make_model(root, vec![requirement, driver_one, driver_two]);

	let graph = build_graph(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string(), "Driver".to_string()],
	)
	.expect("graph should build");

	assert!(graph.roots.contains(&"MIS-001".to_string()));
	assert!(!graph.allowed_nodes.contains("DRI-001"));
	assert!(!graph.allowed_nodes.contains("DRI-002"));
	assert!(!graph.roots.contains(&"DRI-001".to_string()));
	assert!(!graph.roots.contains(&"DRI-002".to_string()));
	validate_graph(&graph).expect("graph should validate");
}

#[test]
fn classify_edges_is_deterministic() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &["REQ-002"]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement, requirement_two]);

	let graph = build_graph(
		&model,
		&["Mission".to_string()],
		&["Requirement".to_string()],
	)
	.expect("graph should build");

	let first = classify_edges(&graph);
	let second = classify_edges(&graph);

	assert_eq!(first.backbone, second.backbone);
	assert_eq!(first.loops, second.loops);
}
