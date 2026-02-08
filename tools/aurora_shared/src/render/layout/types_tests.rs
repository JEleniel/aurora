use std::collections::HashMap;

use super::types::{Layout, LayoutEdge, LayoutNode};

#[test]
fn layout_types_hold_values() {
	let mut nodes: HashMap<String, LayoutNode> = HashMap::new();
	nodes.insert(
		"A".to_string(),
		LayoutNode {
			id: "A".to_string(),
			x: 1,
			y: 2,
		},
	);
	let edges = vec![LayoutEdge {
		a: "A".to_string(),
		b: "B".to_string(),
	}];
	let layout = Layout { nodes, edges };

	let node = layout.nodes.get("A").expect("node missing");
	assert_eq!(node.x, 1);
	assert_eq!(node.y, 2);
}

#[test]
fn layout_types_handle_missing_nodes() {
	let layout = Layout {
		nodes: HashMap::new(),
		edges: Vec::new(),
	};

	assert!(layout.nodes.get("missing").is_none());
}

#[test]
fn layout_types_are_deterministic_on_clone() {
	let mut nodes: HashMap<String, LayoutNode> = HashMap::new();
	nodes.insert(
		"A".to_string(),
		LayoutNode {
			id: "A".to_string(),
			x: 1,
			y: 2,
		},
	);
	let edges = vec![LayoutEdge {
		a: "A".to_string(),
		b: "B".to_string(),
	}];
	let layout = Layout { nodes, edges };
	let clone = layout.clone();

	assert_eq!(layout.nodes.len(), clone.nodes.len());
	assert_eq!(layout.edges.len(), clone.edges.len());
	let original = node_snapshot(&layout);
	let cloned = node_snapshot(&clone);
	assert_eq!(original, cloned);
}

fn node_snapshot(layout: &Layout) -> Vec<(String, i32, i32)> {
	let mut nodes: Vec<(String, i32, i32)> = layout
		.nodes
		.iter()
		.map(|(id, node)| (id.clone(), node.x, node.y))
		.collect();
	nodes.sort_by(|left, right| left.0.cmp(&right.0));
	nodes
}
