//! Theme-aware SVG rendering based on Graphviz layout output.
//!
//! Graphviz provides node/edge geometry via `-Tplain`. This module converts that geometry into
//! an SVG that uses CSS variables and `prefers-color-scheme` to support light/dark mode.

use std::collections::BTreeMap;

use crate::model::Card;

use super::graphviz_plain::{PlainEdge, PlainGraph, PlainNode, Point};
use super::{BoundaryCluster, CardColor};

const PX_PER_IN: f64 = 96.0;
const NODE_FONT_SIZE_PX: f64 = 11.0;
const EDGE_FONT_SIZE_PX: f64 = 9.0;
const LINE_HEIGHT_PX: f64 = 14.0;
const ICON_SCALE: f64 = 3.0;
const ICON_OFFSET_PX: f64 = 16.0;
const ICON_GAP_PX: f64 = 8.0;
const BOUNDARY_PAD_PX: f64 = 18.0;
const BOUNDARY_MIN_PAD_PX: f64 = 0.0;

pub(super) fn render_svg(
	layout: &PlainGraph,
	nodes: &BTreeMap<String, &Card>,
	icons: &std::collections::HashMap<String, String>,
	colors: &std::collections::HashMap<String, CardColor>,
	clusters: &[BoundaryCluster],
) -> String {
	let width = layout.width_in * PX_PER_IN;
	let height = layout.height_in * PX_PER_IN;

	let mut svg = String::new();
	svg.push_str(&format!(
		"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.2} {height:.2}\" width=\"{width:.2}\" height=\"{height:.2}\" data-theme=\"light\">\n"
	));
	svg.push_str("<defs>\n");
	svg.push_str(&svg_style(colors));
	svg.push_str(svg_markers());
	svg.push_str("</defs>\n");

	svg.push_str("<rect class=\"bg\" x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" />\n");

	// Boundaries first, so they render behind nodes.
	for (cluster, rect) in resolve_boundary_rects(layout, clusters) {
		svg.push_str(&format!(
			"<g class=\"boundary\">\n<rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"8\" ry=\"8\" />\n",
			x = rect.x,
			y = rect.y,
			w = rect.w,
			h = rect.h
		));
		svg.push_str(&format!(
			"<text class=\"boundary-label\" x=\"{x:.2}\" y=\"{y:.2}\">{label}</text>\n</g>\n",
			x = rect.x + 8.0,
			y = rect.y + 14.0,
			label = escape_xml(&cluster.label)
		));
	}

	// Edges.
	for edge in &layout.edges {
		svg.push_str(&edge_svg(layout, edge, nodes));
	}

	// Nodes.
	for (id, card) in nodes {
		if let Some(node) = layout.nodes.get(id) {
			let icon = icons.get(&card.card_type).map(String::as_str);
			svg.push_str(&node_svg(layout, node, card, icon));
		}
	}

	svg.push_str("</svg>\n");
	svg
}

fn svg_markers() -> &'static str {
	"<marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto\">\n  <path d=\"M 0 0 L 10 5 L 0 10 z\" class=\"edge-arrow\" />\n</marker>\n"
}

fn svg_style(colors: &std::collections::HashMap<String, CardColor>) -> String {
	let mut css = String::new();
	css.push_str("<style>\n");
	// Light by default to match the canonical styling guide.
	css.push_str(
		"svg{--bg:#FFFFFF;--edge:#000000;--edgeLabel:#000000;--nodeStroke:#000000;--boundary:#000000;}\n",
	);
	css.push_str("svg[data-theme=\"dark\"]{--bg:#000000;--edge:#FFFFFF;--edgeLabel:#FFFFFF;--nodeStroke:#FFFFFF;--boundary:#FFFFFF;}\n");
	css.push_str("@media (prefers-color-scheme: dark){svg:not([data-theme=\"light\"]){--bg:#000000;--edge:#FFFFFF;--edgeLabel:#FFFFFF;--nodeStroke:#FFFFFF;--boundary:#FFFFFF;}}\n");
	css.push_str(".bg{fill:var(--bg);}\n");
	css.push_str(
		".edge-path{fill:none;stroke:var(--edge);stroke-width:1.2;marker-end:url(#arrow);}\n",
	);
	css.push_str(".edge-path.note-edge{stroke-dasharray:2 4;stroke-linecap:round;}\n");
	css.push_str(".edge-arrow{fill:var(--edge);}\n");
	css.push_str(&format!(
		".edge-label{{fill:var(--edgeLabel);font-family:Inter,system-ui,sans-serif;font-size:{EDGE_FONT_SIZE_PX}px;}}\n"
	));
	let icon_font_size = LINE_HEIGHT_PX * ICON_SCALE;
	css.push_str(&format!(
		".node-icon{{fill:var(--nodeText,#000000);font-family:Inter,system-ui,sans-serif;font-size:{icon_font_size}px;}}\n"
	));
	css.push_str(&format!(
		".node-label{{fill:var(--nodeText,#000000);font-family:Inter,system-ui,sans-serif;font-size:{NODE_FONT_SIZE_PX}px;}}\n"
	));
	css.push_str(".node-shape{stroke:var(--nodeStroke);stroke-width:1.4;}\n");
	css.push_str(".node-shape-outline{fill:none;}\n");
	css.push_str(
		".boundary rect{fill:transparent;stroke:var(--boundary);stroke-width:4;stroke-dasharray:5 5;}\n",
	);
	css.push_str(
		".boundary-label{fill:var(--boundary);font-family:Inter,system-ui,sans-serif;font-size:11px;}\n",
	);

	// Card-type palette (light + derived dark).
	for (card_type, color) in colors {
		let class = format!("card-type-{}", css_slug(card_type));
		if let Some(fill) = &color.fill {
			let dark = brighten_hex(fill, 0.18).unwrap_or_else(|| fill.to_string());
			css.push_str(&format!(".{class} .node-shape{{fill:{dark};}}\n"));
			css.push_str(&format!(
				"svg[data-theme=\"light\"] .{class} .node-shape{{fill:{fill};}}\n"
			));
			css.push_str(&format!(
				"@media (prefers-color-scheme: light){{svg:not([data-theme=\"dark\"]) .{class} .node-shape{{fill:{fill};}}}}\n"
			));
		}
		if let Some(font) = &color.font {
			css.push_str(&format!(".{class} .node-label{{--nodeText:{font};}}\n"));
		}
	}

	css.push_str("</style>\n");
	css
}

