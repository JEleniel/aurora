//! SVG rendering for Aurora layout graphs.
//!
//! This module renders an Aurora [`Model`](crate::Model) using an existing [`Layout`](super::Layout)
//! into a standalone SVG string based on the `svgtemplate.txt` template.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::registry::CardDefinition;
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

mod geom {
	#[derive(Debug, Clone, Copy)]
	pub struct PointF {
		pub x: f32,
		pub y: f32,
	}

	#[derive(Debug, Clone, Copy)]
	pub struct RectI {
		pub x: i32,
		pub y: i32,
		pub w: i32,
		pub h: i32,
	}

	impl RectI {
		pub fn center(&self) -> PointF {
			PointF {
				x: (self.x as f32) + (self.w as f32) / 2.0,
				y: (self.y as f32) + (self.h as f32) / 2.0,
			}
		}
	}

	#[derive(Debug, Clone, Copy)]
	pub struct Bounds {
		pub min_x: f32,
		pub min_y: f32,
		pub max_x: f32,
		pub max_y: f32,
	}

	impl Bounds {
		pub fn empty() -> Self {
			Self {
				min_x: f32::INFINITY,
				min_y: f32::INFINITY,
				max_x: f32::NEG_INFINITY,
				max_y: f32::NEG_INFINITY,
			}
		}

		pub fn union(mut self, other: Bounds) -> Self {
			self.min_x = self.min_x.min(other.min_x);
			self.min_y = self.min_y.min(other.min_y);
			self.max_x = self.max_x.max(other.max_x);
			self.max_y = self.max_y.max(other.max_y);
			self
		}

		pub fn union_point(self, p: PointF) -> Self {
			self.union(Bounds {
				min_x: p.x,
				min_y: p.y,
				max_x: p.x,
				max_y: p.y,
			})
		}

		pub fn union_rect_i(self, r: RectI) -> Self {
			self.union(Bounds {
				min_x: r.x as f32,
				min_y: r.y as f32,
				max_x: (r.x + r.w) as f32,
				max_y: (r.y + r.h) as f32,
			})
		}
	}
}

mod node {
	use super::{BTreeMap, Card, CardDefinition, HashMap, Layout, RenderError, SvgConfig, geom};

	const WRAP_COLS: usize = 80;

	#[derive(Debug, Clone)]
	pub struct NodeLayout {
		pub id: String,
		pub x: i32,
		pub y: i32,
		pub geom: NodeGeom,
	}

	#[derive(Debug, Clone)]
	pub struct NodeGeom {
		pub width_px: i32,
		pub height_px: i32,
		pub lines: Vec<String>,
		pub bold_line_index: Option<usize>,
	}

	#[derive(Debug, Clone)]
	pub struct PositionedNode {
		pub bbox: geom::RectI,
		pub width_px: i32,
		pub height_px: i32,
		pub geom: NodeGeom,
	}

	pub fn collect_node_layouts(
		layout: &Layout,
		cards_by_id: &HashMap<String, &Card>,
		config: &SvgConfig,
	) -> Result<Vec<NodeLayout>, RenderError> {
		let mut out: Vec<NodeLayout> = Vec::with_capacity(layout.nodes.len());
		for (id, n) in &layout.nodes {
			let card = cards_by_id
				.get(id)
				.ok_or_else(|| RenderError::SvgMissingNode(id.clone()))?;
			let geom = measure_node(card, config);
			out.push(NodeLayout {
				id: id.clone(),
				x: n.x,
				y: n.y,
				geom,
			});
		}
		Ok(out)
	}

