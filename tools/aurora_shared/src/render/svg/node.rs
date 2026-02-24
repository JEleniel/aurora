use std::collections::{HashMap, HashSet};

use crate::registry::{CardDefinition, CardRegistry};
use crate::{Card, Layout};

use super::{RenderError, SvgConfig, geom};

const WRAP_COLS: usize = 65;
const DESCRIPTION_WRAP_COLS: usize = WRAP_COLS - 5;
const LEADING_LINE_FONT_SIZE_PX: i32 = 24;
const SECOND_LINE_FONT_SIZE_PX: i32 = 28;
const SECOND_LINE_VERTICAL_OFFSET_PX: i32 = 8;
const DESCRIPTION_FONT_SIZE_PX: i32 = TEMPLATE_DEFAULT_FONT_SIZE_PX;
const DESCRIPTION_LINE_HEIGHT: f32 = 1.2;
const COLUMN_PITCH_PX: i32 = 1080;
const ROW_PITCH_PX: i32 = 810;
const SYMBOL_BASE_W: f32 = super::SYMBOL_BASE_WIDTH_PX as f32;
const SYMBOL_BASE_H: f32 = super::SYMBOL_BASE_HEIGHT_PX as f32;
const ICON_VIEWBOX_SIZE_PX: i32 = 128;
const ICON_RENDER_SIZE_PX: i32 = 72;
const ICON_OFFSET_X: f32 = 70.0;
const ICON_OFFSET_Y: f32 = 189.0;
const TEMPLATE_DEFAULT_FONT_SIZE_PX: i32 = 16;

