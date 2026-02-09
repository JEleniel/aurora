use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/Aurora.canonical.definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRegistry {
	pub definitions: Vec<CardDefinition>,
}

impl CardRegistry {
	pub fn try_new() -> Result<Self, RegistryError> {
		Ok(serde_json::from_str(DEFINITIONS)?)
	}

	pub fn check(&self, card_type: &str) -> bool {
		matches!(
			self.definitions
				.iter()
				.find(|def| def.card_type == card_type),
			Some(_)
		)
	}

	pub fn check_link(
		&self,
		source_card_type: &str,
		relationship: &str,
		target_card_type: &str,
	) -> bool {
		let source_def = self.try_get_by_type(source_card_type);
		if source_def.is_err() {
			return false;
		}
		let source_def = source_def.unwrap();

		source_def
			.relationships
			.iter()
			.any(|rel| rel.relationship == relationship && rel.target_card_type == target_card_type)
	}

	pub fn try_get_fill(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.fill)
	}

	pub fn try_get_color(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.color)
	}

	pub fn try_get_shape(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.shape)
	}

	pub fn try_get_icon(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.icon)
	}

	pub fn try_get_by_type(&self, card_type: &str) -> Result<CardDefinition, RegistryError> {
		self.definitions
			.iter()
			.find(|def| def.card_type == card_type)
			.cloned()
			.ok_or(RegistryError::CardTypeNotFound(card_type.to_string()))
	}

	pub fn try_get_by_acronym(&self, acronym: &str) -> Result<CardDefinition, RegistryError> {
		self.definitions
			.iter()
			.find(|def| def.acronym == acronym)
			.cloned()
			.ok_or(RegistryError::CardAcronymNotFound(acronym.to_string()))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDefinition {
	pub target_card_type: String,
	pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardDefinition {
	pub acronym: String,
	pub card_type: String,
	pub color: String,
	pub description: String,
	pub fill: String,
	pub icon: String,
	#[serde(default)]
	pub relationships: Vec<RelationshipDefinition>,
	pub shape: String,
}

#[cfg(test)]
#[path = "card_registry_tests.rs"]
mod card_registry_tests;
