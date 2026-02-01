//! Context graph rendering.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::graph::{GraphEdge, GraphNode, GraphShape, GraphView, build_context_graph};
use crate::model::EditorModel;
use crate::state::AppState;

use super::empty_state::render_empty_state;
use super::state_sync::select_card_and_sync;
use super::types::EditorDraft;

/// Renders the context graph panel.
pub(crate) fn render_context(
	model: Option<EditorModel>,
	focused_id: Option<String>,
	state: Signal<AppState>,
	draft: Signal<EditorDraft>,
) -> Element {
	let graph = model.as_ref().and_then(|model| {
		focused_id
			.as_deref()
			.and_then(|id| build_context_graph(model, id))
	});

	rsx! {
		section { class: "panel mindmap-panel",
			h2 { "Mindmap" }
			if let Some(graph) = graph {
				GraphViewPane { graph, state: state.clone(), draft: draft.clone() }
			} else {
				{render_empty_state("Select a card to see its context.")}
			}
		}
	}
}

#[component]
fn GraphViewPane(graph: GraphView, state: Signal<AppState>, draft: Signal<EditorDraft>) -> Element {
	let zoom = use_signal(|| 1.0f32);
	let zoom_value = *zoom.read();
	let zoom_percent = (zoom_value * 100.0).round() as i32;
	let svg_width = graph.width * zoom_value;
	let svg_height = graph.height * zoom_value;
	let view_box = format!("0 0 {} {}", graph.width, graph.height);
	let mut node_lookup = HashMap::new();
	for node in &graph.nodes {
		node_lookup.insert(node.id.clone(), node);
	}

	let on_zoom_in = {
		let mut zoom = zoom.clone();
		move |_| {
			let current = *zoom.read();
			*zoom.write() = (current * 1.1).min(4.0);
		}
	};
	let on_zoom_out = {
		let mut zoom = zoom.clone();
		move |_| {
			let current = *zoom.read();
			*zoom.write() = (current * 0.9).max(0.5);
		}
	};
	let on_zoom_reset = {
		let mut zoom = zoom.clone();
		move |_| {
			*zoom.write() = 1.0;
		}
	};

	rsx! {
		div { class: "mindmap-toolbar",
			button { onclick: on_zoom_out, "-" }
			div { class: "inline-muted", "{zoom_percent}%" }
			button { onclick: on_zoom_in, "+" }
			button { onclick: on_zoom_reset, "Reset" }
		}
		div { class: "graph-container",
			svg {
				width: "{svg_width}",
				height: "{svg_height}",
				view_box: "{view_box}",
				defs {
					marker {
						id: "arrow",
						marker_width: "10",
						marker_height: "10",
						ref_x: "9",
						ref_y: "3",
						orient: "auto",
						marker_units: "strokeWidth",
						path { d: "M0,0 L9,3 L0,6 Z", fill: "#94a3b8" }
					}
				}
				for edge in &graph.edges {
					if let (Some(source), Some(target)) = (
						node_lookup.get(&edge.source),
						node_lookup.get(&edge.target),
					) {
						{render_edge(edge, source, target)}
					}
				}
				for node in &graph.nodes {
					{render_node(node, state.clone(), draft.clone())}
				}
			}
		}
	}
}

fn render_edge(edge: &GraphEdge, source: &GraphNode, target: &GraphNode) -> Element {
	let mid_x = (source.x + target.x) / 2.0;
	let mid_y = (source.y + target.y) / 2.0;
	rsx! {
		g {
			line {
				x1: "{source.x}",
				y1: "{source.y}",
				x2: "{target.x}",
				y2: "{target.y}",
				stroke: "#94a3b8",
				stroke_width: "1.5",
				marker_end: "url(#arrow)",
			}
			if !edge.label.trim().is_empty() {
				text {
					x: "{mid_x}",
					y: "{mid_y - 6.0}",
					text_anchor: "middle",
					font_size: "10",
					fill: "#64748b",
					"{edge.label}"
				}
			}
		}
	}
}

fn render_node(node: &GraphNode, state: Signal<AppState>, draft: Signal<EditorDraft>) -> Element {
	let state = state.clone();
	let draft = draft.clone();
	let node_id = node.id.clone();
	let icon_x = node.x - node.width / 2.0 + 16.0;
	let icon_y = node.y - node.height / 2.0 + 20.0;
	let label_y = node.y - 8.0;
	let id_line = truncate_text(&node.id, 16);
	let name_line = truncate_text(&node.name, 18);
	let type_line = truncate_text(&node.card_type, 16);

	rsx! {
		g { onclick: move |_| select_card_and_sync(state.clone(), draft.clone(), node_id.clone()),
			{render_node_shape(node)}
			if let Some(icon) = node.icon.as_ref() {
				text {
					class: "graph-icon",
					x: "{icon_x}",
					y: "{icon_y}",
					font_size: "16",
					text_anchor: "start",
					fill: "{node.text_color}",
					"{icon}"
				}
			}
			text {
				x: "{node.x}",
				y: "{label_y}",
				text_anchor: "middle",
				font_size: "11",
				fill: "{node.text_color}",
				tspan { x: "{node.x}", dy: "0", "{type_line}" }
				tspan { x: "{node.x}", dy: "14", "{id_line}" }
				tspan { x: "{node.x}", dy: "14", "{name_line}" }
			}
		}
	}
}

