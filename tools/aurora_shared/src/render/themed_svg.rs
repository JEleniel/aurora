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
		"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.2} {height:.2}\" width=\"{width:.2}\" height=\"{height:.2}\" data-theme=\"dark\">\n"
	));
	svg.push_str("<defs>\n");
	svg.push_str(&svg_style(colors));
	svg.push_str(svg_markers());
	svg.push_str("</defs>\n");

	svg.push_str("<rect class=\"bg\" x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" />\n");

	// Boundaries first, so they render behind nodes.
	for cluster in clusters {
		if let Some(rect) = cluster_rect(layout, cluster) {
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
	}

	// Edges.
	for edge in &layout.edges {
		svg.push_str(&edge_svg(layout, edge));
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
	// Dark by default. VS Code's SVG preview does not reliably map editor theme →
	// `prefers-color-scheme`, so we allow an explicit SVG attribute to force the theme.
	css.push_str(
		"svg{--bg:#000000;--edge:#FFFFFF;--edgeLabel:#FFFFFF;--nodeStroke:#FFFFFF;--boundary:#FFFFFF;}\n",
	);
	css.push_str("svg[data-theme=\"light\"]{--bg:#FFFFFF;--edge:#111827;--edgeLabel:#111827;--nodeStroke:#111827;--boundary:#6B7280;}\n");
	css.push_str("@media (prefers-color-scheme: light){svg:not([data-theme=\"dark\"]){--bg:#FFFFFF;--edge:#111827;--edgeLabel:#111827;--nodeStroke:#111827;--boundary:#6B7280;}}\n");
	css.push_str(".bg{fill:var(--bg);}\n");
	css.push_str(
		".edge-path{fill:none;stroke:var(--edge);stroke-width:1.2;marker-end:url(#arrow);}\n",
	);
	css.push_str(".edge-arrow{fill:var(--edge);}\n");
	css.push_str(&format!(
		".edge-label{{fill:var(--edgeLabel);font-family:Inter,system-ui,sans-serif;font-size:{EDGE_FONT_SIZE_PX}px;}}\n"
	));
	let icon_font_size = LINE_HEIGHT_PX * 3.0;
	css.push_str(&format!(
		".node-icon{{fill:var(--nodeText,#FFFFFF);font-family:Inter,system-ui,sans-serif;font-size:{icon_font_size}px;}}\n"
	));
	css.push_str(&format!(
		".node-label{{fill:var(--nodeText,#FFFFFF);font-family:Inter,system-ui,sans-serif;font-size:{NODE_FONT_SIZE_PX}px;}}\n"
	));
	css.push_str(".node-shape{stroke:var(--nodeStroke);stroke-width:1.4;}\n");
	css.push_str(".node-shape-outline{fill:none;}\n");
	css.push_str(
		".boundary rect{fill:transparent;stroke:var(--boundary);stroke-width:2;stroke-dasharray:6 6;}\n",
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
	let card_class = format!("card-type-{}", css_slug(&card.card_type));
	let mut group = String::new();
	group.push_str(&format!(
		"<g class=\"node {card_class}\" data-card-id=\"{id}\" data-layout-node=\"{layout_node}\">\n",
		id = escape_xml(&card.id),
		layout_node = escape_xml(&node.name)
	));

	group.push_str(&shape_svg(&node.shape, &rect));
	group.push_str(&text_svg(&rect, card, icon));
	group.push_str("</g>\n");
	group
}

fn edge_svg(layout: &PlainGraph, edge: &PlainEdge) -> String {
	let mut out = String::new();
	let d = edge_path_d(layout, &edge.points);
	out.push_str(&format!(
		"<path class=\"edge-path\" d=\"{d}\" data-tail=\"{tail}\" data-head=\"{head}\" />\n",
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

fn edge_path_d(layout: &PlainGraph, points: &[Point]) -> String {
	let mut d = String::new();
	if points.is_empty() {
		return d;
	}
	let svg_points = points
		.iter()
		.copied()
		.map(|point| to_svg_point(layout, point))
		.collect::<Vec<_>>();
	let start = svg_points[0];
	d.push_str(&format!("M {x:.2} {y:.2}", x = start.x, y = start.y));
	let has_bezier_points = svg_points.len() >= 4 && (svg_points.len() - 1) % 3 == 0;
	if has_bezier_points {
		for chunk in svg_points[1..].chunks(3) {
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
	for point in svg_points.iter().skip(1) {
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

fn text_svg(rect: &SvgRect, card: &Card, icon: Option<&str>) -> String {
	let cy = rect.y + (rect.h / 2.0);
	let subtype = card
		.card_subtype
		.as_deref()
		.map(str::trim)
		.filter(|value| !value.is_empty());
	let description = card.description.trim();
	let left_pad = 10.0;
	let right_pad = 10.0;
	let icon_font_size = LINE_HEIGHT_PX * 3.0;
	let icon_width = if icon.is_some() {
		icon_font_size + 16.0
	} else {
		0.0
	};
	let text_x = rect.x + left_pad + icon_width;
	let available_width = (rect.w - left_pad - right_pad - icon_width).max(0.0);
	let max_chars = (available_width / (NODE_FONT_SIZE_PX * 0.60)).floor() as usize;
	let max_chars = max_chars.clamp(18, 72);
	let mut description_lines = Vec::new();
	if !description.is_empty() {
		description_lines = wrap_text(description, max_chars);
	}

	let mut line_count = 2usize; // type line + name
	if !description_lines.is_empty() {
		line_count += 1; // blank line
		line_count += description_lines.len();
	}
	let block_lines = line_count.max(3usize);
	let total_h = (block_lines as f64 - 1.0).max(0.0) * LINE_HEIGHT_PX;
	let start_y = cy - (total_h / 2.0);
	let mut text = String::new();
	if let Some(icon) = icon {
		let icon_top = start_y - (LINE_HEIGHT_PX / 2.0);
		text.push_str(&format!(
			"<text class=\"node-icon\" x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"start\" dominant-baseline=\"hanging\">{icon}</text>\n",
			x = rect.x + left_pad,
			y = icon_top,
			icon = escape_xml(icon)
		));
	}
	text.push_str(&format!(
		"<text class=\"node-label\" x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"start\" dominant-baseline=\"middle\">\n",
		x = text_x,
		y = start_y
	));
	// Line 1: id + **card_type** (subtype)
	let id_label = format!("{}: ", card.id);
	text.push_str(&format!(
		"<tspan x=\"{x:.2}\">{id_label}</tspan><tspan font-weight=\"700\">{card_type}</tspan>",
		x = text_x,
		id_label = escape_xml(&id_label),
		card_type = escape_xml(&card.card_type)
	));
	if let Some(subtype) = subtype {
		text.push_str(&format!(
			"<tspan>{}</tspan>",
			escape_xml(&format!(" ({subtype})"))
		));
	}
	text.push_str("\n");

	// Line 2: name
	text.push_str(&format!(
		"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\">{line}</tspan>\n",
		x = text_x,
		dy = LINE_HEIGHT_PX,
		line = escape_xml(&card.name)
	));

	// Blank line + description
	if !description_lines.is_empty() {
		text.push_str(&format!(
			"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\">&#160;</tspan>\n",
			x = text_x,
			dy = LINE_HEIGHT_PX
		));
		for line in &description_lines {
			text.push_str(&format!(
				"<tspan x=\"{x:.2}\" dy=\"{dy:.2}\">{line}</tspan>\n",
				x = text_x,
				dy = LINE_HEIGHT_PX,
				line = escape_xml(line)
			));
		}
	}
	text.push_str("</text>\n");
	text
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

fn cluster_rect(layout: &PlainGraph, cluster: &BoundaryCluster) -> Option<SvgRect> {
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

	let pad = 18.0;
	Some(SvgRect {
		x: (min_x - pad).max(0.0),
		y: (min_y - pad).max(0.0),
		w: (max_x - min_x) + (2.0 * pad),
		h: (max_y - min_y) + (2.0 * pad),
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

	use crate::model::{AuditTrail, Card};
	use super::{SvgRect, edge_path_d, text_svg};
	use super::super::graphviz_plain::{PlainGraph, Point};

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
		let svg = text_svg(&rect, &card, Some("✨"));
		assert!(svg.contains("class=\"node-icon\""));
		assert!(svg.contains(">✨<"));
		assert!(svg.contains("CAP-001: "));
		assert!(svg.contains("<tspan font-weight=\"700\">Capability</tspan>"));
		assert!(svg.contains("(struct)"));
		assert!(svg.contains("Model IO"));
		assert!(svg.contains("This is a test description."));
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
			Point { x_in: 0.2, y_in: 0.2 },
			Point { x_in: 0.6, y_in: 0.4 },
			Point { x_in: 1.2, y_in: 0.8 },
			Point { x_in: 1.6, y_in: 1.4 },
		];
		let path = edge_path_d(&layout, &points);
		assert!(path.contains(" C "));
		assert!(!path.contains(" L "));
	}
}
