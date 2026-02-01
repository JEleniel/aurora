use serde::{Deserialize, Serialize};

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/details/3-View_Definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
	pub name: String,
	pub description: String,
	pub root_card_types: Vec<String>,
	pub included_card_types: Vec<String>,
	pub optional_card_types: Vec<String>,
}

impl ViewDefinition {
	pub fn _try_get_by_name(name: &str) -> Option<ViewDefinition> {
		let definitions = serde_json::from_str::<Vec<ViewDefinition>>(DEFINITIONS).ok()?;

		definitions.iter().find(|def| def.name == name).cloned()
	}

	pub fn get_all() -> Vec<ViewDefinition> {
		serde_json::from_str::<Vec<ViewDefinition>>(DEFINITIONS).unwrap()
	}
}
