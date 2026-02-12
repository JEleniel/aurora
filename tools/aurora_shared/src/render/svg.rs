//! SVG rendering for Aurora layout graphs.
//!
//! This module renders an Aurora [`Model`](crate::Model) using an existing [`Layout`](super::Layout)
//! into a standalone SVG string based on the `svgtemplate.txt` template.

use super::render_error::RenderError;
use super::{Layout, LayoutEdge};
use crate::{Card, Model};
use std::collections::HashMap;
use std::path::Path;

const SVG_TEMPLATE: &str = include_str!("svgtemplate.txt");

// Limit the size to prevent runaway rendering in case of very large graphs.
const SVG_MAX_SIZE: u32 = 50 * 1024 * 1024; // 50 MiB, limit of many SVG renderers

const SYMBOL_BASE_WIDTH_PX: i32 = 160;
const SYMBOL_BASE_HEIGHT_PX: i32 = 100;

/// Renders an Aurora model graph as SVG.
#[derive(Debug, Default, Clone, Copy)]
pub struct Svg;

/// Rendering configuration for SVG output.
#[derive(Debug, Clone)]
pub struct SvgConfig {
	/// Minimum space between nodes (both horizontally and vertically) in pixels.
	pub node_spacing_px: i32,
	/// Base font size in pixels.
	pub base_font_size_px: i32,
	/// How edges should be drawn.
	pub edge_style: EdgeStyle,
}

impl Default for SvgConfig {
	fn default() -> Self {
		// Default: 16px root font size, generous spacing.
		let base_font_size_px = 16;
		let rem_px = base_font_size_px;
		Self {
			// Previously 2rem; triple spacing for readability.
			node_spacing_px: 6 * rem_px,
			base_font_size_px,
			edge_style: EdgeStyle::Curved,
		}
	}
}

/// Styling choices for edge rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
	/// Orthogonal polyline routing (right angles).
	Orthogonal,
	/// Smooth cubic Bezier edges built from the routed polyline.
	Curved,
}

impl Svg {
	/// Render a model+layout into an SVG string.
	pub fn render(
		model: &Model,
		layout: &Layout,
		config: Option<SvgConfig>,
	) -> Result<String, RenderError> {
		let base_size: u32 = SVG_TEMPLATE.len() as u32;

		let config = config.unwrap_or_default();

		let cards_by_id = index_cards(model)?;
		let node_layouts = node::collect_node_layouts(layout, &cards_by_id, &config)?;
		let positioned = node::position_nodes(&node_layouts, &config);

		let mut edges_svg = String::new();
		let mut edge_bounds: Vec<geom::Bounds> = Vec::new();
		let mut edge_points: Vec<geom::PointF> = Vec::new();
		let mut node_shapes_svg = String::new();
		let mut node_labels_svg = String::new();

		let rem_px = config.base_font_size_px.max(1);
		let node_bboxes: Vec<geom::RectI> = positioned.values().map(|n| n.bbox).collect();
		let node_obstacle_pad = rem_px.max(6);
		let node_obstacles: Vec<geom::RectI> = node_bboxes
			.iter()
			.copied()
			.map(|bbox| grow_rect(bbox, node_obstacle_pad))
			.collect();
		let mut edge_obstacles: Vec<(String, geom::RectI)> = Vec::new();
		let edge_obstacle_pad_px = (rem_px / 4).max(4);
		let cell_px = config.base_font_size_px.max(1);

		let mut edges = layout.edges.clone();
		edges.sort_by(|left, right| {
			let left_key = edge_route_order_key(left, &positioned);
			let right_key = edge_route_order_key(right, &positioned);
			left_key
				.target_y
				.cmp(&right_key.target_y)
				.then_with(|| left_key.target_x.cmp(&right_key.target_x))
				.then_with(|| left_key.target_id.cmp(right_key.target_id))
				.then_with(|| left_key.target_side.cmp(&right_key.target_side))
				.then_with(|| left_key.lane.cmp(&right_key.lane))
				.then_with(|| right_key.span.cmp(&left_key.span))
				.then_with(|| left.a.cmp(&right.a))
				.then_with(|| left.b.cmp(&right.b))
		});

		for e in &edges {
			let a = positioned
				.get(e.a.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.a.clone()))?;
			let b = positioned
				.get(e.b.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.b.clone()))?;

