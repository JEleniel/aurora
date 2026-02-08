use serde::{Deserialize, Serialize};

const DEFINITIONS: &str = include_str!("../../../../.github/agents/aurora/View.Definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewDefinitionsFile {
	pub definitions: Vec<ViewDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
	pub name: String,
	pub description: String,
	pub root_card_types: Vec<String>,
	pub included_card_types: Vec<String>,
	#[serde(default)]
	pub optional_card_types: Vec<String>,
}

impl ViewDefinition {
	pub fn _try_get_by_name(name: &str) -> Option<ViewDefinition> {
		let definitions = Self::get_all();
		definitions.iter().find(|def| def.name == name).cloned()
	}

	pub fn get_all() -> Vec<ViewDefinition> {
		serde_json::from_str::<ViewDefinitionsFile>(DEFINITIONS)
			.map(|file| file.definitions)
			.unwrap_or_default()
	}
}

const _: () = {
	let _ = ViewDefinition::get_all as fn() -> Vec<ViewDefinition>;
};

#[cfg(test)]
#[path = "view_definition_tests.rs"]
mod view_definition_tests;
