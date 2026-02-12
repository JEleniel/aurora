use std::collections::{BTreeMap, HashMap};

use crate::registry::{CardDefinition, CardRegistry, RegistryError};
use crate::{Card, Layout};

use super::{RenderError, SvgConfig, geom};

const WRAP_COLS: usize = 80;
const SYMBOL_BASE_W: f32 = super::SYMBOL_BASE_WIDTH_PX as f32;
const SYMBOL_BASE_H: f32 = super::SYMBOL_BASE_HEIGHT_PX as f32;
const KNOWN_SHAPES: &[&str] = &[
	"rectangle",
	"box",
	"cylinder",
	"diamond",
	"document",
	"double-rectangle",
	"ellipse",
	"hexagon",
	"interface",
	"octagon",
	"folder",
	"ruler",
	"lolipop",
	"curly-braces",
	"poploli",
	"rounded-rectangle",
	"trapezoid",
	"component",
];

fn lookup_card_definition(card_type: &str) -> Result<CardDefinition, RegistryError> {
	let registry = CardRegistry::try_new()?;

	Ok(registry.try_get_by_type(card_type)?)
}

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

#[derive(Debug, Clone)]
pub struct RenderedNode {
	pub shape: String,
	pub labels: String,
}

#[derive(Debug, Clone, Copy)]
struct SymbolPlacement {
	offset_x: f32,
	offset_y: f32,
	width: f32,
	height: f32,
	scale_x: f32,
	scale_y: f32,
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
	if node_layouts.is_empty() {
		return HashMap::new();
	}

	let mut used_x: Vec<i32> = node_layouts.iter().map(|n| n.x).collect();
	used_x.sort();
	used_x.dedup();
	let mut used_y: Vec<i32> = node_layouts.iter().map(|n| n.y).collect();
	used_y.sort();
	used_y.dedup();

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
	for x in &used_x {
		let w = node_layouts
			.iter()
			.filter(|n| n.x == *x)
			.map(|n| n.geom.width_px)
			.max()
			.unwrap_or(0);
		col_width.insert(*x, w.max(0));
	}
	let mut row_height: BTreeMap<i32, i32> = BTreeMap::new();
	for y in &used_y {
		let h = node_layouts
			.iter()
			.filter(|n| n.y == *y)
			.map(|n| n.geom.height_px)
			.max()
			.unwrap_or(0);
		row_height.insert(*y, h.max(0));
	}