	pub fn position_nodes(
		node_layouts: &[NodeLayout],
		config: &SvgConfig,
	) -> HashMap<String, PositionedNode> {
		let spacing = config.node_spacing_px.max(0);

		let min_x = node_layouts.iter().map(|n| n.x).min().unwrap_or(0);
		let max_x = node_layouts.iter().map(|n| n.x).max().unwrap_or(0);
		let min_y = node_layouts.iter().map(|n| n.y).min().unwrap_or(0);
		let max_y = node_layouts.iter().map(|n| n.y).max().unwrap_or(0);

		let max_width = node_layouts
			.iter()
			.map(|n| n.geom.width_px)
			.max()
			.unwrap_or(super::SYMBOL_BASE_WIDTH_PX);
		let max_height = node_layouts
			.iter()
			.map(|n| n.geom.height_px)
			.max()
			.unwrap_or(super::SYMBOL_BASE_HEIGHT_PX);

		let mut col_width: BTreeMap<i32, i32> = BTreeMap::new();
		for x in min_x..=max_x {
			let w = node_layouts
				.iter()
				.filter(|n| n.x == x)
				.map(|n| n.geom.width_px)
				.max()
				.unwrap_or(max_width);
			col_width.insert(x, if w > 0 { w } else { max_width });
		}
		let mut row_height: BTreeMap<i32, i32> = BTreeMap::new();
		for y in min_y..=max_y {
			let h = node_layouts
				.iter()
				.filter(|n| n.y == y)
				.map(|n| n.geom.height_px)
				.max()
				.unwrap_or(max_height);
			row_height.insert(y, if h > 0 { h } else { max_height });
		}

		let mut col_origin: BTreeMap<i32, i32> = BTreeMap::new();
		let mut current_x = 0;
		for (x, w) in col_width.iter() {
			col_origin.insert(*x, current_x);
			current_x += *w + spacing;
		}
		let mut row_origin: BTreeMap<i32, i32> = BTreeMap::new();
		let mut current_y = 0;
		for (y, h) in row_height.iter() {
			row_origin.insert(*y, current_y);
			current_y += *h + spacing;
		}

		let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
		for n in node_layouts {
			let cw = col_width.get(&n.x).copied().unwrap_or(max_width);
			let rh = row_height.get(&n.y).copied().unwrap_or(max_height);
			let origin_x = col_origin.get(&n.x).copied().unwrap_or(0);
			let origin_y = row_origin.get(&n.y).copied().unwrap_or(0);
			let px = origin_x + (cw - n.geom.width_px) / 2;
			let py = origin_y + (rh - n.geom.height_px) / 2;
			positioned.insert(
				n.id.clone(),
				PositionedNode {
					bbox: geom::RectI {
						x: px,
						y: py,
						w: n.geom.width_px,
						h: n.geom.height_px,
					},
					width_px: n.geom.width_px,
					height_px: n.geom.height_px,
					geom: n.geom.clone(),
				},
			);
		}
		positioned
	}

	pub fn render_node(card: &Card, node: &PositionedNode, config: &SvgConfig) -> String {
		let def = CardDefinition::get_by_type(&card.card_type);
		let fill = def.fill;
		let stroke = def.color.clone();
		let text_color = def.color;
		let mut icon = def.icon;
		if icon == "question" {
			icon = "?".to_string();
		}

		let geom = &node.geom;
		let rem_px = config.base_font_size_px.max(1);
		let line_height_px = ((config.base_font_size_px as f32) * 1.2).ceil() as i32;
		let top_padding_px = rem_px;
		let start_y = top_padding_px + config.base_font_size_px;
		let center_x = (node.width_px as f32) / 2.0;

		let shape_id = def.shape;
		let sx = (node.width_px as f32) / (super::SYMBOL_BASE_WIDTH_PX as f32);
		let sy = (node.height_px as f32) / (super::SYMBOL_BASE_HEIGHT_PX as f32);

		let mut out = String::new();
		out.push_str(&format!(
			"<g id=\"{}\" transform=\"translate({}, {})\">",
			escape_attr(&card.id),
			node.bbox.x,
			node.bbox.y
		));
		out.push_str(&format!(
			"<g transform=\"scale({:.6}, {:.6})\"><use href=\"#{}\" style=\"fill:{};stroke:{};\" /></g>",
			sx,
			sy,
			escape_attr(&shape_id),
			fill,
			stroke
		));

		// Icon: 2rem x 2rem, 3rem font-size.
		let icon_x = (2 * rem_px) as f32;
		let icon_y = (2 * rem_px) as f32;
		out.push_str(&format!(
			"<text x=\"{:.2}\" y=\"{:.2}\" style=\"font-size:{}px;text-anchor:middle;dominant-baseline:middle;fill:{};\">{}</text>",
			icon_x,
			icon_y,
			3 * rem_px,
			text_color,
			escape_text(&icon)
		));

		for (i, line) in geom.lines.iter().enumerate() {
			let y = start_y + (i as i32) * line_height_px;
			let mut style = format!(
				"fill:{};text-anchor:middle;dominant-baseline:middle;font-size:{}px;",
				text_color, config.base_font_size_px
			);
			if geom.bold_line_index == Some(i) {
				style.push_str("font-weight:bold;");
			}
			out.push_str(&format!(
				"<text x=\"{:.2}\" y=\"{}\" style=\"{}\">{}</text>",
				center_x,
				y,
				style,
				escape_text(line)
			));
		}
		out.push_str("</g>");
		out
	}

