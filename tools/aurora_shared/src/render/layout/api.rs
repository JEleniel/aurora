//! Public entry points for layout computation.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{Model, render::render_error::RenderError};

use super::graph::{LayoutGraph, build_graph, validate_graph};
use super::types::{Layout, LayoutEdge, LayoutNode};

#[derive(Debug, Clone)]
struct CollapsedGraph {
	nodes: Vec<String>,
	roots: Vec<String>,
	edges: Vec<(String, String)>,
	scc_representatives: HashSet<String>,
}

#[derive(Debug, Clone)]
struct NormalizedGraph {
	nodes: Vec<String>,
	edges: Vec<(String, String)>,
	outgoing: HashMap<String, Vec<String>>,
	incoming: HashMap<String, Vec<String>>,
}

/// Compute a deterministic spine-based layout for a view selection over a model.
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

	let collapsed = collapse_strongly_connected_components(&graph)?;
	let normalized = remove_transit_nodes(collapsed)?;
	let topo = topo_sort_nodes(
		&normalized.nodes,
		&normalized.outgoing,
		&normalized.incoming,
	)?;
	let spine = longest_spine_path(&topo, &normalized.incoming);
	let coords = assign_spine_and_branches(&normalized, &spine, &topo)?;

	build_layout(&normalized, &coords)
}

fn build_layout(
	graph: &NormalizedGraph,
	coords: &HashMap<String, (i32, i32)>,
) -> Result<Layout, RenderError> {
	let mut nodes: HashMap<String, LayoutNode> = HashMap::new();
	for node_id in &graph.nodes {
		let (x, y) = coords
			.get(node_id)
			.copied()
			.ok_or_else(|| RenderError::MissingX(node_id.clone()))?;
		nodes.insert(
			node_id.clone(),
			LayoutNode {
				id: node_id.clone(),
				x,
				y,
			},
		);
	}

	let mut edges: Vec<LayoutEdge> = graph
		.edges
		.iter()
		.filter(|(a, b)| nodes.contains_key(a) && nodes.contains_key(b))
		.map(|(a, b)| LayoutEdge {
			a: a.clone(),
			b: b.clone(),
		})
		.collect();
	edges.sort_by(|left, right| left.a.cmp(&right.a).then_with(|| left.b.cmp(&right.b)));

	Ok(Layout { nodes, edges })
}

fn collapse_strongly_connected_components(
	graph: &LayoutGraph,
) -> Result<CollapsedGraph, RenderError> {
	let mut nodes: Vec<String> = graph.allowed_nodes.iter().cloned().collect();
	nodes.sort();

	let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
	for node in &nodes {
		adjacency.entry(node.clone()).or_default();
	}
	for (a, b) in &graph.edges {
		adjacency.entry(a.clone()).or_default().push(b.clone());
	}
	for neighbors in adjacency.values_mut() {
		neighbors.sort();
		neighbors.dedup();
	}

	let components = tarjan_scc(nodes.as_slice(), &adjacency);
	let mut representative_for: HashMap<String, String> = HashMap::new();
	let mut representatives: Vec<String> = Vec::new();
	let mut scc_representatives: HashSet<String> = HashSet::new();

	for component in components {
		let mut members = component;
		members.sort();
		let representative = members
			.first()
			.cloned()
			.ok_or(RenderError::BackboneOrderFailed)?;
		let has_self_loop = if members.len() == 1 {
			let node_id = members[0].as_str();
			graph
				.edges
				.contains(&(node_id.to_string(), node_id.to_string()))
		} else {
			false
		};
		if members.len() > 1 || has_self_loop {
			scc_representatives.insert(representative.clone());
		}
		for member in members {
			representative_for.insert(member, representative.clone());
		}
		representatives.push(representative);
	}

	representatives.sort();
	representatives.dedup();

	let mut edge_set: BTreeSet<(String, String)> = BTreeSet::new();
	for (a, b) in &graph.edges {
		let Some(rep_a) = representative_for.get(a) else {
			return Err(RenderError::MissingNode(a.clone()));
		};
		let Some(rep_b) = representative_for.get(b) else {
			return Err(RenderError::MissingNode(b.clone()));
		};
		if rep_a != rep_b {
			edge_set.insert((rep_a.clone(), rep_b.clone()));
		}
	}

	let mut roots: Vec<String> = graph
		.roots
		.iter()
		.filter_map(|root| representative_for.get(root).cloned())
		.collect();
	roots.sort();
	roots.dedup();

	Ok(CollapsedGraph {
		nodes: representatives,
		roots,
		edges: edge_set.into_iter().collect(),
		scc_representatives,
	})
}

