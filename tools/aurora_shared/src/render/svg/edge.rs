use super::{EdgeStyle, geom};

const EDGE_STROKE: &str = "#000000";
const EDGE_STROKE_WIDTH_PX: i32 = 4;
const ARROW_SIZE_PX: f32 = 40.0;
const JUMP_RADIUS_PX: f32 = 16.0;
const JUMP_CLEAR_STROKE_WIDTH_PX: i32 = EDGE_STROKE_WIDTH_PX + 2;
pub(super) const LANE_COUNT: i32 = 10;

#[derive(Debug, Clone)]
pub struct Route {
	pub edge_id: String,
	pub source_id: String,
	pub target_id: String,
	pub points: Vec<geom::PointF>,
	pub junction_ids: Vec<String>,
	pub track_id: String,
	pub arrow: [geom::PointF; 3],
	pub bounds: geom::Bounds,
}

#[derive(Debug, Default, Clone)]
pub struct EdgeRenderLayers {
	pub base: String,
	pub overlay: String,
}

pub fn finalize_route(route: &mut Route) {
	route.points = compress_polyline(route.points.clone());
	route.arrow = arrowhead(route.points.as_slice());
	route.bounds = bounds_for_points(route.points.as_slice())
		.union_point(route.arrow[0])
		.union_point(route.arrow[1])
		.union_point(route.arrow[2]);
}

pub fn build_jump_overlay(routes: &[Route]) -> String {
	let mut jumps_by_route: Vec<Vec<geom::PointF>> = vec![Vec::new(); routes.len()];
	let stroke_routes: Vec<Vec<geom::PointF>> = routes.iter().map(route_stroke_points).collect();

	for left in 0..stroke_routes.len() {
		for right in left + 1..stroke_routes.len() {
			collect_pair_jumps(
				left,
				right,
				stroke_routes[left].as_slice(),
				stroke_routes[right].as_slice(),
				&mut jumps_by_route,
			);
		}
	}

	let mut out = String::new();
	for points in &jumps_by_route {
		for point in points {
			let d = jump_path(*point);
			out.push_str(&format!(
				"<path d=\"{}\" style=\"fill:none;stroke:#ffffff;stroke-width:{}px;stroke-linecap:round;stroke-linejoin:round;\" />",
				d, JUMP_CLEAR_STROKE_WIDTH_PX
			));
			out.push_str(&format!(
				"<path d=\"{}\" style=\"fill:none;stroke:{};stroke-width:{}px;stroke-linecap:round;stroke-linejoin:round;\" />",
				d, EDGE_STROKE, EDGE_STROKE_WIDTH_PX
			));
		}
	}

	out
}

fn collect_pair_jumps(
	left_index: usize,
	right_index: usize,
	left_route: &[geom::PointF],
	right_route: &[geom::PointF],
	jumps_by_route: &mut [Vec<geom::PointF>],
) {
	for left_segment in left_route.windows(2) {
		for right_segment in right_route.windows(2) {
			let Some((owner, point)) = segment_crossing_jump(
				left_index,
				right_index,
				left_segment[0],
				left_segment[1],
				right_segment[0],
				right_segment[1],
			) else {
				continue;
			};

			let jumps = &mut jumps_by_route[owner];
			if jumps.iter().any(|existing| {
				(existing.x - point.x).abs() < 0.5 && (existing.y - point.y).abs() < 0.5
			}) {
				continue;
			}
			jumps.push(point);
		}
	}
	for jumps in jumps_by_route.iter_mut() {
		jumps.sort_by(|left, right| {
			left.x
				.total_cmp(&right.x)
				.then_with(|| left.y.total_cmp(&right.y))
		});
	}
}

fn segment_crossing_jump(
	left_index: usize,
	right_index: usize,
	left_a: geom::PointF,
	left_b: geom::PointF,
	right_a: geom::PointF,
	right_b: geom::PointF,
) -> Option<(usize, geom::PointF)> {
	let left_horizontal = (left_a.y - left_b.y).abs() < 0.01;
	let left_vertical = (left_a.x - left_b.x).abs() < 0.01;
	let right_horizontal = (right_a.y - right_b.y).abs() < 0.01;
	let right_vertical = (right_a.x - right_b.x).abs() < 0.01;

	if left_horizontal && right_vertical {
		let point = orthogonal_segment_intersection(left_a, left_b, right_a, right_b)?;
		if jump_too_close_to_ends(point, left_a, left_b)
			|| jump_too_close_to_ends(point, right_a, right_b)
		{
			return None;
		}
		return Some((left_index, point));
	}

	if left_vertical && right_horizontal {
		let point = orthogonal_segment_intersection(right_a, right_b, left_a, left_b)?;
		if jump_too_close_to_ends(point, left_a, left_b)
			|| jump_too_close_to_ends(point, right_a, right_b)
		{
			return None;
		}
		return Some((right_index, point));
	}

	None
}

