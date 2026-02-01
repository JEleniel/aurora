//! Editor-friendly model helpers for Aurora cards.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use aurora_shared::{AuroraModel, Card};

use crate::error::EditorError;

/// Summary data used for list displays and quick lookups.
#[derive(Debug, Clone)]
pub struct CardSummary {
	pub id: String,
	pub card_type: String,
	pub name: String,
}

/// Flattened tree entry for rendering the navigation pane.
#[derive(Debug, Clone)]
pub struct TreeEntry {
	pub depth: usize,
	pub summary: CardSummary,
}

/// Context view for the focused card.
#[derive(Debug, Clone)]
pub struct ContextView {
	pub parent: Option<CardSummary>,
	pub siblings: Vec<CardSummary>,
	pub children: Vec<CardSummary>,
}

/// Editor-friendly representation of a mission-scoped model.
#[derive(Debug, Clone)]
pub struct EditorModel {
	model: AuroraModel,
	mission_id: String,
	card_index: HashMap<String, CardSummary>,
	parent_by_id: HashMap<String, String>,
	tree_children: HashMap<String, Vec<String>>,
	outgoing_by_id: HashMap<String, Vec<String>>,
	tree_entries: Vec<TreeEntry>,
}

impl EditorModel {
	/// Build an editor-friendly snapshot for a mission-scoped model.
	pub fn new(model: AuroraModel) -> Result<Self, EditorError> {
		let mission = model
			.iter_cards()
			.find(|card| card.card_type == "Mission")
			.ok_or(EditorError::MissingMission)?;
		let mission_id = mission.id.clone();
		let card_index = build_card_index(&model);
		let outgoing_by_id = build_outgoing_links(&model);
		let (parent_by_id, tree_children) = build_shortest_path_tree(&mission_id, &outgoing_by_id);
		let tree_entries = build_tree_entries(&mission_id, &card_index, &tree_children);

		Ok(EditorModel {
			model,
			mission_id,
			card_index,
			parent_by_id,
			tree_children,
			outgoing_by_id,
			tree_entries,
		})
	}

	/// Return the mission id for this model snapshot.
	pub fn mission_id(&self) -> &str {
		&self.mission_id
	}

	/// Return the mission name if it is available.
	pub fn mission_name(&self) -> Option<&str> {
		self.model
			.iter_cards()
			.find(|card| card.card_type == "Mission")
			.map(|card| card.name.as_str())
	}

	/// Look up a card by id.
	pub fn card(&self, id: &str) -> Option<&Card> {
		self.model.get(id)
	}

	/// Return the flattened navigation tree entries.
	pub fn tree_entries(&self) -> &[TreeEntry] {
		&self.tree_entries
	}

	/// Build a parent/sibling/child context view for a focused card.
	pub fn context_view(&self, focused_id: &str) -> ContextView {
		let parent = self
			.parent_by_id
			.get(focused_id)
			.and_then(|parent_id| self.card_index.get(parent_id))
			.cloned();
		let siblings = self.collect_siblings(focused_id);
		let children = self.collect_children(focused_id);
		ContextView {
			parent,
			siblings,
			children,
		}
	}

	/// Find cards matching the query by id, name, or card type.
	pub fn search_cards(&self, query: &str) -> Vec<CardSummary> {
		let trimmed = query.trim().to_lowercase();
		if trimmed.is_empty() {
			return Vec::new();
		}
		let mut matches = self
			.card_index
			.values()
			.filter(|summary| {
				let id = summary.id.to_lowercase();
				let name = summary.name.to_lowercase();
				let card_type = summary.card_type.to_lowercase();
				id.contains(&trimmed) || name.contains(&trimmed) || card_type.contains(&trimmed)
			})
			.cloned()
			.collect::<Vec<_>>();
		matches.sort_by(|left, right| left.id.cmp(&right.id));
		matches
	}

	fn collect_siblings(&self, focused_id: &str) -> Vec<CardSummary> {
		let Some(parent_id) = self.parent_by_id.get(focused_id) else {
			return Vec::new();
		};
		let focused_type = self
			.card_index
			.get(focused_id)
			.map(|summary| summary.card_type.clone())
			.unwrap_or_default();
		let mut siblings = self
			.tree_children
			.get(parent_id)
			.into_iter()
			.flat_map(|children| children.iter())
			.filter(|child_id| *child_id != focused_id)
			.filter_map(|child_id| self.card_index.get(child_id))
			.filter(|summary| summary.card_type == focused_type)
			.cloned()
			.collect::<Vec<_>>();
		siblings.sort_by(|left, right| left.id.cmp(&right.id));
		siblings
	}

