use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::discovery::ModelHome;

/// Representation of an Aurora card link.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Link {
	pub target: String,
	pub relationship: String,
}

fn default_links() -> Vec<Link> {
	Vec::new()
}

fn default_value() -> Value {
	Value::Null
}

/// Audit history entry stored on a card.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AuditEvent {
	pub editor: String,
	pub timestamp: String,
	pub event: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hash: Option<String>,
}

fn default_history() -> Vec<AuditEvent> {
	Vec::new()
}

/// Audit trail metadata for a card.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AuditTrail {
	pub version: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hash: Option<String>,
	#[serde(default = "default_history")]
	pub history: Vec<AuditEvent>,
}

/// Aurora card representation used across tooling surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
	#[serde(rename = "$schema", default)]
	pub schema: Option<String>,
	pub id: String,
	pub card_type: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	#[serde(default = "default_links")]
	pub links: Vec<Link>,
	pub audit_trail: AuditTrail,
	#[serde(default = "default_value")]
	pub attributes: Value,
	#[serde(skip_serializing, skip_deserializing)]
	pub source_path: Option<PathBuf>,
	#[serde(flatten, default)]
	pub extra: BTreeMap<String, Value>,
}

impl Card {
	/// Sets the source path where the card was loaded from.
	pub fn with_source_path(mut self, path: PathBuf) -> Self {
		self.source_path = Some(path);
		self
	}

	pub fn source_path(&self) -> Option<&Path> {
		self.source_path.as_deref()
	}
}

/// Loaded Aurora model plus derived indexes for fast lookup.
#[derive(Debug, Clone)]
pub struct AuroraModel {
	home: ModelHome,
	cards: Vec<Card>,
	index_by_id: HashMap<String, Vec<usize>>,
}

impl AuroraModel {
	pub(crate) fn new(home: ModelHome, cards: Vec<Card>) -> Self {
		let mut index_by_id: HashMap<String, Vec<usize>> = HashMap::new();
		for (idx, card) in cards.iter().enumerate() {
			index_by_id.entry(card.id.clone()).or_default().push(idx);
		}
		AuroraModel {
			home,
			cards,
			index_by_id,
		}
	}

	pub fn home(&self) -> &ModelHome {
		&self.home
	}

	pub fn cards(&self) -> &[Card] {
		&self.cards
	}

	pub fn iter_cards(&self) -> impl Iterator<Item = &Card> {
		self.cards.iter()
	}

	pub fn len(&self) -> usize {
		self.cards.len()
	}

	pub fn is_empty(&self) -> bool {
		self.cards.is_empty()
	}

	/// Split a model home into mission-scoped models when multiple missions exist.
	pub fn split_by_mission(&self) -> Vec<AuroraModel> {
		let mission_ids = collect_mission_ids(self);
		if mission_ids.len() <= 1 {
			return vec![self.clone()];
		}

		let mission_set: BTreeSet<String> = mission_ids.iter().cloned().collect();
		let mut by_mission: BTreeMap<String, Vec<Card>> = BTreeMap::new();
		for mission_id in &mission_ids {
			let mut mission_cards = Vec::new();
			for card in self.iter_cards() {
				if card.card_type == "Mission" && card.id == *mission_id {
					mission_cards.push(card.clone());
					break;
				}
			}
			by_mission.insert(mission_id.clone(), mission_cards);
		}

		let mut shared_cards = Vec::new();
		for card in self.iter_cards().filter(|card| card.card_type != "Mission") {
			if let Some(mission_id) = mission_id_for_card(card, &mission_set, self.home.root()) {
				if let Some(cards) = by_mission.get_mut(&mission_id) {
					cards.push(card.clone());
				}
			} else {
				shared_cards.push(card.clone());
			}
		}

		if !shared_cards.is_empty() {
			for cards in by_mission.values_mut() {
				cards.extend(shared_cards.iter().cloned());
			}
		}

		by_mission
			.into_iter()
			.map(|(_, cards)| AuroraModel::new(self.home.clone(), cards))
			.collect()
	}

	pub fn get(&self, id: &str) -> Option<&Card> {
		self.index_by_id
			.get(id)
			.and_then(|entries| entries.first())
			.map(|idx| &self.cards[*idx])
	}

	pub(crate) fn index_by_id(&self) -> &HashMap<String, Vec<usize>> {
		&self.index_by_id
	}
}

fn collect_mission_ids(model: &AuroraModel) -> Vec<String> {
	let mut ids: BTreeSet<String> = BTreeSet::new();
	for card in model.iter_cards() {
		if card.card_type == "Mission" {
			ids.insert(card.id.clone());
		}
	}
	ids.into_iter().collect()
}

fn mission_id_for_card(card: &Card, mission_ids: &BTreeSet<String>, root: &Path) -> Option<String> {
	let source = card.source_path()?;
	let relative = source.strip_prefix(root).ok()?;
	let mut components = relative.components();
	let first = components.next()?;
	let candidate = first.as_os_str().to_str()?;
	if mission_ids.contains(candidate) {
		Some(candidate.to_string())
	} else {
		None
	}
}

#[cfg(test)]
mod tests;
