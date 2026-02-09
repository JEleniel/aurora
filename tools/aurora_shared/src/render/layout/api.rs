//! Public entry points for layout computation.

use std::collections::{HashMap, HashSet};

use crate::{Model, render::render_error::RenderError};

use super::graph::{EdgeClassification, LayoutGraph, build_graph, classify_edges, validate_graph};
use super::ordering::{
	assign_ranks, assign_x_positions, build_backbone_adjacency, build_layers, center_children,
	center_parents, compact_layer_tight, layer_neighbors, median_sweeps, topo_sort,
};
use super::types::{Layout, LayoutEdge, LayoutNode};

type NeighborMap = HashMap<String, Vec<String>>;

/// Compute a hierarchical layout for a view selection over a model.
pub fn layout_model(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<Layout, RenderError> {
	if root_card_types.is_empty() {
		return Err(RenderError::MissingRoots);
	}

	let graph = build_graph(model, root_card_types, included_card_types)?;
	validate_graph(&graph)?;
	let edge_classification = classify_edges(&graph);

	let (outgoing_backbone, incoming_backbone) =
		build_backbone_adjacency(&graph.allowed_nodes, &edge_classification.backbone);
	let topological_order =
		topo_sort(&graph.allowed_nodes, &outgoing_backbone, &incoming_backbone)?;
	let rank = assign_ranks(&graph.roots, &topological_order, &incoming_backbone)?;

	let (mut layers, max_layer_index) = build_layers(&rank, &graph.incoming_count)?;
	let (predecessor_by_layer, successor_by_layer) =
		layer_neighbors(&rank, &edge_classification.backbone)?;
	median_sweeps(
		&mut layers,
		max_layer_index,
		&predecessor_by_layer,
		&successor_by_layer,
	)?;
	let mut x_positions = assign_x_positions(&layers);

	// Use full (non-adjacent) predecessor/successor maps when centering so that long edges
	// influence positioning even without dummy-node expansion.
	let (all_predecessors, all_successors) = layer_neighbors_all(&rank, &graph.edges)?;

	// Keep the bottom-most rank tight (minimum spacing), but do not force it to start at x=0.
	// We compact around its current position so multi-parent midpoints are preserved.
	for _ in 0..8 {
		center_children(
			&mut x_positions,
			&layers,
			max_layer_index,
			&all_predecessors,
		)?;
		if let Some(bottom) = layers.get(max_layer_index) {
			compact_layer_tight(bottom, &mut x_positions);
		}
		center_parents(&mut x_positions, &layers, max_layer_index, &all_successors)?;
	}
	// Final refinement: ensure the bottom is tight, then center each higher rank above children.
	center_children(
		&mut x_positions,
		&layers,
		max_layer_index,
		&all_predecessors,
	)?;
	if let Some(bottom) = layers.get(max_layer_index) {
		compact_layer_tight(bottom, &mut x_positions);
	}
	center_parents(&mut x_positions, &layers, max_layer_index, &all_successors)?;

	validate_layout(&graph, &edge_classification, &rank)?;
	build_layout(&graph, &rank, &x_positions)
}

fn layer_neighbors_all(
	rank: &HashMap<String, i32>,
	edges: &std::collections::HashSet<(String, String)>,
) -> Result<(NeighborMap, NeighborMap), RenderError> {
	let mut predecessor: NeighborMap = HashMap::new();
	let mut successor: NeighborMap = HashMap::new();
	for (a, b) in edges {
		let rank_a = rank
			.get(a)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(a.clone()))?;
		let rank_b = rank
			.get(b)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(b.clone()))?;
		if rank_b > rank_a {
			predecessor.entry(b.clone()).or_default().push(a.clone());
			successor.entry(a.clone()).or_default().push(b.clone());
		}
	}
	for list in predecessor.values_mut() {
		list.sort();
		list.dedup();
	}
	for list in successor.values_mut() {
		list.sort();
		list.dedup();
	}
	Ok((predecessor, successor))
}

fn validate_layout(
	graph: &LayoutGraph,
	edges: &EdgeClassification,
	rank: &HashMap<String, i32>,
) -> Result<(), RenderError> {
	let classified_edges = edges.backbone.len() + edges.loops.len();
	if classified_edges != graph.edges.len() {
		return Err(RenderError::EdgeClassificationMismatch {
			backbone_count: edges.backbone.len(),
			loop_count: edges.loops.len(),
			total_count: graph.edges.len(),
		});
	}

	for root in &graph.roots {
		let root_rank = rank
			.get(root)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(root.clone()))?;
		if root_rank != 0 {
			return Err(RenderError::RootRankNotZero(root.clone()));
		}
	}
	for (a, b) in &edges.backbone {
		let rank_a = rank
			.get(a)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(a.clone()))?;
		let rank_b = rank
			.get(b)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(b.clone()))?;
		if rank_b <= rank_a {
			return Err(RenderError::BackboneRankOrder(a.clone(), b.clone()));
		}
	}
	Ok(())
}

fn build_layout(
	graph: &LayoutGraph,
	rank: &HashMap<String, i32>,
	x_positions: &HashMap<String, i32>,
) -> Result<Layout, RenderError> {
	let nodes = build_nodes(&graph.allowed_nodes, rank, x_positions)?;
	let edges = build_edges(&graph.edges);
	Ok(Layout { nodes, edges })
}

fn build_nodes(
	allowed_nodes: &HashSet<String>,
	rank: &HashMap<String, i32>,
	x_positions: &HashMap<String, i32>,
) -> Result<HashMap<String, LayoutNode>, RenderError> {
	let mut nodes: HashMap<String, LayoutNode> = HashMap::new();
	for node_id in allowed_nodes {
		let x = x_positions
			.get(node_id)
			.copied()
			.ok_or_else(|| RenderError::MissingX(node_id.clone()))?;
		let y = rank
			.get(node_id)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(node_id.clone()))?;
		nodes.insert(
			node_id.clone(),
			LayoutNode {
				id: node_id.clone(),
				x,
				y,
			},
		);
	}
	Ok(nodes)
}

fn build_edges(edges: &HashSet<(String, String)>) -> Vec<LayoutEdge> {
	let mut edges_out: Vec<LayoutEdge> = edges
		.iter()
		.map(|(a, b)| LayoutEdge {
			a: a.clone(),
			b: b.clone(),
		})
		.collect();
	edges_out.sort_by(|left, right| left.a.cmp(&right.a).then_with(|| left.b.cmp(&right.b)));
	edges_out
}
