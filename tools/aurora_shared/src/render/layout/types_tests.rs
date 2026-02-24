use std::collections::HashMap;

use super::types::{Layout, LayoutEdge, LayoutFamily, LayoutNode};

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
	let layout = Layout {
		family: None,
		nodes,
		edges,
	};

	let node = layout.nodes.get("A").expect("node missing");
	assert_eq!(node.x, 1);
	assert_eq!(node.y, 2);
}

#[test]
fn layout_types_handle_missing_nodes() {
	let layout = Layout {
		family: None,
		nodes: HashMap::new(),
		edges: Vec::new(),
	};

	assert!(!layout.nodes.contains_key("missing"));
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
	let layout = Layout {
		family: None,
		nodes,
		edges,
	};
	let clone = layout.clone();

	assert_eq!(layout.nodes.len(), clone.nodes.len());
	assert_eq!(layout.edges.len(), clone.edges.len());
	let original = node_snapshot(&layout);
	let cloned = node_snapshot(&clone);
	assert_eq!(original, cloned);
}

#[test]
fn layout_family_parse_is_case_insensitive() {
	assert_eq!(
		LayoutFamily::parse("vertical-tree"),
		Some(LayoutFamily::VerticalTree)
	);
	assert_eq!(
		LayoutFamily::parse("Horizontal"),
		Some(LayoutFamily::HorizontalTree)
	);
	assert_eq!(
		LayoutFamily::parse("RADIAL_SUBTREE"),
		Some(LayoutFamily::RadialSubtree)
	);
	assert_eq!(LayoutFamily::parse("unknown"), None);
}

#[test]
fn layout_family_order_matches_tie_break_precedence() {
	assert_eq!(
		LayoutFamily::ordered(),
		&[
			LayoutFamily::VerticalTree,
			LayoutFamily::HorizontalTree,
			LayoutFamily::RadialSubtree,
		]
	);
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
