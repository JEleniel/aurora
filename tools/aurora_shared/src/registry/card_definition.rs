use serde::{Deserialize, Serialize};

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/details/1-Card_Definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardDefinition {
	pub card_type: &'static str,
	pub acronym: &'static str,
	pub shape: &'static str,
	pub icon: &'static str,
	pub description: &'static str,
	pub fill: &'static str,
	pub color: &'static str,
}

impl CardDefinition {
	pub const EXTENDED: CardDefinition = CardDefinition {
		card_type: "Extended",
		acronym: "EXT",
		icon: "question",
		shape: "rectangle",
		description: "Extended card type",
		fill: "#606060",
		color: "#FFFFFF",
	};

	pub fn validate(card_type: &str) -> bool {
		Self::get_by_type(card_type) != Self::EXTENDED
	}

	pub fn get_fill(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.fill.to_string()
	}

	pub fn get_color(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.color.to_string()
	}

	pub fn get_shape(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.shape.to_string()
	}

	pub fn get_icon(card_type: &str) -> String {
		let def = Self::get_by_type(card_type);
		def.icon.to_string()
	}

	pub fn get_by_type(card_type: &str) -> CardDefinition {
		let definitions =
			serde_json::from_str::<Vec<CardDefinition>>(DEFINITIONS).unwrap_or_default();

		definitions
			.iter()
			.find(|def| def.card_type == card_type)
			.cloned()
			.unwrap_or(Self::EXTENDED)
	}

	pub fn get_by_acronym(acronym: &str) -> CardDefinition {
		let definitions =
			serde_json::from_str::<Vec<CardDefinition>>(DEFINITIONS).unwrap_or_default();

		definitions
			.iter()
			.find(|def| def.acronym == acronym)
			.cloned()
			.unwrap_or(Self::EXTENDED)
	}

	pub fn get_all() -> Vec<CardDefinition> {
		serde_json::from_str::<Vec<CardDefinition>>(DEFINITIONS).unwrap_or_default()
	}
}
