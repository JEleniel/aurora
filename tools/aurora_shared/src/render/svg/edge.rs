use super::{EdgeStyle, RenderError, SvgConfig, geom};

const EDGE_STROKE: &str = "#000000";
const EDGE_STROKE_WIDTH_PX: i32 = 2;
const ARROW_SIZE_PX: f32 = 40.0;
const JUMP_RADIUS_PX: f32 = 8.0;
const JUMP_CLEAR_STROKE_WIDTH_PX: i32 = EDGE_STROKE_WIDTH_PX + 2;
const TERMINAL_DIRECTION_PENALTY: i64 = 120;
pub(super) const LANE_COUNT: i32 = 10;
pub(super) const LANE_PITCH_PX: f32 = 16.0;
const CHANNEL_WIDTH_PX: f32 = 160.0;
const CHANNEL_OUTER_CLEAR_PX: f32 = 80.0;

#[derive(Debug, Clone)]
pub struct Route {
	pub points: Vec<geom::PointF>,
	pub arrow: [geom::PointF; 3],
	pub bounds: geom::Bounds,
}

#[derive(Debug, Default, Clone)]
pub struct EdgeRenderLayers {
	pub base: String,
	pub overlay: String,
}

pub fn route_edge(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	_edge_obstacles: &[geom::RectI],
	symbol_bboxes: &[geom::RectI],
	source_anchor_bias: f32,
	target_anchor_bias: f32,
	source_merge: bool,
	target_merge: bool,
	config: &SvgConfig,
) -> Result<Route, RenderError> {
	let _ = config;
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();
	let start_anchor = preferred_source_anchor_point(source_bbox, source_anchor_bias);
	let end_anchor = preferred_target_anchor_point(target_bbox, target_anchor_bias);

	let prefer_vertical_terminals = source_center.y <= target_center.y;
	let primary_is_vertical_channel = !prefer_vertical_terminals
		&& (target_center.x - source_center.x).abs() >= (target_center.y - source_center.y).abs();
	let primary = if primary_is_vertical_channel {
		route_through_vertical_channel(
			source_bbox,
			target_bbox,
			start_anchor,
			end_anchor,
			source_anchor_bias,
			target_anchor_bias,
			source_merge,
			target_merge,
		)
	} else {
		route_through_horizontal_channel(
			source_bbox,
			target_bbox,
			start_anchor,
			end_anchor,
			source_anchor_bias,
			target_anchor_bias,
			source_merge,
			target_merge,
		)
	};
	let secondary = if primary_is_vertical_channel {
		route_through_horizontal_channel(
			source_bbox,
			target_bbox,
			start_anchor,
			end_anchor,
			source_anchor_bias,
			target_anchor_bias,
			source_merge,
			target_merge,
		)
	} else {
		route_through_vertical_channel(
			source_bbox,
			target_bbox,
			start_anchor,
			end_anchor,
			source_anchor_bias,
			target_anchor_bias,
			source_merge,
			target_merge,
		)
	};

	let symbol_obstacles =
		collect_symbol_obstacles(source_bbox, target_bbox, node_obstacles, symbol_bboxes);

	let mut candidates: Vec<Vec<geom::PointF>> = vec![primary, secondary];
	candidates.extend(detour_routes(start_anchor, end_anchor, &symbol_obstacles));
	candidates.extend(expanded_candidate_routes(
		start_anchor,
		end_anchor,
		symbol_obstacles.as_slice(),
	));

	let points = choose_non_crossing_route(
		candidates.as_slice(),
		symbol_obstacles.as_slice(),
		start_anchor,
		end_anchor,
	)
	.ok_or(RenderError::SvgRouteFailed)?;
	let points = enforce_terminal_clearance(points, start_anchor, end_anchor);
	let points = compress_polyline(points);
	let arrow = arrowhead(points.as_slice());
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

fn collect_symbol_obstacles(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	node_obstacles: &[geom::RectI],
	symbol_bboxes: &[geom::RectI],
) -> Vec<geom::RectI> {
	let mut obstacles: Vec<geom::RectI> = Vec::new();
	for obstacle in symbol_bboxes {
		if !same_rect(*obstacle, *source_bbox) && !same_rect(*obstacle, *target_bbox) {
			obstacles.push(inflate_rect(*obstacle, CHANNEL_OUTER_CLEAR_PX as i32));
		}
	}
	for obstacle in node_obstacles {
		if !rect_overlaps(*obstacle, *source_bbox) && !rect_overlaps(*obstacle, *target_bbox) {
			obstacles.push(inflate_rect(*obstacle, CHANNEL_OUTER_CLEAR_PX as i32));
		}
	}
	obstacles.sort_by(|left, right| {
		left.x
			.cmp(&right.x)
			.then_with(|| left.y.cmp(&right.y))
			.then_with(|| left.w.cmp(&right.w))
			.then_with(|| left.h.cmp(&right.h))
	});
	obstacles.dedup_by(|left, right| {
		left.x == right.x && left.y == right.y && left.w == right.w && left.h == right.h
	});
	obstacles
}

fn same_rect(a: geom::RectI, b: geom::RectI) -> bool {
	a.x == b.x && a.y == b.y && a.w == b.w && a.h == b.h
}

fn inflate_rect(rect: geom::RectI, pad: i32) -> geom::RectI {
	let p = pad.max(0);
	geom::RectI {
		x: rect.x - p,
		y: rect.y - p,
		w: rect.w + (2 * p),
		h: rect.h + (2 * p),
	}
}

fn detour_routes(
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
	obstacles: &[geom::RectI],
) -> Vec<Vec<geom::PointF>> {
	if obstacles.is_empty() {
		return Vec::new();
	}

	let min_x = obstacles.iter().map(|rect| rect.x).min().unwrap_or(0) as f32;
	let min_y = obstacles.iter().map(|rect| rect.y).min().unwrap_or(0) as f32;
	let max_x = obstacles
		.iter()
		.map(|rect| rect.x + rect.w)
		.max()
		.unwrap_or(0) as f32;
	let max_y = obstacles
		.iter()
		.map(|rect| rect.y + rect.h)
		.max()
		.unwrap_or(0) as f32;
	let clearance = CHANNEL_OUTER_CLEAR_PX + CHANNEL_WIDTH_PX / 2.0;

	vec![
		vec![
			start_anchor,
			geom::PointF {
				x: start_anchor.x,
				y: min_y - clearance,
			},
			geom::PointF {
				x: end_anchor.x,
				y: min_y - clearance,
			},
			end_anchor,
		],
		vec![
			start_anchor,
			geom::PointF {
				x: start_anchor.x,
				y: max_y + clearance,
			},
			geom::PointF {
				x: end_anchor.x,
				y: max_y + clearance,
			},
			end_anchor,
		],
		vec![
			start_anchor,
			geom::PointF {
				x: min_x - clearance,
				y: start_anchor.y,
			},
			geom::PointF {
				x: min_x - clearance,
				y: end_anchor.y,
			},
			end_anchor,
		],
		vec![
			start_anchor,
			geom::PointF {
				x: max_x + clearance,
				y: start_anchor.y,
			},
			geom::PointF {
				x: max_x + clearance,
				y: end_anchor.y,
			},
			end_anchor,
		],
	]
}

fn expanded_candidate_routes(
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
	obstacles: &[geom::RectI],
) -> Vec<Vec<geom::PointF>> {
	if obstacles.is_empty() {
		return Vec::new();
	}

	let (x_lines, y_lines) = candidate_lines(start_anchor, end_anchor, obstacles);
	let mut candidates: Vec<Vec<geom::PointF>> = Vec::new();

	for x in &x_lines {
		candidates.push(vec![
			start_anchor,
			geom::PointF {
				x: *x,
				y: start_anchor.y,
			},
			geom::PointF {
				x: *x,
				y: end_anchor.y,
			},
			end_anchor,
		]);
		for y in &y_lines {
			candidates.push(vec![
				start_anchor,
				geom::PointF {
					x: *x,
					y: start_anchor.y,
				},
				geom::PointF { x: *x, y: *y },
				geom::PointF {
					x: end_anchor.x,
					y: *y,
				},
				end_anchor,
			]);
		}
	}

	for y in &y_lines {
		candidates.push(vec![
			start_anchor,
			geom::PointF {
				x: start_anchor.x,
				y: *y,
			},
			geom::PointF {
				x: end_anchor.x,
				y: *y,
			},
			end_anchor,
		]);
		for x in &x_lines {
			candidates.push(vec![
				start_anchor,
				geom::PointF {
					x: start_anchor.x,
					y: *y,
				},
				geom::PointF { x: *x, y: *y },
				geom::PointF {
					x: *x,
					y: end_anchor.y,
				},
				end_anchor,
			]);
		}
	}

	candidates
}

fn candidate_lines(
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
	obstacles: &[geom::RectI],
) -> (Vec<f32>, Vec<f32>) {
	let mut x_lines = vec![start_anchor.x, end_anchor.x];
	let mut y_lines = vec![start_anchor.y, end_anchor.y];

	let min_x = obstacles.iter().map(|rect| rect.x).min().unwrap_or(0) as f32;
	let min_y = obstacles.iter().map(|rect| rect.y).min().unwrap_or(0) as f32;
	let max_x = obstacles
		.iter()
		.map(|rect| rect.x + rect.w)
		.max()
		.unwrap_or(0) as f32;
	let max_y = obstacles
		.iter()
		.map(|rect| rect.y + rect.h)
		.max()
		.unwrap_or(0) as f32;
	let clearance = CHANNEL_OUTER_CLEAR_PX + CHANNEL_WIDTH_PX / 2.0;

	x_lines.push(min_x - clearance);
	x_lines.push(max_x + clearance);
	y_lines.push(min_y - clearance);
	y_lines.push(max_y + clearance);

	for rect in obstacles {
		x_lines.push(rect.x as f32 - clearance);
		x_lines.push((rect.x + rect.w) as f32 + clearance);
		y_lines.push(rect.y as f32 - clearance);
		y_lines.push((rect.y + rect.h) as f32 + clearance);
	}

	quantize_and_dedup_lines(&mut x_lines);
	quantize_and_dedup_lines(&mut y_lines);
	(x_lines, y_lines)
}

fn quantize_and_dedup_lines(lines: &mut Vec<f32>) {
	lines.sort_by(|left, right| left.total_cmp(right));
	lines.dedup_by(|left, right| (*left - *right).abs() < 0.5);
}

fn choose_non_crossing_route(
	candidates: &[Vec<geom::PointF>],
	obstacles: &[geom::RectI],
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
) -> Option<Vec<geom::PointF>> {
	let mut best_strict: Option<(i64, usize, Vec<geom::PointF>)> = None;
	let mut best_relaxed: Option<(i64, usize, Vec<geom::PointF>)> = None;

	let better = |candidate_score: i64,
	              candidate_index: usize,
	              current: &Option<(i64, usize, Vec<geom::PointF>)>| {
		match current {
			None => true,
			Some((best_score, best_index, _)) => {
				candidate_score < *best_score
					|| (candidate_score == *best_score && candidate_index < *best_index)
			}
		}
	};

	for (index, candidate) in candidates.iter().enumerate() {
		let points = compress_polyline(candidate.clone());
		if points.len() < 2 || route_crosses_symbols(points.as_slice(), obstacles) {
			continue;
		}
		let score = route_score(points.as_slice(), start_anchor, end_anchor);
		if starts_and_ends_vertical(points.as_slice(), start_anchor, end_anchor) {
			if better(score, index, &best_strict) {
				best_strict = Some((score, index, points));
			}
		} else if better(score, index, &best_relaxed) {
			best_relaxed = Some((score, index, points));
		}
	}

	best_strict.or(best_relaxed).map(|(_, _, points)| points)
}

fn starts_and_ends_vertical(
	points: &[geom::PointF],
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
) -> bool {
	if points.len() < 2 {
		return false;
	}
	let first_vertical = (points[1].x - start_anchor.x).abs() < 0.01;
	let last_vertical = (points[points.len() - 2].x - end_anchor.x).abs() < 0.01;
	first_vertical && last_vertical
}

fn enforce_terminal_clearance(
	mut points: Vec<geom::PointF>,
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
) -> Vec<geom::PointF> {
	if points.len() < 2 {
		return points;
	}

	let direction = if end_anchor.y >= start_anchor.y {
		1.0
	} else {
		-1.0
	};
	let mut start_depart_y = start_anchor.y + direction * CHANNEL_OUTER_CLEAR_PX;
	let mut end_entry_y = end_anchor.y - direction * CHANNEL_OUTER_CLEAR_PX;
	if (direction > 0.0 && start_depart_y > end_entry_y)
		|| (direction < 0.0 && start_depart_y < end_entry_y)
	{
		let mid = (start_anchor.y + end_anchor.y) / 2.0;
		start_depart_y = mid;
		end_entry_y = mid;
	}

	if (points[1].y - start_anchor.y).abs() < 0.01 && (points[1].x - start_anchor.x).abs() >= 0.01 {
		points.insert(
			1,
			geom::PointF {
				x: start_anchor.x,
				y: start_depart_y,
			},
		);
		if let Some(first_turn) = points.get_mut(2) {
			first_turn.y = start_depart_y;
		}
	}

	if points.len() >= 2 {
		let penultimate_index = points.len() - 2;
		if (points[penultimate_index].y - end_anchor.y).abs() < 0.01
			&& (points[penultimate_index].x - end_anchor.x).abs() >= 0.01
		{
			if let Some(last_turn) = points.get_mut(penultimate_index) {
				last_turn.y = end_entry_y;
			}
			points.insert(
				points.len() - 1,
				geom::PointF {
					x: end_anchor.x,
					y: end_entry_y,
				},
			);
		}
	}

	points
}

fn route_score(
	points: &[geom::PointF],
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
) -> i64 {
	let mut length = 0.0f64;
	for segment in points.windows(2) {
		let dx = segment[1].x as f64 - segment[0].x as f64;
		let dy = segment[1].y as f64 - segment[0].y as f64;
		length += (dx * dx + dy * dy).sqrt();
	}
	let bends = points.len().saturating_sub(2) as i64;
	(length.round() as i64)
		+ bends * 16
		+ terminal_direction_penalty(points, start_anchor, end_anchor)
}

fn terminal_direction_penalty(
	points: &[geom::PointF],
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
) -> i64 {
	if points.len() < 2 {
		return 0;
	}

	let first_vertical = (points[1].x - start_anchor.x).abs() < 0.01;
	let last_index = points.len() - 1;
	let last_vertical = (points[last_index - 1].x - end_anchor.x).abs() < 0.01;

	let mut penalty = 0;
	if !first_vertical {
		penalty += TERMINAL_DIRECTION_PENALTY;
	}
	if !last_vertical {
		penalty += TERMINAL_DIRECTION_PENALTY;
	}
	penalty
}

fn route_crosses_symbols(points: &[geom::PointF], obstacles: &[geom::RectI]) -> bool {
	for segment in points.windows(2) {
		for obstacle in obstacles {
			if segment_intersects_rect(segment[0], segment[1], *obstacle) {
				return true;
			}
		}
	}
	false
}

fn segment_intersects_rect(a: geom::PointF, b: geom::PointF, rect: geom::RectI) -> bool {
	const EPS: f32 = 0.01;
	let left = rect.x as f32;
	let right = (rect.x + rect.w) as f32;
	let top = rect.y as f32;
	let bottom = (rect.y + rect.h) as f32;

	if (a.x - b.x).abs() < 0.01 {
		let x = a.x;
		let y_min = a.y.min(b.y);
		let y_max = a.y.max(b.y);
		let x_in = x > left + EPS && x < right - EPS;
		let y_overlap = y_max > top + EPS && y_min < bottom - EPS;
		return x_in && y_overlap;
	}

	if (a.y - b.y).abs() < 0.01 {
		let y = a.y;
		let x_min = a.x.min(b.x);
		let x_max = a.x.max(b.x);
		let y_in = y > top + EPS && y < bottom - EPS;
		let x_overlap = x_max > left + EPS && x_min < right - EPS;
		return y_in && x_overlap;
	}

	let seg_rect = segment_rect(a, b, 0);
	rect_overlaps(seg_rect, rect)
}

fn rect_overlaps(a: geom::RectI, b: geom::RectI) -> bool {
	let a_right = a.x + a.w;
	let a_bottom = a.y + a.h;
	let b_right = b.x + b.w;
	let b_bottom = b.y + b.h;
	a.x < b_right && a_right > b.x && a.y < b_bottom && a_bottom > b.y
}

fn route_through_vertical_channel(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
	source_bias: f32,
	target_bias: f32,
	source_merge: bool,
	target_merge: bool,
) -> Vec<geom::PointF> {
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();
	let lane_bias = resolve_lane_bias(source_bias, target_bias, source_merge, target_merge);
	let terminal_leg = CHANNEL_OUTER_CLEAR_PX;

	let (left_edge, right_edge) = if source_center.x <= target_center.x {
		((source_bbox.x + source_bbox.w) as f32, target_bbox.x as f32)
	} else {
		((target_bbox.x + target_bbox.w) as f32, source_bbox.x as f32)
	};
	let source_left = source_bbox.x as f32;
	let source_right = (source_bbox.x + source_bbox.w) as f32;
	let target_left = target_bbox.x as f32;
	let target_right = (target_bbox.x + target_bbox.w) as f32;
	let overlap_x = source_left < target_right && source_right > target_left;
	let lane_x = if overlap_x {
		(start_anchor.x + end_anchor.x) / 2.0
	} else {
		lane_position(left_edge, right_edge, lane_bias)
	};

	let mut start_exit_y = start_anchor.y + terminal_leg;
	let mut end_entry_y = end_anchor.y - terminal_leg;
	if start_exit_y > end_entry_y {
		let mid = (start_anchor.y + end_anchor.y) / 2.0;
		start_exit_y = mid;
		end_entry_y = mid;
	}

	vec![
		start_anchor,
		geom::PointF {
			x: start_anchor.x,
			y: start_exit_y,
		},
		geom::PointF {
			x: lane_x,
			y: start_exit_y,
		},
		geom::PointF {
			x: lane_x,
			y: end_entry_y,
		},
		geom::PointF {
			x: end_anchor.x,
			y: end_entry_y,
		},
		end_anchor,
	]
}

fn route_through_horizontal_channel(
	source_bbox: &geom::RectI,
	target_bbox: &geom::RectI,
	start_anchor: geom::PointF,
	end_anchor: geom::PointF,
	source_bias: f32,
	target_bias: f32,
	source_merge: bool,
	target_merge: bool,
) -> Vec<geom::PointF> {
	let source_center = source_bbox.center();
	let target_center = target_bbox.center();
	let lane_bias = resolve_lane_bias(source_bias, target_bias, source_merge, target_merge);
	let terminal_leg = CHANNEL_OUTER_CLEAR_PX;
	let source_above_target = source_center.y <= target_center.y;

	let start_depart_y = if source_above_target {
		start_anchor.y + terminal_leg
	} else {
		start_anchor.y - terminal_leg
	};
	let end_entry_y = if source_above_target {
		end_anchor.y - terminal_leg
	} else {
		end_anchor.y + terminal_leg
	};

	let lane_y = lane_position(start_depart_y, end_entry_y, lane_bias);

	vec![
		start_anchor,
		geom::PointF {
			x: start_anchor.x,
			y: start_depart_y,
		},
		geom::PointF {
			x: start_anchor.x,
			y: lane_y,
		},
		geom::PointF {
			x: end_anchor.x,
			y: lane_y,
		},
		geom::PointF {
			x: end_anchor.x,
			y: end_entry_y,
		},
		end_anchor,
	]
}

fn resolve_lane_bias(
	source_bias: f32,
	target_bias: f32,
	source_merge: bool,
	target_merge: bool,
) -> f32 {
	if source_merge || target_merge {
		0.0
	} else {
		((source_bias + target_bias) / 2.0).clamp(-1.0, 1.0)
	}
}

fn lane_position(start_edge: f32, end_edge: f32, bias: f32) -> f32 {
	let low = start_edge.min(end_edge);
	let high = start_edge.max(end_edge);
	let gap = (high - low).max(1.0);

	let lane_band_start = if gap >= 320.0 {
		let extra = gap - 320.0;
		low + (extra / 2.0) + CHANNEL_OUTER_CLEAR_PX
	} else {
		let shortfall = (CHANNEL_WIDTH_PX - gap).max(0.0);
		(low - shortfall / 2.0).min(low)
	};

	let lane_index = bias_to_lane_index(bias);
	let lane_mid = lane_band_start + (lane_index as f32 + 0.5) * LANE_PITCH_PX;
	lane_mid.clamp(low, high)
}

fn bias_to_lane_index(bias: f32) -> i32 {
	let normalized = ((bias.clamp(-1.0, 1.0) + 1.0) / 2.0).clamp(0.0, 1.0);
	(normalized * (LANE_COUNT - 1) as f32).round() as i32
}

fn preferred_source_anchor_point(from: &geom::RectI, bias: f32) -> geom::PointF {
	let x0 = from.x as f32;
	let x1 = (from.x + from.w) as f32;
	let y1 = (from.y + from.h) as f32;
	geom::PointF {
		x: biased_coordinate(x0, x1, bias),
		y: y1,
	}
}

fn preferred_target_anchor_point(from: &geom::RectI, bias: f32) -> geom::PointF {
	let x0 = from.x as f32;
	let x1 = (from.x + from.w) as f32;
	let y0 = from.y as f32;
	geom::PointF {
		x: biased_coordinate(x0, x1, bias),
		y: y0,
	}
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

fn biased_coordinate(min: f32, max: f32, bias: f32) -> f32 {
	let inner_min = min + CHANNEL_OUTER_CLEAR_PX;
	let inner_max = max - CHANNEL_OUTER_CLEAR_PX;
	if inner_max <= inner_min {
		return (min + max) / 2.0;
	}

	let center = (inner_min + inner_max) / 2.0;
	let max_lane_offset = ((LANE_COUNT - 1) as f32 / 2.0) * LANE_PITCH_PX;
	let lane_offset = bias.clamp(-1.0, 1.0) * max_lane_offset;
	(center + lane_offset).clamp(inner_min, inner_max)
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
) -> Vec<geom::RectI> {
	let mut obstacles = Vec::new();
	for segment in route.points.windows(2) {
		obstacles.push(segment_rect(segment[0], segment[1], padding_px.max(1)));
	}
	obstacles
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
mod tests {
	use super::*;

	#[test]
	fn arrowhead_uses_fixed_40px_length() {
		let points = vec![
			geom::PointF { x: 0.0, y: 0.0 },
			geom::PointF { x: 120.0, y: 0.0 },
		];
		let arrow = arrowhead(points.as_slice());
		let dx = arrow[0].x - arrow[1].x;
		let dy = arrow[0].y - arrow[1].y;
		let side = (dx * dx + dy * dy).sqrt();
		assert!(
			side > 39.0,
			"arrow side should reflect fixed 40px length, got: {side}"
		);
	}

	#[test]
	fn orthogonal_rendering_uses_single_2px_stroke() {
		let route = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 0.0 },
				geom::PointF { x: 100.0, y: 0.0 },
			],
			arrow: [
				geom::PointF { x: 100.0, y: 0.0 },
				geom::PointF { x: 60.0, y: 12.0 },
				geom::PointF { x: 60.0, y: -12.0 },
			],
			bounds: geom::Bounds::empty(),
		};

		let layers = render_edge_layers(&route, EdgeStyle::Orthogonal);
		assert!(layers.base.contains("stroke:#000000;stroke-width:2px"));
		assert!(!layers.base.contains("stroke:#ffffff"));
	}

	#[test]
	fn route_prefers_channel_lanes_between_columns() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 1080,
			y: 0,
			w: 720,
			h: 450,
		};
		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, target],
			0.0,
			0.0,
			false,
			false,
			&SvgConfig::default(),
		)
		.expect("route");

		assert!(route.points.len() >= 2);
		let source_bottom = (source.y + source.h) as f32;
		let target_top = target.y as f32;
		assert!(
			route
				.points
				.iter()
				.any(|point| point.y > target_top && point.y < source_bottom),
			"expected at least one routed point in the inter-row channel: {:?}",
			route.points
		);
	}

	#[test]
	fn route_prefers_bottom_origin_top_target() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 1080,
			y: 810,
			w: 720,
			h: 450,
		};

		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, target],
			0.0,
			0.0,
			false,
			false,
			&SvgConfig::default(),
		)
		.expect("route");

		let start = route.points.first().expect("start");
		let end = route.points.last().expect("end");
		assert_eq!(start.y, (source.y + source.h) as f32);
		assert_eq!(end.y, target.y as f32);

		if route.points.len() >= 3 {
			assert!((route.points[1].x - start.x).abs() < 0.01);
			assert!((route.points[route.points.len() - 2].x - end.x).abs() < 0.01);
			assert!(
				route.points[1].y > start.y,
				"route should depart source edge before lateral travel: {:?}",
				route.points
			);
		}
	}

	#[test]
	fn build_jump_overlay_renders_arc_for_crossing() {
		let horizontal = Route {
			points: vec![
				geom::PointF { x: 0.0, y: 50.0 },
				geom::PointF { x: 200.0, y: 50.0 },
			],
			arrow: [
				geom::PointF { x: 200.0, y: 50.0 },
				geom::PointF { x: 160.0, y: 62.0 },
				geom::PointF { x: 160.0, y: 38.0 },
			],
			bounds: geom::Bounds::empty(),
		};
		let vertical = Route {
			points: vec![
				geom::PointF { x: 50.0, y: 0.0 },
				geom::PointF { x: 50.0, y: 200.0 },
			],
			arrow: [
				geom::PointF { x: 50.0, y: 200.0 },
				geom::PointF { x: 38.0, y: 160.0 },
				geom::PointF { x: 62.0, y: 160.0 },
			],
			bounds: geom::Bounds::empty(),
		};

		let jumps = build_jump_overlay(&[horizontal, vertical]);
		assert!(jumps.contains("stroke:#ffffff"));
		assert!(jumps.contains("stroke:#000000"));
		assert!(jumps.contains("Q 50.00 42.00"));
	}

	#[test]
	fn route_does_not_cross_intermediate_symbol() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let middle = geom::RectI {
			x: 1080,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 2160,
			y: 0,
			w: 720,
			h: 450,
		};

		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, middle, target],
			0.0,
			0.0,
			false,
			false,
			&SvgConfig::default(),
		)
		.expect("route");

		assert!(
			!route_crosses_symbols(route.points.as_slice(), &[middle]),
			"route crossed intermediate symbol: {:?}",
			route.points
		);

		let clearance_rect = inflate_rect(middle, CHANNEL_OUTER_CLEAR_PX as i32);
		assert!(
			!route_crosses_symbols(route.points.as_slice(), &[clearance_rect]),
			"route violated {}px symbol clearance: {:?}",
			CHANNEL_OUTER_CLEAR_PX,
			route.points
		);
	}

	#[test]
	fn stacked_nodes_route_straight_without_left_jog() {
		let source = geom::RectI {
			x: 0,
			y: 0,
			w: 720,
			h: 450,
		};
		let target = geom::RectI {
			x: 0,
			y: 810,
			w: 720,
			h: 450,
		};

		let route = route_edge(
			&source,
			&target,
			&[],
			&[],
			&[source, target],
			0.0,
			0.0,
			false,
			false,
			&SvgConfig::default(),
		)
		.expect("route");

		for point in &route.points {
			assert!(
				(point.x - 360.0).abs() < 0.01,
				"expected straight vertical routing, got {:?}",
				route.points
			);
		}
	}
}
