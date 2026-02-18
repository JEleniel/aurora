use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::{EdgeStyle, RenderError, SvgConfig, geom};

const EDGE_STROKE: &str = "#000000";
const EDGE_MASK_STROKE: &str = "#ffffff";
const EDGE_STROKE_WIDTH_PX: i32 = 4;
const EDGE_MASK_STROKE_WIDTH_PX: i32 = 19;
const SYMBOL_PARALLEL_EDGE_CLEARANCE_PX: f32 = 80.0;
const SYMBOL_ANCHOR_BIAS_SCALE: f32 = 0.35;
const MIN_TERMINAL_SEGMENT_PX: i32 = 80;
const ARROW_SIZE_PX: f32 = 30.0;
const ADJACENT_STRAIGHT_GAP_PX: i32 = 220;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
	x: i32,
	y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Dir {
	dx: i8,
	dy: i8,
}

const DIR_NONE: Dir = Dir { dx: 0, dy: 0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
	cell: Cell,
	dir: Dir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeapItem {
	f: i32,
	g: i32,
	state: State,
}

impl Ord for HeapItem {
	fn cmp(&self, other: &Self) -> Ordering {
		other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
	}
}

impl PartialOrd for HeapItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone)]
pub struct Route {
	pub points: Vec<geom::PointF>,
	pub arrow: [geom::PointF; 3],
	pub bounds: geom::Bounds,
}

#[derive(Debug, Clone, Copy)]
struct GridBounds {
	min_x: i32,
	max_x: i32,
	min_y: i32,
	max_y: i32,
}

#[derive(Debug, Clone, Copy)]
struct RoutingEnvelope {
	min_x: f32,
	max_x: f32,
	min_y: f32,
	max_y: f32,
}

pub fn route_edge(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	symbol_bboxes: &[geom::RectI],
	source_anchor_bias: f32,
	target_anchor_bias: f32,
	config: &SvgConfig,
) -> Result<Route, RenderError> {
	let rem_px = config.base_font_size_px.max(1);
	let clearance = (rem_px / 2).max(8);
	let terminal_leg = clearance.max(MIN_TERMINAL_SEGMENT_PX);
	let cell_px = rem_px.max(8);
	let envelope = routing_envelope(symbol_bboxes, source_bbox, target_bbox);

	let start_anchor =
		preferred_anchor_point(source_bbox, target_bbox.center(), source_anchor_bias);
	let end_anchor = preferred_anchor_point(target_bbox, source_bbox.center(), target_anchor_bias);
	let start = clamp_point_to_envelope(
		push_outside_bbox(source_bbox, start_anchor, terminal_leg),
		&envelope,
	);
	let end = clamp_point_to_envelope(
		push_outside_bbox(target_bbox, end_anchor, terminal_leg),
		&envelope,
	);
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();

	let node_blocks: Vec<geom::RectI> = node_obstacles
		.iter()
		.copied()
		.filter(|o| {
			!rect_contains_point(*o, source_center) && !rect_contains_point(*o, target_center)
		})
		.collect();
	let edge_blocks: Vec<geom::RectI> = edge_obstacles.to_vec();
	let detour_blocks = nearby_obstacles_for_detours(start, end, node_blocks.as_slice(), clearance);

	if let Some(points) = straight_adjacent_route(
		source_bbox,
		target_bbox,
		node_blocks.as_slice(),
		edge_blocks.as_slice(),
		(clearance / 2).max(1),
	) {
		let arrow = arrowhead(points.as_slice(), rem_px as f32);
		let bounds = bounds_for_points(points.as_slice())
			.union_point(arrow[0])
			.union_point(arrow[1])
			.union_point(arrow[2]);
		return Ok(Route {
			points,
			arrow,
			bounds,
		});
	}

	let mut candidates = orthogonal_candidates(start, end, source_bbox, target_bbox, clearance);
	candidates.extend(obstacle_detour_candidates(
		start,
		end,
		source_bbox,
		target_bbox,
		detour_blocks.as_slice(),
		clearance,
	));
	let candidates = clamp_candidates_to_envelope(candidates, &envelope);

	let mut best = select_best_candidate(
		candidates.as_slice(),
		&node_blocks,
		&edge_blocks,
		clearance,
		true,
	);

	if best.is_none() {
		best = grid_route(
			start,
			end,
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			cell_px,
			clearance,
			&envelope,
			true,
		);
	}

	if best.is_none() {
		best = select_best_candidate(
			candidates.as_slice(),
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			clearance,
			false,
		);
	}

	if best.is_none() {
		best = grid_route(
			start,
			end,
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			cell_px,
			clearance,
			&envelope,
			false,
		);
	}

	if best.is_none() {
		best = select_best_candidate(
			candidates.as_slice(),
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			clearance,
			false,
		);
	}

	if best.is_none() {
		best = grid_route(
			start,
			end,
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			cell_px,
			clearance,
			&envelope,
			false,
		);
	}

	let mut core = best.unwrap_or_else(|| orthogonal_fallback_route(start, end));
	let simplify_pad = clearance.max((rem_px / 2).max(6));
	if polyline_hits_obstacles(core.as_slice(), node_blocks.as_slice(), clearance / 2) {
		if let Some(recovery) = select_best_candidate(
			emergency_detour_candidates(
				start,
				end,
				source_bbox,
				target_bbox,
				detour_blocks.as_slice(),
				clearance,
			)
			.as_slice(),
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			clearance,
			true,
		) {
			core = recovery;
		}
	}
	core = simplify_stair_jogs(
		core,
		node_blocks.as_slice(),
		edge_blocks.as_slice(),
		simplify_pad,
	);
	let mut points = Vec::with_capacity(core.len() + 2);
	points.push(start_anchor);
	points.extend(core);
	points.push(end_anchor);
	let points = simplify_stair_jogs(points, node_blocks.as_slice(), &[], simplify_pad);
	let points = clamp_route_to_envelope(compress_polyline(points), &envelope);
	let points = simplify_stair_jogs(
		points,
		node_blocks.as_slice(),
		&[],
		(simplify_pad / 2).max(4),
	);
	let points = clamp_route_to_envelope(compress_polyline(points), &envelope);

	let arrow = arrowhead(points.as_slice(), rem_px as f32);
	let bounds = bounds_for_points(points.as_slice())
		.union_point(arrow[0])
		.union_point(arrow[1])
		.union_point(arrow[2]);

	Ok(Route {
		points,
		arrow,
		bounds,
	})
}

fn nearby_obstacles_for_detours(
	start: geom::PointF,
	end: geom::PointF,
	node_obstacles: &[geom::RectI],
	clearance: i32,
) -> Vec<geom::RectI> {
	if node_obstacles.is_empty() {
		return Vec::new();
	}

	let corridor = segment_rect(start, end, (clearance * 6).max(120));
	let mut nearby: Vec<geom::RectI> = node_obstacles
		.iter()
		.copied()
		.filter(|obstacle| rect_intersects(*obstacle, corridor))
		.collect();

	if nearby.is_empty() {
		nearby.extend_from_slice(node_obstacles);
	}

	nearby
}

