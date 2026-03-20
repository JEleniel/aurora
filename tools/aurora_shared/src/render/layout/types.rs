//! Layout output types.

use std::collections::HashMap;

/// Supported layout families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutFamily {
	TreeTopDown,
	TreeLeftRight,
	Radial,
	Radial1,
	Circular,
}

impl LayoutFamily {
	pub const ORDERED: [Self; 5] = [
		Self::TreeTopDown,
		Self::TreeLeftRight,
		Self::Radial,
		Self::Radial1,
		Self::Circular,
	];

	pub fn ordered() -> &'static [Self] {
		&Self::ORDERED
	}

	pub fn parse(value: &str) -> Option<Self> {
		let normalized = value.trim().to_ascii_lowercase();
		match normalized.as_str() {
			"tree_top_down" | "tree-top-down" | "treetopdown" | "vertical" | "vertical_tree"
			| "vertical-tree" => Some(Self::TreeTopDown),
			"tree_left_right" | "tree-left-right" | "treeleftright" | "horizontal"
			| "horizontal_tree" | "horizontal-tree" => Some(Self::TreeLeftRight),
			"radial" | "radial_subtree" | "radial-subtree" => Some(Self::Radial),
			"radial1" | "radial_1" | "radial-1" => Some(Self::Radial1),
			"circular" => Some(Self::Circular),
			_ => None,
		}
	}
}

/// Coordinate space used by layout node positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutCoordinateSpace {
	Grid,
	Pixels,
}

/// Explicit route point emitted by the layout engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPoint {
	pub x: f32,
	pub y: f32,
}

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
	pub family: Option<LayoutFamily>,
	pub coordinate_space: LayoutCoordinateSpace,
	pub nodes: HashMap<String, LayoutNode>,
	pub edges: Vec<LayoutEdge>,
	pub routes: HashMap<(String, String), Vec<LayoutPoint>>,
}
