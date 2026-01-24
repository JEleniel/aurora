use std::collections::{BTreeMap, HashMap};
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