fn remove_transit_nodes(graph: CollapsedGraph) -> Result<NormalizedGraph, RenderError> {
	let mut active_nodes: HashSet<String> = graph.nodes.iter().cloned().collect();
	let mut edges = graph.edges;
	let root_set: HashSet<String> = graph.roots.iter().cloned().collect();

	loop {
		let (outgoing, incoming) =
			build_neighbor_maps(active_nodes.iter().cloned().collect(), edges.as_slice());
		let mut candidates: Vec<String> = active_nodes
			.iter()
			.filter(|node_id| !root_set.contains(*node_id))
			.filter(|node_id| !graph.scc_representatives.contains(*node_id))
			.filter(|node_id| incoming.get(*node_id).map_or(0, Vec::len) == 1)
			.filter(|node_id| outgoing.get(*node_id).map_or(0, Vec::len) == 1)
			.cloned()
			.collect();
		candidates.sort();

		let Some(transit) = candidates.into_iter().next() else {
			break;
		};

		let predecessor = incoming
			.get(&transit)
			.and_then(|nodes| nodes.first())
			.cloned()
			.ok_or_else(|| RenderError::MissingNode(transit.clone()))?;
		let successor = outgoing
			.get(&transit)
			.and_then(|nodes| nodes.first())
			.cloned()
			.ok_or_else(|| RenderError::MissingNode(transit.clone()))?;

		edges.retain(|(a, b)| a != &transit && b != &transit);
		if predecessor != successor {
			edges.push((predecessor, successor));
		}
		active_nodes.remove(&transit);
	}

	let mut nodes: Vec<String> = active_nodes.into_iter().collect();
	nodes.sort();
	edges.retain(|(a, b)| nodes.binary_search(a).is_ok() && nodes.binary_search(b).is_ok());
	edges.sort();

	let (outgoing, incoming) = build_neighbor_maps(nodes.clone(), edges.as_slice());
	let mut roots: Vec<String> = graph
		.roots
		.into_iter()
		.filter(|node_id| nodes.binary_search(node_id).is_ok())
		.collect();
	roots.sort();
	roots.dedup();
	if roots.is_empty() {
		roots.extend(
			nodes
				.iter()
				.filter(|node_id| incoming.get(*node_id).map_or(0, Vec::len) == 0)
				.cloned(),
		);
		roots.sort();
		roots.dedup();
	}

	Ok(NormalizedGraph {
		nodes,
		edges,
		outgoing,
		incoming,
	})
}

fn build_neighbor_maps(
	nodes: Vec<String>,
	edges: &[(String, String)],
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
	let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
	let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
	for node_id in nodes {
		outgoing.entry(node_id.clone()).or_default();
		incoming.entry(node_id).or_default();
	}
	for (a, b) in edges {
		outgoing.entry(a.clone()).or_default().push(b.clone());
		incoming.entry(b.clone()).or_default().push(a.clone());
	}
	for values in outgoing.values_mut() {
		values.sort();
		values.dedup();
	}
	for values in incoming.values_mut() {
		values.sort();
		values.dedup();
	}
	(outgoing, incoming)
}

