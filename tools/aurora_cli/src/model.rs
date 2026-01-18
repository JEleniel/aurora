//! In-memory representation of Aurora cards and models.

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Represents a directional link from one card to another.
#[derive(Debug, Clone, Deserialize)]
pub struct Link {
	pub target: String,
	#[serde(default)]
	pub relationship: Option<String>,
}

/// Aurora card metadata loaded from JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct Card {
	#[serde(skip)]
	pub source_path: PathBuf,
	#[serde(skip)]
	pub relative_path: PathBuf,
	#[serde(skip)]
	pub raw: Value,
	#[serde(rename = "$schema", default)]
	pub schema_ref: Option<String>,
	pub id: String,
	#[serde(rename = "card_type")]
	pub card_type: String,
	#[serde(default)]
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub status: Option<String>,
	#[serde(default)]
	pub links: Vec<Link>,
	#[serde(default)]
	pub attributes: Value,
	#[serde(default)]
	pub audit_trail: Value,
}

impl Card {
	/// Attach filesystem metadata to a card after deserialization.
	pub fn with_paths(mut self, source_path: PathBuf, relative_path: PathBuf, raw: Value) -> Self {
		self.source_path = source_path;
		self.relative_path = relative_path;
		self.raw = raw;
		self
	}

	/// Returns `true` when the card type is Mission.
	pub fn is_mission(&self) -> bool {
		self.card_type.eq_ignore_ascii_case("Mission")
	}
}

/// Fully materialized Aurora model state.
#[derive(Debug, Clone)]
pub struct Model {
	root: PathBuf,
	schema_path: PathBuf,
	cards: IndexMap<String, Card>,
	mission_id: String,
}

impl Model {
	pub fn new(
		root: PathBuf,
		schema_path: PathBuf,
		cards: IndexMap<String, Card>,
		mission_id: String,
	) -> Self {
		Self {
			root,
			schema_path,
			cards,
			mission_id,
		}
	}

	pub fn root(&self) -> &Path {
		&self.root
	}

	pub fn schema_path(&self) -> &Path {
		&self.schema_path
	}

	pub fn cards(&self) -> impl Iterator<Item = &Card> {
		self.cards.values()
	}

	pub fn card(&self, id: &str) -> Option<&Card> {
		self.cards.get(id)
	}

	pub fn contains(&self, id: &str) -> bool {
		self.cards.contains_key(id)
	}

	pub fn mission(&self) -> &Card {
		self.cards
			.get(&self.mission_id)
			.expect("mission id set to missing card")
	}

	pub fn mission_id(&self) -> &str {
		&self.mission_id
	}

	pub fn len(&self) -> usize {
		self.cards.len()
	}

	pub fn is_empty(&self) -> bool {
		self.cards.is_empty()
	}
}
