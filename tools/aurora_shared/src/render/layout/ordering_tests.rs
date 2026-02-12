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

#[test]
fn median_sweeps_reduces_simple_crossing() {
	let mut layers = vec![
		vec!["A".to_string(), "B".to_string()],
		vec!["C".to_string(), "D".to_string()],
	];
	let predecessor_by_layer: HashMap<String, Vec<String>> = HashMap::from([
		("C".to_string(), vec!["B".to_string()]),
		("D".to_string(), vec!["A".to_string()]),
	]);
	let successor_by_layer: HashMap<String, Vec<String>> = HashMap::from([
		("A".to_string(), vec!["D".to_string()]),
		("B".to_string(), vec!["C".to_string()]),
	]);

	median_sweeps(&mut layers, 1, &predecessor_by_layer, &successor_by_layer).expect("sweeps");

	let crossings = simple_two_layer_crossings(&layers[0], &layers[1], &successor_by_layer);
	assert_eq!(crossings, 0, "layers not uncrossed: {:?}", layers);
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

fn simple_two_layer_crossings(
	upper: &[String],
	lower: &[String],
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> usize {
	let mut upper_index: HashMap<String, usize> = HashMap::new();
	for (index, node) in upper.iter().enumerate() {
		upper_index.insert(node.clone(), index);
	}
	let mut lower_index: HashMap<String, usize> = HashMap::new();
	for (index, node) in lower.iter().enumerate() {
		lower_index.insert(node.clone(), index);
	}

	let mut edges: Vec<(usize, usize)> = Vec::new();
	for source in upper {
		let source_pos = upper_index[source];
		if let Some(targets) = successor_by_layer.get(source) {
			for target in targets {
				if let Some(target_pos) = lower_index.get(target).copied() {
					edges.push((source_pos, target_pos));
				}
			}
		}
	}

	let mut crossings = 0usize;
	for i in 0..edges.len() {
		for j in i + 1..edges.len() {
			let (a0, b0) = edges[i];
			let (a1, b1) = edges[j];
			if (a0 < a1 && b0 > b1) || (a0 > a1 && b0 < b1) {
				crossings += 1;
			}
		}
	}

	crossings
}
