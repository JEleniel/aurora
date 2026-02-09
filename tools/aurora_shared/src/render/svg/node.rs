use std::collections::{BTreeMap, HashMap};

use crate::registry::{CardDefinition, CardRegistry, RegistryError};
use crate::{Card, Layout};

use super::{RenderError, SvgConfig, geom};

const WRAP_COLS: usize = 80;

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
			// Gap at x=1: no node uses that column.
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
}

pub fn render_node(card: &Card, node: &PositionedNode, config: &SvgConfig) -> String {
	let card_definition = lookup_card_definition(&card.card_type).unwrap();

	let geom = &node.geom;
	let rem_px = config.base_font_size_px.max(1);
	let line_height_px = ((config.base_font_size_px as f32) * 1.2).ceil() as i32;
	let top_padding_px = rem_px;
	let start_y = top_padding_px + config.base_font_size_px;
	let center_x = (node.width_px as f32) / 2.0;

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
		escape_attr(&card_definition.shape),
		card_definition.fill,
		card_definition.color
	));

	// Icon: 2rem x 2rem, 3rem font-size.
	// Use top-left anchoring so the icon is consistently 1rem (16px) from the top-left,
	// regardless of the glyph's bounding box.
	let icon_x = rem_px as f32 * 1.5;
	let icon_y = rem_px as f32 * 2.0;
	out.push_str(&format!(
		"<text x=\"{:.2}\" y=\"{:.2}\" style=\"font-size:{}px;text-anchor:start;dominant-baseline:hanging;fill:{};stroke:none;\">{}</text>",
		icon_x,
		icon_y,
		3 * rem_px,
		card_definition.color,
		escape_text(&card_definition.icon)
	));

	for (i, line) in geom.lines.iter().enumerate() {
		let y = start_y + (i as i32) * line_height_px;
		let mut style = format!(
			"fill:{};stroke:none;text-anchor:middle;dominant-baseline:middle;font-size:{}px;",
			card_definition.color, config.base_font_size_px
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