	let mut col_origin: BTreeMap<i32, i32> = BTreeMap::new();
	let mut current_x = 0;
	for x in &used_x {
		let w = col_width.get(x).copied().unwrap_or(0);
		col_origin.insert(*x, current_x);
		current_x += w + spacing;
	}
	let mut row_origin: BTreeMap<i32, i32> = BTreeMap::new();
	let mut current_y = 0;
	for y in &used_y {
		let h = row_height.get(y).copied().unwrap_or(0);
		row_origin.insert(*y, current_y);
		current_y += h + spacing;
	}

	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	for n in node_layouts {
		let cw = col_width
			.get(&n.x)
			.copied()
			.unwrap_or_else(|| n.geom.width_px.max(max_width));
		let rh = row_height
			.get(&n.y)
			.copied()
			.unwrap_or_else(|| n.geom.height_px.max(max_height));
		let cw = cw.max(n.geom.width_px);
		let rh = rh.max(n.geom.height_px);
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

pub fn render_node(card: &Card, node: &PositionedNode, config: &SvgConfig) -> RenderedNode {
	let card_definition = lookup_card_definition(&card.card_type).unwrap();
	let shape_name = card_definition.shape.trim().to_lowercase();
	let shape_id = if KNOWN_SHAPES.contains(&shape_name.as_str()) {
		shape_name.as_str()
	} else {
		"rectangle"
	};

	let geom = &node.geom;
	let rem_px = config.base_font_size_px.max(1);
	let line_height_px = ((config.base_font_size_px as f32) * 1.2).ceil() as i32;
	let center_x = (node.width_px as f32) / 2.0;

	let symbol = fit_symbol(node.width_px, node.height_px);

	let escaped_id = escape_attr(&card.id);
	let label_id = escape_attr(&format!("{}-label", card.id));

	let mut shape = String::new();
	shape.push_str(&format!(
		"<g id=\"{}\" transform=\"translate({}, {})\">",
		escaped_id, node.bbox.x, node.bbox.y
	));
	shape.push_str(&format!(
		"<g class=\"aurora-symbol\" transform=\"translate({:.2}, {:.2}) scale({:.6}, {:.6})\"><use href=\"#{}\" style=\"fill:{};stroke:{};\" /></g>",
		symbol.offset_x,
		symbol.offset_y,
		symbol.scale_x,
		symbol.scale_y,
		escape_attr(shape_id),
		card_definition.fill,
		card_definition.color
	));
	shape.push_str("</g>");

	let icon_x = symbol.offset_x + (symbol.width * 0.13);
	let icon_y = symbol.offset_y + (symbol.height * 0.18);
	let icon_font_size = (symbol.height * 0.24).clamp(rem_px as f32 * 1.2, rem_px as f32 * 2.8);

	let mut labels = String::new();
	labels.push_str(&format!(
		"<g id=\"{}\" transform=\"translate({}, {})\">",
		label_id, node.bbox.x, node.bbox.y
	));
	labels.push_str(&format!(
		"<text x=\"{:.2}\" y=\"{:.2}\" style=\"font-size:{:.2}px;text-anchor:start;dominant-baseline:hanging;fill:{};stroke:none;\">{}</text>",
		icon_x,
		icon_y,
		icon_font_size,
		card_definition.color,
		escape_text(&card_definition.icon)
	));

	let start_y = text_start_with_top_inset(symbol, geom.lines.len(), line_height_px, rem_px);
	for (i, line) in geom.lines.iter().enumerate() {
		let y = start_y + (i as f32) * (line_height_px as f32);
		let mut style = format!(
			"fill:{};stroke:none;text-anchor:middle;dominant-baseline:middle;font-size:{}px;",
			card_definition.color, config.base_font_size_px
		);
		if geom.bold_line_index == Some(i) {
			style.push_str("font-weight:bold;");
		}
		labels.push_str(&format!(
			"<text x=\"{:.2}\" y=\"{:.2}\" style=\"{}\">{}</text>",
			center_x,
			y,
			style,
			escape_text(line)
		));
	}
	labels.push_str("</g>");

	RenderedNode { shape, labels }
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
	let left_padding_px = 2 * rem_px;
	let right_padding_px = 5 * rem_px;
	let width_px = (longest as i32) * char_width_px + left_padding_px + right_padding_px;
	let width_px = width_px.max(super::SYMBOL_BASE_WIDTH_PX);

	let mut height_px = (lines.len() as i32) * line_height_px + (6 * rem_px);
	height_px = height_px.max(super::SYMBOL_BASE_HEIGHT_PX + (2 * rem_px));

	NodeGeom {
		width_px,
		height_px,
		lines,
		bold_line_index,
	}
}

fn fit_symbol(node_width_px: i32, node_height_px: i32) -> SymbolPlacement {
	let width = node_width_px.max(1) as f32;
	let height = node_height_px.max(1) as f32;
	let symbol_w = (width * 0.98).max(SYMBOL_BASE_W);
	let symbol_h = height.max(SYMBOL_BASE_H);
	let scale_x = (symbol_w / SYMBOL_BASE_W).max(0.001);
	let scale_y = (symbol_h / SYMBOL_BASE_H).max(0.001);
	let offset_x = (width - symbol_w) / 2.0;
	let offset_y = (height - symbol_h) / 2.0;

	SymbolPlacement {
		offset_x,
		offset_y,
		width: symbol_w,
		height: symbol_h,
		scale_x,
		scale_y,
	}
}

fn centered_text_start(symbol: SymbolPlacement, line_count: usize, line_height_px: i32) -> f32 {
	let text_block_h = (line_count as f32) * (line_height_px.max(1) as f32);
	let symbol_center_y = symbol.offset_y + (symbol.height / 2.0);
	symbol_center_y - (text_block_h / 2.0) + ((line_height_px.max(1) as f32) / 2.0)
}

fn text_start_with_top_inset(
	symbol: SymbolPlacement,
	line_count: usize,
	line_height_px: i32,
	top_inset_px: i32,
) -> f32 {
	centered_text_start(symbol, line_count, line_height_px) + (top_inset_px.max(0) as f32)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn position_nodes_does_not_insert_empty_columns() {
		let geom = NodeGeom {
			width_px: 100,
			height_px: 60,
			lines: Vec::new(),
			bold_line_index: None,
		};
		let nodes = vec![
			NodeLayout {
				id: "a".to_string(),
				x: 0,
				y: 0,
				geom: geom.clone(),
			},
			NodeLayout {
				id: "b".to_string(),
				x: 2,
				y: 0,
				geom: geom.clone(),
			},
		];
		let config = SvgConfig {
			node_spacing_px: 10,
			base_font_size_px: 16,
			edge_style: super::super::EdgeStyle::Orthogonal,
		};

		let positioned = position_nodes(&nodes, &config);
		let a = positioned.get("a").expect("a should be positioned");
		let b = positioned.get("b").expect("b should be positioned");
		assert_eq!(a.bbox.x, 0);
		assert_eq!(b.bbox.x, 100 + 10);
	}

	#[test]
	fn fit_symbol_stays_within_node_bounds() {
		let symbol = fit_symbol(640, 220);
		assert!(symbol.width <= 640.0 + 0.01);
		assert!(symbol.height <= 220.0 + 0.01);
		assert!((symbol.scale_x - (symbol.width / SYMBOL_BASE_W)).abs() < 0.0001);
		assert!((symbol.scale_y - (symbol.height / SYMBOL_BASE_H)).abs() < 0.0001);
		assert!(symbol.offset_x >= -0.01);
		assert!(symbol.offset_y >= -0.01);
	}

	#[test]
	fn centered_text_start_centers_block_in_symbol() {
		let symbol = fit_symbol(320, 200);
		let line_height = 20;
		let line_count = 5;
		let start_y = centered_text_start(symbol, line_count, line_height);
		let first_center = start_y;
		let last_center = start_y + ((line_count - 1) as f32) * (line_height as f32);
		let block_center = (first_center + last_center) / 2.0;
		let symbol_center = symbol.offset_y + (symbol.height / 2.0);
		assert!((block_center - symbol_center).abs() < 0.001);
	}

	#[test]
	fn text_start_with_top_inset_adds_requested_padding() {
		let symbol = fit_symbol(320, 200);
		let line_height = 20;
		let line_count = 5;
		let centered = centered_text_start(symbol, line_count, line_height);
		let with_inset = text_start_with_top_inset(symbol, line_count, line_height, 16);
		assert!((with_inset - centered - 16.0).abs() < 0.001);
	}
}
