//! Visibility graph + shortest-path utilities for orthogonal edge routing.
//!
//! This module is intentionally geometry-focused and does not depend on the higher-level routing
//! policy (bundling, per-source grouping, etc.).

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::geom;

pub(super) const ROUTE_PAD_PX: i32 = 1;

// Integer-weighted routing constants. We scale length by 2 so collector edges can be half-cost.
const LENGTH_SCALE: i64 = 2;
const BEND_PENALTY: i64 = 32 * LENGTH_SCALE;
const CONGESTION_PENALTY: i64 = 64 * LENGTH_SCALE;
const SAME_SOURCE_OVERLAP_PENALTY: i64 = 6 * LENGTH_SCALE;
const MAX_CONGESTION_MULTIPLIER: u32 = 6;
const MAX_SAME_SOURCE_MULTIPLIER: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PointI {
	pub(super) x: i32,
	pub(super) y: i32,
}

impl PointI {
	pub(super) fn to_f(self) -> geom::PointF {
		geom::PointF {
			x: self.x as f32,
			y: self.y as f32,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
	None,
	H,
	V,
}

impl Dir {
	fn index(self) -> usize {
		match self {
			Self::None => 0,
			Self::H => 1,
			Self::V => 2,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct EdgeKey {
	a: usize,
	b: usize,
}

impl EdgeKey {
	pub(super) fn new(a: usize, b: usize) -> Self {
		if a <= b {
			Self { a, b }
		} else {
			Self { a: b, b: a }
		}
	}
}

#[derive(Debug, Clone)]
pub(super) struct AdjEdge {
	pub(super) to: usize,
	dir: Dir,
	len: i32,
	pub(super) key: EdgeKey,
	collector_target: Option<usize>,
}

impl AdjEdge {
	pub(super) fn collector_target(&self) -> Option<usize> {
		self.collector_target
	}
}

#[derive(Debug, Clone)]
pub(super) struct Graph {
	pub(super) points: Vec<PointI>,
	pub(super) index: HashMap<PointI, usize>,
	pub(super) adj: Vec<Vec<AdjEdge>>,
}

impl Graph {
	pub(super) fn new() -> Self {
		Self {
			points: Vec::new(),
			index: HashMap::new(),
			adj: Vec::new(),
		}
	}

	fn vertex(&mut self, p: PointI) -> usize {
		if let Some(&idx) = self.index.get(&p) {
			return idx;
		}
		let idx = self.points.len();
		self.points.push(p);
		self.index.insert(p, idx);
		self.adj.push(Vec::new());
		idx
	}

	fn add_undirected_edge(&mut self, a: PointI, b: PointI, collector_target: Option<usize>) {
		if a == b {
			return;
		}
		let ai = self.vertex(a);
		let bi = self.vertex(b);
		let key = EdgeKey::new(ai, bi);
		let (dir, len) = if a.y == b.y {
			(Dir::H, (a.x - b.x).abs())
		} else {
			(Dir::V, (a.y - b.y).abs())
		};
		if len <= 0 {
			return;
		}
		self.adj[ai].push(AdjEdge {
			to: bi,
			dir,
			len,
			key,
			collector_target,
		});
		self.adj[bi].push(AdjEdge {
			to: ai,
			dir,
			len,
			key,
			collector_target,
		});
	}
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RectI {
	pub(super) x0: i32,
	pub(super) y0: i32,
	pub(super) x1: i32,
	pub(super) y1: i32,
}

impl RectI {
	pub(super) fn from_geom(r: geom::RectI) -> Self {
		Self {
			x0: r.x,
			y0: r.y,
			x1: r.x + r.w,
			y1: r.y + r.h,
		}
	}

	pub(super) fn contains_point(&self, p: PointI) -> bool {
		p.x >= self.x0 && p.x <= self.x1 && p.y >= self.y0 && p.y <= self.y1
	}
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CollectorSpan {
	pub(super) target_ord: usize,
	pub(super) x: i32,
	pub(super) y_top: i32,
	pub(super) y_bottom: i32,
}

pub(super) fn inflate(rect: geom::RectI, pad: i32) -> geom::RectI {
	let p = pad.max(0);
	geom::RectI {
		x: rect.x - p,
		y: rect.y - p,
		w: rect.w + 2 * p,
		h: rect.h + 2 * p,
	}
}

pub(super) fn is_free_point(p: PointI, obstacles: &[RectI]) -> bool {
	!obstacles.iter().any(|r| r.contains_point(p))
}

fn blocked_intervals_x(obstacles: &[RectI], y: i32) -> Vec<(i32, i32)> {
	let mut out: Vec<(i32, i32)> = obstacles
		.iter()
		.filter(|r| y >= r.y0 && y <= r.y1)
		.map(|r| (r.x0, r.x1))
		.collect();
	out.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
	let mut merged: Vec<(i32, i32)> = Vec::new();
	for (x0, x1) in out {
		if let Some(last) = merged.last_mut() {
			if x0 <= last.1 + 1 {
				last.1 = last.1.max(x1);
				continue;
			}
		}
		merged.push((x0, x1));
	}
	merged
}

fn blocked_intervals_y(obstacles: &[RectI], x: i32) -> Vec<(i32, i32)> {
	let mut out: Vec<(i32, i32)> = obstacles
		.iter()
		.filter(|r| x >= r.x0 && x <= r.x1)
		.map(|r| (r.y0, r.y1))
		.collect();
	out.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
	let mut merged: Vec<(i32, i32)> = Vec::new();
	for (y0, y1) in out {
		if let Some(last) = merged.last_mut() {
			if y0 <= last.1 + 1 {
				last.1 = last.1.max(y1);
				continue;
			}
		}
		merged.push((y0, y1));
	}
	merged
}

fn collector_target_for_segment(
	x: i32,
	y0: i32,
	y1: i32,
	by_x: &HashMap<i32, Vec<CollectorSpan>>,
) -> Option<usize> {
	let Some(spans) = by_x.get(&x) else {
		return None;
	};
	let a0 = y0.min(y1);
	let a1 = y0.max(y1);
	for s in spans {
		let c0 = s.y_top.min(s.y_bottom);
		let c1 = s.y_top.max(s.y_bottom);
		if a0 >= c0 && a1 <= c1 {
			return Some(s.target_ord);
		}
	}
	None
}

pub(super) fn build_visibility_graph(
	obstacles: &[RectI],
	xs: &[i32],
	ys: &[i32],
	bounds: (i32, i32, i32, i32),
	collectors: &[CollectorSpan],
) -> Graph {
	let (min_x, max_x, min_y, max_y) = bounds;
	let mut by_x: HashMap<i32, Vec<CollectorSpan>> = HashMap::new();
	for c in collectors {
		by_x.entry(c.x).or_default().push(*c);
	}
	for spans in by_x.values_mut() {
		spans.sort_by(|l, r| {
			l.y_top
				.cmp(&r.y_top)
				.then_with(|| l.y_bottom.cmp(&r.y_bottom))
		});
	}

	let mut graph = Graph::new();

	for &y in ys {
		if y < min_y || y > max_y {
			continue;
		}
		let blocked = blocked_intervals_x(obstacles, y);
		let mut start = min_x;
		let mut free_spans: Vec<(i32, i32)> = Vec::new();
		for (bx0, bx1) in blocked {
			if start <= bx0 - 1 {
				free_spans.push((start, bx0 - 1));
			}
			start = bx1 + 1;
		}
		if start <= max_x {
			free_spans.push((start, max_x));
		}
		for (fx0, fx1) in free_spans {
			let mut run: Vec<i32> = xs
				.iter()
				.copied()
				.filter(|x| *x >= fx0 && *x <= fx1)
				.collect();
			run.sort();
			run.dedup();
			for w in run.windows(2) {
				let a = PointI { x: w[0], y };
				let b = PointI { x: w[1], y };
				if !is_free_point(a, obstacles) || !is_free_point(b, obstacles) {
					continue;
				}
				graph.add_undirected_edge(a, b, None);
			}
		}
	}

	for &x in xs {
		if x < min_x || x > max_x {
			continue;
		}
		let blocked = blocked_intervals_y(obstacles, x);
		let mut start = min_y;
		let mut free_spans: Vec<(i32, i32)> = Vec::new();
		for (by0, by1) in blocked {
			if start <= by0 - 1 {
				free_spans.push((start, by0 - 1));
			}
			start = by1 + 1;
		}
		if start <= max_y {
			free_spans.push((start, max_y));
		}
		for (fy0, fy1) in free_spans {
			let mut run: Vec<i32> = ys
				.iter()
				.copied()
				.filter(|y| *y >= fy0 && *y <= fy1)
				.collect();
			run.sort();
			run.dedup();
			for w in run.windows(2) {
				let a = PointI { x, y: w[0] };
				let b = PointI { x, y: w[1] };
				if !is_free_point(a, obstacles) || !is_free_point(b, obstacles) {
					continue;
				}
				let collector_target = collector_target_for_segment(x, w[0], w[1], &by_x);
				graph.add_undirected_edge(a, b, collector_target);
			}
		}
	}

	graph
}

#[derive(Debug, Clone)]
pub(super) enum Reservation {
	Source { primary: usize, count: u32 },
	TargetCollector { target: usize, count: u32 },
}

#[derive(Debug, Clone, Copy)]
struct HeapItem {
	cost: i64,
	goal_dist: i32,
	pos_y: i32,
	pos_x: i32,
	node: usize,
	dir: Dir,
}

impl Ord for HeapItem {
	fn cmp(&self, other: &Self) -> Ordering {
		// Min-heap by (cost, distance-to-goal, y, x, dir, node)
		other
			.cost
			.cmp(&self.cost)
			.then_with(|| other.goal_dist.cmp(&self.goal_dist))
			.then_with(|| other.pos_y.cmp(&self.pos_y))
			.then_with(|| other.pos_x.cmp(&self.pos_x))
			.then_with(|| other.dir.index().cmp(&self.dir.index()))
			.then_with(|| other.node.cmp(&self.node))
	}
}

impl PartialOrd for HeapItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for HeapItem {
	fn eq(&self, other: &Self) -> bool {
		self.cost == other.cost
			&& self.goal_dist == other.goal_dist
			&& self.node == other.node
			&& self.pos_x == other.pos_x
			&& self.pos_y == other.pos_y
			&& self.dir == other.dir
	}
}

impl Eq for HeapItem {}

fn step_cost(edge: &AdjEdge, bend: bool, congestion: u32, same_source_overlap: u32) -> i64 {
	let length = edge.len as i64;
	let collector = edge.collector_target.is_some();
	let length_cost = if collector {
		length
	} else {
		length * LENGTH_SCALE
	};
	let bend_cost = if bend { BEND_PENALTY } else { 0 };
	let congestion_cost = if collector || congestion == 0 {
		0
	} else {
		let capped = congestion.min(MAX_CONGESTION_MULTIPLIER) as i64;
		CONGESTION_PENALTY.saturating_mul(capped)
	};
	let same_source_cost = if collector || same_source_overlap == 0 {
		0
	} else {
		let capped = same_source_overlap.min(MAX_SAME_SOURCE_MULTIPLIER) as i64;
		SAME_SOURCE_OVERLAP_PENALTY.saturating_mul(capped)
	};
	length_cost + bend_cost + congestion_cost + same_source_cost
}

pub(super) fn dijkstra_path(
	graph: &Graph,
	starts: &HashSet<usize>,
	goal: usize,
	reservations: &HashMap<EdgeKey, Reservation>,
	this_source: usize,
	allowed_target: usize,
) -> Option<Vec<usize>> {
	let n = graph.points.len();
	let goal_point = graph.points.get(goal).copied()?;
	let mut dist: Vec<[i64; 3]> = vec![[i64::MAX; 3]; n];
	let mut prev: Vec<[Option<(usize, Dir)>; 3]> = vec![[None, None, None]; n];
	let mut heap = BinaryHeap::new();

	for &s in starts {
		let p = graph.points[s];
		dist[s][Dir::None.index()] = 0;
		heap.push(HeapItem {
			cost: 0,
			goal_dist: (p.x - goal_point.x).abs() + (p.y - goal_point.y).abs(),
			pos_y: p.y,
			pos_x: p.x,
			node: s,
			dir: Dir::None,
		});
	}

	while let Some(item) = heap.pop() {
		let idx = item.node;
		let dir = item.dir;
		if item.cost != dist[idx][dir.index()] {
			continue;
		}
		if idx == goal {
			let mut out: Vec<usize> = Vec::new();
			let mut cur = (idx, dir);
			out.push(cur.0);
			while let Some((pidx, pdir)) = prev[cur.0][cur.1.index()] {
				cur = (pidx, pdir);
				out.push(cur.0);
			}
			out.reverse();
			return Some(out);
		}
		for edge in &graph.adj[idx] {
			if let Some(t) = edge.collector_target() {
				if t != allowed_target {
					continue;
				}
			}
			let mut congestion: u32 = 0;
			let mut same_source_overlap: u32 = 0;
			match reservations.get(&edge.key) {
				None => {}
				Some(Reservation::Source { primary, count }) => {
					if *primary == this_source {
						same_source_overlap = *count;
					} else {
						congestion = *count;
					}
				}
				Some(Reservation::TargetCollector { target, .. }) => {
					if *target != allowed_target {
						congestion = u32::MAX;
					}
				}
			}
			if congestion == u32::MAX {
				continue;
			}
			let next = edge.to;
			let next_dir = edge.dir;
			let bend = dir != Dir::None && dir != next_dir;
			let next_cost = item.cost + step_cost(edge, bend, congestion, same_source_overlap);
			let slot = &mut dist[next][next_dir.index()];
			if next_cost < *slot {
				*slot = next_cost;
				prev[next][next_dir.index()] = Some((idx, dir));
				let p = graph.points[next];
				heap.push(HeapItem {
					cost: next_cost,
					goal_dist: (p.x - goal_point.x).abs() + (p.y - goal_point.y).abs(),
					pos_y: p.y,
					pos_x: p.x,
					node: next,
					dir: next_dir,
				});
			}
		}
	}

	None
}

pub(super) fn add_lane_lines(lines: &mut Vec<i32>, base: i32, spacing_px: i32) {
	for k in -4..=4 {
		lines.push(base + k * spacing_px);
	}
}
