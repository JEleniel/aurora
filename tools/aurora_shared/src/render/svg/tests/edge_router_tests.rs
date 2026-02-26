use std::collections::HashMap;

use super::*;
use crate::render::svg::node::NodeGeom;

fn make_node(x: i32, y: i32, w: i32, h: i32) -> PositionedNode {
	PositionedNode {
		bbox: geom::RectI { x, y, w, h },
		width_px: w,
		height_px: h,
		geom: NodeGeom {
			width_px: w,
			height_px: h,
			lines: Vec::new(),
			bold_line_index: None,
			description_start_index: 0,
		},
	}
}

fn contains(rect: geom::RectI, p: geom::PointF) -> bool {
	p.x >= rect.x as f32
		&& p.x <= (rect.x + rect.w) as f32
		&& p.y >= rect.y as f32
		&& p.y <= (rect.y + rect.h) as f32
}

#[test]
fn router_is_deterministic_for_fixed_input() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("S".to_string(), make_node(100, 100, 80, 40));
	positioned.insert("A".to_string(), make_node(100, 260, 80, 40));
	positioned.insert("B".to_string(), make_node(260, 260, 80, 40));
	positioned.insert("T".to_string(), make_node(180, 420, 80, 40));

	let edges = vec![
		LayoutEdge {
			a: "S".to_string(),
			b: "A".to_string(),
		},
		LayoutEdge {
			a: "S".to_string(),
			b: "B".to_string(),
		},
		LayoutEdge {
			a: "A".to_string(),
			b: "T".to_string(),
		},
		LayoutEdge {
			a: "B".to_string(),
			b: "T".to_string(),
		},
	];

	let config = SvgConfig::default();
	let r1 = route_edges(&positioned, edges.as_slice(), &config).expect("routes should succeed");
	let r2 = route_edges(&positioned, edges.as_slice(), &config).expect("routes should succeed");

	let mut k1: Vec<(String, Vec<(i32, i32)>)> = r1
		.iter()
		.map(|r| {
			(
				r.edge_id.clone(),
				r.points
					.iter()
					.map(|p| (p.x.round() as i32, p.y.round() as i32))
					.collect(),
			)
		})
		.collect();
	let mut k2: Vec<(String, Vec<(i32, i32)>)> = r2
		.iter()
		.map(|r| {
			(
				r.edge_id.clone(),
				r.points
					.iter()
					.map(|p| (p.x.round() as i32, p.y.round() as i32))
					.collect(),
			)
		})
		.collect();
	k1.sort_by(|l, r| l.0.cmp(&r.0));
	k2.sort_by(|l, r| l.0.cmp(&r.0));
	assert_eq!(k1, k2);
}

#[test]
fn router_routes_are_orthogonal_and_clear_of_obstacles() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("S".to_string(), make_node(100, 100, 80, 40));
	positioned.insert("T".to_string(), make_node(300, 300, 80, 40));
	positioned.insert("O".to_string(), make_node(200, 180, 80, 140));

	let edges = vec![LayoutEdge {
		a: "S".to_string(),
		b: "T".to_string(),
	}];
	let config = SvgConfig::default();
	let routes = route_edges(&positioned, edges.as_slice(), &config).expect("route should succeed");
	assert_eq!(routes.len(), 1);

	let inflated_obstacles: Vec<geom::RectI> = positioned
		.values()
		.map(|n| inflate(n.bbox, super::CLEARANCE_PX))
		.collect();

	for route in &routes {
		assert!(route.points.len() >= 2);
		for seg in route.points.windows(2) {
			let a = seg[0];
			let b = seg[1];
			assert!(
				(a.x - b.x).abs() < 0.01 || (a.y - b.y).abs() < 0.01,
				"segment must be orthogonal: {a:?} -> {b:?}"
			);
		}

		for p in route
			.points
			.iter()
			.copied()
			.skip(1)
			.take(route.points.len().saturating_sub(2))
		{
			assert!(
				!inflated_obstacles.iter().any(|r| contains(*r, p)),
				"non-incident point should not enter inflated obstacles: {p:?}"
			);
		}
	}
}

#[test]
fn router_keeps_expanded_terminal_clearance() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("S".to_string(), make_node(100, 100, 80, 40));
	positioned.insert("T".to_string(), make_node(260, 360, 80, 40));

	let source = positioned.get("S").expect("source exists").bbox;
	let target = positioned.get("T").expect("target exists").bbox;
	let obstacles = super::build_obstacles(&positioned);
	let ports = super::ports_for_edge(source, target, obstacles.as_slice());

	let source_bottom_y = source.y + source.h;
	let target_top_y = target.y;
	let source_exit_y = ports.source_exit.y;
	let target_entry_y = ports.target_entry.y;

	assert!(
		source_exit_y - source_bottom_y >= super::CLEARANCE_PX,
		"source exit should stay outside expanded clearance"
	);
	assert!(
		target_top_y - target_entry_y >= super::CLEARANCE_PX,
		"target approach should stay outside expanded clearance"
	);
}

