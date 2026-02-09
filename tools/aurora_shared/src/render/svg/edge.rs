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

#[derive(Debug, Clone)]
pub struct Route {
	pub points: Vec<geom::PointF>,
	pub arrow: [geom::PointF; 3],
	pub bounds: geom::Bounds,
}

pub fn route_edge(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	edge_obstacles: &[geom::RectI],
	config: &SvgConfig,
) -> Result<Route, RenderError> {
	let cell_px = config.base_font_size_px.max(1);
	let start_hint = anchor_point(source_bbox, target_bbox.center());
	let end_hint = anchor_point(target_bbox, source_bbox.center());

	let start_cell = outward_cell(source_bbox, start_hint, cell_px);
	let end_cell = outward_cell(target_bbox, end_hint, cell_px);

	let hard_blocked = blocked_cells(node_obstacles, cell_px);
	let soft_cells = blocked_cells(edge_obstacles, cell_px);

	let local_bounds = grid_bounds_pair(source_bbox, target_bbox, cell_px, config.node_spacing_px);
	let mut route_cells = astar(
		start_cell,
		end_cell,
		&hard_blocked,
		&soft_cells,
		&local_bounds,
	);
	if route_cells.is_none() {
		let mut pad = (config.node_spacing_px.saturating_mul(2)).max(config.node_spacing_px);
		for _ in 0..3 {
			let global_bounds =
				grid_bounds_global(source_bbox, target_bbox, node_obstacles, cell_px, pad);
			route_cells = astar(
				start_cell,
				end_cell,
				&hard_blocked,
				&soft_cells,
				&global_bounds,
			);
			if route_cells.is_some() {
				break;
			}
			pad = pad.saturating_mul(2);
		}
	}
	if route_cells.is_none() {
		// Last attempt: allow crossing earlier edges, but never nodes.
		let no_soft: HashSet<Cell> = HashSet::new();
		let mut pad = (config.node_spacing_px.saturating_mul(2)).max(config.node_spacing_px);
		for _ in 0..3 {
			let global_bounds =
				grid_bounds_global(source_bbox, target_bbox, node_obstacles, cell_px, pad);
			route_cells = astar(
				start_cell,
				end_cell,
				&hard_blocked,
				&no_soft,
				&global_bounds,
			);
			if route_cells.is_some() {
				break;
			}
			pad = pad.saturating_mul(2);
		}
	}
	let cells = route_cells.ok_or(RenderError::SvgRouteFailed)?;

	let cell_points: Vec<geom::PointF> = cells
		.iter()
		.copied()
		.map(|c| cell_center(c, cell_px))
		.collect();
	let start_dir = if cells.len() >= 2 {
		Cell {
			x: (cells[1].x - cells[0].x).signum(),
			y: (cells[1].y - cells[0].y).signum(),
		}
	} else {
		Cell {
			x: (end_cell.x - start_cell.x).signum(),
			y: (end_cell.y - start_cell.y).signum(),
		}
	};
	let end_dir = if cells.len() >= 2 {
		let last = cells.len() - 1;
		Cell {
			x: (cells[last].x - cells[last - 1].x).signum(),
			y: (cells[last].y - cells[last - 1].y).signum(),
		}
	} else {
		Cell {
			x: (end_cell.x - start_cell.x).signum(),
			y: (end_cell.y - start_cell.y).signum(),
		}
	};

	let start = ray_rect_intersection(
		cell_points[0],
		geom::PointF {
			x: (-start_dir.x) as f32,
			y: (-start_dir.y) as f32,
		},
		source_bbox,
	)
	.unwrap_or(start_hint);
	let end = ray_rect_intersection(
		cell_points[cell_points.len() - 1],
		geom::PointF {
			x: end_dir.x as f32,
			y: end_dir.y as f32,
		},
		target_bbox,
	)
	.unwrap_or(end_hint);

	let mut points: Vec<geom::PointF> = Vec::with_capacity(cell_points.len() + 2);
	points.push(start);
	points.extend(cell_points.iter().copied());
	points.push(end);

	let points = compress_polyline(points);
	let arrow = arrowhead(points.as_slice(), cell_px as f32);
	let b = bounds_for_points(points.as_slice())
		.union_point(arrow[0])
		.union_point(arrow[1])
		.union_point(arrow[2]);

	Ok(Route {
		points,
		arrow,
		bounds: b,
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

/// Convert a routed edge into rectangular obstacles that should be avoided by later routes.
///
/// This intentionally skips any segment portions that intersect the source/target node boxes
/// (expanded by the given padding) so that multiple edges can still cleanly exit/enter nodes.
pub fn route_obstacles_for_later_edges(
	route: &Route,
	cell_px: i32,
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	padding_px: i32,
) -> Vec<geom::RectI> {
	let pad = padding_px.max(0) as f32;
	let source_exclusion = grow_rect(*source_bbox, padding_px * 2);
	let target_exclusion = grow_rect(*target_bbox, padding_px * 2);

	let mut out: Vec<geom::RectI> = Vec::new();
	for seg in route.points.windows(2) {
		let a = seg[0];
		let b = seg[1];

		let dx = b.x - a.x;
		let dy = b.y - a.y;
		let diagonal = dx.abs() > f32::EPSILON && dy.abs() > f32::EPSILON;
		if diagonal
			&& is_near_cell_center_point(a, cell_px)
			&& is_near_cell_center_point(b, cell_px)
		{
			let start_cell = cell_from_center_point(a, cell_px);
			let end_cell = cell_from_center_point(b, cell_px);
			let ddx = (end_cell.x - start_cell.x).abs();
			let ddy = (end_cell.y - start_cell.y).abs();
			if ddx == ddy {
				for c in cells_on_straight_run(start_cell, end_cell) {
					let rect = rect_for_cell(c, cell_px, padding_px);
					if rect_intersects(rect, source_exclusion)
						|| rect_intersects(rect, target_exclusion)
					{
						continue;
					}
					out.push(rect);
				}
				continue;
			}
		}

		let min_x = (a.x.min(b.x) - pad).floor() as i32;
		let min_y = (a.y.min(b.y) - pad).floor() as i32;
		let max_x = (a.x.max(b.x) + pad).ceil() as i32;
		let max_y = (a.y.max(b.y) + pad).ceil() as i32;
		let rect = geom::RectI {
			x: min_x,
			y: min_y,
			w: (max_x - min_x).max(1),
			h: (max_y - min_y).max(1),
		};

		if rect_intersects(rect, source_exclusion) || rect_intersects(rect, target_exclusion) {
			continue;
		}
		out.push(rect);
	}
	out
}

fn rect_intersects(a: geom::RectI, b: geom::RectI) -> bool {
	let ax2 = a.x + a.w;
	let ay2 = a.y + a.h;
	let bx2 = b.x + b.w;
	let by2 = b.y + b.h;
	a.x < bx2 && ax2 > b.x && a.y < by2 && ay2 > b.y
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

	// Intersections with vertical sides.
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
	// Intersections with horizontal sides.
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

fn outward_cell(bbox: &geom::RectI, anchor: geom::PointF, cell_px: i32) -> Cell {
	let eps = 0.01;
	let x0 = bbox.x as f32;
	let x1 = (bbox.x + bbox.w) as f32;
	let y0 = bbox.y as f32;
	let y1 = (bbox.y + bbox.h) as f32;
	let left = (anchor.x - x0).abs() <= eps;
	let right = (anchor.x - x1).abs() <= eps;
	let top = (anchor.y - y0).abs() <= eps;
	let bottom = (anchor.y - y1).abs() <= eps;

	let mut x = (anchor.x / (cell_px as f32)).round() as i32;
	let mut y = (anchor.y / (cell_px as f32)).round() as i32;

	if right {
		x = div_ceil(bbox.x + bbox.w, cell_px);
	} else if left {
		x = div_floor(bbox.x, cell_px) - 1;
	}
	if bottom {
		y = div_ceil(bbox.y + bbox.h, cell_px);
	} else if top {
		y = div_floor(bbox.y, cell_px) - 1;
	}

	Cell { x, y }
}

fn div_floor(v: i32, d: i32) -> i32 {
	let dv = v / d;
	let rv = v % d;
	if (rv != 0) && ((rv < 0) != (d < 0)) {
		dv - 1
	} else {
		dv
	}
}

fn div_ceil(v: i32, d: i32) -> i32 {
	let dv = v / d;
	let rv = v % d;
	if (rv != 0) && ((rv > 0) == (d > 0)) {
		dv + 1
	} else {
		dv
	}
}

fn cell_center(c: Cell, cell_px: i32) -> geom::PointF {
	geom::PointF {
		x: (c.x as f32) * (cell_px as f32) + (cell_px as f32) / 2.0,
		y: (c.y as f32) * (cell_px as f32) + (cell_px as f32) / 2.0,
	}
}

#[derive(Debug, Clone, Copy)]
struct GridBounds {
	min_x: i32,
	max_x: i32,
	min_y: i32,
	max_y: i32,
}

fn grid_bounds_pair(a: &geom::RectI, b: &geom::RectI, cell_px: i32, padding_px: i32) -> GridBounds {
	let min_x = a.x.min(b.x) - padding_px;
	let min_y = a.y.min(b.y) - padding_px;
	let max_x = (a.x + a.w).max(b.x + b.w) + padding_px;
	let max_y = (a.y + a.h).max(b.y + b.h) + padding_px;
	GridBounds {
		min_x: div_floor(min_x, cell_px) - 2,
		max_x: div_ceil(max_x, cell_px) + 2,
		min_y: div_floor(min_y, cell_px) - 2,
		max_y: div_ceil(max_y, cell_px) + 2,
	}
}

fn grid_bounds_global(
	a: &geom::RectI,
	b: &geom::RectI,
	obstacles: &[geom::RectI],
	cell_px: i32,
	padding_px: i32,
) -> GridBounds {
	let mut min_x = a.x.min(b.x);
	let mut min_y = a.y.min(b.y);
	let mut max_x = (a.x + a.w).max(b.x + b.w);
	let mut max_y = (a.y + a.h).max(b.y + b.h);
	for o in obstacles {
		min_x = min_x.min(o.x);
		min_y = min_y.min(o.y);
		max_x = max_x.max(o.x + o.w);
		max_y = max_y.max(o.y + o.h);
	}
	min_x -= padding_px;
	min_y -= padding_px;
	max_x += padding_px;
	max_y += padding_px;
	GridBounds {
		min_x: div_floor(min_x, cell_px) - 2,
		max_x: div_ceil(max_x, cell_px) + 2,
		min_y: div_floor(min_y, cell_px) - 2,
		max_y: div_ceil(max_y, cell_px) + 2,
	}
}

fn blocked_cells(obstacles: &[geom::RectI], cell_px: i32) -> HashSet<Cell> {
	let mut blocked: HashSet<Cell> = HashSet::new();
	for o in obstacles {
		// Block the cells covered by the node's bounding box.
		// Use inclusive max on the last covered pixel so the first cell outside the box
		// (used for start/end routing anchors) remains unblocked.
		let max_x_px = o.x.saturating_add(o.w.saturating_sub(1));
		let max_y_px = o.y.saturating_add(o.h.saturating_sub(1));
		let min_x = div_floor(o.x, cell_px);
		let max_x = div_floor(max_x_px, cell_px);
		let min_y = div_floor(o.y, cell_px);
		let max_y = div_floor(max_y_px, cell_px);
		for x in min_x..=max_x {
			for y in min_y..=max_y {
				blocked.insert(Cell { x, y });
			}
		}
	}
	blocked
}

fn ray_rect_intersection(
	p: geom::PointF,
	dir: geom::PointF,
	r: &geom::RectI,
) -> Option<geom::PointF> {
	let eps = 0.000_1;
	if dir.x.abs() < eps && dir.y.abs() < eps {
		return None;
	}
	let x0 = r.x as f32;
	let x1 = (r.x + r.w) as f32;
	let y0 = r.y as f32;
	let y1 = (r.y + r.h) as f32;

	let mut best_t: f32 = f32::INFINITY;
	let mut best: Option<geom::PointF> = None;

	if dir.x.abs() >= eps {
		for x in [x0, x1] {
			let t = (x - p.x) / dir.x;
			if t < 0.0 {
				continue;
			}
			let y = p.y + dir.y * t;
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
			let t = (y - p.y) / dir.y;
			if t < 0.0 {
				continue;
			}
			let x = p.x + dir.x * t;
			if x < x0 - 0.01 || x > x1 + 0.01 {
				continue;
			}
			if t < best_t {
				best_t = t;
				best = Some(geom::PointF { x, y });
			}
		}
	}

	best
}

fn cell_from_center_point(p: geom::PointF, cell_px: i32) -> Cell {
	let half = (cell_px as f32) / 2.0;
	Cell {
		x: ((p.x - half) / (cell_px as f32)).round() as i32,
		y: ((p.y - half) / (cell_px as f32)).round() as i32,
	}
}

fn is_near_cell_center_point(p: geom::PointF, cell_px: i32) -> bool {
	let cell = cell_from_center_point(p, cell_px);
	let c = cell_center(cell, cell_px);
	(p.x - c.x).abs() <= 0.6 && (p.y - c.y).abs() <= 0.6
}

fn rect_for_cell(c: Cell, cell_px: i32, padding_px: i32) -> geom::RectI {
	let pad = padding_px.max(0);
	geom::RectI {
		x: c.x.saturating_mul(cell_px).saturating_sub(pad),
		y: c.y.saturating_mul(cell_px).saturating_sub(pad),
		w: cell_px.saturating_add(pad.saturating_mul(2)).max(1),
		h: cell_px.saturating_add(pad.saturating_mul(2)).max(1),
	}
}

fn cells_on_straight_run(a: Cell, b: Cell) -> Vec<Cell> {
	let dx = (b.x - a.x).signum();
	let dy = (b.y - a.y).signum();
	if dx == 0 && dy == 0 {
		return vec![a];
	}

	let mut out: Vec<Cell> = Vec::new();
	let mut cur = a;
	out.push(cur);
	while cur != b {
		cur = Cell {
			x: cur.x + dx,
			y: cur.y + dy,
		};
		out.push(cur);
	}
	out
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

fn astar(
	start: Cell,
	goal: Cell,
	hard_blocked: &HashSet<Cell>,
	soft_cells: &HashSet<Cell>,
	bounds: &GridBounds,
) -> Option<Vec<Cell>> {
	let straight_cost: i32 = 10;
	let diagonal_cost: i32 = 14;
	let turn_penalty: i32 = 25;
	let soft_penalty: i32 = 50;

	let mut open: BinaryHeap<HeapItem> = BinaryHeap::new();
	let mut came_from: HashMap<State, State> = HashMap::new();
	let mut g_score: HashMap<State, i32> = HashMap::new();

	if hard_blocked.contains(&start) || hard_blocked.contains(&goal) {
		return None;
	}

	let start_state = State {
		cell: start,
		dir: DIR_NONE,
	};

	g_score.insert(start_state, 0);
	open.push(HeapItem {
		f: heuristic(start, goal, straight_cost, diagonal_cost),
		g: 0,
		state: start_state,
	});

	while let Some(current) = open.pop() {
		if current.state.cell == goal {
			return Some(reconstruct_path(came_from, current.state));
		}

		// 8-connected grid for smoother routes.
		//
		// Diagonal moves are cheaper than a horizontal+vertical combo (14 vs 20) which
		// tends to reduce the "stair-step" look in long routes.
		let neighbors: [(i32, i32); 8] = [
			(1, 0),
			(-1, 0),
			(0, 1),
			(0, -1),
			(1, 1),
			(1, -1),
			(-1, 1),
			(-1, -1),
		];

		for (dx, dy) in neighbors {
			let nb = Cell {
				x: current.state.cell.x + dx,
				y: current.state.cell.y + dy,
			};
			if nb.x < bounds.min_x
				|| nb.x > bounds.max_x
				|| nb.y < bounds.min_y
				|| nb.y > bounds.max_y
			{
				continue;
			}
			if hard_blocked.contains(&nb) {
				continue;
			}

			// Prevent corner-cutting: for diagonal moves, require both orthogonal
			// adjacent cells to be clear.
			if dx != 0 && dy != 0 {
				let ortho1 = Cell {
					x: current.state.cell.x + dx,
					y: current.state.cell.y,
				};
				let ortho2 = Cell {
					x: current.state.cell.x,
					y: current.state.cell.y + dy,
				};
				if hard_blocked.contains(&ortho1) || hard_blocked.contains(&ortho2) {
					continue;
				}
			}

			let mut step = if dx != 0 && dy != 0 {
				diagonal_cost
			} else {
				straight_cost
			};
			let new_dir = Dir {
				dx: dx as i8,
				dy: dy as i8,
			};
			if current.state.dir != DIR_NONE && current.state.dir != new_dir {
				step += turn_penalty;
			}
			if soft_cells.contains(&nb) {
				step += soft_penalty;
			}
			let tentative_g = current.g + step;
			let nb_state = State {
				cell: nb,
				dir: new_dir,
			};
			let best = g_score.get(&nb_state).copied();
			if best.is_some_and(|g| tentative_g >= g) {
				continue;
			}

			came_from.insert(nb_state, current.state);
			g_score.insert(nb_state, tentative_g);
			let f = tentative_g + heuristic(nb, goal, straight_cost, diagonal_cost);
			open.push(HeapItem {
				f,
				g: tentative_g,
				state: nb_state,
			});
		}
	}

	None
}

fn heuristic(a: Cell, b: Cell, straight_cost: i32, diagonal_cost: i32) -> i32 {
	// Octile distance (scaled integer costs) for 8-connected grids.
	let dx = (a.x - b.x).abs();
	let dy = (a.y - b.y).abs();
	let min_d = dx.min(dy);
	let max_d = dx.max(dy);
	(diagonal_cost * min_d) + (straight_cost * (max_d - min_d))
}

fn reconstruct_path(mut came_from: HashMap<State, State>, mut current: State) -> Vec<Cell> {
	let mut path: Vec<Cell> = vec![current.cell];
	while let Some(prev) = came_from.remove(&current) {
		current = prev;
		path.push(current.cell);
	}
	path.reverse();
	path
}

fn compress_polyline(points: Vec<geom::PointF>) -> Vec<geom::PointF> {
	if points.len() <= 2 {
		return points;
	}
	let mut out: Vec<geom::PointF> = Vec::with_capacity(points.len());
	out.push(points[0]);
	let mut prev_dir: Option<(i32, i32)> = None;
	for p in points.into_iter().skip(1) {
		let last = *out.last().unwrap_or(&p);
		let dx = p.x - last.x;
		let dy = p.y - last.y;
		let dir = (dx.signum() as i32, dy.signum() as i32);
		if prev_dir == Some(dir) {
			if let Some(tail) = out.last_mut() {
				*tail = p;
			}
			continue;
		}
		out.push(p);
		prev_dir = Some(dir);
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn astar_prefers_diagonals_when_clear() {
		let start = Cell { x: 0, y: 0 };
		let goal = Cell { x: 3, y: 3 };
		let hard_blocked: HashSet<Cell> = HashSet::new();
		let soft_cells: HashSet<Cell> = HashSet::new();
		let bounds = GridBounds {
			min_x: -5,
			max_x: 5,
			min_y: -5,
			max_y: 5,
		};
		let path =
			astar(start, goal, &hard_blocked, &soft_cells, &bounds).expect("path should exist");
		assert_eq!(path.first().copied(), Some(start));
		assert_eq!(path.last().copied(), Some(goal));
		// With diagonal moves allowed, (0,0)->(3,3) should be three diagonal steps.
		assert_eq!(path.len(), 4);
	}

	#[test]
	fn astar_disallows_corner_cutting() {
		let start = Cell { x: 0, y: 0 };
		let goal = Cell { x: 1, y: 1 };
		let mut hard_blocked: HashSet<Cell> = HashSet::new();
		hard_blocked.insert(Cell { x: 1, y: 0 });
		hard_blocked.insert(Cell { x: 0, y: 1 });
		let soft_cells: HashSet<Cell> = HashSet::new();
		let bounds = GridBounds {
			min_x: 0,
			max_x: 1,
			min_y: 0,
			max_y: 1,
		};
		assert!(astar(start, goal, &hard_blocked, &soft_cells, &bounds).is_none());
	}

	#[test]
	fn compress_polyline_merges_diagonal_runs() {
		let pts = vec![
			geom::PointF { x: 0.0, y: 0.0 },
			geom::PointF { x: 1.0, y: 1.0 },
			geom::PointF { x: 2.0, y: 2.0 },
		];
		let out = compress_polyline(pts);
		assert_eq!(out.len(), 2);
		assert_eq!(out[0].x, 0.0);
		assert_eq!(out[0].y, 0.0);
		assert_eq!(out[1].x, 2.0);
		assert_eq!(out[1].y, 2.0);
	}
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
	let mut d = String::new();
	d.push_str(&format!("M {:.2} {:.2}", points[0].x, points[0].y));

	for i in 1..(points.len() - 1) {
		let p0 = points[i - 1];
		let p1 = points[i];
		let p2 = points[i + 1];
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
	let last = points[points.len() - 1];
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
	let size = min_size_px.max(16.0);
	let width = size * 0.6;
	let end = *points.last().unwrap_or(&geom::PointF { x: 0.0, y: 0.0 });
	let prev = if points.len() >= 2 {
		points[points.len() - 2]
	} else {
		geom::PointF {
			x: end.x - 1.0,
			y: end.y,
		}
	};
	let dir = norm(geom::PointF {
		x: end.x - prev.x,
		y: end.y - prev.y,
	});
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

fn bounds_for_points(points: &[geom::PointF]) -> geom::Bounds {
	let mut b = geom::Bounds::empty();
	for p in points {
		b = b.union_point(*p);
	}
	b
}
