use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use crate::registry::CardRegistry;
use crate::{AuditLog, Card, Layout, LayoutEdge, LayoutNode, Model};

use super::render_error::RenderError;
use super::svg::Svg;

const SYMBOL_WIDTH_PX: i32 = 720;
const SYMBOL_HEIGHT_PX: i32 = 450;
const COLUMN_PITCH_PX: i32 = 1112;
const ROW_PITCH_PX: i32 = 842;
const VIEWBOX_MARGIN_PX: i32 = 150;

/// Role of a visible card in the focused graph neighborhood.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FocusedGraphRole {
	Center,
	Parent,
	Sibling,
	Child,
}

impl FocusedGraphRole {
	/// Human-readable section label for the role.
	pub fn label(self) -> &'static str {
		match self {
			Self::Center => "Focused card",
			Self::Parent => "Parents",
			Self::Sibling => "Siblings",
			Self::Child => "Descendants",
		}
	}
}

/// Focused card neighborhood used by the editor graph view.
#[derive(Debug, Clone)]
pub struct FocusedGraph {
	pub center: Card,
	pub parents: Vec<Card>,
	pub siblings: Vec<Card>,
	pub children: Vec<Card>,
	pub model_home: PathBuf,
	pub mission_home: PathBuf,
}

impl FocusedGraph {
	/// Construct a focused graph, keeping role buckets sorted and deduplicated by card ID.
	pub fn new(
		center: Card,
		parents: Vec<Card>,
		siblings: Vec<Card>,
		children: Vec<Card>,
		model_home: PathBuf,
		mission_home: PathBuf,
	) -> Self {
		Self {
			center,
			parents: sort_and_dedup_cards(parents),
			siblings: sort_and_dedup_cards(siblings),
			children: sort_and_dedup_cards(children),
			model_home,
			mission_home,
		}
	}

	fn all_cards(&self) -> impl Iterator<Item = (&Card, FocusedGraphRole)> {
		std::iter::once((&self.center, FocusedGraphRole::Center))
			.chain(
				self.parents
					.iter()
					.map(|card| (card, FocusedGraphRole::Parent)),
			)
			.chain(
				self.siblings
					.iter()
					.map(|card| (card, FocusedGraphRole::Sibling)),
			)
			.chain(
				self.children
					.iter()
					.map(|card| (card, FocusedGraphRole::Child)),
			)
	}

	fn to_model(&self) -> Model {
		let cards = self
			.parents
			.iter()
			.chain(self.siblings.iter())
			.chain(self.children.iter())
			.cloned()
			.collect();
		Model {
			root_card: self.center.clone(),
			cards,
			audit_log: AuditLog {
				schema: None,
				history: Vec::new(),
				source_path: self.mission_home.join("AuditLog.ndjson"),
				validation_errors: Vec::new(),
			},
			model_home: self.model_home.clone(),
			mission_home: self.mission_home.clone(),
		}
	}
}

/// Clickable card hotspot aligned to the focused SVG graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedGraphHotspot {
	pub card_id: String,
	pub card_label: String,
	pub role: FocusedGraphRole,
	pub x_px: i32,
	pub y_px: i32,
	pub width_px: i32,
	pub height_px: i32,
}

/// Focused graph SVG plus hotspot metadata for the editor shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedGraphDocument {
	pub svg: String,
	pub hotspots: Vec<FocusedGraphHotspot>,
	pub width_px: i32,
	pub height_px: i32,
}

/// Render the editor-style focused graph using the shared SVG pipeline.
pub fn render_focused_graph(
	graph: &FocusedGraph,
	card_registry: &CardRegistry,
	svg_template: &str,
) -> Result<FocusedGraphDocument, RenderError> {
	let layout = focused_layout(graph);
	let hotspots = focused_hotspots(graph, &layout);
	let (width_px, height_px) = canvas_dimensions(hotspots.as_slice());
	let svg = Svg::render(
		&graph.to_model(),
		&layout,
		card_registry,
		svg_template,
		None,
	)?;
	Ok(FocusedGraphDocument {
		svg,
		hotspots,
		width_px,
		height_px,
	})
}

