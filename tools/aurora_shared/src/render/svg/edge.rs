use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::{EdgeStyle, RenderError, SvgConfig, geom};

const EDGE_STROKE: &str = "#000000";
const EDGE_STROKE_WIDTH_PX: i32 = 2;

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
	config: &SvgConfig,
) -> Result<Route, RenderError> {
	let rem_px = config.base_font_size_px.max(1);
	let clearance = (rem_px / 2).max(8);
	let cell_px = rem_px.max(8);
	let envelope = routing_envelope(symbol_bboxes, source_bbox, target_bbox);

	let start_anchor = anchor_point(source_bbox, target_bbox.center());
	let end_anchor = anchor_point(target_bbox, source_bbox.center());
	let start = clamp_point_to_envelope(
		push_outside_bbox(source_bbox, start_anchor, clearance),
		&envelope,
	);
	let end = clamp_point_to_envelope(
		push_outside_bbox(target_bbox, end_anchor, clearance),
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
	let terminal_merge_zone = grow_rect(*target_bbox, clearance * 3);
	let edge_blocks: Vec<geom::RectI> = edge_obstacles
		.iter()
		.copied()
		.filter(|obstacle| !rect_intersects(*obstacle, terminal_merge_zone))
		.collect();

	let mut candidates = orthogonal_candidates(start, end, source_bbox, target_bbox, clearance);
	candidates.extend(obstacle_detour_candidates(
		start,
		end,
		source_bbox,
		target_bbox,
		&node_blocks,
		clearance,
	));
	let fallback_straight = vec![start, end];
	candidates.push(fallback_straight);
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

	let mut core = best.unwrap_or_else(|| vec![start, end]);
	let simplify_pad = clearance.max((rem_px / 2).max(6));
	if polyline_hits_obstacles(core.as_slice(), node_blocks.as_slice(), clearance / 2) {
		if let Some(recovery) = select_best_candidate(
			emergency_detour_candidates(
				start,
				end,
				source_bbox,
				target_bbox,
				node_blocks.as_slice(),
				clearance,
			)
			.as_slice(),
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			clearance,
			true,
		) {
			core = recovery;
		} else if let Some(recovery) = select_best_candidate(
			emergency_detour_candidates(
				start,
				end,
				source_bbox,
				target_bbox,
				node_blocks.as_slice(),
				clearance,
			)
			.as_slice(),
			node_blocks.as_slice(),
			edge_blocks.as_slice(),
			clearance,
			false,
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

pub fn render_edge(route: &Route, style: EdgeStyle) -> String {
	let d = match style {
		EdgeStyle::Orthogonal => path_polyline(route.points.as_slice()),
		EdgeStyle::Curved => path_curved(route.points.as_slice()),
	};

	let mut out = String::new();
	out.push_str(&format!(
		"<path d=\"{}\" style=\"fill:none;stroke:{};stroke-width:{}px;\" />",
		d, EDGE_STROKE, EDGE_STROKE_WIDTH_PX
	));
	out.push_str(&format!(
		"<path d=\"M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Z\" style=\"fill:{};stroke:none;\" />",
		route.arrow[0].x,
		route.arrow[0].y,
		route.arrow[1].x,
		route.arrow[1].y,
		route.arrow[2].x,
		route.arrow[2].y,
		EDGE_STROKE
	));
	out
}

pub fn route_obstacles_for_later_edges(
	route: &Route,
	_cell_px: i32,
	_source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	padding_px: i32,
) -> Vec<geom::RectI> {
	let target_exclusion = grow_rect(*target_bbox, padding_px * 2);
	let mut out: Vec<geom::RectI> = Vec::new();
	let last_segment_index = route.points.len().saturating_sub(2);

	for (segment_index, seg) in route.points.windows(2).enumerate() {
		if segment_index == last_segment_index {
			continue;
		}
		let a = seg[0];
		let b = seg[1];
		let rect = segment_rect(a, b, padding_px.max(1));
		if rect_intersects(rect, target_exclusion) {
			continue;
		}
		out.push(rect);
	}

	out
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
			adjusted_score += edge_hits * 40;
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
	(distance.round() as i32) + bends * 18
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
	let mut min_x = ((envelope.min_x / (cell_px as f32)).floor() as i32) - 1;
	let mut min_y = ((envelope.min_y / (cell_px as f32)).floor() as i32) - 1;
	let mut max_x = ((envelope.max_x / (cell_px as f32)).ceil() as i32) + 1;
	let mut max_y = ((envelope.max_y / (cell_px as f32)).ceil() as i32) + 1;

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

fn anchor_point(from: &geom::RectI, to: geom::PointF) -> geom::PointF {
	let c = from.center();
	let dir = geom::PointF {
		x: to.x - c.x,
		y: to.y - c.y,
	};
	let eps = 0.000_1;
	if dir.x.abs() < eps && dir.y.abs() < eps {
		return geom::PointF {
			x: (from.x + from.w) as f32,
			y: c.y,
		};
	}

	let x0 = from.x as f32;
	let x1 = (from.x + from.w) as f32;
	let y0 = from.y as f32;
	let y1 = (from.y + from.h) as f32;

	let mut best_t: f32 = f32::INFINITY;
	let mut best: Option<geom::PointF> = None;

	if dir.x.abs() >= eps {
		for x in [x0, x1] {
			let t = (x - c.x) / dir.x;
			if t <= 0.0 {
				continue;
			}
			let y = c.y + dir.y * t;
			if y < y0 - 0.01 || y > y1 + 0.01 {
				continue;
			}
			if t < best_t {
				best_t = t;
				best = Some(geom::PointF { x, y });
			}
		}
	}

	if dir.y.abs() >= eps {
		for y in [y0, y1] {
			let t = (y - c.y) / dir.y;
			if t <= 0.0 {
				continue;
			}
			let x = c.x + dir.x * t;
			if x < x0 - 0.01 || x > x1 + 0.01 {
				continue;
			}
			if t < best_t {
				best_t = t;
				best = Some(geom::PointF { x, y });
			}
		}
	}

	best.unwrap_or(c)
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

	for symbol_bbox in symbol_bboxes {
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

fn path_polyline(points: &[geom::PointF]) -> String {
	let mut d = String::new();
	if let Some(first) = points.first() {
		d.push_str(&format!("M {:.2} {:.2}", first.x, first.y));
	}
	for p in points.iter().skip(1) {
		d.push_str(&format!(" L {:.2} {:.2}", p.x, p.y));
	}
	d
}

fn path_curved(points: &[geom::PointF]) -> String {
	if points.len() < 3 {
		return path_polyline(points);
	}

	let base_radius: f32 = 8.0;
	let k: f32 = 0.552_284_8;
	let last = points[points.len() - 1];
	let mut d = String::new();
	d.push_str(&format!("M {:.2} {:.2}", points[0].x, points[0].y));

	for i in 1..(points.len() - 1) {
		let p0 = points[i - 1];
		let p1 = points[i];
		let p2 = points[i + 1];
		if i == points.len() - 2 || segment_length(p1, last) <= base_radius * 3.0 {
			d.push_str(&format!(" L {:.2} {:.2}", p1.x, p1.y));
			continue;
		}
		let seg1 = vec_sub(p1, p0);
		let seg2 = vec_sub(p2, p1);
		let len1 = (seg1.x * seg1.x + seg1.y * seg1.y).sqrt();
		let len2 = (seg2.x * seg2.x + seg2.y * seg2.y).sqrt();
		let radius = base_radius.min(len1 / 2.0).min(len2 / 2.0);
		let v1 = norm(seg1);
		let v2 = norm(seg2);
		let dot = v1.x * v2.x + v1.y * v2.y;
		if dot > 0.999 {
			d.push_str(&format!(" L {:.2} {:.2}", p1.x, p1.y));
			continue;
		}

		let in_pt = geom::PointF {
			x: p1.x - v1.x * radius,
			y: p1.y - v1.y * radius,
		};
		let out_pt = geom::PointF {
			x: p1.x + v2.x * radius,
			y: p1.y + v2.y * radius,
		};
		let c1 = geom::PointF {
			x: in_pt.x + v1.x * radius * k,
			y: in_pt.y + v1.y * radius * k,
		};
		let c2 = geom::PointF {
			x: out_pt.x - v2.x * radius * k,
			y: out_pt.y - v2.y * radius * k,
		};

		d.push_str(&format!(" L {:.2} {:.2}", in_pt.x, in_pt.y));
		d.push_str(&format!(
			" C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
			c1.x, c1.y, c2.x, c2.y, out_pt.x, out_pt.y
		));
	}

	d.push_str(&format!(" L {:.2} {:.2}", last.x, last.y));
	d
}

fn vec_sub(a: geom::PointF, b: geom::PointF) -> geom::PointF {
	geom::PointF {
		x: a.x - b.x,
		y: a.y - b.y,
	}
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
	let size = min_size_px.max(12.0);
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

fn arrow_direction(points: &[geom::PointF], min_len: f32) -> geom::PointF {
	if points.len() < 2 {
		return geom::PointF { x: 1.0, y: 0.0 };
	}

	for seg in points.windows(2).rev() {
		let len = segment_length(seg[0], seg[1]);
		if len >= min_len {
			let dir = norm(geom::PointF {
				x: seg[1].x - seg[0].x,
				y: seg[1].y - seg[0].y,
			});
			if dir.x.abs() > 0.0001 || dir.y.abs() > 0.0001 {
				return dir;
			}
		}
	}

	let last = points[points.len() - 1];
	let prev = points[points.len() - 2];
	let fallback = norm(geom::PointF {
		x: last.x - prev.x,
		y: last.y - prev.y,
	});
	if fallback.x.abs() > 0.0001 || fallback.y.abs() > 0.0001 {
		fallback
	} else {
		geom::PointF { x: 1.0, y: 0.0 }
	}
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
	fn route_obstacles_skip_last_segment_only() {
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
		assert_eq!(obstacles.len(), 1);
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
		assert!(obstacles.is_empty());
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
}
