//! SVG rendering for Aurora layout graphs.
//!
//! This module renders an Aurora [`Model`](crate::Model) using an existing [`Layout`](super::Layout)
//! into a standalone SVG string based on the `svgtemplate.txt` template.

use std::collections::HashMap;
use std::path::Path;

use crate::{Card, Model};

use super::Layout;
use super::render_error::RenderError;

const SVG_TEMPLATE: &str = include_str!("svgtemplate.txt");

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
		// Default: 16px root font size, 2rem spacing.
		let base_font_size_px = 16;
		let rem_px = base_font_size_px;
		Self {
			node_spacing_px: 2 * rem_px,
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
		let config = config.unwrap_or_default();

		let cards_by_id = index_cards(model)?;
		let node_layouts = node::collect_node_layouts(layout, &cards_by_id, &config)?;
		let positioned = node::position_nodes(&node_layouts, &config);

		let mut edges_svg = String::new();
		let mut edge_bounds: Vec<geom::Bounds> = Vec::new();
		let mut edge_points: Vec<geom::PointF> = Vec::new();

		let obstacle_bboxes: Vec<geom::RectI> = positioned.values().map(|n| n.bbox).collect();

		for e in &layout.edges {
			let a = positioned
				.get(e.a.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.a.clone()))?;
			let b = positioned
				.get(e.b.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(e.b.clone()))?;

			let route = edge::route_edge(&a.bbox, &b.bbox, &obstacle_bboxes, &config)?;
			edge_points.extend(route.points.iter().copied());
			edge_bounds.push(route.bounds);
			edges_svg.push_str(&edge::render_edge(&route, config.edge_style));
		}

		let mut nodes_svg = String::new();
		for (id, node) in positioned.iter() {
			let card = cards_by_id
				.get(id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(id.clone()))?;
			nodes_svg.push_str(&node::render_node(card, node, &config));
		}

		let drawing = format!(
			"<g id=\"edges\">{}</g><g id=\"nodes\">{}</g>",
			edges_svg, nodes_svg
		);

		let viewbox = compute_viewbox(
			positioned.values().map(|n| n.bbox),
			&edge_bounds,
			&edge_points,
			&config,
		);
		let mut svg = fill_template(SVG_TEMPLATE, &drawing, &viewbox)?;

		// Keep template compatibility, but make width/height match the viewBox size.
		svg = svg
			.replace("width=\"1600\"", &format!("width=\"{}\"", viewbox.w))
			.replace("height=\"400\"", &format!("height=\"{}\"", viewbox.h));

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
	let min_y = (bounds.min_y.floor() as i32) - margin;
	let max_x = (bounds.max_x.ceil() as i32) + margin;
	let max_y = (bounds.max_y.ceil() as i32) + margin;

	geom::RectI {
		x: min_x,
		y: min_y,
		w: (max_x - min_x).max(1),
		h: (max_y - min_y).max(1),
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
}