fn render_node_shape(node: &GraphNode) -> Element {
	let x = node.x - node.width / 2.0;
	let y = node.y - node.height / 2.0;
	let stroke_dasharray = if matches!(node.shape, GraphShape::Rect { dashed: true, .. }) {
		"4 4"
	} else {
		"0"
	};
	match node.shape {
		GraphShape::Rect { radius, .. } => rsx! {
			rect {
				x: "{x}",
				y: "{y}",
				width: "{node.width}",
				height: "{node.height}",
				rx: "{radius}",
				ry: "{radius}",
				fill: "{node.fill}",
				stroke: "{node.stroke}",
				stroke_width: "2",
				stroke_dasharray: "{stroke_dasharray}",
			}
		},
		GraphShape::Ellipse => rsx! {
			ellipse {
				cx: "{node.x}",
				cy: "{node.y}",
				rx: "{node.width / 2.0}",
				ry: "{node.height / 2.0}",
				fill: "{node.fill}",
				stroke: "{node.stroke}",
				stroke_width: "2",
			}
		},
		GraphShape::Diamond => rsx! {
			polygon {
				points: "{diamond_points(node)}",
				fill: "{node.fill}",
				stroke: "{node.stroke}",
				stroke_width: "2",
			}
		},
		GraphShape::Hexagon => rsx! {
			polygon {
				points: "{hexagon_points(node)}",
				fill: "{node.fill}",
				stroke: "{node.stroke}",
				stroke_width: "2",
			}
		},
		GraphShape::Octagon => rsx! {
			polygon {
				points: "{octagon_points(node)}",
				fill: "{node.fill}",
				stroke: "{node.stroke}",
				stroke_width: "2",
			}
		},
		GraphShape::Cylinder => rsx! {
			g {
				rect {
					x: "{x}",
					y: "{y + node.height * 0.15}",
					width: "{node.width}",
					height: "{node.height * 0.7}",
					fill: "{node.fill}",
					stroke: "{node.stroke}",
					stroke_width: "2",
				}
				ellipse {
					cx: "{node.x}",
					cy: "{y + node.height * 0.15}",
					rx: "{node.width / 2.0}",
					ry: "{node.height * 0.15}",
					fill: "{node.fill}",
					stroke: "{node.stroke}",
					stroke_width: "2",
				}
			}
		},
	}
}

fn truncate_text(value: &str, max_len: usize) -> String {
	let trimmed = value.trim();
	if trimmed.chars().count() <= max_len {
		return trimmed.to_string();
	}
	let mut shortened = trimmed
		.chars()
		.take(max_len.saturating_sub(1))
		.collect::<String>();
	shortened.push('…');
	shortened
}

fn diamond_points(node: &GraphNode) -> String {
	let left = node.x - node.width / 2.0;
	let right = node.x + node.width / 2.0;
	let top = node.y - node.height / 2.0;
	let bottom = node.y + node.height / 2.0;
	format!(
		"{x1},{y1} {x2},{y2} {x3},{y3} {x4},{y4}",
		x1 = node.x,
		y1 = top,
		x2 = right,
		y2 = node.y,
		x3 = node.x,
		y3 = bottom,
		x4 = left,
		y4 = node.y,
	)
}

fn hexagon_points(node: &GraphNode) -> String {
	let left = node.x - node.width / 2.0;
	let right = node.x + node.width / 2.0;
	let top = node.y - node.height / 2.0;
	let bottom = node.y + node.height / 2.0;
	let inset = node.width * 0.2;
	format!(
		"{x1},{y1} {x2},{y2} {x3},{y3} {x4},{y4} {x5},{y5} {x6},{y6}",
		x1 = left + inset,
		y1 = top,
		x2 = right - inset,
		y2 = top,
		x3 = right,
		y3 = node.y,
		x4 = right - inset,
		y4 = bottom,
		x5 = left + inset,
		y5 = bottom,
		x6 = left,
		y6 = node.y,
	)
}

fn octagon_points(node: &GraphNode) -> String {
	let left = node.x - node.width / 2.0;
	let right = node.x + node.width / 2.0;
	let top = node.y - node.height / 2.0;
	let bottom = node.y + node.height / 2.0;
	let inset = node.width * 0.18;
	let inset_y = node.height * 0.18;
	format!(
		"{x1},{y1} {x2},{y2} {x3},{y3} {x4},{y4} {x5},{y5} {x6},{y6} {x7},{y7} {x8},{y8}",
		x1 = left + inset,
		y1 = top,
		x2 = right - inset,
		y2 = top,
		x3 = right,
		y3 = top + inset_y,
		x4 = right,
		y4 = bottom - inset_y,
		x5 = right - inset,
		y5 = bottom,
		x6 = left + inset,
		y6 = bottom,
		x7 = left,
		y7 = bottom - inset_y,
		x8 = left,
		y8 = top + inset_y,
	)
}
