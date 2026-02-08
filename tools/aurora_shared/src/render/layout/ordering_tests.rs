use std::collections::{HashMap, HashSet};

use crate::render::render_error::RenderError;

use super::ordering::{
	assign_ranks, build_backbone_adjacency, build_layers, layer_neighbors, median_sweeps, topo_sort,
};

#[test]
fn assign_ranks_prefers_longest_path() {
	let allowed_nodes = set_of(["A", "B", "C", "D"]);
	let edges = edge_set([("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]);
	let (outgoing, incoming) = build_backbone_adjacency(&allowed_nodes, &edges);
	let order = topo_sort(&allowed_nodes, &outgoing, &incoming).expect("topo order");
	let ranks = assign_ranks(&["A".to_string()], &order, &incoming).expect("ranks");

	assert_eq!(ranks.get("A"), Some(&0));
	assert_eq!(ranks.get("B"), Some(&1));
	assert_eq!(ranks.get("C"), Some(&1));
	assert_eq!(ranks.get("D"), Some(&2));
}

#[test]
fn topo_sort_rejects_cycles() {
	let allowed_nodes = set_of(["A", "B"]);
	let edges = edge_set([("A", "B"), ("B", "A")]);
	let (outgoing, incoming) = build_backbone_adjacency(&allowed_nodes, &edges);
	let result = topo_sort(&allowed_nodes, &outgoing, &incoming);

	assert!(matches!(result, Err(RenderError::BackboneOrderFailed)));
}

#[test]
fn median_sweeps_are_deterministic() {
	let allowed_nodes = set_of(["A", "B", "C", "D"]);
	let edges = edge_set([("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]);
	let (outgoing, incoming) = build_backbone_adjacency(&allowed_nodes, &edges);
	let order = topo_sort(&allowed_nodes, &outgoing, &incoming).expect("topo order");
	let ranks = assign_ranks(&["A".to_string()], &order, &incoming).expect("ranks");
	let incoming_count = incoming_count(&incoming);
	let (mut layers, max_layer_index) = build_layers(&ranks, &incoming_count).expect("layers");
	let (predecessor_by_layer, successor_by_layer) =
		layer_neighbors(&ranks, &edges).expect("neighbors");

	let mut second_layers = layers.clone();
	median_sweeps(
		&mut layers,
		max_layer_index,
		&predecessor_by_layer,
		&successor_by_layer,
	)
	.expect("sweeps");
	median_sweeps(
		&mut second_layers,
		max_layer_index,
		&predecessor_by_layer,
		&successor_by_layer,
	)
	.expect("sweeps");

	assert_eq!(layers, second_layers);
}

fn set_of<const N: usize>(values: [&str; N]) -> HashSet<String> {
	values.iter().map(|value| value.to_string()).collect()
}

fn edge_set<const N: usize>(values: [(&str, &str); N]) -> HashSet<(String, String)> {
	values
		.iter()
		.map(|(from, to)| (from.to_string(), to.to_string()))
		.collect()
}

fn incoming_count(incoming: &HashMap<String, Vec<String>>) -> HashMap<String, usize> {
	let mut counts: HashMap<String, usize> = HashMap::new();
	for (node_id, predecessors) in incoming {
		counts.insert(node_id.clone(), predecessors.len());
	}
	counts
}