fn node_svg(layout: &PlainGraph, node: &PlainNode, card: &Card, icon: Option<&str>) -> String {
	let rect = node_rect(layout, node);
	let shape_rect = shape_rect_for_card(&rect, &card.card_type, &node.shape);
	let card_class = format!("card-type-{}", css_slug(&card.card_type));
	let mut group = String::new();
	group.push_str(&format!(
		"<g class=\"node {card_class}\" data-card-id=\"{id}\" data-layout-node=\"{layout_node}\">\n",
		id = escape_xml(&card.id),
		layout_node = escape_xml(&node.name)
	));

	group.push_str(&shape_svg(&node.shape, &shape_rect));
	group.push_str(&text_svg(&rect, &shape_rect, card, &node.shape, icon));
	group.push_str("</g>\n");
	group
}

fn shape_rect_for_card(rect: &SvgRect, card_type: &str, shape: &str) -> SvgRect {
	let scale = shape_scale_for_card_type(card_type);
	let mut out = scale_rect(rect, scale, scale);
	let width_scale = shape_width_scale_for_shape(shape);
	if card_type != "State" && (width_scale - 1.0).abs() >= f64::EPSILON {
		out = scale_rect(&out, width_scale, 1.0);
	}
	if card_type == "State" && matches!(shape, "circle" | "ellipse") {
		out = circle_rect(&out);
	}
	out
}

fn shape_scale_for_card_type(card_type: &str) -> f64 {
	match card_type {
		"State" => 0.55,
		"Event" | "Condition" | "Actor" | "Mission" => 0.75,
		_ => 1.0,
	}
}

fn shape_width_scale_for_shape(shape: &str) -> f64 {
	match shape {
		"octagon" | "doubleoctagon" | "hexagon" => 0.75,
		"ellipse" | "circle" => 0.75,
		_ => 1.0,
	}
}

fn scale_rect(rect: &SvgRect, scale_x: f64, scale_y: f64) -> SvgRect {
	if (scale_x - 1.0).abs() < f64::EPSILON && (scale_y - 1.0).abs() < f64::EPSILON {
		return *rect;
	}
	let w = (rect.w * scale_x).max(0.0);
	let h = (rect.h * scale_y).max(0.0);
	let dx = (rect.w - w) / 2.0;
	let dy = (rect.h - h) / 2.0;
	SvgRect {
		x: rect.x + dx,
		y: rect.y + dy,
		w,
		h,
	}
}

fn circle_rect(rect: &SvgRect) -> SvgRect {
	let size = rect.w.min(rect.h);
	let dx = (rect.w - size) / 2.0;
	let dy = (rect.h - size) / 2.0;
	SvgRect {
		x: rect.x + dx,
		y: rect.y + dy,
		w: size,
		h: size,
	}
}

