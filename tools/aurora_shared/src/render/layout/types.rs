//! Layout output types.

use std::collections::HashMap;

/// Supported layout families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutFamily {
	VerticalTree,
	HorizontalTree,
	RadialSubtree,
}

impl LayoutFamily {
	pub const ORDERED: [Self; 3] = [
		Self::VerticalTree,
		Self::HorizontalTree,
		Self::RadialSubtree,
	];

	pub fn ordered() -> &'static [Self] {
		&Self::ORDERED
	}

	pub fn parse(value: &str) -> Option<Self> {
		let normalized = value.trim().to_ascii_lowercase();
		match normalized.as_str() {
			"vertical" | "vertical_tree" | "vertical-tree" => Some(Self::VerticalTree),
			"horizontal" | "horizontal_tree" | "horizontal-tree" => Some(Self::HorizontalTree),
			"radial" | "radial_subtree" | "radial-subtree" => Some(Self::RadialSubtree),
			_ => None,
		}
	}
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
	pub nodes: HashMap<String, LayoutNode>,
	pub edges: Vec<LayoutEdge>,
}