	fn measure_node(card: &Card, config: &SvgConfig) -> NodeGeom {
		let rem_px = config.base_font_size_px.max(1);
		let line_height_px = ((config.base_font_size_px as f32) * 1.2).ceil() as i32;
		let char_width_px = ((config.base_font_size_px as f32) * 0.5).ceil() as i32;

		let title = match &card.card_subtype {
			Some(subtype) if !subtype.trim().is_empty() => {
				format!("{} ({})", card.card_type, subtype)
			}
			_ => card.card_type.clone(),
		};

		let mut lines: Vec<String> = Vec::new();
		lines.extend(wrap_text(&title, WRAP_COLS));
		let bold_line_index = Some(lines.len());
		lines.extend(wrap_text(&card.name, WRAP_COLS));
		lines.push(String::new());
		lines.extend(wrap_text(&card.description, WRAP_COLS));

		let longest = lines.iter().map(|s| s.chars().count()).max().unwrap_or(0);
		let left_padding_px = rem_px;
		let right_padding_px = 4 * rem_px;
		let width_px = (longest as i32) * char_width_px + left_padding_px + right_padding_px;
		let width_px = width_px.max(super::SYMBOL_BASE_WIDTH_PX);

		let height_px = (lines.len() as i32) * line_height_px + (2 * rem_px);
		let height_px = height_px.max(112);

		NodeGeom {
			width_px,
			height_px,
			lines,
			bold_line_index,
		}
	}

	pub(super) fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
		let mut out: Vec<String> = Vec::new();
		for raw_line in text.split('\n') {
			if raw_line.is_empty() {
				out.push(String::new());
				continue;
			}
			let mut current = raw_line.trim_end().to_string();
			while current.chars().count() > max_cols {
				let mut cut = None;
				let mut count = 0;
				for (idx, ch) in current.char_indices() {
					count += 1;
					if count > max_cols {
						break;
					}
					if ch.is_whitespace() {
						cut = Some(idx);
					}
				}

				let split_at = cut.unwrap_or_else(|| {
					current
						.char_indices()
						.nth(max_cols)
						.map(|(idx, _)| idx)
						.unwrap_or(current.len())
				});

				let (head, tail) = current.split_at(split_at);
				out.push(head.trim().to_string());
				current = tail.trim().to_string();
			}
			out.push(current.to_string());
		}
		out
	}

	pub(super) fn escape_text(s: &str) -> String {
		s.replace('&', "&amp;")
			.replace('<', "&lt;")
			.replace('>', "&gt;")
	}

	fn escape_attr(s: &str) -> String {
		escape_text(s)
			.replace('"', "&quot;")
			.replace('\'', "&apos;")
	}
}

mod edge {
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
}

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
