//! Bundled orthogonal edge router.
//!
//! This is the routing entrypoint used by SVG rendering. It constructs an orthogonal visibility
//! graph, grows a bundled Steiner-style tree per source, and materializes per-edge routes.

use std::collections::{HashMap, HashSet};

use super::super::LayoutEdge;
use super::super::render_error::RenderError;
use super::{SvgConfig, geom};

use super::edge::Route;
use super::node::PositionedNode;

use super::edge_router_graph::{
	CollectorSpan, EdgeKey, Graph, PointI, ROUTE_PAD_PX, Reservation, add_lane_lines,
	dijkstra_path, inflate, is_free_point,
};
use super::edge_router_unicode::unicode_cmp;
use super::edge_router_utils::{
	compress_points, junction_ids_for_route, path_in_tree, tree_parent_map,
};

const CLEARANCE_PX: i32 = 64;
const SPACING_PX: i32 = 32;
const DENSE_EDGE_THRESHOLD: usize = 24;

#[derive(Debug, Clone)]
struct EdgeSpec {
	edge_id: String,
	source_id: String,
	target_id: String,
}

#[derive(Debug, Clone, Copy)]
struct NodePorts {
	source_anchor: PointI,
	source_exit: PointI,
	target_entry: PointI,
	target_anchor: PointI,
}

#[derive(Debug, Clone)]
struct LeafSpec {
	spec: EdgeSpec,
	target_ord: usize,
	ports: NodePorts,
	leaf_idx: usize,
}

fn node_center_x(bbox: geom::RectI) -> i32 {
	bbox.x + bbox.w / 2
}

fn push_below_obstacles(x: i32, mut y: i32, obstacles: &[super::edge_router_graph::RectI]) -> i32 {
	loop {
		let mut moved = false;
		for r in obstacles {
			if x >= r.x0 && x <= r.x1 && y >= r.y0 && y <= r.y1 {
				y = r.y1 + ROUTE_PAD_PX;
				moved = true;
				break;
			}
		}
		if !moved {
			return y;
		}
	}
}

fn push_above_obstacles(x: i32, mut y: i32, obstacles: &[super::edge_router_graph::RectI]) -> i32 {
	loop {
		let mut moved = false;
		for r in obstacles {
			if x >= r.x0 && x <= r.x1 && y >= r.y0 && y <= r.y1 {
				y = r.y0 - ROUTE_PAD_PX;
				moved = true;
				break;
			}
		}
		if !moved {
			return y;
		}
	}
}

fn ports_for_edge(
	source: geom::RectI,
	target: geom::RectI,
	obstacles: &[super::edge_router_graph::RectI],
) -> NodePorts {
	let sx = node_center_x(source);
	let tx = node_center_x(target);
	let source_anchor = PointI {
		x: sx,
		y: source.y + source.h,
	};
	let source_exit = PointI {
		x: sx,
		y: push_below_obstacles(
			sx,
			source.y + source.h + CLEARANCE_PX + ROUTE_PAD_PX,
			obstacles,
		),
	};
	let target_entry = PointI {
		x: tx,
		y: push_above_obstacles(tx, target.y - CLEARANCE_PX - ROUTE_PAD_PX, obstacles),
	};
	let target_anchor = PointI { x: tx, y: target.y };
	NodePorts {
		source_anchor,
		source_exit,
		target_entry,
		target_anchor,
	}
}

fn build_edge_specs(edges: &[LayoutEdge]) -> Vec<EdgeSpec> {
	let mut keyed: Vec<(usize, String, String)> = edges
		.iter()
		.enumerate()
		.map(|(i, e)| (i, e.a.clone(), e.b.clone()))
		.collect();
	keyed.sort_by(|left, right| {
		unicode_cmp(left.1.as_str(), right.1.as_str())
			.then_with(|| unicode_cmp(left.2.as_str(), right.2.as_str()))
			.then_with(|| left.0.cmp(&right.0))
	});
	let mut next: HashMap<(String, String), usize> = HashMap::new();
	let mut ids: Vec<Option<String>> = vec![None; edges.len()];
	for (orig_index, a, b) in keyed {
		let counter = next.entry((a.clone(), b.clone())).or_insert(0);
		ids[orig_index] = Some(format!("{}->{}#{}", a, b, *counter));
		*counter += 1;
	}
	let mut out = Vec::with_capacity(edges.len());
	for (i, e) in edges.iter().enumerate() {
		out.push(EdgeSpec {
			edge_id: ids[i]
				.clone()
				.unwrap_or_else(|| format!("{}->{}#0", e.a, e.b)),
			source_id: e.a.clone(),
			target_id: e.b.clone(),
		});
	}
	out
}