fn edge_svg(layout: &PlainGraph, edge: &PlainEdge, nodes: &BTreeMap<String, &Card>) -> String {
	let mut out = String::new();
	let svg_points = edge
		.points
		.iter()
		.copied()
		.map(|point| to_svg_point(layout, point))
		.collect::<Vec<_>>();
	let tail_rect = node_shape_rect(layout, &edge.tail, nodes);
	let head_rect = node_shape_rect(layout, &edge.head, nodes);
	let adjusted_points = adjust_edge_points(&svg_points, tail_rect.as_ref(), head_rect.as_ref());
	let d = edge_path_d_svg(&adjusted_points);
	let is_note_edge = nodes
		.get(&edge.head)
		.is_some_and(|card| card.card_type == "Note")
		|| nodes
			.get(&edge.tail)
			.is_some_and(|card| card.card_type == "Note");
	let class = if is_note_edge {
		"edge-path note-edge"
	} else {
		"edge-path"
	};
	out.push_str(&format!(
		"<path class=\"{class}\" d=\"{d}\" data-tail=\"{tail}\" data-head=\"{head}\" />\n",
		tail = escape_xml(&edge.tail),
		head = escape_xml(&edge.head)
	));
	if let (Some(label), Some(pos)) = (&edge.label, edge.label_pos) {
		let p = to_svg_point(layout, pos);
		out.push_str(&format!(
			"<text class=\"edge-label\" x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{label}</text>\n",
			x = p.x,
			y = p.y,
			label = escape_xml(label)
		));
	}
	out
}

fn node_shape_rect(
	layout: &PlainGraph,
	node_id: &str,
	nodes: &BTreeMap<String, &Card>,
) -> Option<SvgRect> {
	let card = nodes.get(node_id)?;
	let node = layout.nodes.get(node_id)?;
	let rect = node_rect(layout, node);
	Some(shape_rect_for_card(&rect, &card.card_type, &node.shape))
}

fn adjust_edge_points(
	points: &[SvgPoint],
	tail_rect: Option<&SvgRect>,
	head_rect: Option<&SvgRect>,
) -> Vec<SvgPoint> {
	let mut adjusted = points.to_vec();
	if adjusted.len() < 2 {
		return adjusted;
	}
	if let Some(rect) = tail_rect {
		let center = rect_center(rect);
		let target = adjusted[1];
		adjusted[0] = intersect_ray_with_rect(center, target, rect);
	}
	if let Some(rect) = head_rect {
		let center = rect_center(rect);
		let target = adjusted[adjusted.len() - 2];
		let last = adjusted.len() - 1;
		adjusted[last] = intersect_ray_with_rect(center, target, rect);
	}
	adjusted
}

fn rect_center(rect: &SvgRect) -> SvgPoint {
	SvgPoint {
		x: rect.x + (rect.w / 2.0),
		y: rect.y + (rect.h / 2.0),
	}
}

fn intersect_ray_with_rect(center: SvgPoint, target: SvgPoint, rect: &SvgRect) -> SvgPoint {
	let dx = target.x - center.x;
	let dy = target.y - center.y;
	if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
		return center;
	}
	let mut best: Option<(f64, SvgPoint)> = None;
	let rect_left = rect.x;
	let rect_right = rect.x + rect.w;
	let rect_top = rect.y;
	let rect_bottom = rect.y + rect.h;

	if dx.abs() >= f64::EPSILON {
		for x in [rect_left, rect_right] {
			let t = (x - center.x) / dx;
			if t > 0.0 {
				let y = center.y + (t * dy);
				if y >= rect_top && y <= rect_bottom {
					best = pick_closer_intersection(best, t, SvgPoint { x, y });
				}
			}
		}
	}

	if dy.abs() >= f64::EPSILON {
		for y in [rect_top, rect_bottom] {
			let t = (y - center.y) / dy;
			if t > 0.0 {
				let x = center.x + (t * dx);
				if x >= rect_left && x <= rect_right {
					best = pick_closer_intersection(best, t, SvgPoint { x, y });
				}
			}
		}
	}

	best.map(|(_, point)| point).unwrap_or(target)
}

fn pick_closer_intersection(
	current: Option<(f64, SvgPoint)>,
	candidate_t: f64,
	candidate: SvgPoint,
) -> Option<(f64, SvgPoint)> {
	match current {
		Some((t, point)) if t <= candidate_t => Some((t, point)),
		_ => Some((candidate_t, candidate)),
	}
}

fn edge_path_d_svg(points: &[SvgPoint]) -> String {
	let mut d = String::new();
	if points.is_empty() {
		return d;
	}
	let start = points[0];
	d.push_str(&format!("M {x:.2} {y:.2}", x = start.x, y = start.y));
	let has_bezier_points = points.len() >= 4 && (points.len() - 1) % 3 == 0;
	if has_bezier_points {
		for chunk in points[1..].chunks(3) {
			if let [c1, c2, end] = chunk {
				d.push_str(&format!(
					" C {x1:.2} {y1:.2} {x2:.2} {y2:.2} {x3:.2} {y3:.2}",
					x1 = c1.x,
					y1 = c1.y,
					x2 = c2.x,
					y2 = c2.y,
					x3 = end.x,
					y3 = end.y
				));
			}
		}
		return d;
	}
	for point in points.iter().skip(1) {
		d.push_str(&format!(" L {x:.2} {y:.2}", x = point.x, y = point.y));
	}
	d
}

fn wrap_text(text: &str, max_len: usize) -> Vec<String> {
	let mut lines = Vec::new();
	let mut current = String::new();
	for word in text.split_whitespace() {
		let extra = if current.is_empty() {
			word.len()
		} else {
			word.len() + 1
		};
		if !current.is_empty() && current.len() + extra > max_len {
			lines.push(current);
			current = String::new();
		}
		if !current.is_empty() {
			current.push(' ');
		}
		current.push_str(word);
	}
	if !current.is_empty() {
		lines.push(current);
	}
	lines
}