fn topo_sort_nodes(
	nodes: &[String],
	outgoing: &HashMap<String, Vec<String>>,
	incoming: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, RenderError> {
	let mut in_degree: HashMap<String, usize> = HashMap::new();
	for node_id in nodes {
		in_degree.insert(node_id.clone(), incoming.get(node_id).map_or(0, Vec::len));
	}

	let mut available: BTreeSet<String> = in_degree
		.iter()
		.filter_map(|(node_id, degree)| (*degree == 0).then_some(node_id.clone()))
		.collect();

	let mut order: Vec<String> = Vec::new();
	while let Some(next) = available.pop_first() {
		order.push(next.clone());
		if let Some(children) = outgoing.get(&next) {
			for child in children {
				if let Some(entry) = in_degree.get_mut(child) {
					*entry = entry.saturating_sub(1);
					if *entry == 0 {
						available.insert(child.clone());
					}
				}
			}
		}
	}

	if order.len() != nodes.len() {
		return Err(RenderError::BackboneOrderFailed);
	}

	Ok(order)
}

fn longest_spine_path(topo: &[String], incoming: &HashMap<String, Vec<String>>) -> Vec<String> {
	let mut best_path_to: HashMap<String, Vec<String>> = HashMap::new();

	for node_id in topo {
		let mut best: Vec<String> = vec![node_id.clone()];
		if let Some(predecessors) = incoming.get(node_id) {
			for predecessor in predecessors {
				if let Some(path_to_predecessor) = best_path_to.get(predecessor) {
					let mut candidate = path_to_predecessor.clone();
					candidate.push(node_id.clone());
					if path_is_better(candidate.as_slice(), best.as_slice()) {
						best = candidate;
					}
				}
			}
		}
		best_path_to.insert(node_id.clone(), best);
	}

	let mut spine: Vec<String> = Vec::new();
	for node_id in topo {
		if let Some(candidate) = best_path_to.get(node_id)
			&& path_is_better(candidate.as_slice(), spine.as_slice())
		{
			spine = candidate.clone();
		}
	}

	spine
}

fn path_is_better(candidate: &[String], current: &[String]) -> bool {
	candidate.len() > current.len() || (candidate.len() == current.len() && candidate < current)
}

fn assign_spine_and_branches(
	graph: &NormalizedGraph,
	spine: &[String],
	topo: &[String],
) -> Result<HashMap<String, (i32, i32)>, RenderError> {
	let mut coords: HashMap<String, (i32, i32)> = HashMap::new();
	let spine_index: HashMap<String, usize> = spine
		.iter()
		.enumerate()
		.map(|(index, node_id)| (node_id.clone(), index))
		.collect();

	for (index, node_id) in spine.iter().enumerate() {
		coords.insert(node_id.clone(), (0, index as i32));
	}

	let mut anchor_index: HashMap<String, usize> = HashMap::new();
	for node_id in topo {
		if let Some(index) = spine_index.get(node_id).copied() {
			anchor_index.insert(node_id.clone(), index);
			continue;
		}

		let mut best_anchor = 0usize;
		if let Some(predecessors) = graph.incoming.get(node_id) {
			for predecessor in predecessors {
				if let Some(index) = spine_index.get(predecessor).copied() {
					best_anchor = best_anchor.max(index);
				}
				if let Some(index) = anchor_index.get(predecessor).copied() {
					best_anchor = best_anchor.max(index);
				}
			}
		}
		anchor_index.insert(node_id.clone(), best_anchor);
	}

	let non_spine: Vec<String> = topo
		.iter()
		.filter(|node_id| !spine_index.contains_key(*node_id))
		.cloned()
		.collect();
	let width_bound = estimate_width_bound(non_spine.len());

	let mut row_load: HashMap<i32, usize> = HashMap::new();
	let mut nodes_by_row: BTreeMap<i32, Vec<String>> = BTreeMap::new();
	for node_id in &non_spine {
		let anchor_row = anchor_index
			.get(node_id)
			.copied()
			.unwrap_or(0)
			.try_into()
			.unwrap_or(0i32);
		let mut min_row = anchor_row + 1;

		if let Some(predecessors) = graph.incoming.get(node_id) {
			for predecessor in predecessors {
				if let Some((_, predecessor_row)) = coords.get(predecessor).copied() {
					min_row = min_row.max(predecessor_row + 1);
				}
			}
		}

		let mut row = min_row;
		while row_load.get(&row).copied().unwrap_or(0) >= width_bound {
			row += 1;
		}

		*row_load.entry(row).or_insert(0) += 1;
		nodes_by_row.entry(row).or_default().push(node_id.clone());
	}

	for (row, nodes) in &mut nodes_by_row {
		nodes.sort_by(|left, right| {
			let left_anchor = anchor_index.get(left).copied().unwrap_or(0);
			let right_anchor = anchor_index.get(right).copied().unwrap_or(0);
			left_anchor
				.cmp(&right_anchor)
				.then_with(|| {
					let left_bary =
						predecessor_barycenter(left, &coords, &graph.incoming).unwrap_or(0.0);
					let right_bary =
						predecessor_barycenter(right, &coords, &graph.incoming).unwrap_or(0.0);
					left_bary
						.partial_cmp(&right_bary)
						.unwrap_or(Ordering::Equal)
				})
				.then_with(|| left.cmp(right))
		});

		for (index, node_id) in nodes.iter().enumerate() {
			let column = 1 + index as i32;
			coords.insert(node_id.clone(), (column, *row));
		}
	}

	for node_id in &graph.nodes {
		if !coords.contains_key(node_id) {
			let row = anchor_index
				.get(node_id)
				.copied()
				.unwrap_or(0)
				.try_into()
				.unwrap_or(0i32);
			coords.insert(node_id.clone(), (0, row));
		}
	}

	Ok(coords)
}

fn predecessor_barycenter(
	node_id: &str,
	coords: &HashMap<String, (i32, i32)>,
	incoming: &HashMap<String, Vec<String>>,
) -> Option<f32> {
	let predecessors = incoming.get(node_id)?;
	if predecessors.is_empty() {
		return None;
	}

	let mut sum = 0.0f32;
	let mut count = 0usize;
	for predecessor in predecessors {
		if let Some((column, _)) = coords.get(predecessor).copied() {
			sum += column as f32;
			count += 1;
		}
	}

	(count > 0).then_some(sum / count as f32)
}

fn estimate_width_bound(non_spine_node_count: usize) -> usize {
	if non_spine_node_count == 0 {
		return 1;
	}
	((0.75f64 * non_spine_node_count as f64).sqrt().ceil() as usize).max(1)
}

fn tarjan_scc(nodes: &[String], adjacency: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
	#[derive(Default)]
	struct Tarjan {
		index: usize,
		stack: Vec<String>,
		on_stack: HashSet<String>,
		indices: HashMap<String, usize>,
		lowlink: HashMap<String, usize>,
		components: Vec<Vec<String>>,
	}

	fn strong_connect(
		node_id: &str,
		adjacency: &HashMap<String, Vec<String>>,
		tarjan: &mut Tarjan,
	) {
		tarjan.indices.insert(node_id.to_string(), tarjan.index);
		tarjan.lowlink.insert(node_id.to_string(), tarjan.index);
		tarjan.index += 1;
		tarjan.stack.push(node_id.to_string());
		tarjan.on_stack.insert(node_id.to_string());

		if let Some(neighbors) = adjacency.get(node_id) {
			for neighbor in neighbors {
				if !tarjan.indices.contains_key(neighbor) {
					strong_connect(neighbor.as_str(), adjacency, tarjan);
					let node_lowlink = tarjan.lowlink.get(node_id).copied().unwrap_or(0);
					let neighbor_lowlink = tarjan.lowlink.get(neighbor).copied().unwrap_or(0);
					tarjan
						.lowlink
						.insert(node_id.to_string(), node_lowlink.min(neighbor_lowlink));
				} else if tarjan.on_stack.contains(neighbor) {
					let node_lowlink = tarjan.lowlink.get(node_id).copied().unwrap_or(0);
					let neighbor_index = tarjan.indices.get(neighbor).copied().unwrap_or(0);
					tarjan
						.lowlink
						.insert(node_id.to_string(), node_lowlink.min(neighbor_index));
				}
			}
		}

		let node_index = tarjan.indices.get(node_id).copied().unwrap_or(0);
		let node_lowlink = tarjan.lowlink.get(node_id).copied().unwrap_or(0);
		if node_index == node_lowlink {
			let mut component: Vec<String> = Vec::new();
			while let Some(member) = tarjan.stack.pop() {
				tarjan.on_stack.remove(member.as_str());
				component.push(member.clone());
				if member == node_id {
					break;
				}
			}
			tarjan.components.push(component);
		}
	}

	let mut tarjan = Tarjan::default();
	for node_id in nodes {
		if !tarjan.indices.contains_key(node_id) {
			strong_connect(node_id.as_str(), adjacency, &mut tarjan);
		}
	}

	tarjan.components.sort_by(|left, right| {
		let mut left_sorted = left.clone();
		let mut right_sorted = right.clone();
		left_sorted.sort();
		right_sorted.sort();
		left_sorted
			.first()
			.cmp(&right_sorted.first())
			.then_with(|| left_sorted.cmp(&right_sorted))
	});

	tarjan.components
}
