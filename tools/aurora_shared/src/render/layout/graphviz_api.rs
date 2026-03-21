//! Public entry points for Graphviz-backed layout computation.

use crate::{Model, render::render_error::RenderError};

use super::graph::{LayoutGraph, build_graph, validate_graph};
use super::graphviz::layout_with_graphviz;
use super::types::{Layout, LayoutCoordinateSpace, LayoutEdge, LayoutFamily, LayoutPoint};

const NODE_WIDTH_PX: f32 = 720.0;
const NODE_HEIGHT_PX: f32 = 450.0;
const TARGET_ASPECT_RATIO: f64 = 1.6;

#[derive(Debug, Clone, Copy)]
struct LayoutScore {
	score_milli: i64,
	crossings: usize,
	bends: usize,
}

/// Compute a default top-down tree layout for a view selection over a model.
pub fn layout_model(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<Layout, RenderError> {
	layout_model_with_family(
		model,
		root_card_types,
		included_card_types,
		LayoutFamily::TreeTopDown,
	)
}

/// Compute a deterministic layout for a specific layout family.
pub fn layout_model_with_family(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
	family: LayoutFamily,
) -> Result<Layout, RenderError> {
	let graph = prepare_layout_graph(model, root_card_types, included_card_types)?;
	layout_graph_with_family(&graph, family)
}

/// Compute deterministic layouts for all families and return the best-scoring one.
pub fn layout_model_best_family(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<Layout, RenderError> {
	let graph = prepare_layout_graph(model, root_card_types, included_card_types)?;
	let mut best_layout: Option<Layout> = None;
	let mut best_score: Option<LayoutScore> = None;
	let mut best_family: Option<LayoutFamily> = None;
	let mut best_width: Option<i32> = None;

	for family in [LayoutFamily::TreeTopDown, LayoutFamily::TreeLeftRight] {
		let layout = layout_graph_with_family(&graph, family)?;
		let width = layout_width(&layout);
		let score = score_layout(&layout);
		let should_replace = match (best_width, best_score, best_family) {
			(None, _, _) => true,
			(Some(_), None, _) => true,
			(Some(current_width), Some(current_score), Some(current_family)) => {
				width < current_width
					|| (width == current_width
						&& is_better_candidate(score, family, current_score, current_family))
			}
			(Some(_), Some(_), None) => true,
		};
		if should_replace {
			best_layout = Some(layout);
			best_width = Some(width);
			best_score = Some(score);
			best_family = Some(family);
		}
	}

	best_layout.ok_or(RenderError::BackboneOrderFailed)
}

fn prepare_layout_graph(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<LayoutGraph, RenderError> {
	if root_card_types.is_empty() {
		return Err(RenderError::MissingRoots);
	}
	let graph = build_graph(model, root_card_types, included_card_types)?;
	validate_graph(&graph)?;
	Ok(graph)
}

fn layout_graph_with_family(
	graph: &LayoutGraph,
	family: LayoutFamily,
) -> Result<Layout, RenderError> {
	let graphviz = layout_with_graphviz(graph, family)?;
	let mut edges = graph
		.edges
		.iter()
		.map(|(a, b)| LayoutEdge {
			a: a.clone(),
			b: b.clone(),
		})
		.collect::<Vec<_>>();
	edges.sort_by(|left, right| left.a.cmp(&right.a).then_with(|| left.b.cmp(&right.b)));
	Ok(Layout {
		family: Some(family),
		coordinate_space: LayoutCoordinateSpace::Pixels,
		nodes: graphviz.nodes,
		edges,
		routes: graphviz.routes,
	})
}

fn score_layout(layout: &Layout) -> LayoutScore {
	let mut min_x = f32::INFINITY;
	let mut min_y = f32::INFINITY;
	let mut max_x = f32::NEG_INFINITY;
	let mut max_y = f32::NEG_INFINITY;
	for node in layout.nodes.values() {
		min_x = min_x.min(node.x as f32);
		min_y = min_y.min(node.y as f32);
		max_x = max_x.max(node.x as f32 + NODE_WIDTH_PX);
		max_y = max_y.max(node.y as f32 + NODE_HEIGHT_PX);
	}
	let width = (max_x - min_x).max(1.0) as f64;
	let height = (max_y - min_y).max(1.0) as f64;
	let aspect_error = ((width / height) / TARGET_ASPECT_RATIO).ln().abs();
	let crossings = edge_crossings(layout);
	let bends = edge_bends(layout);
	let total_length = edge_length(layout);
	let long_span = long_span_penalty(layout);
	let score = 5.0f64 * aspect_error
		+ 10.0f64 * crossings as f64
		+ 2.0f64 * bends as f64
		+ 0.5f64 * total_length
		+ 3.0f64 * long_span as f64;
	LayoutScore {
		score_milli: (score * 1000.0).round() as i64,
		crossings,
		bends,
	}
}

fn layout_width(layout: &Layout) -> i32 {
	let mut min_x = i32::MAX;
	let mut max_x = i32::MIN;
	for node in layout.nodes.values() {
		min_x = min_x.min(node.x);
		max_x = max_x.max(node.x + NODE_WIDTH_PX as i32);
	}
	(max_x - min_x).max(1)
}

fn edge_crossings(layout: &Layout) -> usize {
	let routes = edge_routes(layout);
	let mut crossings = 0usize;
	for left_index in 0..routes.len() {
		for right_index in left_index + 1..routes.len() {
			for left in routes[left_index].windows(2) {
				for right in routes[right_index].windows(2) {
					if shares_endpoint(left[0], left[1], right[0], right[1]) {
						continue;
					}
					if segments_intersect(left[0], left[1], right[0], right[1]) {
						crossings += 1;
					}
				}
			}
		}
	}
	crossings
}

fn edge_bends(layout: &Layout) -> usize {
	let mut bends = 0usize;
	for route in edge_routes(layout) {
		for segment in route.windows(3) {
			if is_bend(segment[0], segment[1], segment[2]) {
				bends += 1;
			}
		}
	}
	bends
}

fn edge_length(layout: &Layout) -> f64 {
	let mut total = 0.0f64;
	for route in edge_routes(layout) {
		for segment in route.windows(2) {
			total += distance(segment[0], segment[1]) as f64 / 1000.0;
		}
	}
	total
}

fn long_span_penalty(layout: &Layout) -> usize {
	let mut penalty = 0usize;
	for edge in &layout.edges {
		let Some(source) = layout.nodes.get(edge.a.as_str()) else {
			continue;
		};
		let Some(target) = layout.nodes.get(edge.b.as_str()) else {
			continue;
		};
		let dx = ((target.x - source.x).abs() as f32) / NODE_WIDTH_PX;
		let dy = ((target.y - source.y).abs() as f32) / NODE_HEIGHT_PX;
		let span = dx.max(dy).ceil() as usize;
		penalty += span.saturating_sub(2);
	}
	penalty
}

fn edge_routes(layout: &Layout) -> Vec<Vec<LayoutPoint>> {
	let mut routes = Vec::with_capacity(layout.edges.len());
	for edge in &layout.edges {
		if let Some(route) = layout.routes.get(&(edge.a.clone(), edge.b.clone())) {
			routes.push(route.clone());
			continue;
		}
		let Some(source) = layout.nodes.get(edge.a.as_str()) else {
			continue;
		};
		let Some(target) = layout.nodes.get(edge.b.as_str()) else {
			continue;
		};
		routes.push(vec![
			center_point(source.x, source.y),
			center_point(target.x, target.y),
		]);
	}
	routes
}

fn center_point(x: i32, y: i32) -> LayoutPoint {
	LayoutPoint {
		x: x as f32 + NODE_WIDTH_PX / 2.0,
		y: y as f32 + NODE_HEIGHT_PX / 2.0,
	}
}

fn is_bend(a: LayoutPoint, b: LayoutPoint, c: LayoutPoint) -> bool {
	let ab = (b.x - a.x, b.y - a.y);
	let bc = (c.x - b.x, c.y - b.y);
	let ab_len = (ab.0 * ab.0 + ab.1 * ab.1).sqrt();
	let bc_len = (bc.0 * bc.0 + bc.1 * bc.1).sqrt();
	if ab_len <= f32::EPSILON || bc_len <= f32::EPSILON {
		return false;
	}
	let cross = ab.0 * bc.1 - ab.1 * bc.0;
	cross.abs() > 0.01
}

fn distance(a: LayoutPoint, b: LayoutPoint) -> f32 {
	let dx = b.x - a.x;
	let dy = b.y - a.y;
	(dx * dx + dy * dy).sqrt()
}

fn shares_endpoint(a1: LayoutPoint, a2: LayoutPoint, b1: LayoutPoint, b2: LayoutPoint) -> bool {
	points_close(a1, b1) || points_close(a1, b2) || points_close(a2, b1) || points_close(a2, b2)
}

fn segments_intersect(a1: LayoutPoint, a2: LayoutPoint, b1: LayoutPoint, b2: LayoutPoint) -> bool {
	let o1 = orientation(a1, a2, b1);
	let o2 = orientation(a1, a2, b2);
	let o3 = orientation(b1, b2, a1);
	let o4 = orientation(b1, b2, a2);
	o1.abs() > 0.01
		&& o2.abs() > 0.01
		&& o3.abs() > 0.01
		&& o4.abs() > 0.01
		&& ((o1 > 0.0 && o2 < 0.0) || (o1 < 0.0 && o2 > 0.0))
		&& ((o3 > 0.0 && o4 < 0.0) || (o3 < 0.0 && o4 > 0.0))
}

fn orientation(a: LayoutPoint, b: LayoutPoint, c: LayoutPoint) -> f32 {
	(b.y - a.y) * (c.x - b.x) - (b.x - a.x) * (c.y - b.y)
}

fn points_close(left: LayoutPoint, right: LayoutPoint) -> bool {
	(left.x - right.x).abs() < 0.01 && (left.y - right.y).abs() < 0.01
}

fn is_better_candidate(
	candidate_score: LayoutScore,
	candidate_family: LayoutFamily,
	current_score: LayoutScore,
	current_family: LayoutFamily,
) -> bool {
	if candidate_score.score_milli != current_score.score_milli {
		return candidate_score.score_milli < current_score.score_milli;
	}
	if candidate_score.crossings != current_score.crossings {
		return candidate_score.crossings < current_score.crossings;
	}
	if candidate_score.bends != current_score.bends {
		return candidate_score.bends < current_score.bends;
	}
	candidate_family < current_family
}

#[cfg(test)]
mod tests {
	use super::{
		Layout, LayoutCoordinateSpace, LayoutEdge, LayoutFamily, LayoutPoint, center_point,
		edge_bends, edge_crossings, layout_model_best_family,
	};
	use crate::render::LayoutNode;
	use crate::render::layout::test_support::{make_card, make_model};
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
}