fn shape_svg(shape: &str, rect: &SvgRect) -> String {
	match shape {
		"ellipse" | "circle" => {
			let cx = rect.x + (rect.w / 2.0);
			let cy = rect.y + (rect.h / 2.0);
			let rx = rect.w / 2.0;
			let ry = rect.h / 2.0;
			format!(
				"<ellipse class=\"node-shape\" cx=\"{cx:.2}\" cy=\"{cy:.2}\" rx=\"{rx:.2}\" ry=\"{ry:.2}\" />\n"
			)
		}
		"diamond" => {
			let cx = rect.x + (rect.w / 2.0);
			let cy = rect.y + (rect.h / 2.0);
			let d = format!(
				"M {cx:.2} {y0:.2} L {x1:.2} {cy:.2} L {cx:.2} {y1:.2} L {x0:.2} {cy:.2} Z",
				y0 = rect.y,
				y1 = rect.y + rect.h,
				x0 = rect.x,
				x1 = rect.x + rect.w
			);
			format!("<path class=\"node-shape\" d=\"{d}\" />\n")
		}
		"octagon" | "doubleoctagon" | "hexagon" => polygon_svg(shape, rect),
		"box3d" => box3d_svg(rect),
		"component" => component_svg(rect),
		"folder" => folder_svg(rect),
		"note" => note_svg(rect),
		"tab" => tab_svg(rect),
		"cylinder" => cylinder_svg(rect),
		"cds" => cds_svg(rect),
		"record" => record_svg(rect),
		_ => format!(
			"<rect class=\"node-shape\" x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"6\" ry=\"6\" />\n",
			x = rect.x,
			y = rect.y,
			w = rect.w,
			h = rect.h
		),
	}
}

fn polygon_svg(shape: &str, rect: &SvgRect) -> String {
	let sides = match shape {
		"hexagon" => 6,
		"octagon" | "doubleoctagon" => 8,
		_ => 8,
	};
	let cx = rect.x + (rect.w / 2.0);
	let cy = rect.y + (rect.h / 2.0);
	let rx = rect.w / 2.0;
	let ry = rect.h / 2.0;
	let points = regular_polygon_points(cx, cy, rx, ry, sides);
	let mut out = String::new();
	out.push_str(&format!(
		"<polygon class=\"node-shape\" points=\"{}\" />\n",
		points
	));
	if shape == "doubleoctagon" {
		let inset = 6.0;
		let points_inner =
			regular_polygon_points(cx, cy, (rx - inset).max(0.0), (ry - inset).max(0.0), 8);
		out.push_str(&format!(
			"<polygon class=\"node-shape node-shape-outline\" points=\"{}\" stroke-width=\"1\" />\n",
			points_inner
		));
	}
	out
}

fn rect_svg(rect: &SvgRect, rx: f64, ry: f64) -> String {
	format!(
		"<rect class=\"node-shape\" x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{rx:.2}\" ry=\"{ry:.2}\" />\n",
		x = rect.x,
		y = rect.y,
		w = rect.w,
		h = rect.h,
		rx = rx,
		ry = ry
	)
}

fn record_svg(rect: &SvgRect) -> String {
	rect_svg(rect, 0.0, 0.0)
}

fn cds_svg(rect: &SvgRect) -> String {
	let cut = (rect.w.min(rect.h) * 0.12).clamp(6.0, 14.0);
	let x0 = rect.x;
	let y0 = rect.y;
	let x1 = rect.x + rect.w;
	let y1 = rect.y + rect.h;
	let points = [
		format!("{:.2},{:.2}", x0 + cut, y0),
		format!("{:.2},{:.2}", x1 - cut, y0),
		format!("{:.2},{:.2}", x1, y0 + cut),
		format!("{:.2},{:.2}", x1, y1 - cut),
		format!("{:.2},{:.2}", x1 - cut, y1),
		format!("{:.2},{:.2}", x0 + cut, y1),
		format!("{:.2},{:.2}", x0, y1 - cut),
		format!("{:.2},{:.2}", x0, y0 + cut),
	]
	.join(" ");
	format!("<polygon class=\"node-shape\" points=\"{points}\" />\n")
}

fn box3d_svg(rect: &SvgRect) -> String {
	let mut out = rect_svg(rect, 6.0, 6.0);
	let inset = (rect.w.min(rect.h) * 0.12).clamp(4.0, 8.0);
	let x0 = rect.x + inset;
	let y0 = rect.y + inset;
	let x1 = rect.x + rect.w - inset;
	let y1 = rect.y + rect.h - inset;
	let d = format!(
		"M {x0:.2} {y0:.2} L {x1:.2} {y0:.2} L {x1:.2} {y1:.2}",
		x0 = x0,
		y0 = y0,
		x1 = x1,
		y1 = y1
	);
	out.push_str(&format!(
		"<path class=\"node-shape node-shape-outline\" d=\"{d}\" />\n"
	));
	out
}

