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
	a: &geom::RectI,
	b: &geom::RectI,
	obstacles: &[geom::RectI],
	config: &SvgConfig,
) -> Result<Route, RenderError> {
	let cell_px = config.base_font_size_px.max(1);
	let start = anchor_point(a, b);
	let end = anchor_point(b, a);

	let start_cell = outward_cell(a, start, cell_px);
	let end_cell = outward_cell(b, end, cell_px);

	let bounds = grid_bounds(a, b, cell_px, config.node_spacing_px);
	let blocked = blocked_cells(obstacles, cell_px);

	let mut route_cells = astar(start_cell, end_cell, &blocked, &bounds);
	if route_cells.is_none() {
		route_cells = Some(fallback_l(start_cell, end_cell, &blocked));
	}
	let cells = route_cells.ok_or(RenderError::SvgRouteFailed)?;

	let mut points: Vec<geom::PointF> = Vec::new();
	points.push(start);
	for c in cells {
		points.push(cell_center(c, cell_px));
	}
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

fn anchor_point(from: &geom::RectI, to: &geom::RectI) -> geom::PointF {
	let c_from = from.center();
	let c_to = to.center();
	let dx = c_to.x - c_from.x;
	let dy = c_to.y - c_from.y;
	let w = (from.w as f32) / 2.0;
	let h = (from.h as f32) / 2.0;
	if dx.abs() / w.max(1.0) > dy.abs() / h.max(1.0) {
		let x = if dx >= 0.0 {
			(from.x + from.w) as f32
		} else {
			from.x as f32
		};
		geom::PointF { x, y: c_from.y }
	} else {
		let y = if dy >= 0.0 {
			(from.y + from.h) as f32
		} else {
			from.y as f32
		};
		geom::PointF { x: c_from.x, y }
	}
}

fn outward_cell(bbox: &geom::RectI, anchor: geom::PointF, cell_px: i32) -> Cell {
	let c = bbox.center();
	let dx = anchor.x - c.x;
	let dy = anchor.y - c.y;
	if dx.abs() > dy.abs() {
		if dx >= 0.0 {
			Cell {
				x: div_ceil(bbox.x + bbox.w, cell_px),
				y: (c.y / (cell_px as f32)).round() as i32,
			}
		} else {
			Cell {
				x: div_floor(bbox.x, cell_px) - 1,
				y: (c.y / (cell_px as f32)).round() as i32,
			}
		}
	} else if dy >= 0.0 {
		Cell {
			x: (c.x / (cell_px as f32)).round() as i32,
			y: div_ceil(bbox.y + bbox.h, cell_px),
		}
	} else {
		Cell {
			x: (c.x / (cell_px as f32)).round() as i32,
			y: div_floor(bbox.y, cell_px) - 1,
		}
	}
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

fn grid_bounds(a: &geom::RectI, b: &geom::RectI, cell_px: i32, padding_px: i32) -> GridBounds {
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

fn fallback_l(start: Cell, goal: Cell, blocked: &HashSet<Cell>) -> Vec<Cell> {
	if start.x == goal.x || start.y == goal.y {
		return vec![start, goal];
	}
	let bend1 = Cell {
		x: start.x,
		y: goal.y,
	};
	let bend2 = Cell {
		x: goal.x,
		y: start.y,
	};
	if !blocked.contains(&bend1) {
		return vec![start, bend1, goal];
	}
	if !blocked.contains(&bend2) {
		return vec![start, bend2, goal];
	}
	vec![start, bend1, goal]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeapItem {
	f: i32,
	g: i32,
	cell: Cell,
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
	blocked: &HashSet<Cell>,
	bounds: &GridBounds,
) -> Option<Vec<Cell>> {
	let mut open: BinaryHeap<HeapItem> = BinaryHeap::new();
	let mut came_from: HashMap<Cell, Cell> = HashMap::new();
	let mut g_score: HashMap<Cell, i32> = HashMap::new();

	if blocked.contains(&start) || blocked.contains(&goal) {
		return None;
	}

	g_score.insert(start, 0);
	open.push(HeapItem {
		f: heuristic(start, goal),
		g: 0,
		cell: start,
	});

	while let Some(current) = open.pop() {
		if current.cell == goal {
			return Some(reconstruct_path(came_from, current.cell));
		}

		let neighbors = [
			Cell {
				x: current.cell.x + 1,
				y: current.cell.y,
			},
			Cell {
				x: current.cell.x - 1,
				y: current.cell.y,
			},
			Cell {
				x: current.cell.x,
				y: current.cell.y + 1,
			},
			Cell {
				x: current.cell.x,
				y: current.cell.y - 1,
			},
		];

		for nb in neighbors {
			if nb.x < bounds.min_x
				|| nb.x > bounds.max_x
				|| nb.y < bounds.min_y
				|| nb.y > bounds.max_y
			{
				continue;
			}
			if blocked.contains(&nb) {
				continue;
			}

			let tentative_g = current.g + 1;
			let best = g_score.get(&nb).copied();
			if best.is_some_and(|g| tentative_g >= g) {
				continue;
			}

			came_from.insert(nb, current.cell);
			g_score.insert(nb, tentative_g);
			let f = tentative_g + heuristic(nb, goal);
			open.push(HeapItem {
				f,
				g: tentative_g,
				cell: nb,
			});
		}
	}

	None
}

fn heuristic(a: Cell, b: Cell) -> i32 {
	(a.x - b.x).abs() + (a.y - b.y).abs()
}

fn reconstruct_path(mut came_from: HashMap<Cell, Cell>, mut current: Cell) -> Vec<Cell> {
	let mut path: Vec<Cell> = vec![current];
	while let Some(prev) = came_from.remove(&current) {
		current = prev;
		path.push(current);
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
		let dir = if dx.abs() >= dy.abs() {
			(dx.signum() as i32, 0)
		} else {
			(0, dy.signum() as i32)
		};
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
