//! Context graph layout helpers for the Aurora editor.

use std::collections::{BTreeSet, HashMap};

use aurora_shared::{Card, CardDefinition};

use crate::constants::{
	GRAPH_CENTER_X, GRAPH_CENTER_Y, GRAPH_COLUMN_OFFSET, GRAPH_HEIGHT, GRAPH_NODE_HEIGHT,
	GRAPH_NODE_WIDTH, GRAPH_ROW_SPACING, GRAPH_WIDTH,
};
use crate::model::EditorModel;

#[derive(Debug, Clone, PartialEq)]
pub struct GraphView {
	pub nodes: Vec<GraphNode>,
	pub edges: Vec<GraphEdge>,
	pub width: f32,
	pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphNode {
	pub id: String,
	pub card_type: String,
	pub name: String,
	pub icon: Option<String>,
	pub shape: GraphShape,
	pub fill: String,
	pub stroke: String,
	pub text_color: String,
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphEdge {
	pub source: String,
	pub target: String,
	pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphShape {
	Rect { radius: f32, dashed: bool },
	Ellipse,
	Diamond,
	Hexagon,
	Octagon,
	Cylinder,
}

pub fn build_context_graph(model: &EditorModel, focused_id: &str) -> Option<GraphView> {
	let focus_card = model.card(focused_id)?;
	let context = model.context_view(focused_id);
	let mut ids = Vec::new();
	ids.push(focused_id.to_string());
	if let Some(parent) = context.parent.as_ref() {
		ids.push(parent.id.clone());
	}
	ids.extend(context.siblings.iter().map(|summary| summary.id.clone()));
	ids.extend(context.children.iter().map(|summary| summary.id.clone()));

	let mut seen = BTreeSet::new();
	let mut ordered_ids = Vec::new();
	for id in ids {
		if seen.insert(id.clone()) {
			ordered_ids.push(id);
		}
	}

	let mut nodes = Vec::new();
	let mut node_index = HashMap::new();

	let center_node = build_node(focus_card, GRAPH_CENTER_X, GRAPH_CENTER_Y);
	node_index.insert(center_node.id.clone(), center_node.clone());
	nodes.push(center_node);

	if let Some(parent) = context.parent.as_ref() {
		if let Some(card) = model.card(&parent.id) {
			let node = build_node(card, GRAPH_CENTER_X, GRAPH_CENTER_Y - GRAPH_COLUMN_OFFSET);
			node_index.insert(node.id.clone(), node.clone());
			nodes.push(node);
		}
	}

	let sibling_positions = layout_column(
		GRAPH_CENTER_X - GRAPH_COLUMN_OFFSET,
		GRAPH_CENTER_Y,
		context.siblings.len(),
	);
	for (summary, (x, y)) in context.siblings.iter().zip(sibling_positions.into_iter()) {
		if let Some(card) = model.card(&summary.id) {
			let node = build_node(card, x, y);
			node_index.insert(node.id.clone(), node.clone());
			nodes.push(node);
		}
	}

	let child_positions = layout_column(
		GRAPH_CENTER_X + GRAPH_COLUMN_OFFSET,
		GRAPH_CENTER_Y,
		context.children.len(),
	);
	for (summary, (x, y)) in context.children.iter().zip(child_positions.into_iter()) {
		if let Some(card) = model.card(&summary.id) {
			let node = build_node(card, x, y);
			node_index.insert(node.id.clone(), node.clone());
			nodes.push(node);
		}
	}

	let edges = build_edges(model, &node_index);

	Some(GraphView {
		nodes,
		edges,
		width: GRAPH_WIDTH,
		height: GRAPH_HEIGHT,
	})
}

fn build_node(card: &Card, x: f32, y: f32) -> GraphNode {
	let definition = CardDefinition::try_get_by_type(&card.card_type);
	let (shape, fill, stroke, text_color) = definition
		.map(definition_styles)
		.unwrap_or_else(default_styles);
	let icon = definition.and_then(|definition| editor_icon_glyph(definition.icon));
	GraphNode {
		id: card.id.clone(),
		card_type: card.card_type.clone(),
		name: card.name.clone(),
		icon,
		shape,
		fill,
		stroke,
		text_color,
		x,
		y,
		width: GRAPH_NODE_WIDTH,
		height: GRAPH_NODE_HEIGHT,
	}
}

fn editor_icon_glyph(icon_name: &str) -> Option<String> {
	let normalized = icon_name.trim();
	if normalized.is_empty() || normalized.eq_ignore_ascii_case("n/a") {
		return None;
	}
	let symbol = match normalized.to_ascii_lowercase().as_str() {
		"target" => Some("O"),
		"compass" => Some("C"),
		"checklist" => Some("V"),
		"spark" => Some("*"),
		"star" => Some("*"),
		"layers" => Some("="),
		"app-window" => Some("A"),
		"cube" => Some("C"),
		"plug" => Some("P"),
		"file-text" => Some("T"),
		"file" => Some("F"),
		"shield-key" => Some("S"),
		"database" => Some("D"),
		"cloud" => Some("C"),
		"server" | "server-cog" => Some("S"),
		"workflow" => Some("W"),
		"steps" => Some(">"),
		"user" => Some("U"),
		"book" => Some("B"),
		"bolt" => Some("B"),
		"timeline" => Some("T"),
		"dot" => Some("."),
		"split" => Some("|"),
		"lock" => Some("L"),
		"ruler" => Some("R"),
		"alert-triangle" => Some("!"),
		"bug" => Some("B"),
		"beaker" => Some("B"),
		"note-sticky" => Some("N"),
		"braces" => Some("{"),
		"square-dashed" => Some("#"),
		_ => None,
	};
	if let Some(symbol) = symbol {
		return Some(symbol.to_string());
	}
	normalized
		.chars()
		.find(|value| value.is_ascii_alphanumeric())
		.map(|value| value.to_ascii_uppercase().to_string())
}

fn definition_styles(definition: &CardDefinition) -> (GraphShape, String, String, String) {
	if definition.card_type == "Boundary" {
		return (
			GraphShape::Rect {
				radius: 10.0,
				dashed: true,
			},
			"transparent".to_string(),
			"#94a3b8".to_string(),
			"#64748b".to_string(),
		);
	}
	let shape = match definition.shape.trim().to_ascii_lowercase().as_str() {
		"rounded box" => GraphShape::Rect {
			radius: 12.0,
			dashed: false,
		},
		"box" | "box3d" | "component" | "tab" | "record" | "folder" | "note" => GraphShape::Rect {
			radius: 6.0,
			dashed: false,
		},
		"cluster (dashed)" => GraphShape::Rect {
			radius: 8.0,
			dashed: true,
		},
		"ellipse" | "oval" => GraphShape::Ellipse,
		"diamond" => GraphShape::Diamond,
		"hexagon" => GraphShape::Hexagon,
		"octagon" | "double-octagon" | "double octagon" => GraphShape::Octagon,
		"cylinder" => GraphShape::Cylinder,
		_ => GraphShape::Rect {
			radius: 6.0,
			dashed: false,
		},
	};
	let fill = definition.fill.trim().to_string();
	let stroke = definition.color.trim().to_string();
	let text_color = definition.color.trim().to_string();
	(shape, fill, stroke, text_color)
}

fn default_styles() -> (GraphShape, String, String, String) {
	(
		GraphShape::Rect {
			radius: 6.0,
			dashed: false,
		},
		"#ffffff".to_string(),
		"#0f172a".to_string(),
		"#0f172a".to_string(),
	)
}

fn layout_column(x: f32, center_y: f32, count: usize) -> Vec<(f32, f32)> {
	if count == 0 {
		return Vec::new();
	}
	let total_height = (count.saturating_sub(1) as f32) * GRAPH_ROW_SPACING;
	let start_y = center_y - total_height / 2.0;
	(0..count)
		.map(|index| (x, start_y + index as f32 * GRAPH_ROW_SPACING))
		.collect()
}

fn build_edges(model: &EditorModel, node_index: &HashMap<String, GraphNode>) -> Vec<GraphEdge> {
	let mut edges = Vec::new();
	let node_ids: BTreeSet<String> = node_index.keys().cloned().collect();
	for node_id in node_ids.iter() {
		let Some(card) = model.card(node_id) else {
			continue;
		};
		for link in &card.links {
			if node_ids.contains(&link.target) {
				edges.push(GraphEdge {
					source: node_id.clone(),
					target: link.target.clone(),
					label: link.relationship.clone(),
				});
			}
		}
	}
	edges
}
