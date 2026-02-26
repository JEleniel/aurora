//! Small helpers for the bundled edge router.
//!
//! Kept in a separate module to keep individual router source files under the repository's
//! size limits, and to keep `edge_router.rs` focused on the routing orchestration.

use std::collections::HashMap;

use super::edge_router_graph::PointI;

pub(super) fn tree_parent_map(
	root: usize,
	tree_adj: &HashMap<usize, Vec<usize>>,
) -> HashMap<usize, usize> {
	let mut parent: HashMap<usize, usize> = HashMap::new();
	let mut stack: Vec<usize> = vec![root];
	parent.insert(root, root);
	while let Some(cur) = stack.pop() {
		if let Some(nexts) = tree_adj.get(&cur) {
			for &n in nexts {
				if parent.contains_key(&n) {
					continue;
				}
				parent.insert(n, cur);
				stack.push(n);
			}
		}
	}
	parent
}

pub(super) fn path_in_tree(parent: &HashMap<usize, usize>, root: usize, leaf: usize) -> Vec<usize> {
	let mut out: Vec<usize> = Vec::new();
	let mut cur = leaf;
	out.push(cur);
	while cur != root {
		cur = *parent.get(&cur).unwrap_or(&root);
		out.push(cur);
	}
	out.reverse();
	out
}

pub(super) fn compress_points(points: Vec<PointI>) -> Vec<PointI> {
	if points.len() <= 2 {
		return points;
	}
	let mut deduped: Vec<PointI> = Vec::new();
	for p in points {
		if deduped
			.last()
			.is_some_and(|last| last.x == p.x && last.y == p.y)
		{
			continue;
		}
		deduped.push(p);
	}
	if deduped.len() <= 2 {
		return deduped;
	}
	let mut out = vec![deduped[0], deduped[1]];
	for p in deduped.iter().skip(2).copied() {
		let a = out[out.len() - 2];
		let b = out[out.len() - 1];
		let same_x = a.x == b.x && b.x == p.x;
		let same_y = a.y == b.y && b.y == p.y;
		if same_x || same_y {
			if let Some(last) = out.last_mut() {
				*last = p;
			} else {
				out.push(p);
			}
		} else {
			out.push(p);
		}
	}
	out
}

pub(super) fn junction_ids_for_route(
	points: &[PointI],
	shared: &HashMap<PointI, usize>,
) -> Vec<String> {
	let mut out = Vec::new();
	for p in points
		.iter()
		.copied()
		.skip(1)
		.take(points.len().saturating_sub(2))
	{
		if shared.get(&p).copied().unwrap_or(0) > 1 {
			out.push(format!("j:{}:{}", p.x, p.y));
		}
	}
	out
}