fn component_svg(rect: &SvgRect) -> String {
	let mut out = rect_svg(rect, 6.0, 6.0);
	let ear_w = (rect.w * 0.12).clamp(6.0, 14.0);
	let ear_h = (rect.h * 0.18).clamp(8.0, 16.0);
	let gap = (rect.h * 0.12).clamp(6.0, 12.0);
	let x = rect.x + 4.0;
	let y_top = rect.y + gap;
	let y_bottom = rect.y + rect.h - gap - ear_h;
	for y in [y_top, y_bottom] {
		out.push_str(&format!(
			"<rect class=\"node-shape node-shape-outline\" x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"2\" ry=\"2\" />\n",
			x = x,
			y = y,
			w = ear_w,
			h = ear_h
		));
	}
	out
}

fn folder_svg(rect: &SvgRect) -> String {
	let tab_h = (rect.h * 0.22).clamp(8.0, 16.0);
	let tab_w = (rect.w * 0.35).clamp(18.0, rect.w * 0.6);
	let tab_x = rect.x + 8.0;
	let tab_right = (tab_x + tab_w).min(rect.x + rect.w - 8.0);
	let x0 = rect.x;
	let y0 = rect.y;
	let x1 = rect.x + rect.w;
	let y1 = rect.y + rect.h;
	let y_tab = y0 + tab_h;
	let d = format!(
		"M {x0:.2} {y_tab:.2} L {x0:.2} {y1:.2} L {x1:.2} {y1:.2} L {x1:.2} {y_tab:.2} L {tab_right:.2} {y_tab:.2} L {tab_right:.2} {y0:.2} L {tab_x:.2} {y0:.2} L {tab_x:.2} {y_tab:.2} Z",
		x0 = x0,
		y_tab = y_tab,
		y1 = y1,
		x1 = x1,
		tab_right = tab_right,
		y0 = y0,
		tab_x = tab_x
	);
	format!("<path class=\"node-shape\" d=\"{d}\" />\n")
}

fn tab_svg(rect: &SvgRect) -> String {
	let tab_h = (rect.h * 0.2).clamp(6.0, 14.0);
	let tab_w = (rect.w * 0.4).clamp(20.0, rect.w * 0.7);
	let tab_x = rect.x + ((rect.w - tab_w) / 2.0);
	let tab_right = tab_x + tab_w;
	let x0 = rect.x;
	let y0 = rect.y;
	let x1 = rect.x + rect.w;
	let y1 = rect.y + rect.h;
	let y_tab = y0 + tab_h;
	let d = format!(
		"M {x0:.2} {y_tab:.2} L {x0:.2} {y1:.2} L {x1:.2} {y1:.2} L {x1:.2} {y_tab:.2} L {tab_right:.2} {y_tab:.2} L {tab_right:.2} {y0:.2} L {tab_x:.2} {y0:.2} L {tab_x:.2} {y_tab:.2} Z",
		x0 = x0,
		y_tab = y_tab,
		y1 = y1,
		x1 = x1,
		tab_right = tab_right,
		y0 = y0,
		tab_x = tab_x
	);
	format!("<path class=\"node-shape\" d=\"{d}\" />\n")
}

fn note_svg(rect: &SvgRect) -> String {
	let fold = (rect.w.min(rect.h) * 0.2).clamp(6.0, 16.0);
	let x0 = rect.x;
	let y0 = rect.y;
	let x1 = rect.x + rect.w;
	let y1 = rect.y + rect.h;
	let fold_x = x1 - fold;
	let fold_y = y0 + fold;
	let d = format!(
		"M {x0:.2} {y0:.2} L {fold_x:.2} {y0:.2} L {x1:.2} {fold_y:.2} L {x1:.2} {y1:.2} L {x0:.2} {y1:.2} Z",
		x0 = x0,
		y0 = y0,
		fold_x = fold_x,
		x1 = x1,
		fold_y = fold_y,
		y1 = y1
	);
	let fold_line = format!(
		"M {fold_x:.2} {y0:.2} L {fold_x:.2} {fold_y:.2} L {x1:.2} {fold_y:.2}",
		fold_x = fold_x,
		y0 = y0,
		fold_y = fold_y,
		x1 = x1
	);
	let mut out = String::new();
	out.push_str(&format!("<path class=\"node-shape\" d=\"{d}\" />\n"));
	out.push_str(&format!(
		"<path class=\"node-shape node-shape-outline\" d=\"{fold_line}\" />\n"
	));
	out
}