fn build_obstacles(
	nodes: &HashMap<String, PositionedNode>,
) -> Vec<super::edge_router_graph::RectI> {
	let mut obstacles: Vec<super::edge_router_graph::RectI> = nodes
		.values()
		.map(|n| super::edge_router_graph::RectI::from_geom(inflate(n.bbox, CLEARANCE_PX)))
		.collect();
	obstacles.sort_by(|l, r| l.y0.cmp(&r.y0).then_with(|| l.x0.cmp(&r.x0)));
	obstacles
}

fn positioned_order_key(id: &str, positioned: &HashMap<String, PositionedNode>) -> (i32, i32) {
	positioned
		.get(id)
		.map(|node| (node.bbox.y, node.bbox.x))
		.unwrap_or((i32::MAX, i32::MAX))
}

fn global_bounds(
	obstacles: &[super::edge_router_graph::RectI],
	terminals: &[PointI],
) -> (i32, i32, i32, i32) {
	let mut min_x = i32::MAX;
	let mut max_x = i32::MIN;
	let mut min_y = i32::MAX;
	let mut max_y = i32::MIN;
	for r in obstacles {
		min_x = min_x.min(r.x0);
		max_x = max_x.max(r.x1);
		min_y = min_y.min(r.y0);
		max_y = max_y.max(r.y1);
	}
	for t in terminals {
		min_x = min_x.min(t.x);
		max_x = max_x.max(t.x);
		min_y = min_y.min(t.y);
		max_y = max_y.max(t.y);
	}
	let margin = CLEARANCE_PX + 64;
	(
		min_x - margin,
		max_x + margin,
		min_y - margin,
		max_y + margin,
	)
}

fn collector_for_target(
	target_ord: usize,
	target_bbox: geom::RectI,
	obstacles: &[super::edge_router_graph::RectI],
	min_y: i32,
	center_y: i32,
) -> CollectorSpan {
	let ports = ports_for_edge(target_bbox, target_bbox, obstacles);
	let x = ports.target_entry.x;
	let entry_y = ports.target_entry.y;
	let mut y_top = center_y.clamp(min_y, entry_y - 1);
	loop {
		let mut moved = false;
		for r in obstacles {
			if x < r.x0 || x > r.x1 {
				continue;
			}
			if y_top <= r.y1 && r.y1 < entry_y {
				y_top = r.y1 + ROUTE_PAD_PX;
				moved = true;
				break;
			}
		}
		if !moved {
			break;
		}
	}
	CollectorSpan {
		target_ord,
		x,
		y_top: y_top.min(entry_y - 1),
		y_bottom: entry_y,
	}
}

