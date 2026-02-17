use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::registry::RegistryError;

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/reference/View.Definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
	pub description: String,
	pub included_card_types: Vec<String>,
	pub name: String,
	pub root_card_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRegistry {
	#[serde(rename = "$schema")]
	schema: String,
	definitions: Vec<ViewDefinition>,
}

impl ViewRegistry {
	pub fn try_new() -> Result<Self, RegistryError> {
		trace!(DEFINITIONS);
		Ok(serde_json::from_str(DEFINITIONS)?)
	}

	pub fn try_get_all(&self) -> Result<Vec<ViewDefinition>, RegistryError> {
		Ok(self.definitions.clone())
	}
}

#[cfg(test)]
#[path = "view_registry_tests.rs"]
mod view_registry_tests;
