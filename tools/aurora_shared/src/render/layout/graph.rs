//! Graph preparation helpers for layout.

use std::collections::{HashMap, HashSet};

use crate::{Card, Model, render::render_error::RenderError};

/// Graph data required by the layout algorithm.
pub(super) struct LayoutGraph {
	pub(super) allowed_nodes: HashSet<String>,
	pub(super) roots: Vec<String>,
	pub(super) edges: HashSet<(String, String)>,
	pub(super) outgoing: HashMap<String, Vec<String>>,
	pub(super) incoming_count: HashMap<String, usize>,
}

/// Backbone and loop edges discovered during traversal.
pub(super) struct EdgeClassification {
	pub(super) backbone: HashSet<(String, String)>,
	pub(super) loops: HashSet<(String, String)>,
}

/// Build a filtered layout graph from a model and view configuration.
pub(super) fn build_graph(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<LayoutGraph, RenderError> {
	let (root_types, allowed_types) = collect_allowed_types(root_card_types, included_card_types);
	let cards_by_id = index_cards(model)?;
	let (allowed_nodes, mut roots) = select_nodes(model, &root_types, &allowed_types);
	if roots.is_empty() {
		return Err(RenderError::MissingRoots);
	}
	roots.sort();

	let edges = collect_edges(&allowed_nodes, &cards_by_id)?;
	let (outgoing, incoming) = build_adjacency(&allowed_nodes, &edges);
	let incoming_count = compute_incoming_count(&allowed_nodes, &incoming);

	Ok(LayoutGraph {
		allowed_nodes,
		roots,
		edges,
		outgoing,
		incoming_count,
	})
}

/// Validate that layout graph invariants are satisfied.
pub(super) fn validate_graph(graph: &LayoutGraph) -> Result<(), RenderError> {
	let root_set: HashSet<&str> = graph.roots.iter().map(|id| id.as_str()).collect();
	for root in &graph.roots {
		if graph.incoming_count.get(root).copied().unwrap_or(0) != 0 {
			return Err(RenderError::RootHasIncoming(root.clone()));
		}
	}
	for node_id in &graph.allowed_nodes {
		if root_set.contains(node_id.as_str()) {
			continue;
		}
		if graph.incoming_count.get(node_id).copied().unwrap_or(0) == 0 {
			return Err(RenderError::NodeHasNoIncoming(node_id.clone()));
		}
	}

	let reachable = collect_reachable(&graph.roots, &graph.outgoing);
	let mut unreachable: Vec<String> = graph
		.allowed_nodes
		.iter()
		.filter(|node_id| !reachable.contains(*node_id))
		.cloned()
		.collect();
	if !unreachable.is_empty() {
		unreachable.sort();
		return Err(RenderError::UnreachableNodes(unreachable));
	}

	Ok(())
}

/// Classify edges into backbone and loop sets.
pub(super) fn classify_edges(graph: &LayoutGraph) -> EdgeClassification {
	let mut backbone_edges: HashSet<(String, String)> = HashSet::new();
	let mut loop_edges: HashSet<(String, String)> = HashSet::new();
	let mut visited: HashSet<String> = graph.roots.iter().cloned().collect();
	let mut stack = graph.roots.clone();
	stack.reverse();
	while let Some(node_id) = stack.pop() {
		if let Some(neighbors) = graph.outgoing.get(&node_id) {
			for neighbor in neighbors {
				if !visited.contains(neighbor) {
					backbone_edges.insert((node_id.clone(), neighbor.clone()));
					visited.insert(neighbor.clone());
					stack.push(neighbor.clone());
				} else {
					loop_edges.insert((node_id.clone(), neighbor.clone()));
				}
			}
		}
	}

	EdgeClassification {
		backbone: backbone_edges,
		loops: loop_edges,
	}
}

fn collect_allowed_types<'a>(
	root_card_types: &'a [String],
	included_card_types: &'a [String],
) -> (HashSet<&'a str>, HashSet<&'a str>) {
	let mut root_types: HashSet<&str> = HashSet::new();
	let mut allowed_types: HashSet<&str> = HashSet::new();
	for card_type in root_card_types {
		root_types.insert(card_type.as_str());
		allowed_types.insert(card_type.as_str());
	}
	for card_type in included_card_types {
		allowed_types.insert(card_type.as_str());
	}
	(root_types, allowed_types)
}