#[derive(Debug, Default, Clone)]
pub struct EdgeRenderLayers {
	pub base: String,
	pub overlay: String,
}

pub fn render_edge_layers(route: &Route, style: EdgeStyle) -> EdgeRenderLayers {
	let mut layers = EdgeRenderLayers::default();
	match style {
		EdgeStyle::Orthogonal | EdgeStyle::Curved => {
			render_orthogonal_with_underlay(&mut layers, route)
		}
	}

	layers.overlay.push_str(&format!(
		"<path d=\"M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Z\" style=\"fill:{};stroke:none;\" />",
		route.arrow[0].x,
		route.arrow[0].y,
		route.arrow[1].x,
		route.arrow[1].y,
		route.arrow[2].x,
		route.arrow[2].y,
		EDGE_STROKE
	));
	layers
}

fn render_orthogonal_with_underlay(layers: &mut EdgeRenderLayers, route: &Route) {
	let stroke_points = route_stroke_points(route);
	if stroke_points.len() < 2 {
		return;
	}
	let d = path_polyline(stroke_points.as_slice());
	push_stroke_path(
		&mut layers.base,
		d.as_str(),
		EDGE_MASK_STROKE,
		EDGE_MASK_STROKE_WIDTH_PX,
	);
	push_stroke_path(
		&mut layers.base,
		d.as_str(),
		EDGE_STROKE,
		EDGE_STROKE_WIDTH_PX,
	);
}

fn path_polyline(points: &[geom::PointF]) -> String {
	let mut d = String::new();
	if let Some(first) = points.first() {
		d.push_str(&format!("M {:.2} {:.2}", first.x, first.y));
	}
	for point in points.iter().skip(1) {
		d.push_str(&format!(" L {:.2} {:.2}", point.x, point.y));
	}
	d
}

fn route_stroke_points(route: &Route) -> Vec<geom::PointF> {
	let mut points = route.points.clone();
	if points.len() < 2 {
		return points;
	}
	let base = geom::PointF {
		x: (route.arrow[1].x + route.arrow[2].x) / 2.0,
		y: (route.arrow[1].y + route.arrow[2].y) / 2.0,
	};
	if let Some(last) = points.last_mut() {
		*last = base;
	}
	compress_polyline(points)
}

fn push_stroke_path(out: &mut String, d: &str, stroke: &str, width_px: i32) {
	out.push_str(&format!(
		"<path d=\"{}\" style=\"fill:none;stroke:{};stroke-width:{}px;stroke-linecap:round;stroke-linejoin:round;\" />",
		d, stroke, width_px
	));
}

pub fn route_obstacles_for_later_edges(
	route: &Route,
	_cell_px: i32,
	_source_bbox: &geom::RectI,
	_target_bbox: &geom::RectI,
	padding_px: i32,
) -> Vec<geom::RectI> {
	let mut out: Vec<geom::RectI> = Vec::new();

	for seg in route.points.windows(2) {
		let a = seg[0];
		let b = seg[1];
		let rect = segment_rect(a, b, padding_px.max(1));
		out.push(rect);
	}

	out
}

fn orthogonal_fallback_route(start: geom::PointF, end: geom::PointF) -> Vec<geom::PointF> {
	if (start.x - end.x).abs() < 0.01 || (start.y - end.y).abs() < 0.01 {
		vec![start, end]
	} else {
		let mid_y = ((start.y + end.y) / 2.0).round();
		vec![
			start,
			geom::PointF {
				x: start.x,
				y: mid_y,
			},
			geom::PointF { x: end.x, y: mid_y },
			end,
		]
	}
}