			let edge_obstacles_for_route: Vec<geom::RectI> = edge_obstacles
				.iter()
				.filter_map(|(target_id, obstacle)| {
					if target_id == &e.b {
						None
					} else {
						Some(*obstacle)
					}
				})
				.collect();

			let route = edge::route_edge(
				&a.bbox,
				&b.bbox,
				&node_obstacles,
				edge_obstacles_for_route.as_slice(),
				node_bboxes.as_slice(),
				&config,
			)?;

			for obstacle in edge::route_obstacles_for_later_edges(
				&route,
				cell_px,
				&a.bbox,
				&b.bbox,
				edge_obstacle_pad_px,
			) {
				edge_obstacles.push((e.b.clone(), obstacle));
			}
			edge_points.extend(route.points.iter().copied());
			edge_bounds.push(route.bounds);
			if base_size + edges_svg.len() as u32 > SVG_MAX_SIZE {
				return Err(RenderError::SvgTooLarge);
			}
			edges_svg.push_str(&edge::render_edge(&route, config.edge_style));
		}

		for (id, node) in positioned.iter() {
			let card = cards_by_id
				.get(id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(id.clone()))?;
			if base_size
				+ edges_svg.len() as u32
				+ node_shapes_svg.len() as u32
				+ node_labels_svg.len() as u32
				> SVG_MAX_SIZE
			{
				return Err(RenderError::SvgTooLarge);
			}
			let rendered = node::render_node(card, node, &config);
			node_shapes_svg.push_str(&rendered.shape);
			node_labels_svg.push_str(&rendered.labels);
		}

		let viewbox = compute_viewbox(
			positioned.values().map(|n| n.bbox),
			&edge_bounds,
			&edge_points,
			&config,
		);

		let background = format!(
			"<style>@media print{{#aurora-bg{{display:none;}}}}</style><rect id=\"aurora-bg\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:#ffffff;stroke:none;\" />",
			viewbox.x, viewbox.y, viewbox.w, viewbox.h
		);
		if base_size
			+ edges_svg.len() as u32
			+ node_shapes_svg.len() as u32
			+ node_labels_svg.len() as u32
			+ background.len() as u32
			> SVG_MAX_SIZE
		{
			return Err(RenderError::SvgTooLarge);
		}

		let drawing = format!(
			"{}<g id=\"edges\">{}</g><g id=\"node-shapes\">{}</g><g id=\"node-labels\">{}</g>",
			background, edges_svg, node_shapes_svg, node_labels_svg
		);
		if base_size + drawing.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}

		let mut svg = fill_template(SVG_TEMPLATE, &drawing, &viewbox)?;
		if svg.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}

		// Keep template compatibility, but make width/height match the viewBox size.
		svg = svg
			.replace("width=\"1600\"", &format!("width=\"{}\"", viewbox.w))
			.replace("height=\"400\"", &format!("height=\"{}\"", viewbox.h));
		if svg.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}

		Ok(svg)
	}

	/// Render SVG and write it to the provided path.
	pub fn write_to_file(
		path: impl AsRef<Path>,
		model: &Model,
		layout: &Layout,
		config: Option<SvgConfig>,
	) -> Result<(), RenderError> {
		let svg = Self::render(model, layout, config)?;
		std::fs::write(path, svg)?;
		Ok(())
	}
}

fn index_cards(model: &Model) -> Result<HashMap<String, &Card>, RenderError> {
	let mut cards_by_id: HashMap<String, &Card> = HashMap::new();
	for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
		if cards_by_id.insert(card.id.clone(), card).is_some() {
			return Err(RenderError::SvgDuplicateCardId(card.id.clone()));
		}
	}
	Ok(cards_by_id)
}

