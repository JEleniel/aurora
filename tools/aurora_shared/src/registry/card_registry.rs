use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;

pub const MODEL_CONFIGURATION_VERSION: &str = "1.1.0";
pub const VIEW_CONFIGURATION_VERSION: &str = "1.1.0";
const DEFAULT_VIEW_CONFIGURATION_JSON: &str =
	include_str!("../../../../.github/aurora/reference/Aurora.viewconfiguration.json");

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
		view_configuration.try_domain_paths()?;
		let available_icons = collect_available_icons(&view_configuration);
		let card_type_by_acronym = build_card_type_by_acronym(&model_configuration)?;

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
				common_properties: definition.common_properties,
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

fn collect_available_icons(view_configuration: &ViewConfiguration) -> HashSet<String> {
	view_configuration
		.available_icons
		.iter()
		.filter_map(|icon| normalize_icon_name(icon))
		.collect()
}

fn build_card_type_by_acronym(
	model_configuration: &ModelConfiguration,
) -> Result<HashMap<String, String>, RegistryError> {
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
	Ok(card_type_by_acronym)
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
	#[serde(default)]
	pub common_properties: Vec<ModelConfigurationCommonPropertyDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelConfigurationCommonPropertyDefinition {
	pub name: String,
	pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ViewDomainDefinition {
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub cards: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub subdomains: BTreeMap<String, ViewSubdomainDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ViewSubdomainDefinition {
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub cards: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub subsubdomains: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfiguration {
	#[serde(default, rename = "$schema")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema: Option<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
	pub cards: Vec<ModelConfigurationCardDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigurationCardDefinition {
	pub acronym: String,
	pub card_type: String,
	pub description: String,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub common_properties: Vec<ModelConfigurationCommonPropertyDefinition>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub relationships: Vec<ModelConfigurationRelationshipDefinition>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub common_subtypes: Vec<String>,
}

/// Rendering and icon configuration used by view and SVG renderers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfiguration {
	#[serde(default, rename = "$schema")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema: Option<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
	pub available_icons: Vec<String>,
	pub cards: Vec<ViewConfigurationCardDefinition>,
	#[serde(default)]
	pub domains: BTreeMap<String, ViewDomainDefinition>,
	#[serde(default)]
	pub views: Vec<super::ViewDefinition>,
}

impl ViewConfiguration {
	pub fn try_domain_paths(&self) -> Result<HashMap<String, Vec<String>>, RegistryError> {
		let domains = self.effective_domains()?;
		let known_acronyms: HashSet<String> =
			self.cards.iter().map(|card| card.acronym.clone()).collect();
		let mut paths: HashMap<String, Vec<String>> = HashMap::new();
		let strict_unknowns = !self.domains.is_empty();
		for (domain_name, definition) in &domains {
			collect_domain_paths(
				domain_name,
				definition,
				&known_acronyms,
				strict_unknowns,
				&mut paths,
			)?;
		}
		Ok(paths)
	}

	fn effective_domains(&self) -> Result<BTreeMap<String, ViewDomainDefinition>, RegistryError> {
		if !self.domains.is_empty() {
			return Ok(self.domains.clone());
		}
		let defaults: ViewConfiguration = serde_json::from_str(DEFAULT_VIEW_CONFIGURATION_JSON)?;
		Ok(defaults.domains)
	}
}

/// Appearance configuration for a single card type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfigurationCardDefinition {
	pub acronym: String,
	pub shape: String,
	pub fill: String,
	#[serde(alias = "color")]
	pub stroke: String,
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub text: Option<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigurationRelationshipDefinition {
	pub target: String,
	pub relationship: String,
}

fn collect_domain_paths(
	domain_name: &str,
	definition: &ViewDomainDefinition,
	known_acronyms: &HashSet<String>,
	strict_unknowns: bool,
	paths: &mut HashMap<String, Vec<String>>,
) -> Result<(), RegistryError> {
	assign_domain_cards(
		&[domain_name.to_string()],
		&definition.cards,
		known_acronyms,
		strict_unknowns,
		paths,
	)?;
	for (subdomain_name, subdomain) in &definition.subdomains {
		let path = vec![domain_name.to_string(), subdomain_name.clone()];
		assign_domain_cards(
			path.as_slice(),
			&subdomain.cards,
			known_acronyms,
			strict_unknowns,
			paths,
		)?;
		for (leaf_name, cards) in &subdomain.subsubdomains {
			let leaf_path = vec![
				domain_name.to_string(),
				subdomain_name.clone(),
				leaf_name.clone(),
			];
			assign_domain_cards(
				leaf_path.as_slice(),
				cards,
				known_acronyms,
				strict_unknowns,
				paths,
			)?;
		}
	}
	Ok(())
}

fn assign_domain_cards(
	path: &[String],
	cards: &[String],
	known_acronyms: &HashSet<String>,
	strict_unknowns: bool,
	paths: &mut HashMap<String, Vec<String>>,
) -> Result<(), RegistryError> {
	for acronym in cards {
		if !known_acronyms.contains(acronym) {
			if strict_unknowns {
				return Err(RegistryError::UnknownDomainCardAcronym(acronym.clone()));
			}
			continue;
		}
		if paths.insert(acronym.clone(), path.to_vec()).is_some() {
			return Err(RegistryError::DuplicateDomainAssignment(acronym.clone()));
		}
	}
	Ok(())
}

#[cfg(test)]
#[path = "card_registry_tests.rs"]
mod card_registry_tests;
