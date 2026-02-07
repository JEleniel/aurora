use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/Aurora.canonical.definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanonicalDefinitionsFile {
	pub definitions: Vec<CardDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardDefinition {
	pub card_type: String,
	pub acronym: String,
	pub shape: String,
	pub icon: String,
	pub description: String,
	pub fill: String,
	pub color: String,
}

impl CardDefinition {
	fn extended() -> &'static CardDefinition {
		static EXTENDED: OnceLock<CardDefinition> = OnceLock::new();
		EXTENDED.get_or_init(|| CardDefinition {
			card_type: "Extended".to_string(),
			acronym: "EXT".to_string(),
			icon: "question".to_string(),
			shape: "rectangle".to_string(),
			description: "Extended card type".to_string(),
			fill: "#606060".to_string(),
			color: "#FFFFFF".to_string(),
		})
	}

	pub fn validate(card_type: &str) -> bool {
		Self::get_by_type(card_type) != *Self::extended()
	}

	pub fn get_fill(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.fill
	}

	pub fn get_color(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.color
	}

	pub fn get_shape(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.shape
	}

	pub fn get_icon(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.icon
	}

	pub fn get_by_type(card_type: &str) -> CardDefinition {
		let definitions = serde_json::from_str::<CanonicalDefinitionsFile>(DEFINITIONS)
			.map(|file| file.definitions)
			.unwrap_or_default();

		definitions
			.iter()
			.find(|def| def.card_type == card_type)
			.cloned()
			.unwrap_or_else(|| Self::extended().clone())
	}

	pub fn get_by_acronym(acronym: &str) -> CardDefinition {
		let definitions = serde_json::from_str::<CanonicalDefinitionsFile>(DEFINITIONS)
			.map(|file| file.definitions)
			.unwrap_or_default();

		definitions
			.iter()
			.find(|def| def.acronym == acronym)
			.cloned()
			.unwrap_or_else(|| Self::extended().clone())
	}

	pub fn get_all() -> Vec<CardDefinition> {
		serde_json::from_str::<CanonicalDefinitionsFile>(DEFINITIONS)
			.map(|file| file.definitions)
			.unwrap_or_default()
	}
}