fn compute_viewbox(
	node_bboxes: impl Iterator<Item = geom::RectI>,
	edge_bounds: &[geom::Bounds],
	edge_points: &[geom::PointF],
	config: &SvgConfig,
) -> geom::RectI {
	let rem_px = config.base_font_size_px.max(1);
	let margin = rem_px;
	let top_margin = margin + rem_px;

	let mut bounds = geom::Bounds::empty();
	for bbox in node_bboxes {
		bounds = bounds.union_rect_i(bbox);
	}
	for b in edge_bounds {
		bounds = bounds.union(*b);
	}
	for p in edge_points {
		bounds = bounds.union_point(*p);
	}

	if !bounds.min_x.is_finite() {
		return geom::RectI {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
		};
	}

	let min_x = (bounds.min_x.floor() as i32) - margin;
	let min_y = (bounds.min_y.floor() as i32) - top_margin;
	let max_x = (bounds.max_x.ceil() as i32) + margin;
	let max_y = (bounds.max_y.ceil() as i32) + margin;

	geom::RectI {
		x: min_x,
		y: min_y,
		w: (max_x - min_x).max(1),
		h: (max_y - min_y).max(1),
	}
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

#[derive(Debug, Clone, Copy)]
struct EdgeRouteOrderKey<'a> {
	target_y: i32,
	target_x: i32,
	target_id: &'a str,
	target_side: i32,
	lane: i32,
	span: i32,
}

fn edge_route_order_key<'a>(
	edge: &'a LayoutEdge,
	positioned: &HashMap<String, node::PositionedNode>,
) -> EdgeRouteOrderKey<'a> {
	let source_center = positioned
		.get(edge.a.as_str())
		.map(|node| node.bbox.center())
		.unwrap_or(geom::PointF { x: 0.0, y: 0.0 });
	let target_center = positioned
		.get(edge.b.as_str())
		.map(|node| node.bbox.center())
		.unwrap_or(geom::PointF { x: 0.0, y: 0.0 });
	let dx = source_center.x - target_center.x;
	let dy = source_center.y - target_center.y;
	let span = dx.abs().round() as i32 + dy.abs().round() as i32;

	let (target_side, lane) = if dx.abs() >= dy.abs() {
		if dx < 0.0 {
			(0, source_center.y.round() as i32)
		} else {
			(1, source_center.y.round() as i32)
		}
	} else if dy < 0.0 {
		(2, source_center.x.round() as i32)
	} else {
		(3, source_center.x.round() as i32)
	};

	EdgeRouteOrderKey {
		target_y: target_center.y.round() as i32,
		target_x: target_center.x.round() as i32,
		target_id: edge.b.as_str(),
		target_side,
		lane,
		span,
	}
}

fn fill_template(
	template: &str,
	drawing: &str,
	viewbox: &geom::RectI,
) -> Result<String, RenderError> {
	if !template.contains("{{viewbox}}") {
		return Err(RenderError::SvgTemplateMissingViewbox);
	}

	let mut out = template.replace(
		"{{viewbox}}",
		&format!("{} {} {} {}", viewbox.x, viewbox.y, viewbox.w, viewbox.h),
	);

	if out.contains("{{diagram}}") {
		out = out.replace("{{diagram}}", drawing);
		return Ok(out);
	}
	if out.contains("{{drawing}}") {
		out = out.replace("{{drawing}}", drawing);
		return Ok(out);
	}

	Err(RenderError::SvgTemplateMissingDiagram)
}

mod edge;
mod geom;
mod node;

#[cfg(test)]
mod tests {
	use super::node;
	use super::{RenderError, fill_template, geom};
	use crate::{Attributes, AuditLog, Card, Link, Model};
	use std::collections::HashMap;
	use std::path::PathBuf;