fn focused_layout(graph: &FocusedGraph) -> Layout {
	let mut nodes = HashMap::new();
	nodes.insert(
		graph.center.id.clone(),
		LayoutNode {
			id: graph.center.id.clone(),
			x: 0,
			y: 0,
		},
	);
	insert_role_row(&mut nodes, graph.parents.as_slice(), -1, false);
	insert_role_row(&mut nodes, graph.children.as_slice(), 1, false);
	insert_role_row(&mut nodes, graph.siblings.as_slice(), 0, true);
	Layout {
		family: None,
		coordinate_space: crate::render::LayoutCoordinateSpace::Grid,
		nodes,
		edges: visible_edges(graph),
		routes: HashMap::new(),
	}
}

fn insert_role_row(
	nodes: &mut HashMap<String, LayoutNode>,
	cards: &[Card],
	y: i32,
	reserve_zero: bool,
) {
	for (card, x) in cards.iter().zip(centered_slots(cards.len(), reserve_zero)) {
		nodes.insert(
			card.id.clone(),
			LayoutNode {
				id: card.id.clone(),
				x,
				y,
			},
		);
	}
}

fn visible_edges(graph: &FocusedGraph) -> Vec<LayoutEdge> {
	let visible_ids = graph
		.all_cards()
		.map(|(card, _)| card.id.clone())
		.collect::<HashSet<_>>();
	let mut edges = graph
		.all_cards()
		.flat_map(|(card, _)| {
			card.links
				.iter()
				.filter(|link| visible_ids.contains(&link.target))
				.map(|link| LayoutEdge {
					a: card.id.clone(),
					b: link.target.clone(),
				})
		})
		.collect::<Vec<_>>();
	edges.sort_by(|left, right| left.a.cmp(&right.a).then_with(|| left.b.cmp(&right.b)));
	edges.dedup_by(|left, right| left.a == right.a && left.b == right.b);
	edges
}

fn centered_slots(count: usize, reserve_zero: bool) -> Vec<i32> {
	if count == 0 {
		return Vec::new();
	}
	if reserve_zero {
		let half = count / 2;
		let mut slots = (1..=half)
			.rev()
			.map(|slot| -(slot as i32))
			.collect::<Vec<_>>();
		let right_count = count - half;
		slots.extend((1..=right_count).map(|slot| slot as i32));
		return slots;
	}

	let center = (count as i32 - 1) / 2;
	(0..count as i32).map(|index| index - center).collect()
}

fn focused_hotspots(graph: &FocusedGraph, layout: &Layout) -> Vec<FocusedGraphHotspot> {
	let bounds = logical_bounds(layout);
	let labels = graph
		.all_cards()
		.map(|(card, role)| {
			(
				card.id.clone(),
				(format!("{}: {}", card.id, card.name), role),
			)
		})
		.collect::<BTreeMap<_, _>>();

	let mut hotspots = layout
		.nodes
		.values()
		.filter_map(|node| {
			labels
				.get(&node.id)
				.map(|(label, role)| FocusedGraphHotspot {
					card_id: node.id.clone(),
					card_label: label.clone(),
					role: *role,
					x_px: node.x * COLUMN_PITCH_PX - bounds.0,
					y_px: node.y * ROW_PITCH_PX - bounds.1,
					width_px: SYMBOL_WIDTH_PX,
					height_px: SYMBOL_HEIGHT_PX,
				})
		})
		.collect::<Vec<_>>();
	hotspots.sort_by(|left, right| {
		left.role
			.cmp(&right.role)
			.then_with(|| left.card_id.cmp(&right.card_id))
	});
	hotspots
}