#[test]
fn router_avoids_below_target_detour_when_path_is_open() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("S".to_string(), make_node(100, 100, 80, 40));
	positioned.insert("T".to_string(), make_node(360, 320, 80, 40));

	let edges = vec![LayoutEdge {
		a: "S".to_string(),
		b: "T".to_string(),
	}];
	let config = SvgConfig::default();
	let routes = route_edges(&positioned, edges.as_slice(), &config).expect("route should succeed");
	assert_eq!(routes.len(), 1);

	let route = &routes[0];
	let target_top_y = 320;
	for p in route
		.points
		.iter()
		.copied()
		.skip(1)
		.take(route.points.len().saturating_sub(2))
	{
		assert!(
			(p.y.round() as i32) <= target_top_y,
			"open route should not dip below target top before final approach: {p:?}"
		);
	}
}

#[test]
fn router_orders_sources_by_geometry_then_id() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("SRC-Z".to_string(), make_node(100, 100, 80, 40));
	positioned.insert("SRC-A".to_string(), make_node(900, 100, 80, 40));
	positioned.insert("T-A".to_string(), make_node(100, 320, 80, 40));
	positioned.insert("T-Z".to_string(), make_node(900, 320, 80, 40));

	let edges = vec![
		LayoutEdge {
			a: "SRC-A".to_string(),
			b: "T-Z".to_string(),
		},
		LayoutEdge {
			a: "SRC-Z".to_string(),
			b: "T-A".to_string(),
		},
	];

	let config = SvgConfig::default();
	let routes = route_edges(&positioned, edges.as_slice(), &config).expect("route should succeed");
	assert_eq!(routes.len(), 2);
	assert_eq!(
		routes[0].source_id, "SRC-Z",
		"left-most source should be routed first"
	);
}

#[test]
fn router_orders_dense_sources_center_first() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	for (idx, x) in [100, 300, 500, 700, 900].iter().copied().enumerate() {
		positioned.insert(format!("S{idx}"), make_node(x, 100, 80, 40));
	}
	for (idx, x) in [120, 280, 500, 720, 880, 1040].iter().copied().enumerate() {
		positioned.insert(format!("T{idx}"), make_node(x, 760, 80, 40));
	}

	let mut edges: Vec<LayoutEdge> = Vec::new();
	for source_idx in 0..5 {
		for target_idx in 0..6 {
			edges.push(LayoutEdge {
				a: format!("S{source_idx}"),
				b: format!("T{target_idx}"),
			});
		}
	}

	let routes = route_edges(&positioned, edges.as_slice(), &SvgConfig::default())
		.expect("dense routes should succeed");
	assert_eq!(routes.len(), edges.len());
	assert_eq!(
		routes[0].source_id, "S2",
		"dense routing should process center-nearest sources first"
	);
}

#[test]
fn router_orders_targets_by_geometry_within_source() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("SRC".to_string(), make_node(500, 100, 80, 40));
	positioned.insert("T-Z".to_string(), make_node(100, 320, 80, 40));
	positioned.insert("T-A".to_string(), make_node(900, 320, 80, 40));

	let edges = vec![
		LayoutEdge {
			a: "SRC".to_string(),
			b: "T-A".to_string(),
		},
		LayoutEdge {
			a: "SRC".to_string(),
			b: "T-Z".to_string(),
		},
	];

	let config = SvgConfig::default();
	let routes = route_edges(&positioned, edges.as_slice(), &config).expect("route should succeed");
	assert_eq!(routes.len(), 2);
	assert_eq!(
		routes[0].target_id, "T-Z",
		"left-most target should be routed first within a source"
	);
}

#[test]
fn collector_span_starts_near_vertical_center() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("SRC".to_string(), make_node(120, 120, 80, 40));
	positioned.insert("TGT".to_string(), make_node(520, 720, 80, 40));

	let obstacles = super::build_obstacles(&positioned);
	let source = positioned.get("SRC").expect("source exists").bbox;
	let target = positioned.get("TGT").expect("target exists").bbox;
	let ports = super::ports_for_edge(source, target, obstacles.as_slice());
	let bounds = super::global_bounds(
		obstacles.as_slice(),
		&[ports.source_exit, ports.target_entry],
	);
	let center_y = (bounds.2 + bounds.3) / 2;
	let collector =
		super::collector_for_target(0, target, obstacles.as_slice(), bounds.2, center_y);

	assert!(
		collector.y_top >= center_y.min(collector.y_bottom - 1),
		"collector should grow from center band upward/downward instead of top boundary"
	);
}

#[test]
fn routes_to_same_target_align_on_collector_axis() {
	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	positioned.insert("S1".to_string(), make_node(100, 120, 80, 40));
	positioned.insert("S2".to_string(), make_node(900, 120, 80, 40));
	positioned.insert("T".to_string(), make_node(500, 760, 80, 40));

	let edges = vec![
		LayoutEdge {
			a: "S1".to_string(),
			b: "T".to_string(),
		},
		LayoutEdge {
			a: "S2".to_string(),
			b: "T".to_string(),
		},
	];

	let routes = route_edges(&positioned, edges.as_slice(), &SvgConfig::default())
		.expect("route should succeed");
	assert_eq!(routes.len(), 2);
	let target_center_x = positioned
		.get("T")
		.expect("target exists")
		.bbox
		.center()
		.x
		.round() as i32;
	let on_collector = |route: &Route| {
		route
			.points
			.iter()
			.copied()
			.skip(1)
			.take(route.points.len().saturating_sub(2))
			.any(|p| p.x.round() as i32 == target_center_x)
	};
	assert!(
		on_collector(&routes[0]) && on_collector(&routes[1]),
		"same-target routes should align onto the target collector axis"
	);
}
