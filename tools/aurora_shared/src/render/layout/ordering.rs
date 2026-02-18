//! Layering, ordering, and coordinate assignment helpers.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::render::render_error::RenderError;

type NeighborMap = HashMap<String, Vec<String>>;

/// Build adjacency lists for the backbone edges.
pub(super) fn build_backbone_adjacency(
	allowed_nodes: &HashSet<String>,
	backbone_edges: &HashSet<(String, String)>,
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
	build_adjacency(allowed_nodes, backbone_edges)
}

/// Return nodes in deterministic topological order.
pub(super) fn topo_sort(
	allowed_nodes: &HashSet<String>,
	outgoing: &HashMap<String, Vec<String>>,
	incoming: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, RenderError> {
	let incoming_degree = initialize_in_degree(allowed_nodes, incoming);
	let mut available = collect_zero_in_degree(&incoming_degree);
	let mut order: Vec<String> = Vec::new();
	let mut remaining = incoming_degree;

	while let Some(next) = available.iter().next().cloned() {
		available.remove(&next);
		order.push(next.clone());
		if let Some(children) = outgoing.get(&next) {
			update_in_degree(children, &mut remaining, &mut available);
		}
	}

	if order.len() != allowed_nodes.len() {
		return Err(RenderError::BackboneOrderFailed);
	}

	Ok(order)
}

/// Assign longest-path ranks from the roots.
pub(super) fn assign_ranks(
	roots: &[String],
	order: &[String],
	incoming: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, i32>, RenderError> {
	let mut rank: HashMap<String, i32> = HashMap::new();
	for root in roots {
		rank.insert(root.clone(), 0);
	}
	for node_id in order {
		if rank.contains_key(node_id) {
			continue;
		}
		let next_rank = max_parent_rank(node_id, incoming, &rank)?;
		rank.insert(node_id.clone(), next_rank);
	}
	Ok(rank)
}

/// Group nodes by rank and apply initial ordering.
pub(super) fn build_layers(
	rank: &HashMap<String, i32>,
	incoming_count: &HashMap<String, usize>,
) -> Result<(Vec<Vec<String>>, usize), RenderError> {
	let max_rank = max_rank(rank);
	let mut layers = group_by_rank(rank, max_rank as usize);
	for layer in &mut layers {
		*layer = incoming_heavy_centered(layer, incoming_count);
	}
	Ok((layers, max_rank as usize))
}

/// Collect predecessor and successor neighbors in adjacent layers.
pub(super) fn layer_neighbors(
	rank: &HashMap<String, i32>,
	backbone_edges: &HashSet<(String, String)>,
) -> Result<(NeighborMap, NeighborMap), RenderError> {
	let mut predecessor_by_layer: NeighborMap = HashMap::new();
	let mut successor_by_layer: NeighborMap = HashMap::new();
	for (a, b) in backbone_edges {
		let rank_a = rank
			.get(a)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(a.clone()))?;
		let rank_b = rank
			.get(b)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(b.clone()))?;
		if rank_b == rank_a + 1 {
			predecessor_by_layer
				.entry(b.clone())
				.or_default()
				.push(a.clone());
			successor_by_layer
				.entry(a.clone())
				.or_default()
				.push(b.clone());
		}
	}
	normalize_neighbors(&mut predecessor_by_layer);
	normalize_neighbors(&mut successor_by_layer);
	Ok((predecessor_by_layer, successor_by_layer))
}

/// Reduce crossings with alternating median sweeps.
pub(super) fn median_sweeps(
	layers: &mut [Vec<String>],
	max_layer_index: usize,
	predecessor_by_layer: &HashMap<String, Vec<String>>,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	let mut best_layers = layers.to_vec();
	let mut best_score =
		total_crossings(best_layers.as_slice(), max_layer_index, successor_by_layer)?;

	let mut seeds: Vec<Vec<Vec<String>>> = vec![layers.to_vec()];
	seeds.push(mirrored_layers(layers));

	for mut candidate_layers in seeds {
		optimize_crossings(
			candidate_layers.as_mut_slice(),
			max_layer_index,
			predecessor_by_layer,
			successor_by_layer,
		)?;
		let score = total_crossings(
			candidate_layers.as_slice(),
			max_layer_index,
			successor_by_layer,
		)?;
		if score < best_score {
			best_score = score;
			best_layers = candidate_layers;
		}
	}

	for (index, layer) in best_layers.into_iter().enumerate() {
		if let Some(target) = layers.get_mut(index) {
			*target = layer;
		}
	}
	Ok(())
}

