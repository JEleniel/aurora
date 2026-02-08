use std::collections::{BTreeMap, HashMap};

use crate::registry::CardDefinition;
use crate::{Card, Layout};

use super::{RenderError, SvgConfig, geom};

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
		"<text x=\"{:.2}\" y=\"{:.2}\" style=\"font-size:{}px;text-anchor:middle;dominant-baseline:middle;fill:{};stroke:none;\">{}</text>",
		icon_x,
		icon_y,
		3 * rem_px,
		text_color,
		escape_text(&icon)
	));

	for (i, line) in geom.lines.iter().enumerate() {
		let y = start_y + (i as i32) * line_height_px;
		let mut style = format!(
			"fill:{};stroke:none;text-anchor:middle;dominant-baseline:middle;font-size:{}px;",
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