	#[test]
	fn wrap_preserves_newlines() {
		let lines = node::wrap_text("a b c\n\n1 2 3", 4);
		assert_eq!(lines[0], "a b");
		assert_eq!(lines[1], "c");
		assert_eq!(lines[2], "");
		assert_eq!(lines[3], "1 2");
		assert_eq!(lines[4], "3");
	}

	#[test]
	fn escape_xml_text() {
		assert_eq!(node::escape_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
	}

	#[test]
	fn template_replaces_diagram_placeholder() {
		let tpl = "<svg viewBox=\"{{viewbox}}\">{{diagram}}</svg>";
		let vb = geom::RectI {
			x: 1,
			y: 2,
			w: 3,
			h: 4,
		};
		let out = fill_template(tpl, "<g/>", &vb).expect("template should render");
		assert!(out.contains("viewBox=\"1 2 3 4\""));
		assert!(out.contains("<g/>"));
	}

	#[test]
	fn template_replaces_drawing_placeholder() {
		let tpl = "<svg viewBox=\"{{viewbox}}\">{{drawing}}</svg>";
		let vb = geom::RectI {
			x: 0,
			y: 0,
			w: 10,
			h: 10,
		};
		let out = fill_template(tpl, "X", &vb).expect("template should render");
		assert!(out.contains("X"));
	}

	#[test]
	fn template_missing_placeholders_errors() {
		let vb = geom::RectI {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
		};
		let err = fill_template("nope", "X", &vb).unwrap_err();
		assert!(matches!(err, RenderError::SvgTemplateMissingViewbox));
	}

	#[test]
	fn render_includes_screen_background_and_no_text_stroke() {
		let root = Card {
			schema: None,
			id: "MIS-001".to_string(),
			card_type: "Mission".to_string(),
			card_subtype: None,
			name: "Test Mission".to_string(),
			description: "desc".to_string(),
			version: "1.0.0".to_string(),
			status: None,
			boundary: None,
			notes: None,
			attributes: Attributes::new(),
			links: vec![Link {
				target: "C-001".to_string(),
				relationship: "rel".to_string(),
			}],
			source_path: PathBuf::from("MIS-001.json"),
			validation_errors: Vec::new(),
		};
		let child = Card {
			schema: None,
			id: "C-001".to_string(),
			card_type: "Activity".to_string(),
			card_subtype: None,
			name: "Child".to_string(),
			description: "desc".to_string(),
			version: "1.0.0".to_string(),
			status: None,
			boundary: None,
			notes: None,
			attributes: Attributes::new(),
			links: Vec::new(),
			source_path: PathBuf::from("C-001.json"),
			validation_errors: Vec::new(),
		};
		let model = Model {
			root_card: root,
			cards: vec![child],
			audit_log: AuditLog {
				schema: None,
				history: Vec::new(),
				source_path: PathBuf::from("AuditLog.json"),
				validation_errors: Vec::new(),
			},
			model_home: PathBuf::from("model"),
			mission_home: PathBuf::from("mission"),
		};

		let mut nodes = HashMap::new();
		nodes.insert(
			"MIS-001".to_string(),
			crate::render::LayoutNode {
				id: "MIS-001".to_string(),
				x: 0,
				y: 0,
			},
		);
		nodes.insert(
			"C-001".to_string(),
			crate::render::LayoutNode {
				id: "C-001".to_string(),
				x: 1,
				y: 0,
			},
		);
		let layout = crate::render::Layout {
			nodes,
			edges: vec![crate::render::LayoutEdge {
				a: "MIS-001".to_string(),
				b: "C-001".to_string(),
			}],
		};

		let svg = super::Svg::render(&model, &layout, None).expect("svg render");
		assert!(!svg.contains("fill:#00000000;stroke:#000000"));
		assert!(svg.contains("non-scaling-stroke"));
		assert!(svg.contains("id=\"aurora-bg\""));
		assert!(svg.contains("stroke:none"));
	}
}