fn optimize_crossings(
	layers: &mut [Vec<String>],
	max_layer_index: usize,
	predecessor_by_layer: &HashMap<String, Vec<String>>,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	let mut positions = rebuild_positions(layers);
	for _ in 0..12 {
		sweep_down(
			layers,
			max_layer_index,
			predecessor_by_layer,
			&mut positions,
		)?;
		sweep_up(layers, max_layer_index, successor_by_layer, &mut positions)?;
	}
	transpose_sweeps(layers, max_layer_index, successor_by_layer)?;
	Ok(())
}

fn mirrored_layers(layers: &[Vec<String>]) -> Vec<Vec<String>> {
	let mut mirrored = layers.to_vec();
	for layer in mirrored.iter_mut() {
		if layer.len() > 1 {
			layer.reverse();
		}
	}
	mirrored
}

fn transpose_sweeps(
	layers: &mut [Vec<String>],
	max_layer_index: usize,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	if layers.is_empty() {
		return Ok(());
	}

	for _ in 0..6 {
		let mut improved = false;
		for layer_index in 0..=max_layer_index {
			if layers[layer_index].len() < 2 {
				continue;
			}

			let mut index = 0usize;
			while index + 1 < layers[layer_index].len() {
				let before = total_crossings(layers, max_layer_index, successor_by_layer)?;

				layers[layer_index].swap(index, index + 1);
				let after = total_crossings(layers, max_layer_index, successor_by_layer)?;

				if after < before {
					improved = true;
				} else {
					layers[layer_index].swap(index, index + 1);
				}

				index += 1;
			}
		}

		if !improved {
			break;
		}
	}

	Ok(())
}