fn cylinder_svg(rect: &SvgRect) -> String {
	let rx = rect.w / 2.0;
	let ry = (rect.h * 0.18).clamp(6.0, rect.h / 2.5);
	let cx = rect.x + rx;
	let top_y = rect.y + ry;
	let body_h = (rect.h - (2.0 * ry)).max(0.0);
	let bottom_y = rect.y + rect.h - ry;
	let mut out = String::new();
	out.push_str(&format!(
		"<ellipse class=\"node-shape\" cx=\"{cx:.2}\" cy=\"{top_y:.2}\" rx=\"{rx:.2}\" ry=\"{ry:.2}\" />\n",
		cx = cx,
		top_y = top_y,
		rx = rx,
		ry = ry
	));
	out.push_str(&format!(
		"<rect class=\"node-shape\" x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" />\n",
		x = rect.x,
		y = rect.y + ry,
		w = rect.w,
		h = body_h
	));
	out.push_str(&format!(
		"<ellipse class=\"node-shape node-shape-outline\" cx=\"{cx:.2}\" cy=\"{bottom_y:.2}\" rx=\"{rx:.2}\" ry=\"{ry:.2}\" />\n",
		cx = cx,
		bottom_y = bottom_y,
		rx = rx,
		ry = ry
	));
	out
}

fn regular_polygon_points(cx: f64, cy: f64, rx: f64, ry: f64, sides: usize) -> String {
	let mut pts = Vec::new();
	let step = std::f64::consts::TAU / (sides as f64);
	// Rotate so a flat edge is on top.
	let rotation = -std::f64::consts::FRAC_PI_2 + (step / 2.0);
	for i in 0..sides {
		let a = rotation + (i as f64 * step);
		let x = cx + (rx * a.cos());
		let y = cy + (ry * a.sin());
		pts.push(format!("{x:.2},{y:.2}"));
	}
	pts.join(" ")
}

fn text_svg(
	rect: &SvgRect,
	shape_rect: &SvgRect,
	card: &Card,
	shape: &str,
	icon: Option<&str>,
) -> String {
	let cy = rect.y + (rect.h / 2.0);
	let subtype = card
		.card_subtype
		.as_deref()
		.map(str::trim)
		.filter(|value| !value.is_empty());
	let description = card.description.trim();
	let left_pad = 8.0;
	let right_pad = 8.0;
	let icon_scale = icon_scale_for_card_type(&card.card_type);
	let icon_font_size = LINE_HEIGHT_PX * ICON_SCALE * icon_scale;
	let icon_offset_extra = icon_offset_for_shape(shape, icon_font_size);
	let icon_width = if icon.is_some() {
		icon_font_size + ICON_GAP_PX + ICON_OFFSET_PX + icon_offset_extra
	} else {
		0.0
	};
	let icon_left = shape_rect.x + left_pad + ICON_OFFSET_PX + icon_offset_extra;
	let text_x = rect.x + left_pad + icon_width;
	let available_width = (rect.w - left_pad - right_pad - icon_width).max(0.0);
	let text_center_x = text_x + (available_width / 2.0);
	let max_chars = (available_width / (NODE_FONT_SIZE_PX * 0.60)).floor() as usize;
	let max_chars = max_chars.clamp(18, 72);
	let detail = if description.is_empty() {
		card.name.trim().to_string()
	} else if card.name.trim().is_empty() {
		description.to_string()
	} else {
		format!("{} — {}", card.name.trim(), description)
	};
	let mut detail_lines = wrap_text(&detail, max_chars);
	if detail_lines.is_empty() {
		detail_lines.push(String::new());
	}

	let line_count = 2usize + 1 + detail_lines.len(); // type + id + blank + details
	let block_lines = line_count.max(3usize);
	let total_h = (block_lines as f64 - 1.0).max(0.0) * LINE_HEIGHT_PX;
	let start_y = cy - (total_h / 2.0);
	let mut text = String::new();
	if let Some(icon) = icon {
		let icon_center_x = icon_left + ((icon_width - ICON_OFFSET_PX) / 2.0);
		let icon_center_y = shape_rect.y + (shape_rect.h / 2.0);
		text.push_str(&format!(
			"<text class=\"node-icon\" x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{size:.2}\">{icon}</text>\n",
			x = icon_center_x,
			y = icon_center_y,
			size = icon_font_size,
			icon = escape_xml(icon)
		));
	}
	text.push_str(&format!(
		"<text class=\"node-label\" x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\">\n",
		x = text_center_x,
		y = start_y
	));
	// Line 1: card_type (subtype)
	let mut type_label = card.card_type.clone();
	if let Some(subtype) = subtype {
		type_label.push(' ');
		type_label.push('(');
		type_label.push_str(subtype);
		type_label.push(')');
	}
	text.push_str(&format!(
		"<tspan x=\"{x:.2}\" font-weight=\"700\">{line}</tspan>\n",
		x = text_center_x,
		line = escape_xml(&type_label)
	));

	// Line 2: id
	text.push_str(&format!(
		"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\" font-weight=\"700\">{line}</tspan>\n",
		x = text_center_x,
		dy = LINE_HEIGHT_PX,
		line = escape_xml(&card.id)
	));

	// Line 3: blank
	text.push_str(&format!(
		"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\">&#160;</tspan>\n",
		x = text_center_x,
		dy = LINE_HEIGHT_PX
	));

	// Line 4+: name + description detail
	for line in &detail_lines {
		text.push_str(&format!(
			"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\">{line}</tspan>\n",
			x = text_center_x,
			dy = LINE_HEIGHT_PX,
			line = escape_xml(line)
		));
	}
	text.push_str("</text>\n");
	text
}

