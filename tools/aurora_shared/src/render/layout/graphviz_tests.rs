use super::{
	GraphvizLayout, LayoutFamily, build_graphviz_input, normalize_edge_endpoint,
	parse_plain_output, spec_for_family,
};
use crate::render::LayoutNode;
use crate::render::layout::graph::build_graph;
use crate::render::layout::graphviz_api::{aspect_preference, layout_model_best_family};
use crate::render::layout::test_support::{make_card, make_model};
use crate::render::layout::types::{Layout, LayoutCoordinateSpace, LayoutEdge, LayoutPoint};
use std::collections::HashMap;

type SyntheticRoute<'a> = ((&'a str, &'a str), Vec<(f32, f32)>);

#[test]
fn graphviz_input_emits_plain_tree_nodes_without_helper_root() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![requirement]);
	let graph = build_graph(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("graph should build");

	let dot = build_graphviz_input(
		&graph,
		spec_for_family(LayoutFamily::TreeTopDown),
		LayoutFamily::TreeTopDown,
	);
	assert!(dot.contains("\"MIS-001\";"));
	assert!(dot.contains("\"REQ-001\";"));
	assert!(!dot.contains("__aurora_layout_root__"));
}

#[test]
fn graphviz_input_uses_expected_spacing_and_rankdir() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![requirement]);
	let graph = build_graph(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("graph should build");

	let dot = build_graphviz_input(
		&graph,
		spec_for_family(LayoutFamily::TreeLeftRight),
		LayoutFamily::TreeLeftRight,
	);
	assert!(dot.contains("rankdir=LR"));
	assert!(dot.contains("nodesep=1.0000"));
	assert!(dot.contains("ranksep=2.0000"));
	assert!(dot.contains("splines=ortho"));
}

#[test]
fn normalize_edge_endpoint_strips_ports() {
	assert_eq!(normalize_edge_endpoint("\"MIS-001:e\""), "MIS-001");
}

#[test]
fn parse_plain_output_scales_nodes_and_routes() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![requirement]);
	let graph = build_graph(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("graph should build");
	let plain = "graph 1 5.0 3.0\nnode MIS-001 1.2 0.75 2.4 1.5 \"\" solid box black lightgrey\nnode REQ-001 3.8 2.25 2.4 1.5 \"\" solid box black lightgrey\nedge MIS-001 REQ-001 4 2.4 0.75 2.9 0.75 3.1 2.25 3.8 2.25 solid black\nstop\n";

	let GraphvizLayout { nodes, routes } =
		parse_plain_output(plain, &graph).expect("plain output should parse");
	let mission = nodes.get("MIS-001").expect("mission node missing");
	assert_eq!(mission.x, 0);
	assert_eq!(mission.y, 0);
	let route = routes
		.get(&("MIS-001".to_string(), "REQ-001".to_string()))
		.expect("route missing");
	assert_eq!(route.len(), 4);
	assert!((route[0].x - 720.0).abs() < 0.1);
}

#[test]
fn parse_plain_output_merges_wrapped_edge_records() {
	let root = make_card("MIS-001", "Mission", &["REQ-001"]);
	let requirement = make_card("REQ-001", "Requirement", &[]);
	let model = make_model(root, vec![requirement]);
	let graph = build_graph(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("graph should build");
	let plain = "graph 1 5.0 3.0\nnode MIS-001 1.2 0.75 2.4 1.5 \"\" solid box black lightgrey\nnode REQ-001 3.8 2.25 2.4 1.5 \"\" solid box black lightgrey\nedge MIS-001 REQ-001 4 2.4 0.75 2.9 0.75 3.1 2.25 3.8 2.25\nsolid black\nstop\n";

	let GraphvizLayout { routes, .. } =
		parse_plain_output(plain, &graph).expect("wrapped plain output should parse");
	assert!(routes.contains_key(&("MIS-001".to_string(), "REQ-001".to_string())));
}

fn synthetic_bounds_layout(x: i32, y: i32) -> Layout {
	let mut layout = synthetic_layout(Vec::new());
	layout.nodes.insert(
		"B".to_string(),
		LayoutNode {
			id: "B".to_string(),
			x,
			y,
		},
	);
	layout
}

fn synthetic_layout(routes: Vec<SyntheticRoute<'_>>) -> Layout {
	let mut nodes = HashMap::new();
	nodes.insert(
		"A".to_string(),
		LayoutNode {
			id: "A".to_string(),
			x: 0,
			y: 0,
		},
	);
	nodes.insert(
		"B".to_string(),
		LayoutNode {
			id: "B".to_string(),
			x: 0,
			y: 0,
		},
	);
	nodes.insert(
		"C".to_string(),
		LayoutNode {
			id: "C".to_string(),
			x: 0,
			y: 0,
		},
	);
	nodes.insert(
		"D".to_string(),
		LayoutNode {
			id: "D".to_string(),
			x: 0,
			y: 0,
		},
	);

	let mut edge_list = Vec::new();
	let mut route_map = HashMap::new();
	for ((source, target), points) in routes {
		edge_list.push(LayoutEdge {
			a: source.to_string(),
			b: target.to_string(),
		});
		route_map.insert(
			(source.to_string(), target.to_string()),
			points
				.into_iter()
				.map(|(x, y)| LayoutPoint { x, y })
				.collect(),
		);
	}

	Layout {
		family: Some(LayoutFamily::TreeTopDown),
		coordinate_space: LayoutCoordinateSpace::Pixels,
		nodes,
		edges: edge_list,
		routes: route_map,
	}
}

#[test]
fn best_family_is_deterministic() {
	let root = make_card("MIS-001", "Mission", &["REQ-001", "REQ-002"]);
	let requirement_one = make_card("REQ-001", "Requirement", &[]);
	let requirement_two = make_card("REQ-002", "Requirement", &[]);
	let model = make_model(root, vec![requirement_one, requirement_two]);

	let first = layout_model_best_family(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("best-family layout should succeed");
	let second = layout_model_best_family(&model, &["MIS".to_string()], &["REQ".to_string()])
		.expect("best-family layout should succeed");

	assert_eq!(first.family, second.family);
	assert_eq!(first.routes, second.routes);
}

#[test]
fn aspect_preference_prefers_ratio_below_target() {
	let far_below = synthetic_bounds_layout(360, 450);
	let close_below = synthetic_bounds_layout(672, 450);
	let close_above = synthetic_bounds_layout(768, 450);

	assert!(aspect_preference(&close_below) < aspect_preference(&far_below));
	assert!(aspect_preference(&close_below) < aspect_preference(&close_above));
}
