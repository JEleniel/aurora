//! Layout output types.

use std::collections::HashMap;

/// Position data for a node in a layout.
#[derive(Debug, Clone)]
pub struct LayoutNode {
	pub id: String,
	pub x: i32,
	pub y: i32,
}

/// Edge data for a layout.
#[derive(Debug, Clone)]
pub struct LayoutEdge {
	pub a: String,
	pub b: String,
}

/// Layout result keyed by node id.
#[derive(Debug, Clone)]
pub struct Layout {
	pub nodes: HashMap<String, LayoutNode>,
	pub edges: Vec<LayoutEdge>,
}