fn icon_scale_for_card_type(card_type: &str) -> f64 {
	match card_type {
		"State" => 0.7,
		"Event" | "Condition" | "Actor" | "Mission" => 0.75,
		"Control" | "Risk" | "Threat" | "State Machine" => 0.75,
		_ => 1.0,
	}
}

fn icon_offset_for_shape(shape: &str, icon_width: f64) -> f64 {
	match shape {
		"octagon" | "doubleoctagon" | "hexagon" => icon_width,
		"diamond" => icon_width * 1.5,
		_ => 0.0,
	}
}
#[derive(Debug, Clone, Copy)]
struct SvgRect {
	x: f64,
	y: f64,
	w: f64,
	h: f64,
}

fn node_rect(layout: &PlainGraph, node: &PlainNode) -> SvgRect {
	let w = node.width_in * PX_PER_IN;
	let h = node.height_in * PX_PER_IN;
	let cx = node.center.x_in * PX_PER_IN;
	let cy = (layout.height_in - node.center.y_in) * PX_PER_IN;
	SvgRect {
		x: cx - (w / 2.0),
		y: cy - (h / 2.0),
		w,
		h,
	}
}

fn to_svg_point(layout: &PlainGraph, point: Point) -> SvgPoint {
	SvgPoint {
		x: point.x_in * PX_PER_IN,
		y: (layout.height_in - point.y_in) * PX_PER_IN,
	}
}

#[derive(Debug, Clone, Copy)]
struct SvgPoint {
	x: f64,
	y: f64,
}

struct BoundaryLayout<'a> {
	cluster: &'a BoundaryCluster,
	base_rect: SvgRect,
	pad: f64,
}

fn resolve_boundary_rects<'a>(
	layout: &PlainGraph,
	clusters: &'a [BoundaryCluster],
) -> Vec<(&'a BoundaryCluster, SvgRect)> {
	let mut layouts = Vec::new();
	for cluster in clusters {
		if let Some(base_rect) = cluster_base_rect(layout, cluster) {
			layouts.push(BoundaryLayout {
				cluster,
				base_rect,
				pad: BOUNDARY_PAD_PX,
			});
		}
	}

	for i in 0..layouts.len() {
		for j in (i + 1)..layouts.len() {
			if clusters_share_members(layouts[i].cluster, layouts[j].cluster) {
				continue;
			}
			let rect_i = apply_cluster_pad(&layouts[i].base_rect, layouts[i].pad);
			let rect_j = apply_cluster_pad(&layouts[j].base_rect, layouts[j].pad);
			if rects_overlap(&rect_i, &rect_j) {
				layouts[i].pad = BOUNDARY_MIN_PAD_PX;
				layouts[j].pad = BOUNDARY_MIN_PAD_PX;
			}
		}
	}

	layouts
		.into_iter()
		.map(|layout| {
			let rect = apply_cluster_pad(&layout.base_rect, layout.pad);
			(layout.cluster, rect)
		})
		.collect()
}

fn clusters_share_members(a: &BoundaryCluster, b: &BoundaryCluster) -> bool {
	let (small, large) = if a.members.len() <= b.members.len() {
		(a, b)
	} else {
		(b, a)
	};
	for member in &small.members {
		if large.members.iter().any(|other| other == member) {
			return true;
		}
	}
	false
}

fn rects_overlap(a: &SvgRect, b: &SvgRect) -> bool {
	let x_overlap = a.x < (b.x + b.w) && (a.x + a.w) > b.x;
	let y_overlap = a.y < (b.y + b.h) && (a.y + a.h) > b.y;
	x_overlap && y_overlap
}

fn apply_cluster_pad(rect: &SvgRect, pad: f64) -> SvgRect {
	SvgRect {
		x: (rect.x - pad).max(0.0),
		y: (rect.y - pad).max(0.0),
		w: rect.w + (2.0 * pad),
		h: rect.h + (2.0 * pad),
	}
}

fn cluster_base_rect(layout: &PlainGraph, cluster: &BoundaryCluster) -> Option<SvgRect> {
	let mut min_x: Option<f64> = None;
	let mut min_y: Option<f64> = None;
	let mut max_x: Option<f64> = None;
	let mut max_y: Option<f64> = None;

	for member in &cluster.members {
		let node = layout.nodes.get(member)?;
		let rect = node_rect(layout, node);
		min_x = Some(min_x.map_or(rect.x, |v| v.min(rect.x)));
		min_y = Some(min_y.map_or(rect.y, |v| v.min(rect.y)));
		max_x = Some(max_x.map_or(rect.x + rect.w, |v| v.max(rect.x + rect.w)));
		max_y = Some(max_y.map_or(rect.y + rect.h, |v| v.max(rect.y + rect.h)));
	}

	let (min_x, min_y, max_x, max_y) = match (min_x, min_y, max_x, max_y) {
		(Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
		_ => return None,
	};

	Some(SvgRect {
		x: min_x,
		y: min_y,
		w: max_x - min_x,
		h: max_y - min_y,
	})
}

fn css_slug(value: &str) -> String {
	let mut out = String::new();
	for ch in value.chars() {
		if matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9') {
			out.push(ch.to_ascii_lowercase());
		} else if !out.ends_with('-') {
			out.push('-');
		}
	}
	out.trim_matches('-').to_string()
}

