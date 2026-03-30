use super::{
	aspect_preference, center_point, edge_bends, edge_crossings, layout_model_best_family,
};
use crate::render::LayoutNode;
use crate::render::layout::test_support::{make_card, make_model};
use crate::render::layout::types::{
	Layout, LayoutCoordinateSpace, LayoutEdge, LayoutFamily, LayoutPoint,
};
use std::collections::HashMap;

type SyntheticRoute<'a> = ((&'a str, &'a str), Vec<(f32, f32)>);

#[test]
fn scoring_detects_crossing_routes() {
	let layout = synthetic_layout(vec![
		(("A", "B"), vec![(0.0, 0.0), (10.0, 10.0)]),
		(("C", "D"), vec![(0.0, 10.0), (10.0, 0.0)]),
	]);
	assert_eq!(edge_crossings(&layout), 1);
}

#[test]
fn scoring_detects_bends() {
	let layout = synthetic_layout(vec![(
		("A", "B"),
		vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
	)]);
	assert_eq!(edge_bends(&layout), 1);
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
	let mut edges = Vec::new();
	let mut route_map = HashMap::new();
	for ((source, target), points) in routes {
		edges.push(LayoutEdge {
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
		edges,
		routes: route_map,
	}
}

#[test]
fn center_point_uses_symbol_bounds() {
	let center = center_point(10, 20);
	assert_eq!(center.x, 370.0);
	assert_eq!(center.y, 245.0);
}