fn straight_adjacent_route(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> Option<Vec<geom::PointF>> {
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();
	let horizontal_overlap = ((source_bbox.y + source_bbox.h).min(target_bbox.y + target_bbox.h)
		- source_bbox.y.max(target_bbox.y)) as f32;
	let vertical_overlap = ((source_bbox.x + source_bbox.w).min(target_bbox.x + target_bbox.w)
		- source_bbox.x.max(target_bbox.x)) as f32;

	let horizontal_gap = if source_center.x <= target_center.x {
		target_bbox.x as f32 - (source_bbox.x + source_bbox.w) as f32
	} else {
		source_bbox.x as f32 - (target_bbox.x + target_bbox.w) as f32
	};
	if horizontal_overlap > 0.0
		&& horizontal_gap >= 0.0
		&& horizontal_gap <= ADJACENT_STRAIGHT_GAP_PX as f32
	{
		let (start, end) = if source_center.x <= target_center.x {
			(
				geom::PointF {
					x: (source_bbox.x + source_bbox.w) as f32,
					y: source_center.y,
				},
				geom::PointF {
					x: target_bbox.x as f32,
					y: target_center.y,
				},
			)
		} else {
			(
				geom::PointF {
					x: source_bbox.x as f32,
					y: source_center.y,
				},
				geom::PointF {
					x: (target_bbox.x + target_bbox.w) as f32,
					y: target_center.y,
				},
			)
		};
		if segment_clear(start, end, node_obstacles, edge_obstacles, pad) {
			return Some(vec![start, end]);
		}
	}

	let vertical_gap = if source_center.y <= target_center.y {
		target_bbox.y as f32 - (source_bbox.y + source_bbox.h) as f32
	} else {
		source_bbox.y as f32 - (target_bbox.y + target_bbox.h) as f32
	};
	if vertical_overlap > 0.0
		&& vertical_gap >= 0.0
		&& vertical_gap <= ADJACENT_STRAIGHT_GAP_PX as f32
	{
		let (start, end) = if source_center.y <= target_center.y {
			(
				geom::PointF {
					x: source_center.x,
					y: (source_bbox.y + source_bbox.h) as f32,
				},
				geom::PointF {
					x: target_center.x,
					y: target_bbox.y as f32,
				},
			)
		} else {
			(
				geom::PointF {
					x: source_center.x,
					y: source_bbox.y as f32,
				},
				geom::PointF {
					x: target_center.x,
					y: (target_bbox.y + target_bbox.h) as f32,
				},
			)
		};
		if segment_clear(start, end, node_obstacles, edge_obstacles, pad) {
			return Some(vec![start, end]);
		}
	}

	None
}

fn orthogonal_candidates(
	start: geom::PointF,
	end: geom::PointF,
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	clearance: i32,
) -> Vec<Vec<geom::PointF>> {
	let mut out: Vec<Vec<geom::PointF>> = Vec::new();

	out.push(vec![
		start,
		geom::PointF {
			x: end.x,
			y: start.y,
		},
		end,
	]);
	out.push(vec![
		start,
		geom::PointF {
			x: start.x,
			y: end.y,
		},
		end,
	]);

	let mut x_candidates = vec![
		source_bbox.x - clearance,
		source_bbox.x + source_bbox.w + clearance,
		target_bbox.x - clearance,
		target_bbox.x + target_bbox.w + clearance,
		((start.x + end.x) / 2.0).round() as i32,
	];
	x_candidates.sort();
	x_candidates.dedup();
	for x in x_candidates {
		let xf = x as f32;
		out.push(vec![
			start,
			geom::PointF { x: xf, y: start.y },
			geom::PointF { x: xf, y: end.y },
			end,
		]);
	}

	let mut y_candidates = vec![
		source_bbox.y - clearance,
		source_bbox.y + source_bbox.h + clearance,
		target_bbox.y - clearance,
		target_bbox.y + target_bbox.h + clearance,
		((start.y + end.y) / 2.0).round() as i32,
	];
	y_candidates.sort();
	y_candidates.dedup();
	for y in y_candidates {
		let yf = y as f32;
		out.push(vec![
			start,
			geom::PointF { x: start.x, y: yf },
			geom::PointF { x: end.x, y: yf },
			end,
		]);
	}

	out
}

fn obstacle_detour_candidates(
	start: geom::PointF,
	end: geom::PointF,
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	clearance: i32,
) -> Vec<Vec<geom::PointF>> {
	if node_obstacles.is_empty() {
		return Vec::new();
	}

	let min_y = node_obstacles.iter().map(|o| o.y).min().unwrap_or(0);
	let max_y = node_obstacles.iter().map(|o| o.y + o.h).max().unwrap_or(0);
	let lift = (clearance * 2).max(16) as f32;
	let top_y = min_y as f32 - lift;
	let bottom_y = max_y as f32 + lift;
	let lane_pad = (clearance * 2).max(16) as f32;

	let mut approach_xs = vec![
		source_bbox.x as f32 - lane_pad,
		(source_bbox.x + source_bbox.w) as f32 + lane_pad,
		target_bbox.x as f32 - lane_pad,
		(target_bbox.x + target_bbox.w) as f32 + lane_pad,
	];
	approach_xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
	approach_xs.dedup_by(|a, b| (*a - *b).abs() < 0.01);

	let mut out: Vec<Vec<geom::PointF>> = vec![
		vec![
			start,
			geom::PointF {
				x: start.x,
				y: top_y,
			},
			geom::PointF { x: end.x, y: top_y },
			end,
		],
		vec![
			start,
			geom::PointF {
				x: start.x,
				y: bottom_y,
			},
			geom::PointF {
				x: end.x,
				y: bottom_y,
			},
			end,
		],
	];

	for approach_x in approach_xs {
		out.push(vec![
			start,
			geom::PointF {
				x: start.x,
				y: top_y,
			},
			geom::PointF {
				x: approach_x,
				y: top_y,
			},
			geom::PointF {
				x: approach_x,
				y: end.y,
			},
			end,
		]);
		out.push(vec![
			start,
			geom::PointF {
				x: start.x,
				y: bottom_y,
			},
			geom::PointF {
				x: approach_x,
				y: bottom_y,
			},
			geom::PointF {
				x: approach_x,
				y: end.y,
			},
			end,
		]);
	}

	out
}

fn emergency_detour_candidates(
	start: geom::PointF,
	end: geom::PointF,
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	clearance: i32,
) -> Vec<Vec<geom::PointF>> {
	if node_obstacles.is_empty() {
		return Vec::new();
	}

	let min_x = node_obstacles.iter().map(|o| o.x).min().unwrap_or(0) as f32;
	let max_x = node_obstacles.iter().map(|o| o.x + o.w).max().unwrap_or(0) as f32;
	let min_y = node_obstacles.iter().map(|o| o.y).min().unwrap_or(0) as f32;
	let max_y = node_obstacles.iter().map(|o| o.y + o.h).max().unwrap_or(0) as f32;

	let lane_pad = (clearance * 3).max(24) as f32;
	let y_lanes = [min_y - lane_pad, max_y + lane_pad];
	let mut x_lanes = vec![
		min_x - lane_pad,
		max_x + lane_pad,
		source_bbox.x as f32 - lane_pad,
		(source_bbox.x + source_bbox.w) as f32 + lane_pad,
		target_bbox.x as f32 - lane_pad,
		(target_bbox.x + target_bbox.w) as f32 + lane_pad,
	];
	x_lanes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
	x_lanes.dedup_by(|a, b| (*a - *b).abs() < 0.01);

	let mut out = Vec::new();
	for lane_y in y_lanes {
		for lane_x in x_lanes.iter().copied() {
			out.push(vec![
				start,
				geom::PointF {
					x: start.x,
					y: lane_y,
				},
				geom::PointF {
					x: lane_x,
					y: lane_y,
				},
				geom::PointF {
					x: lane_x,
					y: end.y,
				},
				end,
			]);
		}
	}
	out
}

fn select_best_candidate(
	candidates: &[Vec<geom::PointF>],
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	clearance: i32,
	strict_no_overlap: bool,
) -> Option<Vec<geom::PointF>> {
	let mut best: Option<(i32, Vec<geom::PointF>)> = None;

	for candidate in candidates {
		let compressed = compress_polyline(candidate.clone());
		if compressed.len() < 2 {
			continue;
		}
		let score = score_polyline(compressed.as_slice());
		let mut node_hits = 0;
		let mut edge_hits = 0;
		for seg in compressed.windows(2) {
			if segment_hits_obstacles(seg[0], seg[1], node_obstacles, clearance / 2) {
				node_hits += 1;
			}
			if segment_hits_obstacles(seg[0], seg[1], edge_obstacles, clearance / 2) {
				edge_hits += 1;
			}
		}
		if node_hits > 0 {
			continue;
		}
		if strict_no_overlap && edge_hits > 0 {
			continue;
		}
		let mut adjusted_score = score;
		if !strict_no_overlap {
			adjusted_score += edge_hits * 200;
		}

		match &best {
			Some((best_score, _)) if adjusted_score >= *best_score => {}
			_ => best = Some((adjusted_score, compressed)),
		}
	}

	best.map(|(_, route)| route)
}

fn score_polyline(points: &[geom::PointF]) -> i32 {
	if points.len() < 2 {
		return i32::MAX;
	}

	let mut distance = 0.0;
	for seg in points.windows(2) {
		distance += segment_length(seg[0], seg[1]);
	}

	let bends = (points.len() as i32).saturating_sub(2);
	(distance.round() as i32) + bends * 18 + route_deviation_penalty(points)
}

fn route_deviation_penalty(points: &[geom::PointF]) -> i32 {
	if points.len() < 2 {
		return 0;
	}

	let start = points[0];
	let end = *points.last().unwrap_or(&start);
	let min_x = start.x.min(end.x);
	let max_x = start.x.max(end.x);
	let min_y = start.y.min(end.y);
	let max_y = start.y.max(end.y);
	let allowance = 160.0;

	let mut penalty = 0.0f32;
	for point in points {
		if point.x < min_x - allowance {
			penalty += (min_x - allowance) - point.x;
		}
		if point.x > max_x + allowance {
			penalty += point.x - (max_x + allowance);
		}
		if point.y < min_y - allowance {
			penalty += (min_y - allowance) - point.y;
		}
		if point.y > max_y + allowance {
			penalty += point.y - (max_y + allowance);
		}
	}

	(penalty * 0.5).round() as i32
}

fn polyline_hits_obstacles(points: &[geom::PointF], obstacles: &[geom::RectI], pad: i32) -> bool {
	for seg in points.windows(2) {
		if segment_hits_obstacles(seg[0], seg[1], obstacles, pad) {
			return true;
		}
	}
	false
}

fn grid_route(
	start: geom::PointF,
	end: geom::PointF,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	cell_px: i32,
	clearance: i32,
	envelope: &RoutingEnvelope,
	strict_no_overlap: bool,
) -> Option<Vec<geom::PointF>> {
	let start_cell = point_to_cell(start, cell_px);
	let goal_cell = point_to_cell(end, cell_px);

	let hard_blocked = blocked_cells(node_obstacles, cell_px, (clearance / 2).max(1));
	let soft_blocked = blocked_cells(edge_obstacles, cell_px, (clearance / 3).max(1));
	if hard_blocked.contains(&start_cell) || hard_blocked.contains(&goal_cell) {
		return None;
	}

	let bounds = grid_bounds(start_cell, goal_cell, cell_px, envelope);
	let cells = astar(
		start_cell,
		goal_cell,
		&hard_blocked,
		&soft_blocked,
		&bounds,
		strict_no_overlap,
	)?;

	let mut points: Vec<geom::PointF> = cells.iter().map(|c| cell_center(*c, cell_px)).collect();
	if points.is_empty() {
		return None;
	}
	if let Some(first) = points.first_mut() {
		*first = start;
	}
	if let Some(last) = points.last_mut() {
		*last = end;
	}
	Some(compress_polyline(points))
}

fn astar(
	start: Cell,
	goal: Cell,
	hard_blocked: &HashSet<Cell>,
	soft_blocked: &HashSet<Cell>,
	bounds: &GridBounds,
	strict_no_overlap: bool,
) -> Option<Vec<Cell>> {
	let move_cost: i32 = 10;
	let turn_penalty: i32 = 6;
	let soft_penalty: i32 = 18;
	let neighbors: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

	let mut open: BinaryHeap<HeapItem> = BinaryHeap::new();
	let mut came_from: HashMap<State, State> = HashMap::new();
	let mut g_score: HashMap<State, i32> = HashMap::new();

	let start_state = State {
		cell: start,
		dir: DIR_NONE,
	};
	g_score.insert(start_state, 0);
	open.push(HeapItem {
		f: heuristic(start, goal),
		g: 0,
		state: start_state,
	});

	while let Some(current) = open.pop() {
		if current.state.cell == goal {
			return Some(reconstruct_path(came_from, current.state));
		}

		for (dx, dy) in neighbors {
			let next_cell = Cell {
				x: current.state.cell.x + dx,
				y: current.state.cell.y + dy,
			};
			if next_cell.x < bounds.min_x
				|| next_cell.x > bounds.max_x
				|| next_cell.y < bounds.min_y
				|| next_cell.y > bounds.max_y
			{
				continue;
			}
			if hard_blocked.contains(&next_cell) {
				continue;
			}
			if strict_no_overlap && soft_blocked.contains(&next_cell) {
				continue;
			}

			let mut step_cost = move_cost;
			let next_dir = Dir {
				dx: dx as i8,
				dy: dy as i8,
			};
			if current.state.dir != DIR_NONE && current.state.dir != next_dir {
				step_cost += turn_penalty;
			}
			if !strict_no_overlap && soft_blocked.contains(&next_cell) {
				step_cost += soft_penalty;
			}

			let next_state = State {
				cell: next_cell,
				dir: next_dir,
			};
			let tentative_g = current.g + step_cost;
			if g_score
				.get(&next_state)
				.is_some_and(|existing| tentative_g >= *existing)
			{
				continue;
			}

			came_from.insert(next_state, current.state);
			g_score.insert(next_state, tentative_g);
			open.push(HeapItem {
				f: tentative_g + heuristic(next_cell, goal),
				g: tentative_g,
				state: next_state,
			});
		}
	}

	None
}

fn heuristic(a: Cell, b: Cell) -> i32 {
	((a.x - b.x).abs() + (a.y - b.y).abs()) * 10
}

fn reconstruct_path(mut came_from: HashMap<State, State>, mut current: State) -> Vec<Cell> {
	let mut path = vec![current.cell];
	while let Some(prev) = came_from.remove(&current) {
		current = prev;
		path.push(current.cell);
	}
	path.reverse();
	path
}

fn grid_bounds(start: Cell, goal: Cell, cell_px: i32, envelope: &RoutingEnvelope) -> GridBounds {
	let mut min_x = (envelope.min_x / (cell_px as f32)).floor() as i32;
	let mut min_y = (envelope.min_y / (cell_px as f32)).floor() as i32;
	let mut max_x = (envelope.max_x / (cell_px as f32)).ceil() as i32;
	let mut max_y = (envelope.max_y / (cell_px as f32)).ceil() as i32;

	min_x = min_x.min(start.x.min(goal.x));
	min_y = min_y.min(start.y.min(goal.y));
	max_x = max_x.max(start.x.max(goal.x));
	max_y = max_y.max(start.y.max(goal.y));

	GridBounds {
		min_x,
		max_x,
		min_y,
		max_y,
	}
}

fn blocked_cells(obstacles: &[geom::RectI], cell_px: i32, pad: i32) -> HashSet<Cell> {
	let mut blocked = HashSet::new();
	for o in obstacles {
		let grown = grow_rect(*o, pad);
		let min_x = (grown.x / cell_px) - 1;
		let max_x = ((grown.x + grown.w) / cell_px) + 1;
		let min_y = (grown.y / cell_px) - 1;
		let max_y = ((grown.y + grown.h) / cell_px) + 1;
		for x in min_x..=max_x {
			for y in min_y..=max_y {
				blocked.insert(Cell { x, y });
			}
		}
	}
	blocked
}

fn point_to_cell(p: geom::PointF, cell_px: i32) -> Cell {
	Cell {
		x: (p.x / cell_px as f32).round() as i32,
		y: (p.y / cell_px as f32).round() as i32,
	}
}

fn cell_center(c: Cell, cell_px: i32) -> geom::PointF {
	geom::PointF {
		x: (c.x as f32) * (cell_px as f32),
		y: (c.y as f32) * (cell_px as f32),
	}
}

fn push_outside_bbox(bbox: &geom::RectI, anchor: geom::PointF, clearance: i32) -> geom::PointF {
	let x0 = bbox.x as f32;
	let x1 = (bbox.x + bbox.w) as f32;
	let y0 = bbox.y as f32;
	let y1 = (bbox.y + bbox.h) as f32;
	let c = clearance as f32;
	let eps = 0.01;

	if (anchor.x - x0).abs() <= eps {
		return geom::PointF {
			x: anchor.x - c,
			y: anchor.y,
		};
	}
	if (anchor.x - x1).abs() <= eps {
		return geom::PointF {
			x: anchor.x + c,
			y: anchor.y,
		};
	}
	if (anchor.y - y0).abs() <= eps {
		return geom::PointF {
			x: anchor.x,
			y: anchor.y - c,
		};
	}
	if (anchor.y - y1).abs() <= eps {
		return geom::PointF {
			x: anchor.x,
			y: anchor.y + c,
		};
	}

	let center = bbox.center();
	if (anchor.x - center.x).abs() >= (anchor.y - center.y).abs() {
		geom::PointF {
			x: if anchor.x >= center.x {
				anchor.x + c
			} else {
				anchor.x - c
			},
			y: anchor.y,
		}
	} else {
		geom::PointF {
			x: anchor.x,
			y: if anchor.y >= center.y {
				anchor.y + c
			} else {
				anchor.y - c
			},
		}
	}
}

fn preferred_anchor_point(from: &geom::RectI, to: geom::PointF, bias: f32) -> geom::PointF {
	let c = from.center();
	let x0 = from.x as f32;
	let x1 = (from.x + from.w) as f32;
	let y0 = from.y as f32;
	let y1 = (from.y + from.h) as f32;
	let dx = to.x - c.x;
	let dy = to.y - c.y;
	let near_same_row = dy.abs() < (from.h as f32 * 0.2);
	let strongly_horizontal = dx.abs() > dy.abs() * 2.0;
	let allow_side_anchor = near_same_row && strongly_horizontal;

	if !allow_side_anchor {
		let y = if dy >= 0.0 { y1 } else { y0 };
		let x =
			weighted_anchor_coordinate(x0, x1, SYMBOL_PARALLEL_EDGE_CLEARANCE_PX, to.x, c.x, bias);
		return geom::PointF { x, y };
	}

	let x = if dx >= 0.0 { x1 } else { x0 };
	let y = weighted_anchor_coordinate(y0, y1, SYMBOL_PARALLEL_EDGE_CLEARANCE_PX, to.y, c.y, bias);
	geom::PointF { x, y }
}

fn weighted_anchor_coordinate(
	min: f32,
	max: f32,
	margin: f32,
	_preferred: f32,
	_center_hint: f32,
	bias: f32,
) -> f32 {
	let low = min + margin;
	let high = max - margin;
	if low > high {
		return (min + max) / 2.0;
	}

	let center = (low + high) / 2.0;
	let spread = (high - low) / 2.0;
	let biased = center + bias.clamp(-1.0, 1.0) * spread * SYMBOL_ANCHOR_BIAS_SCALE;
	biased.clamp(low, high)
}

fn segment_rect(a: geom::PointF, b: geom::PointF, pad: i32) -> geom::RectI {
	let p = pad.max(0) as f32;
	let min_x = (a.x.min(b.x) - p).floor() as i32;
	let min_y = (a.y.min(b.y) - p).floor() as i32;
	let max_x = (a.x.max(b.x) + p).ceil() as i32;
	let max_y = (a.y.max(b.y) + p).ceil() as i32;
	geom::RectI {
		x: min_x,
		y: min_y,
		w: (max_x - min_x).max(1),
		h: (max_y - min_y).max(1),
	}
}

fn segment_length(a: geom::PointF, b: geom::PointF) -> f32 {
	let dx = b.x - a.x;
	let dy = b.y - a.y;
	(dx * dx + dy * dy).sqrt()
}

fn compress_polyline(points: Vec<geom::PointF>) -> Vec<geom::PointF> {
	if points.len() <= 2 {
		return points;
	}
	const EPS: f32 = 0.01;

	let mut deduped: Vec<geom::PointF> = Vec::new();
	for p in points {
		if deduped
			.last()
			.is_some_and(|last| (last.x - p.x).abs() < EPS && (last.y - p.y).abs() < EPS)
		{
			continue;
		}
		deduped.push(p);
	}

	if deduped.len() <= 2 {
		return deduped;
	}

	let mut out: Vec<geom::PointF> = vec![deduped[0], deduped[1]];
	for p in deduped.iter().skip(2).copied() {
		let a = out[out.len() - 2];
		let b = out[out.len() - 1];
		let same_x = (a.x - b.x).abs() < EPS && (b.x - p.x).abs() < EPS;
		let same_y = (a.y - b.y).abs() < EPS && (b.y - p.y).abs() < EPS;
		if same_x || same_y {
			if let Some(last) = out.last_mut() {
				*last = p;
			}
		} else {
			out.push(p);
		}
	}
	out
}

fn simplify_stair_jogs(
	points: Vec<geom::PointF>,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> Vec<geom::PointF> {
	let mut out = compress_polyline(points);
	if out.len() < 4 {
		return out;
	}

	let check_pad = (pad / 2).max(1);
	let jog_limit = (pad.max(2) as f32) * 2.5;
	let mut changed = true;
	while changed {
		changed = false;
		if out.len() < 4 {
			break;
		}

		let mut i = 0usize;
		while i + 3 < out.len() {
			let a = out[i];
			let b = out[i + 1];
			let c = out[i + 2];
			let d = out[i + 3];

			if let Some(pivot) = replacement_pivot(
				a,
				b,
				c,
				d,
				jog_limit,
				node_obstacles,
				edge_obstacles,
				check_pad,
			) {
				out.splice(i + 1..=i + 2, [pivot]);
				changed = true;
				i = i.saturating_sub(1);
				continue;
			}

			i += 1;
		}
	}

	out = axis_shortcuts(out, node_obstacles, edge_obstacles, check_pad);
	compress_polyline(out)
}

fn axis_shortcuts(
	points: Vec<geom::PointF>,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> Vec<geom::PointF> {
	if points.len() < 4 {
		return points;
	}

	let mut out: Vec<geom::PointF> = vec![points[0]];
	let mut i = 0usize;
	while i < points.len() - 1 {
		let from = *out.last().unwrap_or(&points[i]);
		let mut advanced = false;
		for j in (i + 2..points.len()).rev() {
			let to = points[j];
			if !is_axis_aligned(from, to) {
				continue;
			}
			if !segment_clear(from, to, node_obstacles, edge_obstacles, pad) {
				continue;
			}
			out.push(to);
			i = j;
			advanced = true;
			break;
		}
		if advanced {
			continue;
		}
		out.push(points[i + 1]);
		i += 1;
	}

	compress_polyline(out)
}

fn replacement_pivot(
	a: geom::PointF,
	b: geom::PointF,
	c: geom::PointF,
	d: geom::PointF,
	jog_limit: f32,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> Option<geom::PointF> {
	let bc_len = segment_length(b, c);
	if bc_len > jog_limit {
		return None;
	}

	if is_horizontal(a, b)
		&& is_vertical(b, c)
		&& is_horizontal(c, d)
		&& same_direction_sign(b.x - a.x, d.x - c.x)
	{
		let preferred = geom::PointF { x: d.x, y: a.y };
		let fallback = geom::PointF { x: a.x, y: d.y };
		return best_pivot(
			a,
			d,
			preferred,
			fallback,
			node_obstacles,
			edge_obstacles,
			pad,
		);
	}

	if is_vertical(a, b)
		&& is_horizontal(b, c)
		&& is_vertical(c, d)
		&& same_direction_sign(b.y - a.y, d.y - c.y)
	{
		let preferred = geom::PointF { x: a.x, y: d.y };
		let fallback = geom::PointF { x: d.x, y: a.y };
		return best_pivot(
			a,
			d,
			preferred,
			fallback,
			node_obstacles,
			edge_obstacles,
			pad,
		);
	}

	None
}

fn best_pivot(
	a: geom::PointF,
	d: geom::PointF,
	preferred: geom::PointF,
	fallback: geom::PointF,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> Option<geom::PointF> {
	for pivot in [preferred, fallback] {
		if !is_axis_aligned(a, pivot) || !is_axis_aligned(pivot, d) {
			continue;
		}
		if !segment_clear(a, pivot, node_obstacles, edge_obstacles, pad) {
			continue;
		}
		if !segment_clear(pivot, d, node_obstacles, edge_obstacles, pad) {
			continue;
		}
		return Some(pivot);
	}

	None
}

fn segment_clear(
	a: geom::PointF,
	b: geom::PointF,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	pad: i32,
) -> bool {
	!segment_hits_obstacles(a, b, node_obstacles, pad)
		&& !segment_hits_obstacles(a, b, edge_obstacles, pad)
}

fn is_axis_aligned(a: geom::PointF, b: geom::PointF) -> bool {
	(a.x - b.x).abs() < 0.01 || (a.y - b.y).abs() < 0.01
}

fn is_horizontal(a: geom::PointF, b: geom::PointF) -> bool {
	(a.y - b.y).abs() < 0.01 && (a.x - b.x).abs() >= 0.01
}

fn is_vertical(a: geom::PointF, b: geom::PointF) -> bool {
	(a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() >= 0.01
}

fn same_direction_sign(a: f32, b: f32) -> bool {
	a.abs() > 0.01 && b.abs() > 0.01 && a.signum() == b.signum()
}

#[cfg(test)]
fn polyline_clear(route: &[geom::PointF], obstacles: &[geom::RectI], pad: i32) -> bool {
	for seg in route.windows(2) {
		if segment_hits_obstacles(seg[0], seg[1], obstacles, pad) {
			return false;
		}
	}
	true
}

fn segment_hits_obstacles(
	a: geom::PointF,
	b: geom::PointF,
	obstacles: &[geom::RectI],
	pad: i32,
) -> bool {
	for o in obstacles {
		if segment_intersects_rect(a, b, o, pad) {
			return true;
		}
	}
	false
}

fn segment_intersects_rect(a: geom::PointF, b: geom::PointF, r: &geom::RectI, pad: i32) -> bool {
	let grown = grow_rect(*r, pad.max(0));

	if (a.x - b.x).abs() < 0.01 {
		let x = a.x;
		let y_min = a.y.min(b.y);
		let y_max = a.y.max(b.y);
		let rx0 = grown.x as f32;
		let rx1 = (grown.x + grown.w) as f32;
		let ry0 = grown.y as f32;
		let ry1 = (grown.y + grown.h) as f32;
		return x >= rx0 && x <= rx1 && y_max >= ry0 && y_min <= ry1;
	}

	if (a.y - b.y).abs() < 0.01 {
		let y = a.y;
		let x_min = a.x.min(b.x);
		let x_max = a.x.max(b.x);
		let rx0 = grown.x as f32;
		let rx1 = (grown.x + grown.w) as f32;
		let ry0 = grown.y as f32;
		let ry1 = (grown.y + grown.h) as f32;
		return y >= ry0 && y <= ry1 && x_max >= rx0 && x_min <= rx1;
	}

	let seg_rect = segment_rect(a, b, pad.max(0));
	rect_intersects(seg_rect, grown)
}

fn rect_intersects(a: geom::RectI, b: geom::RectI) -> bool {
	let ax2 = a.x + a.w;
	let ay2 = a.y + a.h;
	let bx2 = b.x + b.w;
	let by2 = b.y + b.h;
	a.x < bx2 && ax2 > b.x && a.y < by2 && ay2 > b.y
}

fn rect_contains_point(rect: geom::RectI, point: geom::PointF) -> bool {
	point.x >= rect.x as f32
		&& point.x <= (rect.x + rect.w) as f32
		&& point.y >= rect.y as f32
		&& point.y <= (rect.y + rect.h) as f32
}

fn grow_rect(r: geom::RectI, pad: i32) -> geom::RectI {
	let p = pad.max(0);
	geom::RectI {
		x: r.x - p,
		y: r.y - p,
		w: r.w + 2 * p,
		h: r.h + 2 * p,
	}
}

fn routing_envelope(
	symbol_bboxes: &[geom::RectI],
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
) -> RoutingEnvelope {
	let mut min_x = source_bbox.x.min(target_bbox.x) as f32;
	let mut min_y = source_bbox.y.min(target_bbox.y) as f32;
	let mut max_x = (source_bbox.x + source_bbox.w).max(target_bbox.x + target_bbox.w) as f32;
	let mut max_y = (source_bbox.y + source_bbox.h).max(target_bbox.y + target_bbox.h) as f32;
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();
	let corridor_pad = ((source_bbox.h.max(target_bbox.h)) / 2).max(120);
	let corridor = grow_rect(
		segment_rect(source_center, target_center, corridor_pad),
		corridor_pad,
	);

	for symbol_bbox in symbol_bboxes {
		if !rect_intersects(*symbol_bbox, corridor) {
			continue;
		}
		min_x = min_x.min(symbol_bbox.x as f32);
		min_y = min_y.min(symbol_bbox.y as f32);
		max_x = max_x.max((symbol_bbox.x + symbol_bbox.w) as f32);
		max_y = max_y.max((symbol_bbox.y + symbol_bbox.h) as f32);
	}

	RoutingEnvelope {
		min_x,
		max_x,
		min_y,
		max_y,
	}
}

fn clamp_point_to_envelope(point: geom::PointF, envelope: &RoutingEnvelope) -> geom::PointF {
	geom::PointF {
		x: point.x.clamp(envelope.min_x, envelope.max_x),
		y: point.y.clamp(envelope.min_y, envelope.max_y),
	}
}

fn clamp_route_to_envelope(
	points: Vec<geom::PointF>,
	envelope: &RoutingEnvelope,
) -> Vec<geom::PointF> {
	let clamped: Vec<geom::PointF> = points
		.into_iter()
		.map(|point| clamp_point_to_envelope(point, envelope))
		.collect();
	compress_polyline(clamped)
}

fn clamp_candidates_to_envelope(
	candidates: Vec<Vec<geom::PointF>>,
	envelope: &RoutingEnvelope,
) -> Vec<Vec<geom::PointF>> {
	candidates
		.into_iter()
		.map(|candidate| clamp_route_to_envelope(candidate, envelope))
		.collect()
}

fn norm(v: geom::PointF) -> geom::PointF {
	let len = (v.x * v.x + v.y * v.y).sqrt();
	if len <= f32::EPSILON {
		geom::PointF { x: 0.0, y: 0.0 }
	} else {
		geom::PointF {
			x: v.x / len,
			y: v.y / len,
		}
	}
}

fn arrowhead(points: &[geom::PointF], min_size_px: f32) -> [geom::PointF; 3] {
	let _ = min_size_px;
	let size = ARROW_SIZE_PX;
	let width = size * 0.6;
	let end = *points.last().unwrap_or(&geom::PointF { x: 0.0, y: 0.0 });
	let dir = arrow_direction(points, (size * 0.75).max(8.0));
	let base = geom::PointF {
		x: end.x - dir.x * size,
		y: end.y - dir.y * size,
	};
	let perp = geom::PointF {
		x: -dir.y,
		y: dir.x,
	};
	let p1 = geom::PointF {
		x: base.x + perp.x * (width / 2.0),
		y: base.y + perp.y * (width / 2.0),
	};
	let p2 = geom::PointF {
		x: base.x - perp.x * (width / 2.0),
		y: base.y - perp.y * (width / 2.0),
	};
	[end, p1, p2]
}

fn arrow_direction(points: &[geom::PointF], _min_len: f32) -> geom::PointF {
	if points.len() < 2 {
		return geom::PointF { x: 1.0, y: 0.0 };
	}

	for seg in points.windows(2).rev() {
		let dir = norm(geom::PointF {
			x: seg[1].x - seg[0].x,
			y: seg[1].y - seg[0].y,
		});
		if dir.x.abs() > 0.0001 || dir.y.abs() > 0.0001 {
			return dir;
		}
	}

	geom::PointF { x: 1.0, y: 0.0 }
}

fn bounds_for_points(points: &[geom::PointF]) -> geom::Bounds {
	let mut b = geom::Bounds::empty();
	for p in points {
		b = b.union_point(*p);
	}
	b
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn candidate_route_can_avoid_middle_obstacle() {
		let start = geom::PointF { x: 0.0, y: 0.0 };
		let end = geom::PointF { x: 200.0, y: 0.0 };
		let source = geom::RectI {
			x: -80,
			y: -50,
			w: 80,
			h: 100,
		};
		let target = geom::RectI {
			x: 200,
			y: -50,
			w: 80,
			h: 100,
		};
		let obstacle = geom::RectI {
			x: 80,
			y: -40,
			w: 40,
			h: 80,
		};

		let candidates = orthogonal_candidates(start, end, &source, &target, 20);
		let best = select_best_candidate(candidates.as_slice(), &[obstacle], &[], 8, true)
			.expect("should find a route around obstacle");
		assert!(best.len() >= 3);
		assert!(polyline_clear(best.as_slice(), &[obstacle], 4));
	}

	#[test]
	fn grid_route_finds_path_when_candidates_fail() {
		let start = geom::PointF { x: 0.0, y: 0.0 };
		let end = geom::PointF { x: 160.0, y: 0.0 };
		let obstacle = geom::RectI {
			x: 40,
			y: -40,
			w: 80,
			h: 80,
		};
		let envelope = RoutingEnvelope {
			min_x: -40.0,
			max_x: 200.0,
			min_y: -80.0,
			max_y: 80.0,
		};
		let out = grid_route(start, end, &[obstacle], &[], 16, 8, &envelope, true)
			.expect("grid route should exist");
		assert!(out.len() >= 3);
		assert!(
			polyline_clear(out.as_slice(), &[obstacle], 4),
			"route intersected obstacle: {:?}",
			out
		);
	}

	#[test]
	fn route_obstacles_include_all_segments() {
		let route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 0.0 },
				geom::PointF { x: 50.0, y: 0.0 },
				geom::PointF { x: 100.0, y: 0.0 },
			],
			arrow: [
				geom::PointF { x: 100.0, y: 0.0 },
				geom::PointF { x: 90.0, y: 4.0 },
				geom::PointF { x: 90.0, y: -4.0 },
			],
			bounds: geom::Bounds::empty(),
		};
		let source = geom::RectI {
			x: -20,
			y: -20,
			w: 30,
			h: 40,
		};
		let target = geom::RectI {
			x: 80,
			y: -20,
			w: 30,
			h: 40,
		};
		let obstacles = route_obstacles_for_later_edges(&route, 16, &source, &target, 4);
		assert_eq!(obstacles.len(), 2);
	}

	#[test]
	fn clamped_route_stays_inside_envelope() {
		let envelope = RoutingEnvelope {
			min_x: 10.0,
			max_x: 50.0,
			min_y: 20.0,
			max_y: 60.0,
		};
		let points = vec![
			geom::PointF { x: -10.0, y: 30.0 },
			geom::PointF { x: 30.0, y: 100.0 },
			geom::PointF { x: 90.0, y: 40.0 },
		];
		let clamped = clamp_route_to_envelope(points, &envelope);
		assert!(clamped.iter().all(|point| {
			point.x >= envelope.min_x
				&& point.x <= envelope.max_x
				&& point.y >= envelope.min_y
				&& point.y <= envelope.max_y
		}));
	}

	#[test]
	fn route_obstacles_exclude_node_boxes() {
		let route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 0.0 },
				geom::PointF { x: 100.0, y: 0.0 },
			],
			arrow: [
				geom::PointF { x: 100.0, y: 0.0 },
				geom::PointF { x: 90.0, y: 4.0 },
				geom::PointF { x: 90.0, y: -4.0 },
			],
			bounds: geom::Bounds::empty(),
		};
		let source = geom::RectI {
			x: -30,
			y: -30,
			w: 60,
			h: 60,
		};
		let target = geom::RectI {
			x: 70,
			y: -30,
			w: 60,
			h: 60,
		};
		let obstacles = route_obstacles_for_later_edges(&route, 16, &source, &target, 6);
		assert_eq!(obstacles.len(), 1);
	}

	#[test]
	fn long_same_row_route_avoids_intermediate_nodes() {
		let source = geom::RectI {
			x: 0,
			y: 310,
			w: 728,
			h: 184,
		};
		let target = geom::RectI {
			x: 11476,
			y: 310,
			w: 728,
			h: 184,
		};

		let node_obstacles = vec![
			grow_rect(source, 16),
			grow_rect(target, 16),
			grow_rect(
				geom::RectI {
					x: 2452,
					y: 310,
					w: 716,
					h: 184,
				},
				16,
			),
			grow_rect(
				geom::RectI {
					x: 8968,
					y: 310,
					w: 736,
					h: 184,
				},
				16,
			),
			grow_rect(
				geom::RectI {
					x: 10620,
					y: 310,
					w: 744,
					h: 184,
				},
				16,
			),
			grow_rect(
				geom::RectI {
					x: 11464,
					y: 10,
					w: 744,
					h: 184,
				},
				16,
			),
		];

		let config = SvgConfig::default();
		let symbol_bboxes = vec![
			source,
			target,
			geom::RectI {
				x: 2452,
				y: 310,
				w: 716,
				h: 184,
			},
			geom::RectI {
				x: 8968,
				y: 310,
				w: 736,
				h: 184,
			},
			geom::RectI {
				x: 10620,
				y: 310,
				w: 744,
				h: 184,
			},
			geom::RectI {
				x: 11464,
				y: 10,
				w: 744,
				h: 184,
			},
		];
		let route = route_edge(
			&source,
			&target,
			&node_obstacles,
			&[],
			&symbol_bboxes,
			0.0,
			0.0,
			&config,
		)
		.expect("route");

		let blocking = vec![
			grow_rect(
				geom::RectI {
					x: 2452,
					y: 310,
					w: 716,
					h: 184,
				},
				16,
			),
			grow_rect(
				geom::RectI {
					x: 8968,
					y: 310,
					w: 736,
					h: 184,
				},
				16,
			),
			grow_rect(
				geom::RectI {
					x: 10620,
					y: 310,
					w: 744,
					h: 184,
				},
				16,
			),
		];

		assert!(
			polyline_clear(route.points.as_slice(), &blocking, 0),
			"route intersects row nodes: {:?}",
			route.points
		);
	}

	#[test]
	fn arrow_direction_prefers_terminal_segment() {
		let points = vec![
			geom::PointF { x: 0.0, y: 0.0 },
			geom::PointF { x: 20.0, y: 0.0 },
			geom::PointF { x: 10.0, y: 0.0 },
		];

		let direction = arrow_direction(points.as_slice(), 8.0);
		assert!(direction.x < -0.99, "unexpected direction: {direction:?}");
		assert!(
			direction.y.abs() < 0.01,
			"unexpected direction: {direction:?}"
		);
	}

	#[test]
	fn preferred_anchor_point_keeps_80px_parallel_clearance() {
		let node = geom::RectI {
			x: 100,
			y: 200,
			w: 720,
			h: 450,
		};
		let below = geom::PointF {
			x: 120.0,
			y: 1000.0,
		};
		let anchor = preferred_anchor_point(&node, below, 0.0);
		assert_eq!(anchor.y, (node.y + node.h) as f32);
		assert!(anchor.x >= (node.x as f32 + 80.0));
		assert!(anchor.x <= ((node.x + node.w) as f32 - 80.0));
	}

	#[test]
	fn preferred_anchor_point_applies_bias_but_stays_inside_clearance() {
		let node = geom::RectI {
			x: 100,
			y: 200,
			w: 720,
			h: 450,
		};
		let below = geom::PointF {
			x: 460.0,
			y: 1000.0,
		};
		let left_bias = preferred_anchor_point(&node, below, -1.0);
		let right_bias = preferred_anchor_point(&node, below, 1.0);
		assert!(left_bias.x < right_bias.x, "bias did not shift anchors");
		assert!(left_bias.x >= (node.x as f32 + 80.0));
		assert!(right_bias.x <= ((node.x + node.w) as f32 - 80.0));
	}

	#[test]
	fn route_edge_has_terminal_segment_at_least_80px() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 1200,
			y: 0,
			w: 720,
			h: 450,
		};
		let config = SvgConfig::default();
		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, target],
			0.0,
			0.0,
			&config,
		)
		.expect("route should exist");

		let segment = route
			.points
			.windows(2)
			.next_back()
			.expect("route should have terminal segment");
		let length = segment_length(segment[0], segment[1]);
		assert!(length >= 80.0, "terminal segment too short: {length}");
	}

	#[test]
	fn orthogonal_rendering_adds_white_underlay_for_vertical_segments() {
		let base_route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 0.0 },
				geom::PointF { x: 0.0, y: 200.0 },
			],
			arrow: [
				geom::PointF { x: 0.0, y: 200.0 },
				geom::PointF { x: -5.0, y: 190.0 },
				geom::PointF { x: 5.0, y: 190.0 },
			],
			bounds: geom::Bounds::empty(),
		};

		let layers = render_edge_layers(&base_route, EdgeStyle::Orthogonal);
		let svg = format!("{}{}", layers.base, layers.overlay);
		assert!(
			svg.contains("stroke:#ffffff;stroke-width:19px"),
			"expected white mask stroke, got: {svg}"
		);
		assert!(
			svg.contains("stroke:#000000;stroke-width:4px"),
			"expected foreground stroke, got: {svg}"
		);
	}

	#[test]
	fn orthogonal_rendering_adds_white_underlay_for_horizontal_segments() {
		let route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 50.0 },
				geom::PointF { x: 200.0, y: 50.0 },
			],
			arrow: [
				geom::PointF { x: 200.0, y: 50.0 },
				geom::PointF { x: 188.0, y: 56.0 },
				geom::PointF { x: 188.0, y: 44.0 },
			],
			bounds: geom::Bounds::empty(),
		};

		let layers = render_edge_layers(&route, EdgeStyle::Orthogonal);
		assert!(layers.base.contains("stroke:#ffffff;stroke-width:19px"));
		assert!(layers.base.contains("stroke:#000000;stroke-width:4px"));
	}

	#[test]
	fn adjacent_symbols_route_straight_line() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 880,
			y: 0,
			w: 720,
			h: 450,
		};
		let config = SvgConfig::default();
		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, target],
			0.0,
			0.0,
			&config,
		)
		.expect("adjacent route");

		assert_eq!(
			route.points.len(),
			2,
			"expected straight path: {:?}",
			route.points
		);
		assert!((route.points[0].y - route.points[1].y).abs() < 0.01);
	}

	#[test]
	fn orthogonal_rendering_stops_stroke_at_arrow_base() {
		let route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 0.0 },
				geom::PointF { x: 100.0, y: 0.0 },
			],
			arrow: [
				geom::PointF { x: 100.0, y: 0.0 },
				geom::PointF { x: 70.0, y: 9.0 },
				geom::PointF { x: 70.0, y: -9.0 },
			],
			bounds: geom::Bounds::empty(),
		};

		let layers = render_edge_layers(&route, EdgeStyle::Orthogonal);
		assert!(
			layers.base.contains("L 70.00 0.00"),
			"expected stroke to end at arrow base, got: {}",
			layers.base
		);
		assert!(
			!layers.base.contains("L 100.00 0.00"),
			"stroke should not continue through arrow tip"
		);
	}

	#[test]
	fn arrowhead_uses_fixed_30px_length() {
		let points = vec![
			geom::PointF { x: 0.0, y: 0.0 },
			geom::PointF { x: 100.0, y: 0.0 },
		];
		let arrow = arrowhead(points.as_slice(), 8.0);
		let side = segment_length(arrow[0], arrow[1]);
		assert!(
			side > 29.0,
			"arrow side should reflect fixed 30px length, got: {side}"
		);
	}
}
