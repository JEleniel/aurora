use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;

const CANONICAL_DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/Aurora.canonical.definitions.json");
const APPEARANCE_DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/Aurora.appearance.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRegistry {
	pub definitions: Vec<CardDefinition>,
}

impl CardRegistry {
	pub fn try_new() -> Result<Self, RegistryError> {
		let canonical: CanonicalRegistry = serde_json::from_str(CANONICAL_DEFINITIONS)?;
		let appearance: AppearanceRegistry = serde_json::from_str(APPEARANCE_DEFINITIONS)?;

		let appearance_by_acronym: HashMap<String, CardAppearance> = appearance
			.definitions
			.into_iter()
			.map(|entry| (entry.acronym.clone(), entry))
			.collect();

		let card_type_by_acronym: HashMap<String, String> = canonical
			.definitions
			.iter()
			.map(|definition| (definition.acronym.clone(), definition.card_type.clone()))
			.collect();

		for rel in &canonical.relationships {
			if !card_type_by_acronym.contains_key(&rel.source_card_type) {
				return Err(RegistryError::UnknownRelationshipSource(
					rel.source_card_type.clone(),
				));
			}
			if !card_type_by_acronym.contains_key(&rel.target_card_type) {
				return Err(RegistryError::UnknownRelationshipTarget(
					rel.target_card_type.clone(),
				));
			}
		}

		for acronym in appearance_by_acronym.keys() {
			if !card_type_by_acronym.contains_key(acronym) {
				return Err(RegistryError::UnknownAppearanceAcronym(acronym.clone()));
			}
		}

		let mut definitions: Vec<CardDefinition> = Vec::with_capacity(canonical.definitions.len());
		for definition in canonical.definitions {
			let appearance = appearance_by_acronym
				.get(&definition.acronym)
				.ok_or_else(|| RegistryError::MissingAppearance(definition.acronym.clone()))?;

			let relationships = canonical
				.relationships
				.iter()
				.filter(|rel| rel.source_card_type == definition.acronym)
				.map(|rel| {
					let target_card_type = card_type_by_acronym
						.get(&rel.target_card_type)
						.ok_or_else(|| {
							RegistryError::UnknownRelationshipTarget(rel.target_card_type.clone())
						})?;
					Ok(RelationshipDefinition {
						target_card_type: target_card_type.clone(),
						relationship: rel.relationship.clone(),
					})
				})
				.collect::<Result<Vec<_>, RegistryError>>()?;

			definitions.push(CardDefinition {
				acronym: definition.acronym,
				card_type: definition.card_type,
				color: appearance.color.clone(),
				description: definition.description,
				fill: appearance.fill.clone(),
				icon: appearance.icon.clone(),
				relationships,
				shape: appearance.shape.clone(),
				common_subtypes: definition.common_subtypes,
			});
		}

		Ok(Self { definitions })
	}

	pub fn check(&self, card_type: &str) -> bool {
		self.definitions
			.iter()
			.any(|def| def.card_type == card_type)
	}

	pub fn check_link(
		&self,
		source_card_type: &str,
		relationship: &str,
		target_card_type: &str,
	) -> bool {
		let source_def = match self.try_get_by_type(source_card_type) {
			Ok(definition) => definition,
			Err(_) => return false,
		};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipDefinition {
	pub target_card_type: String,
	pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
	#[serde(default)]
	pub common_subtypes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CanonicalRegistry {
	pub definitions: Vec<CanonicalCardDefinition>,
	pub relationships: Vec<CanonicalRelationshipDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
struct CanonicalCardDefinition {
	pub card_type: String,
	pub acronym: String,
	pub description: String,
	#[serde(default)]
	pub common_subtypes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CanonicalRelationshipDefinition {
	pub source_card_type: String,
	pub target_card_type: String,
	pub relationship: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AppearanceRegistry {
	pub definitions: Vec<CardAppearance>,
}

#[derive(Debug, Clone, Deserialize)]
struct CardAppearance {
	pub acronym: String,
	pub shape: String,
	pub icon: String,
	pub fill: String,
	pub color: String,
}

#[cfg(test)]
#[path = "card_registry_tests.rs"]
mod card_registry_tests;