fn collect_lines(
	obstacles: &[super::edge_router_graph::RectI],
	terminals: &[PointI],
	collectors: &[CollectorSpan],
	bounds: (i32, i32, i32, i32),
) -> (Vec<i32>, Vec<i32>) {
	let (min_x, max_x, min_y, max_y) = bounds;
	let mut xs: Vec<i32> = Vec::new();
	let mut ys: Vec<i32> = Vec::new();
	for r in obstacles {
		add_lane_lines(&mut xs, r.x0 - ROUTE_PAD_PX, SPACING_PX);
		add_lane_lines(&mut xs, r.x1 + ROUTE_PAD_PX, SPACING_PX);
		add_lane_lines(&mut ys, r.y0 - ROUTE_PAD_PX, SPACING_PX);
		add_lane_lines(&mut ys, r.y1 + ROUTE_PAD_PX, SPACING_PX);
	}
	for t in terminals {
		add_lane_lines(&mut xs, t.x, SPACING_PX);
		add_lane_lines(&mut ys, t.y, SPACING_PX);
	}
	for c in collectors {
		add_lane_lines(&mut xs, c.x, SPACING_PX);
		add_lane_lines(&mut ys, c.y_top, SPACING_PX);
		add_lane_lines(&mut ys, c.y_bottom, SPACING_PX);
	}
	add_lane_lines(&mut xs, min_x, SPACING_PX);
	add_lane_lines(&mut xs, max_x, SPACING_PX);
	add_lane_lines(&mut ys, min_y, SPACING_PX);
	add_lane_lines(&mut ys, max_y, SPACING_PX);

	for x in xs.iter_mut() {
		*x = (*x).clamp(min_x, max_x);
	}
	for y in ys.iter_mut() {
		*y = (*y).clamp(min_y, max_y);
	}

	xs.sort();
	xs.dedup();
	ys.sort();
	ys.dedup();
	(xs, ys)
}
pub fn route_edges(
	positioned: &HashMap<String, PositionedNode>,
	edges: &[LayoutEdge],
	_config: &SvgConfig,
) -> Result<Vec<Route>, RenderError> {
	if edges.is_empty() {
		return Ok(Vec::new());
	}

	let specs = build_edge_specs(edges);
	let dense_mode = specs.len() >= DENSE_EDGE_THRESHOLD;
	let obstacles = build_obstacles(positioned);

	let mut terminals: Vec<PointI> = Vec::new();
	for e in &specs {
		let a = positioned
			.get(e.source_id.as_str())
			.ok_or_else(|| RenderError::SvgMissingNode(e.source_id.clone()))?;
		let b = positioned
			.get(e.target_id.as_str())
			.ok_or_else(|| RenderError::SvgMissingNode(e.target_id.clone()))?;
		let ports = ports_for_edge(a.bbox, b.bbox, obstacles.as_slice());
		terminals.push(ports.source_exit);
		terminals.push(ports.target_entry);
	}
	let bounds = global_bounds(obstacles.as_slice(), terminals.as_slice());
	let bounds_center_x = (bounds.0 + bounds.1) / 2;
	let collector_center_y = (bounds.2 + bounds.3) / 2;

	// Deterministic target indexing for collector segments.
	let mut target_ids: Vec<String> = specs.iter().map(|e| e.target_id.clone()).collect();
	target_ids.sort_by(|l, r| {
		let lk = positioned_order_key(l.as_str(), positioned);
		let rk = positioned_order_key(r.as_str(), positioned);
		if dense_mode {
			let lx = positioned
				.get(l.as_str())
				.map(|node| node.bbox.center().x.round() as i32)
				.unwrap_or(i32::MAX / 4);
			let rx = positioned
				.get(r.as_str())
				.map(|node| node.bbox.center().x.round() as i32)
				.unwrap_or(i32::MAX / 4);
			(lx - bounds_center_x)
				.abs()
				.cmp(&(rx - bounds_center_x).abs())
				.then_with(|| lk.0.cmp(&rk.0))
				.then_with(|| lk.1.cmp(&rk.1))
				.then_with(|| unicode_cmp(l.as_str(), r.as_str()))
		} else {
			lk.0.cmp(&rk.0)
				.then_with(|| lk.1.cmp(&rk.1))
				.then_with(|| unicode_cmp(l.as_str(), r.as_str()))
		}
	});
	target_ids.dedup();
	let mut target_ord_by_id: HashMap<String, usize> = HashMap::new();
	for (i, id) in target_ids.iter().enumerate() {
		target_ord_by_id.insert(id.clone(), i);
	}

	let collectors: Vec<CollectorSpan> = target_ids
		.iter()
		.enumerate()
		.map(|(ord, id)| {
			let bbox = positioned
				.get(id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(id.clone()))?
				.bbox;
			Ok(collector_for_target(
				ord,
				bbox,
				obstacles.as_slice(),
				bounds.2,
				collector_center_y,
			))
		})
		.collect::<Result<Vec<_>, RenderError>>()?;

	let (xs, ys) = collect_lines(
		obstacles.as_slice(),
		terminals.as_slice(),
		collectors.as_slice(),
		bounds,
	);
	let graph: Graph = super::edge_router_graph::build_visibility_graph(
		obstacles.as_slice(),
		xs.as_slice(),
		ys.as_slice(),
		bounds,
		collectors.as_slice(),
	);

	let mut by_source: HashMap<String, Vec<EdgeSpec>> = HashMap::new();
	for e in specs {
		by_source.entry(e.source_id.clone()).or_default().push(e);
	}
	let mut sources: Vec<String> = by_source.keys().cloned().collect();
	sources.sort_by(|l, r| {
		let lk = positioned_order_key(l.as_str(), positioned);
		let rk = positioned_order_key(r.as_str(), positioned);
		if dense_mode {
			let lx = positioned
				.get(l.as_str())
				.map(|node| node.bbox.center().x.round() as i32)
				.unwrap_or(i32::MAX / 4);
			let rx = positioned
				.get(r.as_str())
				.map(|node| node.bbox.center().x.round() as i32)
				.unwrap_or(i32::MAX / 4);
			(lx - bounds_center_x)
				.abs()
				.cmp(&(rx - bounds_center_x).abs())
				.then_with(|| lk.0.cmp(&rk.0))
				.then_with(|| lk.1.cmp(&rk.1))
				.then_with(|| unicode_cmp(l.as_str(), r.as_str()))
		} else {
			lk.0.cmp(&rk.0)
				.then_with(|| lk.1.cmp(&rk.1))
				.then_with(|| unicode_cmp(l.as_str(), r.as_str()))
		}
	});

	let mut reservations: HashMap<EdgeKey, Reservation> = HashMap::new();
	let mut out_routes: Vec<Route> = Vec::new();

	for (source_ord, source_id) in sources.iter().enumerate() {
		let group = by_source
			.get(source_id.as_str())
			.cloned()
			.unwrap_or_default();
		let source_node = positioned
			.get(source_id.as_str())
			.ok_or_else(|| RenderError::SvgMissingNode(source_id.clone()))?;
		let root_ports = ports_for_edge(source_node.bbox, source_node.bbox, obstacles.as_slice());
		if !is_free_point(root_ports.source_exit, obstacles.as_slice()) {
			return Err(RenderError::SvgRouteFailed);
		}
		let root_idx = graph
			.index
			.get(&root_ports.source_exit)
			.copied()
			.ok_or(RenderError::SvgRouteFailed)?;

		let mut leaves: Vec<LeafSpec> = Vec::new();
		for e in group {
			let a = positioned
				.get(e.source_id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.source_id.clone()))?;
			let b = positioned
				.get(e.target_id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.target_id.clone()))?;
			let ports = ports_for_edge(a.bbox, b.bbox, obstacles.as_slice());
			let leaf_idx = graph
				.index
				.get(&ports.target_entry)
				.copied()
				.ok_or(RenderError::SvgRouteFailed)?;
			let target_ord = *target_ord_by_id.get(e.target_id.as_str()).unwrap_or(&0);
			leaves.push(LeafSpec {
				spec: e,
				target_ord,
				ports,
				leaf_idx,
			});
		}
		let unknown_bbox = geom::RectI {
			x: 0,
			y: 0,
			w: 0,
			h: 0,
		};
		let source_center = source_node.bbox.center();
		let mut sum_dx = 0.0f32;
		let mut sum_dy = 0.0f32;
		for leaf in &leaves {
			let bbox = positioned
				.get(leaf.spec.target_id.as_str())
				.map(|node| node.bbox)
				.unwrap_or(unknown_bbox);
			let center = bbox.center();
			sum_dx += (center.x - source_center.x).abs();
			sum_dy += (center.y - source_center.y).abs();
		}
		let primary_is_x = sum_dx >= sum_dy;
		leaves.sort_by(|l, r| {
			let lb = positioned
				.get(l.spec.target_id.as_str())
				.map(|n| n.bbox)
				.unwrap_or(unknown_bbox);
			let rb = positioned
				.get(r.spec.target_id.as_str())
				.map(|n| n.bbox)
				.unwrap_or(unknown_bbox);
			let l_center = lb.center();
			let r_center = rb.center();
			let (l_primary, l_secondary) = if primary_is_x {
				(l_center.x.round() as i32, l_center.y.round() as i32)
			} else {
				(l_center.y.round() as i32, l_center.x.round() as i32)
			};
			let (r_primary, r_secondary) = if primary_is_x {
				(r_center.x.round() as i32, r_center.y.round() as i32)
			} else {
				(r_center.y.round() as i32, r_center.x.round() as i32)
			};
			if dense_mode {
				let source_primary = if primary_is_x {
					source_center.x.round() as i32
				} else {
					source_center.y.round() as i32
				};
				(l_primary - source_primary)
					.abs()
					.cmp(&(r_primary - source_primary).abs())
					.then_with(|| l_primary.cmp(&r_primary))
					.then_with(|| l_secondary.cmp(&r_secondary))
					.then_with(|| lb.y.cmp(&rb.y))
					.then_with(|| lb.x.cmp(&rb.x))
					.then_with(|| unicode_cmp(l.spec.edge_id.as_str(), r.spec.edge_id.as_str()))
			} else {
				l_primary
					.cmp(&r_primary)
					.then_with(|| l_secondary.cmp(&r_secondary))
					.then_with(|| lb.y.cmp(&rb.y))
					.then_with(|| lb.x.cmp(&rb.x))
					.then_with(|| unicode_cmp(l.spec.edge_id.as_str(), r.spec.edge_id.as_str()))
			}
		});

		let mut tree_adj: HashMap<usize, Vec<usize>> = HashMap::new();
		let mut tree_vertices: HashSet<usize> = HashSet::new();
		tree_vertices.insert(root_idx);

		for leaf in &leaves {
			let Some(path) = dijkstra_path(
				&graph,
				&tree_vertices,
				leaf.leaf_idx,
				&reservations,
				source_ord,
				leaf.target_ord,
			) else {
				return Err(RenderError::SvgRouteFailed);
			};
			for w in path.windows(2) {
				let a = w[0];
				let b = w[1];
				tree_adj.entry(a).or_default().push(b);
				tree_adj.entry(b).or_default().push(a);
				let key = EdgeKey::new(a, b);
				let is_collector = graph.adj[a]
					.iter()
					.find(|e| e.to == b)
					.and_then(|e| e.collector_target())
					.is_some();
				reservations
					.entry(key)
					.and_modify(|reserved| match reserved {
						Reservation::Source { primary, count } => {
							if *primary != source_ord {
								*count = count.saturating_add(1);
							}
						}
						Reservation::TargetCollector { target, count } => {
							if *target == leaf.target_ord {
								*count = count.saturating_add(1);
							}
						}
					})
					.or_insert_with(|| {
						if is_collector {
							Reservation::TargetCollector {
								target: leaf.target_ord,
								count: 1,
							}
						} else {
							Reservation::Source {
								primary: source_ord,
								count: 1,
							}
						}
					});
			}
			for &v in &path {
				tree_vertices.insert(v);
			}
		}

		let parent = tree_parent_map(root_idx, &tree_adj);
		for leaf in leaves {
			let path = path_in_tree(&parent, root_idx, leaf.leaf_idx);
			let mut points: Vec<PointI> = Vec::new();
			points.push(leaf.ports.source_anchor);
			points.push(leaf.ports.source_exit);
			for &v in path.iter().skip(1) {
				points.push(graph.points[v]);
			}
			points.push(leaf.ports.target_anchor);
			let points = compress_points(points);
			out_routes.push(Route {
				edge_id: leaf.spec.edge_id,
				source_id: leaf.spec.source_id,
				target_id: leaf.spec.target_id,
				points: points.iter().copied().map(PointI::to_f).collect(),
				junction_ids: Vec::new(),
				track_id: "auto".to_string(),
				arrow: [geom::PointF { x: 0.0, y: 0.0 }; 3],
				bounds: geom::Bounds::empty(),
			});
		}
	}

	// Junction ids, then arrowheads/bounds.
	let mut shared: HashMap<PointI, usize> = HashMap::new();
	let all_points_i: Vec<Vec<PointI>> = out_routes
		.iter()
		.map(|r| {
			r.points
				.iter()
				.map(|p| PointI {
					x: p.x.round() as i32,
					y: p.y.round() as i32,
				})
				.collect::<Vec<_>>()
		})
		.collect();
	for pts in &all_points_i {
		for p in pts.iter().copied() {
			*shared.entry(p).or_insert(0) += 1;
		}
	}
	for (route, pts_i) in out_routes.iter_mut().zip(all_points_i.iter()) {
		route.junction_ids = junction_ids_for_route(pts_i.as_slice(), &shared);
		super::edge::finalize_route(route);
	}

	Ok(out_routes)
}

#[cfg(test)]
#[path = "tests/edge_router_tests.rs"]
mod edge_router_tests;