fn total_crossings(
	layers: &[Vec<String>],
	max_layer_index: usize,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<usize, RenderError> {
	let mut total = 0usize;
	if max_layer_index == 0 {
		return Ok(0);
	}
	for layer_index in 0..max_layer_index {
		total += crossings_from_successors(
			&layers[layer_index],
			&layers[layer_index + 1],
			successor_by_layer,
		)?;
	}
	Ok(total)
}

fn crossings_from_successors(
	upper_layer: &[String],
	lower_layer: &[String],
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<usize, RenderError> {
	let upper_pos = position_index(upper_layer);
	let lower_pos = position_index(lower_layer);
	let mut edges: Vec<(usize, usize)> = Vec::new();

	for source in upper_layer {
		let source_pos = upper_pos
			.get(source)
			.copied()
			.ok_or_else(|| RenderError::MissingPosition(source.clone()))?;
		if let Some(successors) = successor_by_layer.get(source) {
			for target in successors {
				if let Some(target_pos) = lower_pos.get(target).copied() {
					edges.push((source_pos, target_pos));
				}
			}
		}
	}

	Ok(count_crossings(edges.as_slice()))
}

fn position_index(layer: &[String]) -> HashMap<String, usize> {
	let mut out = HashMap::new();
	for (index, node_id) in layer.iter().enumerate() {
		out.insert(node_id.clone(), index);
	}
	out
}

fn count_crossings(edges: &[(usize, usize)]) -> usize {
	let mut crossings = 0usize;
	for i in 0..edges.len() {
		for j in i + 1..edges.len() {
			let (a0, b0) = edges[i];
			let (a1, b1) = edges[j];
			let source_inverted = (a0 < a1 && b0 > b1) || (a0 > a1 && b0 < b1);
			if source_inverted {
				crossings += 1;
			}
		}
	}
	crossings
}

/// Assign dense integer x positions for each layer.
pub(super) fn assign_x_positions(layers: &[Vec<String>]) -> HashMap<String, i32> {
	let mut x_positions: HashMap<String, i32> = HashMap::new();
	for layer in layers {
		for (index, node_id) in layer.iter().enumerate() {
			// Use half-step coordinates (2 units per column). This allows a node to be
			// centered between two siblings (e.g. median of [0, 2] becomes 1).
			x_positions.insert(node_id.clone(), (index as i32) * 2);
		}
	}
	x_positions
}

/// Pack a layer tightly (minimum spacing) while keeping its overall horizontal placement stable.
///
/// The returned positions are the dense half-step grid ($0, 2, 4, \dots$) plus a single integer
/// offset chosen to keep the packed layer close to its pre-compaction positions.
pub(super) fn compact_layer_tight(layer: &[String], x_positions: &mut HashMap<String, i32>) {
	if layer.is_empty() {
		return;
	}

	let mut deltas: Vec<i32> = Vec::with_capacity(layer.len());
	for (index, node_id) in layer.iter().enumerate() {
		let current = x_positions.get(node_id).copied().unwrap_or(0);
		let packed = (index as i32) * 2;
		deltas.push(current - packed);
	}
	let offset = median_floor_i32(&mut deltas);

	for (index, node_id) in layer.iter().enumerate() {
		let packed = (index as i32) * 2;
		x_positions.insert(node_id.clone(), packed + offset);
	}
}

/// Center nodes over their children while preserving order.
pub(super) fn center_parents(
	x_positions: &mut HashMap<String, i32>,
	layers: &[Vec<String>],
	max_layer_index: usize,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	if max_layer_index == 0 {
		return Ok(());
	}
	for y in (0..max_layer_index).rev() {
		let layer = layers.get(y).ok_or(RenderError::MissingLayer(y))?;
		center_layer(layer, x_positions, successor_by_layer)?;
	}
	Ok(())
}

/// Center nodes under their parents while preserving layer order.
pub(super) fn center_children(
	x_positions: &mut HashMap<String, i32>,
	layers: &[Vec<String>],
	max_layer_index: usize,
	predecessor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	if max_layer_index == 0 {
		return Ok(());
	}
	for y in 1..=max_layer_index {
		let layer = layers.get(y).ok_or(RenderError::MissingLayer(y))?;
		center_layer(layer, x_positions, predecessor_by_layer)?;
	}
	Ok(())
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
	normalize_neighbors(&mut outgoing);
	normalize_neighbors(&mut incoming);
	(outgoing, incoming)
}

fn initialize_in_degree(
	allowed_nodes: &HashSet<String>,
	incoming: &HashMap<String, Vec<String>>,
) -> HashMap<String, usize> {
	let mut incoming_degree: HashMap<String, usize> = HashMap::new();
	for node_id in allowed_nodes {
		incoming_degree.insert(node_id.clone(), 0);
	}
	for (node_id, predecessors) in incoming {
		if let Some(entry) = incoming_degree.get_mut(node_id) {
			*entry = predecessors.len();
		}
	}
	incoming_degree
}

fn collect_zero_in_degree(incoming_degree: &HashMap<String, usize>) -> BTreeSet<String> {
	let mut available: BTreeSet<String> = BTreeSet::new();
	for (node_id, count) in incoming_degree {
		if *count == 0 {
			available.insert(node_id.clone());
		}
	}
	available
}

fn update_in_degree(
	children: &[String],
	remaining: &mut HashMap<String, usize>,
	available: &mut BTreeSet<String>,
) {
	for child in children {
		if let Some(entry) = remaining.get_mut(child) {
			*entry = entry.saturating_sub(1);
			if *entry == 0 {
				available.insert(child.clone());
			}
		}
	}
}

fn max_parent_rank(
	node_id: &str,
	incoming: &HashMap<String, Vec<String>>,
	rank: &HashMap<String, i32>,
) -> Result<i32, RenderError> {
	let predecessors = incoming
		.get(node_id)
		.map(|list| list.as_slice())
		.unwrap_or(&[]);
	if predecessors.is_empty() {
		return Err(RenderError::NodeHasNoIncoming(node_id.to_string()));
	}
	let mut best: Option<i32> = None;
	for predecessor in predecessors {
		let predecessor_rank = rank
			.get(predecessor)
			.copied()
			.ok_or_else(|| RenderError::MissingRank(predecessor.clone()))?;
		let candidate = predecessor_rank + 1;
		best = Some(match best {
			Some(value) => value.max(candidate),
			None => candidate,
		});
	}
	best.ok_or_else(|| RenderError::MissingRank(node_id.to_string()))
}

fn max_rank(rank: &HashMap<String, i32>) -> i32 {
	let mut max_rank = 0;
	for value in rank.values() {
		if *value > max_rank {
			max_rank = *value;
		}
	}
	max_rank
}

fn group_by_rank(rank: &HashMap<String, i32>, max_rank: usize) -> Vec<Vec<String>> {
	let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_rank + 1];
	for (node_id, value) in rank {
		if let Some(layer) = layers.get_mut(*value as usize) {
			layer.push(node_id.clone());
		}
	}
	layers
}

fn incoming_heavy_centered(
	layer: &[String],
	incoming_count: &HashMap<String, usize>,
) -> Vec<String> {
	let mut ordered = layer.to_vec();
	ordered.sort_by(|left, right| {
		let left_count = incoming_count.get(left).copied().unwrap_or(0);
		let right_count = incoming_count.get(right).copied().unwrap_or(0);
		left_count.cmp(&right_count).then_with(|| left.cmp(right))
	});
	let mut odd: Vec<String> = Vec::new();
	let mut even: Vec<String> = Vec::new();
	for (index, node_id) in ordered.into_iter().enumerate() {
		if index % 2 == 0 {
			odd.push(node_id);
		} else {
			even.push(node_id);
		}
	}
	even.reverse();
	odd.extend(even);
	odd
}

fn normalize_neighbors(neighbors: &mut HashMap<String, Vec<String>>) {
	for entries in neighbors.values_mut() {
		entries.sort();
		entries.dedup();
	}
}

fn sweep_down(
	layers: &mut [Vec<String>],
	max_layer_index: usize,
	predecessor_by_layer: &HashMap<String, Vec<String>>,
	positions: &mut HashMap<String, usize>,
) -> Result<(), RenderError> {
	for y in 1..=max_layer_index {
		let layer = layers.get_mut(y).ok_or(RenderError::MissingLayer(y))?;
		reorder_layer(layer, predecessor_by_layer, positions)?;
	}
	Ok(())
}

fn sweep_up(
	layers: &mut [Vec<String>],
	max_layer_index: usize,
	successor_by_layer: &HashMap<String, Vec<String>>,
	positions: &mut HashMap<String, usize>,
) -> Result<(), RenderError> {
	for y in (0..max_layer_index).rev() {
		let layer = layers.get_mut(y).ok_or(RenderError::MissingLayer(y))?;
		reorder_layer(layer, successor_by_layer, positions)?;
	}
	Ok(())
}

fn reorder_layer(
	layer: &mut Vec<String>,
	neighbor_map: &HashMap<String, Vec<String>>,
	positions: &mut HashMap<String, usize>,
) -> Result<(), RenderError> {
	let mut keyed = build_keyed(layer, neighbor_map, positions)?;
	keyed.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
	layer.clear();
	for (node_id, _) in keyed {
		layer.push(node_id);
	}
	update_positions(layer, positions);
	Ok(())
}

fn build_keyed(
	layer: &[String],
	neighbor_map: &HashMap<String, Vec<String>>,
	positions: &HashMap<String, usize>,
) -> Result<Vec<(String, i32)>, RenderError> {
	let mut keyed: Vec<(String, i32)> = Vec::with_capacity(layer.len());
	for node_id in layer.iter() {
		let fallback = positions
			.get(node_id)
			.copied()
			.ok_or_else(|| RenderError::MissingPosition(node_id.clone()))?;
		let mut neighbor_positions = collect_neighbor_positions(node_id, neighbor_map, positions)?;
		let key = median_key(&mut neighbor_positions, fallback);
		keyed.push((node_id.clone(), key));
	}
	Ok(keyed)
}

fn collect_neighbor_positions(
	node_id: &str,
	neighbor_map: &HashMap<String, Vec<String>>,
	positions: &HashMap<String, usize>,
) -> Result<Vec<usize>, RenderError> {
	let mut neighbor_positions: Vec<usize> = Vec::new();
	if let Some(neighbors) = neighbor_map.get(node_id) {
		for neighbor in neighbors {
			let neighbor_pos = positions
				.get(neighbor)
				.copied()
				.ok_or_else(|| RenderError::MissingPosition(neighbor.clone()))?;
			neighbor_positions.push(neighbor_pos);
		}
	}
	Ok(neighbor_positions)
}

fn center_layer(
	layer: &[String],
	x_positions: &mut HashMap<String, i32>,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<(), RenderError> {
	let desired = desired_positions(layer, x_positions, successor_by_layer)?;
	let mut new_x = enforce_left_to_right(&desired);
	tighten_right_to_left(&mut new_x);
	translate_layer_towards_desired(&desired, &mut new_x);
	apply_layer_positions(layer, &new_x, x_positions);
	Ok(())
}

fn translate_layer_towards_desired(desired: &[i32], new_x: &mut [i32]) {
	if desired.is_empty() {
		return;
	}
	let mut offsets: Vec<i32> = Vec::with_capacity(desired.len());
	for (d, x) in desired.iter().zip(new_x.iter()) {
		offsets.push(*d - *x);
	}
	let offset = median_floor_i32(&mut offsets);
	for x in new_x.iter_mut() {
		*x += offset;
	}
}

fn desired_positions(
	layer: &[String],
	x_positions: &HashMap<String, i32>,
	successor_by_layer: &HashMap<String, Vec<String>>,
) -> Result<Vec<i32>, RenderError> {
	let mut desired: Vec<i32> = Vec::with_capacity(layer.len());
	for node_id in layer {
		let current = x_positions
			.get(node_id)
			.copied()
			.ok_or_else(|| RenderError::MissingX(node_id.clone()))?;
		let mut child_positions: Vec<i32> = Vec::new();
		if let Some(children) = successor_by_layer.get(node_id) {
			for child in children {
				let child_x = x_positions
					.get(child)
					.copied()
					.ok_or_else(|| RenderError::MissingX(child.clone()))?;
				child_positions.push(child_x);
			}
		}
		let desired_x = if child_positions.is_empty() {
			current
		} else {
			median_floor_i32(&mut child_positions)
		};
		desired.push(desired_x);
	}
	Ok(desired)
}

fn enforce_left_to_right(desired: &[i32]) -> Vec<i32> {
	let mut new_x: Vec<i32> = Vec::with_capacity(desired.len());
	for (index, desired_x) in desired.iter().enumerate() {
		let value = if index == 0 {
			*desired_x
		} else {
			// Maintain at least one half-step between nodes (2 units).
			let prev = new_x[index - 1] + 2;
			std::cmp::max(*desired_x, prev)
		};
		new_x.push(value);
	}
	new_x
}

fn tighten_right_to_left(new_x: &mut [i32]) {
	if new_x.len() < 2 {
		return;
	}
	for index in (0..new_x.len() - 1).rev() {
		let limit = new_x[index + 1] - 2;
		if new_x[index] > limit {
			new_x[index] = limit;
		}
	}
}

fn apply_layer_positions(layer: &[String], new_x: &[i32], x_positions: &mut HashMap<String, i32>) {
	for (node_id, value) in layer.iter().zip(new_x.iter()) {
		x_positions.insert(node_id.clone(), *value);
	}
}

fn rebuild_positions(layers: &[Vec<String>]) -> HashMap<String, usize> {
	let mut positions: HashMap<String, usize> = HashMap::new();
	for layer in layers {
		update_positions(layer, &mut positions);
	}
	positions
}

fn update_positions(layer: &[String], positions: &mut HashMap<String, usize>) {
	for (index, node_id) in layer.iter().enumerate() {
		positions.insert(node_id.clone(), index);
	}
}

fn median_key(values: &mut [usize], fallback: usize) -> i32 {
	if values.is_empty() {
		return fallback as i32;
	}
	values.sort_unstable();
	let mid = values.len() / 2;
	if values.len() % 2 == 1 {
		values[mid] as i32
	} else {
		((values[mid - 1] + values[mid]) / 2) as i32
	}
}

fn median_floor_i32(values: &mut [i32]) -> i32 {
	if values.is_empty() {
		return 0;
	}
	values.sort_unstable();
	let mid = values.len() / 2;
	if values.len() % 2 == 1 {
		values[mid]
	} else {
		(values[mid - 1] + values[mid]) / 2
	}
}
