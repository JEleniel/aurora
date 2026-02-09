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
	let (mut allowed_nodes, mut roots) = select_nodes(model, &root_types, &allowed_types);
	if roots.is_empty() {
		return Err(RenderError::MissingRoots);
	}
	roots.sort();
	let declared_roots: HashSet<String> = roots.iter().cloned().collect();

	let mut edges = collect_edges(&allowed_nodes, &cards_by_id)?;
	let (mut outgoing, incoming) = build_adjacency(&allowed_nodes, &edges);
	let mut incoming_count = compute_incoming_count(&allowed_nodes, &incoming);

	// Expand roots to keep filtered subgraphs layout-able (nodes can lose all incoming edges
	// when their predecessors are filtered out).
	expand_roots_for_layout(&mut roots, &allowed_nodes, &incoming_count);

	// Drop disconnected components entirely: they show up as unconnected cards and can
	// dramatically widen horizontal spacing.
	let reachable = collect_reachable(&roots, &outgoing);
	allowed_nodes.retain(|node_id| reachable.contains(node_id));
	roots.retain(|node_id| allowed_nodes.contains(node_id));
	edges.retain(|(a, b)| allowed_nodes.contains(a) && allowed_nodes.contains(b));

	// Also drop fully isolated nodes (no incident edges) unless they are explicitly declared
	// roots by type.
	let incident = collect_incident_nodes(&edges);
	allowed_nodes.retain(|node_id| declared_roots.contains(node_id) || incident.contains(node_id));
	roots.retain(|node_id| allowed_nodes.contains(node_id));
	edges.retain(|(a, b)| allowed_nodes.contains(a) && allowed_nodes.contains(b));

	// Rebuild adjacency and incoming counts after pruning.
	let (rebuilt_outgoing, rebuilt_incoming) = build_adjacency(&allowed_nodes, &edges);
	outgoing = rebuilt_outgoing;
	incoming_count = compute_incoming_count(&allowed_nodes, &rebuilt_incoming);

	// Re-expand roots using the pruned graph's incoming counts.
	expand_roots_for_layout(&mut roots, &allowed_nodes, &incoming_count);
	roots.sort();
	roots.dedup();

	Ok(LayoutGraph {
		allowed_nodes,
		roots,
		edges,
		outgoing,
		incoming_count,
	})
}

fn collect_incident_nodes(edges: &HashSet<(String, String)>) -> HashSet<String> {
	let mut out: HashSet<String> = HashSet::new();
	for (a, b) in edges {
		out.insert(a.clone());
		out.insert(b.clone());
	}
	out
}

/// Validate that layout graph invariants are satisfied.
pub(super) fn validate_graph(graph: &LayoutGraph) -> Result<(), RenderError> {
	let root_set: HashSet<&str> = graph.roots.iter().map(|id| id.as_str()).collect();
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

fn expand_roots_for_layout(
	roots: &mut Vec<String>,
	allowed_nodes: &HashSet<String>,
	incoming_count: &HashMap<String, usize>,
) {
	let mut root_set: HashSet<String> = roots.iter().cloned().collect();

	// In a filtered view subgraph, some nodes can lose all incoming edges because their
	// predecessors were excluded. Treat those nodes as additional roots so they can still
	// be ranked and positioned.
	for node_id in allowed_nodes {
		if incoming_count.get(node_id).copied().unwrap_or(0) == 0 {
			root_set.insert(node_id.clone());
		}
	}

	let mut expanded_roots: Vec<String> = root_set.into_iter().collect();
	expanded_roots.sort();
	*roots = expanded_roots;
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

fn iter_cards(model: &Model) -> impl Iterator<Item = &Card> + '_ {
	std::iter::once(&model.root_card).chain(model.cards.iter())
}
