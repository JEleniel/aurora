use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRegistry {
	pub definitions: Vec<CardDefinition>,
	pub available_icons: HashSet<String>,
}

impl CardRegistry {
	pub fn try_new_from_configurations(
		model_configuration_json: &str,
		view_configuration_json: &str,
	) -> Result<Self, RegistryError> {
		let model_configuration: ModelConfiguration =
			serde_json::from_str(model_configuration_json)?;
		let view_configuration: ViewConfiguration = serde_json::from_str(view_configuration_json)?;
		Self::try_new_from_structs(model_configuration, view_configuration)
	}

	pub fn try_new_from_structs(
		model_configuration: ModelConfiguration,
		view_configuration: ViewConfiguration,
	) -> Result<Self, RegistryError> {
		let available_icons: HashSet<String> = view_configuration
			.available_icons
			.iter()
			.filter_map(|icon| normalize_icon_name(icon))
			.collect();

		let mut card_type_by_acronym: HashMap<String, String> = HashMap::new();
		for definition in &model_configuration.cards {
			if card_type_by_acronym
				.insert(definition.acronym.clone(), definition.card_type.clone())
				.is_some()
			{
				return Err(RegistryError::DuplicateCardAcronym(
					definition.acronym.clone(),
				));
			}
		}

		let mut appearance_by_acronym: HashMap<String, ViewConfigurationCardDefinition> =
			HashMap::new();
		for appearance in view_configuration.cards {
			let acronym = appearance.acronym.clone();
			if !card_type_by_acronym.contains_key(&appearance.acronym) {
				return Err(RegistryError::UnknownAppearanceAcronym(appearance.acronym));
			}
			if appearance_by_acronym
				.insert(acronym.clone(), appearance)
				.is_some()
			{
				return Err(RegistryError::DuplicateAppearanceAcronym(acronym));
			}
		}

		let mut definitions: Vec<CardDefinition> =
			Vec::with_capacity(model_configuration.cards.len());
		for definition in model_configuration.cards {
			let appearance = appearance_by_acronym
				.get(&definition.acronym)
				.ok_or_else(|| RegistryError::MissingAppearance(definition.acronym.clone()))?;

			let stroke = appearance.stroke.clone();
			let text = appearance.text.clone().unwrap_or_else(|| stroke.clone());
			let relationships = definition
				.relationships
				.into_iter()
				.map(|relationship| {
					let target_card_type = card_type_by_acronym
						.get(&relationship.target)
						.ok_or_else(|| {
							RegistryError::UnknownRelationshipTarget(relationship.target.clone())
						})?;
					Ok(RelationshipDefinition {
						target_card_type: target_card_type.clone(),
						relationship: relationship.relationship,
					})
				})
				.collect::<Result<Vec<_>, RegistryError>>()?;

			definitions.push(CardDefinition {
				acronym: definition.acronym,
				card_type: definition.card_type,
				stroke,
				text,
				description: definition.description,
				fill: appearance.fill.clone(),
				icon: appearance.icon.clone(),
				relationships,
				shape: appearance.shape.clone(),
				common_subtypes: definition.common_subtypes,
			});
		}

		Ok(Self {
			definitions,
			available_icons,
		})
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

	pub fn try_get_stroke(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.stroke)
	}

	pub fn try_get_text(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.text)
	}

	pub fn try_get_color(&self, card_type: &str) -> Result<String, RegistryError> {
		self.try_get_stroke(card_type)
	}

	pub fn try_get_shape(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.shape)
	}

	pub fn try_get_icon(&self, card_type: &str) -> Result<Option<String>, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.icon)
	}

	pub fn try_get_acronym_for_type(&self, card_type: &str) -> Result<String, RegistryError> {
		let def = self.try_get_by_type(card_type)?;
		Ok(def.acronym)
	}

	pub fn has_icon(&self, icon: &str) -> bool {
		normalize_icon_name(icon)
			.is_some_and(|normalized| self.available_icons.contains(&normalized))
	}

	pub fn canonical_card_types(&self) -> HashSet<String> {
		self.definitions
			.iter()
			.map(|definition| definition.card_type.clone())
			.collect()
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

fn normalize_icon_name(icon: &str) -> Option<String> {
	let normalized = icon.trim().trim_start_matches('#');
	let normalized = normalized.strip_prefix("i-").unwrap_or(normalized);
	if normalized.is_empty() {
		None
	} else {
		Some(normalized.to_string())
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
	pub stroke: String,
	pub text: String,
	pub description: String,
	pub fill: String,
	pub icon: Option<String>,
	#[serde(default)]
	pub relationships: Vec<RelationshipDefinition>,
	pub shape: String,
	#[serde(default)]
	pub common_subtypes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfiguration {
	pub cards: Vec<ModelConfigurationCardDefinition>,
	pub views: Vec<super::ViewDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfigurationCardDefinition {
	pub acronym: String,
	pub card_type: String,
	pub description: String,
	#[serde(default)]
	pub relationships: Vec<ModelConfigurationRelationshipDefinition>,
	#[serde(default)]
	pub common_subtypes: Vec<String>,
}

/// Rendering and icon configuration used by view and SVG renderers.
#[derive(Debug, Clone, Deserialize)]
pub struct ViewConfiguration {
	pub available_icons: Vec<String>,
	pub cards: Vec<ViewConfigurationCardDefinition>,
}

/// Appearance configuration for a single card type.
#[derive(Debug, Clone, Deserialize)]
pub struct ViewConfigurationCardDefinition {
	pub acronym: String,
	pub shape: String,
	pub fill: String,
	#[serde(alias = "color")]
	pub stroke: String,
	#[serde(default)]
	pub text: Option<String>,
	#[serde(default)]
	pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfigurationRelationshipDefinition {
	pub target: String,
	pub relationship: String,
}

#[cfg(test)]
#[path = "card_registry_tests.rs"]
mod card_registry_tests;