fn lookup_card_definition(card_type: &str, registry: &CardRegistry) -> Option<CardDefinition> {
	registry.try_get_by_type(card_type).ok()
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
	pub description_start_index: usize,
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
	let _ = config;
	if node_layouts.is_empty() {
		return HashMap::new();
	}

	let mut positioned: HashMap<String, PositionedNode> = HashMap::new();
	for n in node_layouts {
		let px = n.x * COLUMN_PITCH_PX;
		let py = n.y * ROW_PITCH_PX;
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

pub fn render_node(
	card: &Card,
	node: &PositionedNode,
	registry: &CardRegistry,
	known_shape_ids: &HashSet<String>,
	config: &SvgConfig,
) -> RenderedNode {
	let card_definition =
		lookup_card_definition(&card.card_type, registry).unwrap_or(CardDefinition {
			acronym: String::new(),
			card_type: card.card_type.clone(),
			stroke: "#000000".to_string(),
			text: "#000000".to_string(),
			description: String::new(),
			fill: "#ffffff".to_string(),
			icon: None,
			relationships: Vec::new(),
			shape: "rectangle".to_string(),
			common_subtypes: Vec::new(),
		});
	let shape_name = card_definition.shape.trim().to_lowercase();
	let shape_id = if known_shape_ids.contains(shape_name.as_str()) {
		shape_name.as_str()
	} else {
		"rectangle"
	};

	let geom = &node.geom;
	let symbol = fit_symbol(node.width_px, node.height_px);
	let center_x = symbol.offset_x + 398.0;

	let escaped_id = escape_attr(&card.id);

	let mut shape = String::new();
	shape.push_str(&format!(
		"<g id=\"{}\" class=\"aurora-symbol\" transform=\"translate({}, {})\">",
		escaped_id, node.bbox.x, node.bbox.y
	));
	let symbol_transform = if symbol.offset_x.abs() < 0.01 && symbol.offset_y.abs() < 0.01 {
		String::new()
	} else {
		format!(
			" transform=\"translate({:.2}, {:.2})\"",
			symbol.offset_x, symbol.offset_y
		)
	};
	shape.push_str(&format!(
		"<use href=\"#{}\"{} style=\"fill:{};stroke:{};\" />",
		escape_attr(shape_id),
		symbol_transform,
		card_definition.fill,
		card_definition.stroke
	));

	let icon_x = symbol.offset_x + ICON_OFFSET_X;
	let icon_y = symbol.offset_y + ICON_OFFSET_Y;
	let icon_scale = ICON_RENDER_SIZE_PX as f32 / ICON_VIEWBOX_SIZE_PX as f32;

	let labels = String::new();
	let icon_name = card
		.icon
		.as_ref()
		.or(card_definition.icon.as_ref())
		.map(String::as_str)
		.unwrap_or("");
	if let Some(icon_id) = normalize_icon_id(icon_name) {
		let icon_href = format!("#i-{}", escape_attr(icon_id.as_str()));
		shape.push_str(&format!(
			"<g class=\"aurora-icon\" transform=\"translate({:.2}, {:.2}) scale({:.6})\"><use href=\"{}\" /></g>",
			icon_x,
			icon_y,
			icon_scale,
			icon_href
		));
	}

	let start_y = symbol.offset_y + 100.2;
	let default_line_step_px = (config.base_font_size_px + 8).max(1) as f32;
	let mut y = start_y;
	for (i, line) in geom.lines.iter().enumerate() {
		if i > 0 {
			let previous_font_px = line_font_size_px(i - 1, geom, config);
			let previous_line_step_px = if i > geom.description_start_index {
				(previous_font_px as f32) * DESCRIPTION_LINE_HEIGHT
			} else {
				default_line_step_px
			};
			y += previous_line_step_px;
		}
		if i == 1 {
			y += SECOND_LINE_VERTICAL_OFFSET_PX as f32;
		}

		let font_size_px = line_font_size_px(i, geom, config);
		let mut style = format!("fill:{};", card_definition.text);
		if font_size_px != TEMPLATE_DEFAULT_FONT_SIZE_PX {
			style.push_str(format!("font-size:{}px;", font_size_px).as_str());
		}
		if i == 1 || geom.bold_line_index == Some(i) {
			style.push_str("font-weight:bold;");
		}
		shape.push_str(&format!(
			"<text x=\"{:.2}\" y=\"{:.2}\" style=\"{}\">{}</text>",
			center_x,
			y,
			style,
			escape_text(line)
		));
	}
	shape.push_str("</g>");

	RenderedNode { shape, labels }
}

fn measure_node(card: &Card, config: &SvgConfig) -> NodeGeom {
	let _ = config;

	let title = match &card.card_subtype {
		Some(subtype) if !subtype.trim().is_empty() => {
			format!("{} ({})", card.card_type, subtype)
		}
		_ => card.card_type.clone(),
	};
	let title = format!("{}: {}", card.id, title);

	let mut lines: Vec<String> = Vec::new();
	lines.extend(wrap_text(&title, WRAP_COLS));
	let bold_line_index = Some(lines.len());
	lines.extend(wrap_text(&card.name, WRAP_COLS));
	lines.push(String::new());
	let description_start_index = lines.len();
	lines.extend(wrap_text(&card.description, DESCRIPTION_WRAP_COLS));

	NodeGeom {
		width_px: super::SYMBOL_BASE_WIDTH_PX,
		height_px: super::SYMBOL_BASE_HEIGHT_PX,
		lines,
		bold_line_index,
		description_start_index,
	}
}

fn line_font_size_px(index: usize, geom: &NodeGeom, config: &SvgConfig) -> i32 {
	if index == 1 {
		SECOND_LINE_FONT_SIZE_PX
	} else if index < 2 {
		LEADING_LINE_FONT_SIZE_PX
	} else if index >= geom.description_start_index {
		DESCRIPTION_FONT_SIZE_PX
	} else {
		config.base_font_size_px
	}
}

fn fit_symbol(node_width_px: i32, node_height_px: i32) -> SymbolPlacement {
	let width = node_width_px.max(1) as f32;
	let height = node_height_px.max(1) as f32;
	let symbol_w = SYMBOL_BASE_W;
	let symbol_h = SYMBOL_BASE_H;
	let offset_x = ((width - symbol_w) / 2.0).max(0.0);
	let offset_y = ((height - symbol_h) / 2.0).max(0.0);

	SymbolPlacement { offset_x, offset_y }
}

fn normalize_icon_id(icon: &str) -> Option<String> {
	let normalized = icon.trim().trim_start_matches('#');
	let normalized = normalized.strip_prefix("i-").unwrap_or(normalized);
	if normalized.is_empty() {
		None
	} else {
		Some(normalized.to_string())
	}
}

pub(super) fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
	let mut out: Vec<String> = Vec::new();
	if max_cols == 0 {
		return vec![text.to_string()];
	}
	for raw_line in text.split('\n') {
		if raw_line.is_empty() {
			out.push(String::new());
			continue;
		}
		let mut current = raw_line.trim_end().to_string();
		while current.chars().count() > max_cols {
			let mut cut_at_whitespace: Option<usize> = None;
			let mut split_at: Option<usize> = None;
			let mut count = 0usize;
			for (idx, ch) in current.char_indices() {
				count += 1;
				if count > max_cols {
					split_at = Some(idx);
					break;
				}
				if ch.is_whitespace() {
					cut_at_whitespace = Some(idx);
				}
			}

			let cut_idx = cut_at_whitespace.or(split_at).unwrap_or(current.len());
			let (head, tail) = current.split_at(cut_idx);
			out.push(head.trim_end().to_string());
			current = tail.trim_start().to_string();
		}
		out.push(current);
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
	use std::collections::HashSet;

	#[test]
	fn position_nodes_does_not_insert_empty_columns() {
		let geom = NodeGeom {
			width_px: 100,
			height_px: 60,
			lines: Vec::new(),
			bold_line_index: None,
			description_start_index: 0,
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
		assert_eq!(b.bbox.x, 2 * COLUMN_PITCH_PX);
	}

	#[test]
	fn position_nodes_uses_fixed_row_pitch() {
		let geom = NodeGeom {
			width_px: 100,
			height_px: 60,
			lines: Vec::new(),
			bold_line_index: None,
			description_start_index: 0,
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
				x: 0,
				y: 1,
				geom,
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
		assert_eq!(a.bbox.y, 0);
		assert_eq!(b.bbox.y, ROW_PITCH_PX);
	}

	#[test]
	fn fit_symbol_stays_within_node_bounds() {
		let symbol = fit_symbol(720, 450);
		assert!((symbol.offset_x - 0.0).abs() < 0.01);
		assert!((symbol.offset_y - 0.0).abs() < 0.01);

		let shifted = fit_symbol(900, 700);
		assert!(shifted.offset_x > 0.0);
		assert!(shifted.offset_y > 0.0);
		assert!(symbol.offset_x >= -0.01);
		assert!(symbol.offset_y >= -0.01);
	}

	#[test]
	fn normalize_icon_id_accepts_prefixed_variants() {
		assert_eq!(normalize_icon_id("wrench").as_deref(), Some("wrench"));
		assert_eq!(normalize_icon_id("i-wrench").as_deref(), Some("wrench"));
		assert_eq!(normalize_icon_id("#i-wrench").as_deref(), Some("wrench"));
		assert_eq!(
			normalize_icon_id("  #i-wrench  ").as_deref(),
			Some("wrench")
		);
		assert!(normalize_icon_id(" ").is_none());
	}

	#[test]
	fn wrap_text_respects_column_limit() {
		let lines = wrap_text(
			"A phrase that should wrap before the final word escapes the card boundary.",
			32,
		);
		assert!(
			lines.iter().all(|line| line.chars().count() <= 32),
			"wrapped lines exceeded width: {lines:?}"
		);
	}

	#[test]
	fn icon_slot_matches_template_layout() {
		assert_eq!(ICON_RENDER_SIZE_PX, 72);
		assert!((ICON_OFFSET_X - 70.0).abs() < 0.01);
		assert!((ICON_OFFSET_Y - 189.0).abs() < 0.01);
	}

	#[test]
	fn measure_node_prefixes_first_line_with_card_id() {
		let card = Card {
			schema: None,
			id: "APP-001".to_string(),
			card_type: "Application".to_string(),
			card_subtype: None,
			name: "Portal".to_string(),
			description: "desc".to_string(),
			version: None,
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: crate::Attributes::new(),
			references: Vec::new(),
			links: Vec::new(),
			source_path: std::path::PathBuf::from("APP-001.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		};
		let geom = measure_node(&card, &SvgConfig::default());
		assert_eq!(
			geom.lines.first().map(String::as_str),
			Some("APP-001: Application")
		);
	}

	#[test]
	fn render_node_uses_stroke_and_text_colors_for_labels() {
		let card = Card {
			schema: None,
			id: "FOO-001".to_string(),
			card_type: "Foo".to_string(),
			card_subtype: None,
			name: "Sample Node".to_string(),
			description: "Description".to_string(),
			version: None,
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: crate::Attributes::new(),
			references: Vec::new(),
			links: Vec::new(),
			source_path: std::path::PathBuf::from("FOO-001.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		};

		let registry = CardRegistry {
			definitions: vec![CardDefinition {
				acronym: "FOO".to_string(),
				card_type: "Foo".to_string(),
				stroke: "#111111".to_string(),
				text: "#eeeeee".to_string(),
				description: "desc".to_string(),
				fill: "#222222".to_string(),
				icon: None,
				relationships: Vec::new(),
				shape: "rectangle".to_string(),
				common_subtypes: Vec::new(),
			}],
			available_icons: HashSet::new(),
		};

		let geom = measure_node(&card, &SvgConfig::default());
		let node = PositionedNode {
			bbox: geom::RectI {
				x: 0,
				y: 0,
				w: geom.width_px,
				h: geom.height_px,
			},
			width_px: geom.width_px,
			height_px: geom.height_px,
			geom,
		};

		let mut shape_ids = HashSet::new();
		shape_ids.insert("rectangle".to_string());
		let rendered = render_node(&card, &node, &registry, &shape_ids, &SvgConfig::default());

		assert!(
			rendered.shape.contains("fill:#222222;stroke:#111111;"),
			"shape style did not apply fill/stroke correctly: {}",
			rendered.shape
		);
		assert!(
			rendered.shape.contains("fill:#eeeeee;"),
			"label style did not apply text color as fill correctly: {}",
			rendered.shape
		);
		assert!(
			!rendered.shape.contains("stroke:#eeeeee;"),
			"label style should not use text color as stroke: {}",
			rendered.shape
		);
		assert!(
			!rendered.shape.contains("text-anchor:middle;"),
			"label style should rely on template default text-anchor: {}",
			rendered.shape
		);
		assert!(
			rendered.labels.is_empty(),
			"render_node should no longer emit a separate labels group: {}",
			rendered.labels
		);
		assert!(
			!rendered.shape.contains("class=\"aurora-node"),
			"rendered node should not include aurora-node class: {}",
			rendered.shape
		);
		assert!(
			!rendered.shape.contains("<svg class=\"aurora-icon\""),
			"icon should be wrapped in a transformed <g> rather than nested <svg>: {}",
			rendered.shape
		);
		assert!(
			!rendered.shape.contains("translate(0.00, 0.00)"),
			"rendered node should not emit redundant zero transforms: {}",
			rendered.shape
		);
	}
}