fn logical_bounds(layout: &Layout) -> (i32, i32, i32, i32) {
	let mut left = i32::MAX;
	let mut top = i32::MAX;
	let mut right = i32::MIN;
	let mut bottom = i32::MIN;
	for node in layout.nodes.values() {
		let x = node.x * COLUMN_PITCH_PX;
		let y = node.y * ROW_PITCH_PX;
		left = left.min(x - VIEWBOX_MARGIN_PX);
		top = top.min(y - VIEWBOX_MARGIN_PX);
		right = right.max(x + SYMBOL_WIDTH_PX + VIEWBOX_MARGIN_PX);
		bottom = bottom.max(y + SYMBOL_HEIGHT_PX + VIEWBOX_MARGIN_PX);
	}
	if layout.nodes.is_empty() {
		(0, 0, 1, 1)
	} else {
		(left, top, right, bottom)
	}
}

fn canvas_dimensions(hotspots: &[FocusedGraphHotspot]) -> (i32, i32) {
	let mut max_right = 1;
	let mut max_bottom = 1;
	for hotspot in hotspots {
		max_right = max_right.max(hotspot.x_px + hotspot.width_px + VIEWBOX_MARGIN_PX);
		max_bottom = max_bottom.max(hotspot.y_px + hotspot.height_px + VIEWBOX_MARGIN_PX);
	}
	(max_right, max_bottom)
}

fn sort_and_dedup_cards(cards: Vec<Card>) -> Vec<Card> {
	let mut by_id = BTreeMap::new();
	for card in cards {
		by_id.entry(card.id.clone()).or_insert(card);
	}
	by_id.into_values().collect()
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use crate::registry::CardRegistry;
	use crate::{Attributes, FocusedGraph, Link, render_focused_graph};

	use super::{FocusedGraphRole, centered_slots};
	use crate::Card;

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn centered_slots_preserve_center_gap_for_siblings() {
		assert_eq!(centered_slots(1, true), vec![1]);
		assert_eq!(centered_slots(2, true), vec![-1, 1]);
		assert_eq!(centered_slots(3, true), vec![-1, 1, 2]);
	}

	#[test]
	fn render_focused_graph_groups_visible_roles() -> Result<()> {
		let graph = FocusedGraph::new(
			make_card("ACT-002", "Activity", "Focus", &["ACT-003"]),
			vec![make_card(
				"ACT-001",
				"Activity",
				"Parent",
				&["ACT-002", "ACT-004"],
			)],
			vec![make_card("ACT-004", "Activity", "Sibling", &[])],
			vec![make_card("ACT-003", "Activity", "Child", &[])],
			PathBuf::from("model"),
			PathBuf::from("model/MIS-001"),
		);
		let registry = CardRegistry::try_new_from_configurations(
			&read_testdata("modelconfiguration/svg_render_registry.json"),
			&read_testdata("modelconfiguration/svg_render_viewconfiguration.json"),
		)?;
		let document = render_focused_graph(
			&graph,
			&registry,
			&read_testdata("svg/template_minimal.svg"),
		)?;

		assert!(document.svg.contains("ACT-002"));
		assert_eq!(document.hotspots.len(), 4);
		assert!(document.hotspots.iter().any(|hotspot| {
			hotspot.card_id == "ACT-002" && hotspot.role == FocusedGraphRole::Center
		}));
		assert!(document.width_px > 0);
		assert!(document.height_px > 0);
		Ok(())
	}

	fn make_card(id: &str, card_type: &str, name: &str, targets: &[&str]) -> Card {
		Card {
			schema: None,
			id: id.to_string(),
			card_type: card_type.to_string(),
			card_subtype: None,
			name: name.to_string(),
			description: format!("{} description", name),
			version: Some("1.0.0".to_string()),
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: Attributes::new(),
			external_references: Vec::new(),
			links: targets
				.iter()
				.map(|target| Link {
					target: (*target).to_string(),
					relationship: "relates".to_string(),
				})
				.collect(),
			source_path: PathBuf::from(format!("{id}.json")),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		}
	}

	fn read_testdata(rel_path: &str) -> String {
		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("src")
			.join("testdata")
			.join(rel_path);
		std::fs::read_to_string(&path)
			.unwrap_or_else(|error| panic!("failed to read testdata {}: {error}", path.display()))
	}
}
