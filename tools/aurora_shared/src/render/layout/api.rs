//! Public entry points for layout computation.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use crate::{Model, render::render_error::RenderError};

use super::graph::{LayoutGraph, build_graph, validate_graph};
use super::types::{Layout, LayoutEdge, LayoutFamily, LayoutNode};

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
	roots: Vec<String>,
	edges: Vec<(String, String)>,
	outgoing: HashMap<String, Vec<String>>,
	incoming: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct PreparedLayoutGraph {
	normalized: NormalizedGraph,
	topo: Vec<String>,
	spine: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct LayoutScore {
	score_milli: i64,
	crossings: usize,
	bends: usize,
}

/// Compute a deterministic spine-based layout for a view selection over a model.
pub fn layout_model(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<Layout, RenderError> {
	layout_model_with_family(
		model,
		root_card_types,
		included_card_types,
		LayoutFamily::VerticalTree,
	)
}

/// Compute a deterministic layout for a specific layout family.
pub fn layout_model_with_family(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
	family: LayoutFamily,
) -> Result<Layout, RenderError> {
	let prepared = prepare_layout_graph(model, root_card_types, included_card_types)?;
	layout_for_family(&prepared, family)
}

/// Compute deterministic layouts for all families and return the best-scoring one.
pub fn layout_model_best_family(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<Layout, RenderError> {
	let prepared = prepare_layout_graph(model, root_card_types, included_card_types)?;

	let mut best_layout: Option<Layout> = None;
	let mut best_score: Option<LayoutScore> = None;
	let mut best_family: Option<LayoutFamily> = None;

	for family in LayoutFamily::ordered() {
		let layout = layout_for_family(&prepared, *family)?;
		let score = score_layout(&prepared.normalized, &layout);

		let should_replace = match (best_score, best_family) {
			(None, _) => true,
			(Some(current_score), Some(current_family)) => {
				is_better_candidate(score, *family, current_score, current_family)
			}
			(Some(_), None) => true,
		};

		if should_replace {
			best_layout = Some(layout);
			best_score = Some(score);
			best_family = Some(*family);
		}
	}

	best_layout.ok_or(RenderError::BackboneOrderFailed)
}

fn prepare_layout_graph(
	model: &Model,
	root_card_types: &[String],
	included_card_types: &[String],
) -> Result<PreparedLayoutGraph, RenderError> {
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

	Ok(PreparedLayoutGraph {
		normalized,
		topo,
		spine,
	})
}

fn layout_for_family(
	prepared: &PreparedLayoutGraph,
	family: LayoutFamily,
) -> Result<Layout, RenderError> {
	let coords = assign_coords_for_family(prepared, family)?;

	build_layout(&prepared.normalized, &coords, family)
}

fn assign_coords_for_family(
	prepared: &PreparedLayoutGraph,
	family: LayoutFamily,
) -> Result<HashMap<String, (i32, i32)>, RenderError> {
	match family {
		LayoutFamily::VerticalTree => {
			assign_spine_and_branches(&prepared.normalized, &prepared.spine, &prepared.topo)
		}
		LayoutFamily::HorizontalTree => {
			let vertical =
				assign_spine_and_branches(&prepared.normalized, &prepared.spine, &prepared.topo)?;
			Ok(transpose_coords(vertical))
		}
		LayoutFamily::RadialSubtree => assign_radial_subtree_coords(&prepared.normalized),
	}
}

fn transpose_coords(coords: HashMap<String, (i32, i32)>) -> HashMap<String, (i32, i32)> {
	coords
		.into_iter()
		.map(|(id, (x, y))| (id, (y, x)))
		.collect()
}

fn build_layout(
	graph: &NormalizedGraph,
	coords: &HashMap<String, (i32, i32)>,
	family: LayoutFamily,
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

	Ok(Layout {
		family: Some(family),
		nodes,
		edges,
	})
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
		roots,
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

#[derive(Debug, Clone)]
struct NodeOwnership {
	root: String,
	depth: usize,
}

#[derive(Debug, Clone, Copy)]
struct ClusterBounds {
	min_x: i32,
	max_x: i32,
	min_y: i32,
	max_y: i32,
}

#[derive(Debug, Clone)]
struct RadialCluster {
	root: String,
	coords: HashMap<String, (i32, i32)>,
	bounds: ClusterBounds,
	packing_radius: f32,
}

fn assign_radial_subtree_coords(
	graph: &NormalizedGraph,
) -> Result<HashMap<String, (i32, i32)>, RenderError> {
	let roots = resolve_radial_roots(graph)?;
	let ownership = assign_node_ownership(graph, roots.as_slice());
	let clusters = build_radial_clusters(graph, roots.as_slice(), &ownership)?;
	let centers = pack_clusters_on_ring(clusters.as_slice());
	let mut coords = translate_clusters(clusters.as_slice(), &centers);
	place_missing_radial_nodes(graph, &mut coords);
	resolve_coordinate_collisions(&mut coords, graph.nodes.as_slice());
	Ok(coords)
}

fn radial_root_ring_radius(root_count: usize) -> f32 {
	if root_count <= 1 {
		return 0.0;
	}

	let min_radius_for_unique_spacing = (root_count as f32 / (2.0 * std::f32::consts::PI)).ceil();
	2.0f32.max(min_radius_for_unique_spacing)
}

fn resolve_radial_roots(graph: &NormalizedGraph) -> Result<Vec<String>, RenderError> {
	let mut roots = graph.roots.clone();
	roots.sort();
	roots.dedup();
	if roots.is_empty() {
		roots.extend(
			graph
				.nodes
				.iter()
				.filter(|node_id| graph.incoming.get(*node_id).map_or(0, Vec::len) == 0)
				.cloned(),
		);
		roots.sort();
		roots.dedup();
	}
	if roots.is_empty() {
		return Err(RenderError::MissingRoots);
	}
	Ok(roots)
}

fn build_radial_clusters(
	graph: &NormalizedGraph,
	roots: &[String],
	ownership: &HashMap<String, NodeOwnership>,
) -> Result<Vec<RadialCluster>, RenderError> {
	let mut clusters = Vec::with_capacity(roots.len());
	for root in roots {
		clusters.push(build_radial_cluster(graph, root.as_str(), ownership)?);
	}
	Ok(clusters)
}

fn build_radial_cluster(
	graph: &NormalizedGraph,
	root: &str,
	ownership: &HashMap<String, NodeOwnership>,
) -> Result<RadialCluster, RenderError> {
	let local_graph = owned_subgraph(graph, root, ownership);
	let topo = topo_sort_nodes(
		&local_graph.nodes,
		&local_graph.outgoing,
		&local_graph.incoming,
	)?;
	let spine = longest_spine_path(&topo, &local_graph.incoming);
	let coords = center_tree_horizontally(assign_spine_and_branches(&local_graph, &spine, &topo)?);
	let bounds = cluster_bounds(&coords, root)?;
	Ok(RadialCluster {
		root: root.to_string(),
		packing_radius: cluster_packing_radius(bounds),
		coords,
		bounds,
	})
}

fn owned_subgraph(
	graph: &NormalizedGraph,
	root: &str,
	ownership: &HashMap<String, NodeOwnership>,
) -> NormalizedGraph {
	let nodes: Vec<String> = graph
		.nodes
		.iter()
		.filter(|node_id| {
			ownership
				.get(*node_id)
				.is_some_and(|info| info.root.as_str() == root)
		})
		.cloned()
		.collect();
	let node_set: HashSet<&str> = nodes.iter().map(String::as_str).collect();
	let edges: Vec<(String, String)> = graph
		.edges
		.iter()
		.filter(|(a, b)| node_set.contains(a.as_str()) && node_set.contains(b.as_str()))
		.cloned()
		.collect();
	let (outgoing, incoming) = build_neighbor_maps(nodes.clone(), edges.as_slice());
	NormalizedGraph {
		nodes,
		roots: vec![root.to_string()],
		edges,
		outgoing,
		incoming,
	}
}

fn center_tree_horizontally(coords: HashMap<String, (i32, i32)>) -> HashMap<String, (i32, i32)> {
	let Some((min_x, max_x)) = coords
		.values()
		.map(|(x, _)| *x)
		.min()
		.zip(coords.values().map(|(x, _)| *x).max())
	else {
		return coords;
	};
	let shift_x = (min_x + max_x) / 2;
	coords
		.into_iter()
		.map(|(node_id, (x, y))| (node_id, (x - shift_x, y)))
		.collect()
}

fn cluster_bounds(
	coords: &HashMap<String, (i32, i32)>,
	root: &str,
) -> Result<ClusterBounds, RenderError> {
	let mut values = coords.values().copied();
	let Some((first_x, first_y)) = values.next() else {
		return Err(RenderError::MissingNode(root.to_string()));
	};
	let mut bounds = ClusterBounds {
		min_x: first_x,
		max_x: first_x,
		min_y: first_y,
		max_y: first_y,
	};
	for (x, y) in values {
		bounds.min_x = bounds.min_x.min(x);
		bounds.max_x = bounds.max_x.max(x);
		bounds.min_y = bounds.min_y.min(y);
		bounds.max_y = bounds.max_y.max(y);
	}
	Ok(bounds)
}

fn cluster_packing_radius(bounds: ClusterBounds) -> f32 {
	let half_width = (bounds.max_x - bounds.min_x).abs() as f32 / 2.0;
	let half_height = (bounds.max_y - bounds.min_y).abs() as f32 / 2.0;
	(half_width.mul_add(half_width, half_height * half_height)).sqrt() + 1.0
}

fn pack_clusters_on_ring(clusters: &[RadialCluster]) -> HashMap<String, (f32, f32)> {
	if clusters.len() <= 1 {
		return clusters
			.iter()
			.map(|cluster| (cluster.root.clone(), (0.0f32, 0.0f32)))
			.collect();
	}

	let gap = 1.5f32;
	let weights: Vec<f32> = clusters
		.iter()
		.map(|cluster| 2.0 * cluster.packing_radius + gap)
		.collect();
	let total_weight = weights.iter().sum::<f32>().max(1.0);
	let min_ring_radius = radial_root_ring_radius(clusters.len());
	let required_circumference = total_weight.max(min_ring_radius * 2.0 * std::f32::consts::PI);
	let max_cluster_radius = clusters
		.iter()
		.map(|cluster| cluster.packing_radius)
		.fold(0.0f32, f32::max);
	let ring_radius = (required_circumference / (2.0 * std::f32::consts::PI))
		.max(min_ring_radius)
		.max(max_cluster_radius);

	let mut centers = HashMap::with_capacity(clusters.len());
	let first_span = (2.0 * std::f32::consts::PI) * (weights[0] / total_weight);
	let mut cursor = -std::f32::consts::FRAC_PI_2 - (first_span / 2.0);
	for (cluster, weight) in clusters.iter().zip(weights) {
		let span = (2.0 * std::f32::consts::PI) * (weight / total_weight);
		let center_angle = cursor + (span / 2.0);
		centers.insert(
			cluster.root.clone(),
			(
				ring_radius * center_angle.cos(),
				ring_radius * center_angle.sin(),
			),
		);
		cursor += span;
	}
	centers
}

fn translate_clusters(
	clusters: &[RadialCluster],
	centers: &HashMap<String, (f32, f32)>,
) -> HashMap<String, (i32, i32)> {
	let mut coords = HashMap::new();
	for cluster in clusters {
		let (center_x, center_y) = centers.get(&cluster.root).copied().unwrap_or((0.0, 0.0));
		let local_center_x = (cluster.bounds.min_x + cluster.bounds.max_x) as f32 / 2.0;
		let local_center_y = (cluster.bounds.min_y + cluster.bounds.max_y) as f32 / 2.0;
		for (node_id, (x, y)) in &cluster.coords {
			coords.insert(
				node_id.clone(),
				(
					(center_x + (*x as f32 - local_center_x)).round() as i32,
					(center_y + (*y as f32 - local_center_y)).round() as i32,
				),
			);
		}
	}
	coords
}

fn place_missing_radial_nodes(graph: &NormalizedGraph, coords: &mut HashMap<String, (i32, i32)>) {
	let mut missing: Vec<String> = graph
		.nodes
		.iter()
		.filter(|node_id| !coords.contains_key(*node_id))
		.cloned()
		.collect();
	missing.sort();
	for (index, node_id) in missing.into_iter().enumerate() {
		let angle = (index as f32) * 0.5;
		let ring = (index / 12) as f32;
		let radius = 1.5 + (ring * 0.75);
		coords.insert(
			node_id,
			(
				(radius * angle.cos()).round() as i32,
				(radius * angle.sin()).round() as i32,
			),
		);
	}
}

fn assign_node_ownership(
	graph: &NormalizedGraph,
	roots: &[String],
) -> HashMap<String, NodeOwnership> {
	let mut ownership: HashMap<String, NodeOwnership> = HashMap::new();

	for root in roots {
		let mut queue: VecDeque<(String, usize)> = VecDeque::new();
		let mut visited: HashSet<String> = HashSet::new();
		queue.push_back((root.clone(), 0));

		while let Some((node_id, depth)) = queue.pop_front() {
			if !visited.insert(node_id.clone()) {
				continue;
			}

			let should_replace = match ownership.get(&node_id) {
				None => true,
				Some(current) => {
					depth < current.depth
						|| (depth == current.depth && root.as_str() < current.root.as_str())
				}
			};
			if should_replace {
				ownership.insert(
					node_id.clone(),
					NodeOwnership {
						root: root.clone(),
						depth,
					},
				);
			}

			if let Some(next_nodes) = graph.outgoing.get(&node_id) {
				for next in next_nodes {
					queue.push_back((next.clone(), depth + 1));
				}
			}
		}
	}

	for root in roots {
		ownership.entry(root.clone()).or_insert(NodeOwnership {
			root: root.clone(),
			depth: 0,
		});
	}

	ownership
}

fn resolve_coordinate_collisions(coords: &mut HashMap<String, (i32, i32)>, node_ids: &[String]) {
	let mut occupied: HashSet<(i32, i32)> = HashSet::new();

	for node_id in node_ids {
		let Some(origin) = coords.get(node_id).copied() else {
			continue;
		};
		if occupied.insert(origin) {
			continue;
		}

		let mut radius = 1i32;
		'find_slot: loop {
			for candidate in collision_ring(origin, radius) {
				if occupied.insert(candidate) {
					coords.insert(node_id.clone(), candidate);
					break 'find_slot;
				}
			}
			radius += 1;
		}
	}
}

fn collision_ring(origin: (i32, i32), radius: i32) -> [(i32, i32); 8] {
	let (x, y) = origin;
	[
		(x + radius, y),
		(x, y + radius),
		(x - radius, y),
		(x, y - radius),
		(x + radius, y + radius),
		(x - radius, y + radius),
		(x - radius, y - radius),
		(x + radius, y - radius),
	]
}

fn score_layout(graph: &NormalizedGraph, layout: &Layout) -> LayoutScore {
	let mut min_x = i32::MAX;
	let mut max_x = i32::MIN;
	let mut min_y = i32::MAX;
	let mut max_y = i32::MIN;
	for node in layout.nodes.values() {
		min_x = min_x.min(node.x);
		max_x = max_x.max(node.x);
		min_y = min_y.min(node.y);
		max_y = max_y.max(node.y);
	}

	let width = (max_x - min_x).max(1) as f64;
	let height = (max_y - min_y).max(1) as f64;
	let aspect_ratio = width / height;
	let aspect_error = ((aspect_ratio / 1.6f64).ln()).abs();

	let crossings = edge_crossings(graph, layout);
	let mut bends = 0usize;
	let mut total_length = 0usize;
	let mut long_span = 0usize;
	for (a, b) in &graph.edges {
		let Some(source) = layout.nodes.get(a) else {
			continue;
		};
		let Some(target) = layout.nodes.get(b) else {
			continue;
		};
		let dx = (target.x - source.x).unsigned_abs() as usize;
		let dy = (target.y - source.y).unsigned_abs() as usize;
		total_length += dx + dy;
		if dx > 0 && dy > 0 {
			bends += 2;
		} else if dx > 0 || dy > 0 {
			bends += 1;
		}
		let span = dx.max(dy);
		long_span += span.saturating_sub(2);
	}

	let score = 5.0f64 * aspect_error
		+ 10.0f64 * crossings as f64
		+ 2.0f64 * bends as f64
		+ 0.5f64 * total_length as f64
		+ 3.0f64 * long_span as f64;

	LayoutScore {
		score_milli: (score * 1000.0).round() as i64,
		crossings,
		bends,
	}
}

fn edge_crossings(graph: &NormalizedGraph, layout: &Layout) -> usize {
	let mut edges: Vec<((i32, i32), (i32, i32))> = Vec::new();
	for (a, b) in &graph.edges {
		let Some(source) = layout.nodes.get(a) else {
			continue;
		};
		let Some(target) = layout.nodes.get(b) else {
			continue;
		};
		edges.push(((source.x, source.y), (target.x, target.y)));
	}

	let mut crossings = 0usize;
	for left_index in 0..edges.len() {
		for right_index in left_index + 1..edges.len() {
			let (left_a, left_b) = edges[left_index];
			let (right_a, right_b) = edges[right_index];
			if shares_endpoint(left_a, left_b, right_a, right_b) {
				continue;
			}
			if segments_intersect(left_a, left_b, right_a, right_b) {
				crossings += 1;
			}
		}
	}

	crossings
}

fn shares_endpoint(a1: (i32, i32), a2: (i32, i32), b1: (i32, i32), b2: (i32, i32)) -> bool {
	a1 == b1 || a1 == b2 || a2 == b1 || a2 == b2
}

fn segments_intersect(a1: (i32, i32), a2: (i32, i32), b1: (i32, i32), b2: (i32, i32)) -> bool {
	let o1 = orientation(a1, a2, b1);
	let o2 = orientation(a1, a2, b2);
	let o3 = orientation(b1, b2, a1);
	let o4 = orientation(b1, b2, a2);

	if o1 == 0 || o2 == 0 || o3 == 0 || o4 == 0 {
		return false;
	}

	(o1 > 0 && o2 < 0 || o1 < 0 && o2 > 0) && (o3 > 0 && o4 < 0 || o3 < 0 && o4 > 0)
}

fn orientation(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i64 {
	let (ax, ay) = (a.0 as i64, a.1 as i64);
	let (bx, by) = (b.0 as i64, b.1 as i64);
	let (cx, cy) = (c.0 as i64, c.1 as i64);
	(by - ay) * (cx - bx) - (bx - ax) * (cy - by)
}

fn is_better_candidate(
	candidate_score: LayoutScore,
	candidate_family: LayoutFamily,
	current_score: LayoutScore,
	current_family: LayoutFamily,
) -> bool {
	if candidate_score.score_milli != current_score.score_milli {
		return candidate_score.score_milli < current_score.score_milli;
	}
	if candidate_score.crossings != current_score.crossings {
		return candidate_score.crossings < current_score.crossings;
	}
	if candidate_score.bends != current_score.bends {
		return candidate_score.bends < current_score.bends;
	}
	candidate_family < current_family
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