	fn collect_children(&self, focused_id: &str) -> Vec<CardSummary> {
		let mut children = self
			.outgoing_by_id
			.get(focused_id)
			.into_iter()
			.flat_map(|targets| targets.iter())
			.filter_map(|child_id| self.card_index.get(child_id))
			.cloned()
			.collect::<Vec<_>>();
		children.sort_by(|left, right| left.id.cmp(&right.id));
		children
	}
}

/// Render a read-only Markdown-style preview for a card.
pub fn render_card_preview(card: &Card) -> String {
	let mut buffer = String::new();
	buffer.push_str(&format!("# {}: **{}**\n\n", card.id, card.card_type));
	buffer.push_str(&format!("**Name:** {}\n\n", card.name));
	buffer.push_str("## Description\n\n");
	buffer.push_str(&card.description);
	buffer.push_str("\n\n## Links\n\n");
	if card.links.is_empty() {
		buffer.push_str("_No outgoing links._\n");
	} else {
		for link in &card.links {
			buffer.push_str(&format!("- `{}` → `{}`\n", link.relationship, link.target));
		}
	}
	buffer
}

fn build_card_index(model: &AuroraModel) -> HashMap<String, CardSummary> {
	let mut index = HashMap::new();
	for card in model.iter_cards() {
		index.insert(
			card.id.clone(),
			CardSummary {
				id: card.id.clone(),
				card_type: card.card_type.clone(),
				name: card.name.clone(),
			},
		);
	}
	index
}

fn build_outgoing_links(model: &AuroraModel) -> HashMap<String, Vec<String>> {
	let mut outgoing = HashMap::new();
	for card in model.iter_cards() {
		let mut targets = card
			.links
			.iter()
			.map(|link| link.target.clone())
			.collect::<BTreeSet<_>>()
			.into_iter()
			.collect::<Vec<_>>();
		targets.sort();
		outgoing.insert(card.id.clone(), targets);
	}
	outgoing
}

fn build_shortest_path_tree(
	mission_id: &str,
	outgoing_by_id: &HashMap<String, Vec<String>>,
) -> (HashMap<String, String>, HashMap<String, Vec<String>>) {
	let mut parent_by_id = HashMap::new();
	let mut visited = HashSet::new();
	let mut queue = VecDeque::new();
	visited.insert(mission_id.to_string());
	queue.push_back(mission_id.to_string());

	while let Some(current) = queue.pop_front() {
		let Some(children) = outgoing_by_id.get(&current) else {
			continue;
		};
		for target in children {
			if !visited.insert(target.clone()) {
				continue;
			}
			parent_by_id.insert(target.clone(), current.clone());
			queue.push_back(target.clone());
		}
	}

	let mut tree_children: HashMap<String, Vec<String>> = HashMap::new();
	for (child, parent) in &parent_by_id {
		if child == mission_id {
			continue;
		}
		tree_children
			.entry(parent.clone())
			.or_default()
			.push(child.clone());
	}
	for children in tree_children.values_mut() {
		children.sort();
	}

	(parent_by_id, tree_children)
}

fn build_tree_entries(
	root_id: &str,
	card_index: &HashMap<String, CardSummary>,
	tree_children: &HashMap<String, Vec<String>>,
) -> Vec<TreeEntry> {
	let mut entries = Vec::new();
	build_tree_entries_recursive(root_id, 0, card_index, tree_children, &mut entries);
	entries
}

fn build_tree_entries_recursive(
	current_id: &str,
	depth: usize,
	card_index: &HashMap<String, CardSummary>,
	tree_children: &HashMap<String, Vec<String>>,
	entries: &mut Vec<TreeEntry>,
) {
	if let Some(summary) = card_index.get(current_id) {
		entries.push(TreeEntry {
			depth,
			summary: summary.clone(),
		});
	}
	let Some(children) = tree_children.get(current_id) else {
		return;
	};
	for child in children {
		build_tree_entries_recursive(child, depth + 1, card_index, tree_children, entries);
	}
}