fn index_cards(model: &Model) -> Result<HashMap<String, &Card>, RenderError> {
	let mut cards_by_id: HashMap<String, &Card> = HashMap::new();
	for card in iter_cards(model) {
		if cards_by_id.insert(card.id.clone(), card).is_some() {
			return Err(RenderError::DuplicateNodeId(card.id.clone()));
		}
	}
	Ok(cards_by_id)
}

fn select_nodes(
	model: &Model,
	root_types: &HashSet<&str>,
	allowed_types: &HashSet<&str>,
) -> (HashSet<String>, Vec<String>) {
	let mut allowed_nodes: HashSet<String> = HashSet::new();
	let mut roots: Vec<String> = Vec::new();
	for card in iter_cards(model) {
		if !allowed_types.contains(card.card_type.as_str()) {
			continue;
		}
		allowed_nodes.insert(card.id.clone());
		if root_types.contains(card.card_type.as_str()) {
			roots.push(card.id.clone());
		}
	}
	(allowed_nodes, roots)
}

fn collect_edges(
	allowed_nodes: &HashSet<String>,
	cards_by_id: &HashMap<String, &Card>,
) -> Result<HashSet<(String, String)>, RenderError> {
	let mut edges: HashSet<(String, String)> = HashSet::new();
	for node_id in allowed_nodes {
		let card = cards_by_id
			.get(node_id)
			.ok_or_else(|| RenderError::MissingNode(node_id.clone()))?;
		for link in &card.links {
			if !cards_by_id.contains_key(&link.target) {
				return Err(RenderError::UnknownTarget(
					node_id.clone(),
					link.target.clone(),
				));
			}
			if allowed_nodes.contains(link.target.as_str()) {
				edges.insert((node_id.clone(), link.target.clone()));
			}
		}
	}
	Ok(edges)
}

fn build_adjacency(
	allowed_nodes: &HashSet<String>,
	edges: &HashSet<(String, String)>,
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
	let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
	let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
	for node_id in allowed_nodes {
		outgoing.entry(node_id.clone()).or_default();
		incoming.entry(node_id.clone()).or_default();
	}
	for (a, b) in edges {
		outgoing.entry(a.clone()).or_default().push(b.clone());
		incoming.entry(b.clone()).or_default().push(a.clone());
	}
	normalize_adjacency(&mut outgoing);
	normalize_adjacency(&mut incoming);
	(outgoing, incoming)
}

fn normalize_adjacency(adjacency: &mut HashMap<String, Vec<String>>) {
	for neighbors in adjacency.values_mut() {
		neighbors.sort();
		neighbors.dedup();
	}
}

fn compute_incoming_count(
	allowed_nodes: &HashSet<String>,
	incoming: &HashMap<String, Vec<String>>,
) -> HashMap<String, usize> {
	let mut incoming_count: HashMap<String, usize> = HashMap::new();
	for node_id in allowed_nodes {
		let count = incoming.get(node_id).map(|list| list.len()).unwrap_or(0);
		incoming_count.insert(node_id.clone(), count);
	}
	incoming_count
}

fn collect_reachable(roots: &[String], outgoing: &HashMap<String, Vec<String>>) -> HashSet<String> {
	let mut visited: HashSet<String> = HashSet::new();
	let mut stack: Vec<String> = roots.to_vec();
	stack.sort();
	stack.reverse();
	while let Some(node_id) = stack.pop() {
		if !visited.insert(node_id.clone()) {
			continue;
		}
		if let Some(neighbors) = outgoing.get(&node_id) {
			for neighbor in neighbors.iter().rev() {
				stack.push(neighbor.clone());
			}
		}
	}
	visited
}

fn iter_cards<'a>(model: &'a Model) -> impl Iterator<Item = &'a Card> {
	std::iter::once(&model.root_card).chain(model.cards.iter())
}