fn orthogonal_segment_intersection(
	horizontal_a: geom::PointF,
	horizontal_b: geom::PointF,
	vertical_a: geom::PointF,
	vertical_b: geom::PointF,
) -> Option<geom::PointF> {
	let x = vertical_a.x;
	let y = horizontal_a.y;
	let h_min = horizontal_a.x.min(horizontal_b.x);
	let h_max = horizontal_a.x.max(horizontal_b.x);
	let v_min = vertical_a.y.min(vertical_b.y);
	let v_max = vertical_a.y.max(vertical_b.y);

	((x > h_min + 0.01) && (x < h_max - 0.01) && (y > v_min + 0.01) && (y < v_max - 0.01))
		.then_some(geom::PointF { x, y })
}

fn jump_too_close_to_ends(point: geom::PointF, a: geom::PointF, b: geom::PointF) -> bool {
	let threshold = JUMP_RADIUS_PX * 1.5;
	if (a.y - b.y).abs() < 0.01 {
		let min_dist = (point.x - a.x).abs().min((point.x - b.x).abs());
		return min_dist < threshold;
	}
	let min_dist = (point.y - a.y).abs().min((point.y - b.y).abs());
	min_dist < threshold
}

fn jump_path(center: geom::PointF) -> String {
	let r = JUMP_RADIUS_PX;
	format!(
		"M {:.2} {:.2} Q {:.2} {:.2} {:.2} {:.2}",
		center.x - r,
		center.y,
		center.x,
		center.y - r,
		center.x + r,
		center.y,
	)
}

pub fn render_edge_layers(route: &Route, style: EdgeStyle) -> EdgeRenderLayers {
	let mut layers = EdgeRenderLayers::default();
	match style {
		EdgeStyle::Orthogonal | EdgeStyle::Curved => {
			let points = route_stroke_points(route);
			if points.len() >= 2 {
				let d = path_polyline(points.as_slice());
				layers.base.push_str(&format!(
					"<path d=\"{}\" style=\"fill:none;stroke:{};stroke-width:{}px;stroke-linecap:round;stroke-linejoin:round;\" />",
					d, EDGE_STROKE, EDGE_STROKE_WIDTH_PX
				));
			}
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

pub fn route_obstacles_for_later_edges(
	route: &Route,
	_cell_px: i32,
	_source_bbox: &geom::RectI,
	_target_bbox: &geom::RectI,
	padding_px: i32,
	source_merge: bool,
	target_merge: bool,
) -> Vec<geom::RectI> {
	let _ = route;
	let _ = _cell_px;
	let _ = _source_bbox;
	let _ = _target_bbox;
	let _ = padding_px;
	let _ = source_merge;
	let _ = target_merge;
	Vec::new()
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

fn arrowhead(points: &[geom::PointF]) -> [geom::PointF; 3] {
	let size = ARROW_SIZE_PX;
	let width = size * 0.6;
	let end = *points.last().unwrap_or(&geom::PointF { x: 0.0, y: 0.0 });
	let dir = arrow_direction(points);
	let base = geom::PointF {
		x: end.x - dir.x * size,
		y: end.y - dir.y * size,
	};
	let perp = geom::PointF {
		x: -dir.y,
		y: dir.x,
	};
	[
		end,
		geom::PointF {
			x: base.x + perp.x * (width / 2.0),
			y: base.y + perp.y * (width / 2.0),
		},
		geom::PointF {
			x: base.x - perp.x * (width / 2.0),
			y: base.y - perp.y * (width / 2.0),
		},
	]
}

fn arrow_direction(points: &[geom::PointF]) -> geom::PointF {
	if points.len() < 2 {
		return geom::PointF { x: 1.0, y: 0.0 };
	}
	for segment in points.windows(2).rev() {
		let dx = segment[1].x - segment[0].x;
		let dy = segment[1].y - segment[0].y;
		let length = (dx * dx + dy * dy).sqrt();
		if length > f32::EPSILON {
			return geom::PointF {
				x: dx / length,
				y: dy / length,
			};
		}
	}
	geom::PointF { x: 1.0, y: 0.0 }
}

fn bounds_for_points(points: &[geom::PointF]) -> geom::Bounds {
	let mut bounds = geom::Bounds::empty();
	for point in points {
		bounds = bounds.union_point(*point);
	}
	bounds
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

fn compress_polyline(points: Vec<geom::PointF>) -> Vec<geom::PointF> {
	if points.len() <= 2 {
		return points;
	}
	const EPS: f32 = 0.01;

	let mut deduped: Vec<geom::PointF> = Vec::new();
	for point in points {
		if deduped
			.last()
			.is_some_and(|last| (last.x - point.x).abs() < EPS && (last.y - point.y).abs() < EPS)
		{
			continue;
		}
		deduped.push(point);
	}
	if deduped.len() <= 2 {
		return deduped;
	}

	let mut out = vec![deduped[0], deduped[1]];
	for point in deduped.iter().skip(2).copied() {
		let a = out[out.len() - 2];
		let b = out[out.len() - 1];
		let same_x = (a.x - b.x).abs() < EPS && (b.x - point.x).abs() < EPS;
		let same_y = (a.y - b.y).abs() < EPS && (b.y - point.y).abs() < EPS;
		if same_x || same_y {
			if let Some(last) = out.last_mut() {
				*last = point;
			}
		} else {
			out.push(point);
		}
	}

	out
}

#[cfg(test)]
#[path = "tests/edge_tests.rs"]
mod edge_tests;