fn escape_xml(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
}

fn brighten_hex(value: &str, amount: f32) -> Option<String> {
	let trimmed = value.trim();
	let hex = trimmed.strip_prefix('#')?;
	if hex.len() != 6 {
		return None;
	}
	let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
	let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
	let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
	Some(format!(
		"#{:02X}{:02X}{:02X}",
		brighten_channel(r, amount),
		brighten_channel(g, amount),
		brighten_channel(b, amount)
	))
}

fn brighten_channel(value: u8, amount: f32) -> u8 {
	let value = value as f32;
	let amount = amount.clamp(0.0, 1.0);
	let out = value + ((255.0 - value) * amount);
	out.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
	use std::collections::{BTreeMap, HashMap};

	use serde_json::Value;

	use super::super::graphviz_plain::{PlainGraph, PlainNode, Point};
	use super::{
		BoundaryCluster, SvgRect, edge_path_d_svg, rects_overlap, resolve_boundary_rects, text_svg,
		to_svg_point,
	};
	use crate::model::{AuditTrail, Card};

	#[test]
	fn node_text_uses_svg_tspans_and_requested_format() {
		let card = Card {
			schema: None,
			id: "CAP-001".into(),
			card_type: "Capability".into(),
			card_subtype: Some("struct".into()),
			name: "Model IO".into(),
			description: "This is a test description.".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let rect = SvgRect {
			x: 0.0,
			y: 0.0,
			w: 400.0,
			h: 200.0,
		};
		let svg = text_svg(&rect, &rect, &card, "box", Some("✨"));
		assert!(svg.contains("class=\"node-icon\""));
		assert!(svg.contains(">✨<"));
		assert!(svg.contains("font-weight=\"700\">Capability (struct)"));
		assert!(svg.contains("font-weight=\"700\">CAP-001"));
		assert!(svg.contains("Model IO — This is a test description."));
		assert!(!svg.contains("<br"));
	}

	#[test]
	fn edge_path_uses_bezier_curves() {
		let layout = PlainGraph {
			width_in: 2.0,
			height_in: 2.0,
			nodes: HashMap::new(),
			edges: Vec::new(),
		};
		let points = vec![
			Point {
				x_in: 0.2,
				y_in: 0.2,
			},
			Point {
				x_in: 0.6,
				y_in: 0.4,
			},
			Point {
				x_in: 1.2,
				y_in: 0.8,
			},
			Point {
				x_in: 1.6,
				y_in: 1.4,
			},
		];
		let svg_points = points
			.iter()
			.copied()
			.map(|point| to_svg_point(&layout, point))
			.collect::<Vec<_>>();
		let path = edge_path_d_svg(&svg_points);
		assert!(path.contains(" C "));
		assert!(!path.contains(" L "));
	}

	#[test]
	fn boundary_rects_avoid_overlap_for_disjoint_members() {
		let mut nodes = HashMap::new();
		nodes.insert(
			"COM-001".to_string(),
			PlainNode {
				name: "COM-001".into(),
				center: Point {
					x_in: 1.0,
					y_in: 1.0,
				},
				width_in: 1.0,
				height_in: 0.6,
				shape: "box".into(),
			},
		);
		nodes.insert(
			"COM-002".to_string(),
			PlainNode {
				name: "COM-002".into(),
				center: Point {
					x_in: 2.2,
					y_in: 1.0,
				},
				width_in: 1.0,
				height_in: 0.6,
				shape: "box".into(),
			},
		);
		let layout = PlainGraph {
			width_in: 4.0,
			height_in: 3.0,
			nodes,
			edges: Vec::new(),
		};
		let clusters = vec![
			BoundaryCluster {
				id: "BND-001".into(),
				label: "Cluster One".into(),
				members: vec!["COM-001".into()],
			},
			BoundaryCluster {
				id: "BND-002".into(),
				label: "Cluster Two".into(),
				members: vec!["COM-002".into()],
			},
		];

		let rects = resolve_boundary_rects(&layout, &clusters);
		let rect_a = rects
			.iter()
			.find(|(cluster, _)| cluster.id == "BND-001")
			.map(|(_, rect)| *rect)
			.expect("cluster one rect");
		let rect_b = rects
			.iter()
			.find(|(cluster, _)| cluster.id == "BND-002")
			.map(|(_, rect)| *rect)
			.expect("cluster two rect");
		assert!(!rects_overlap(&rect_a, &rect_b));
	}
}
