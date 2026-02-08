use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

const CANONICAL_DEFINITIONS: &str =
	include_str!("../../../../.github/agents/aurora/Aurora.canonical.definitions.json");

#[derive(Debug, Clone, Deserialize)]
struct CanonicalDefinitionsFile {
	pub definitions: Vec<CanonicalCardTypeDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
struct CanonicalCardTypeDefinition {
	pub card_type: String,
	pub acronym: String,
	#[serde(default)]
	pub relationships: Vec<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDefinition {
	pub source_card_type: String,
	pub target_card_type: String,
	pub relationship: String,
	pub description: String,
}

impl RelationshipDefinition {
	fn definitions() -> &'static Vec<RelationshipDefinition> {
		static DEFINITIONS: OnceLock<Vec<RelationshipDefinition>> = OnceLock::new();
		DEFINITIONS.get_or_init(|| {
			let canonical = serde_json::from_str::<CanonicalDefinitionsFile>(CANONICAL_DEFINITIONS)
				.map(|file| file.definitions)
				.unwrap_or_default();

			let mut relationships: Vec<RelationshipDefinition> = Vec::new();
			for source in &canonical {
				for mapping in &source.relationships {
					for (target_acronym, verb) in mapping {
						let target = canonical
							.iter()
							.find(|def| def.acronym.as_str() == target_acronym.as_str());
						if let Some(target) = target {
							relationships.push(RelationshipDefinition {
								source_card_type: source.card_type.clone(),
								target_card_type: target.card_type.clone(),
								relationship: verb.clone(),
								description: String::new(),
							});
						}
					}
				}
			}
			relationships
		})
	}

	fn extended() -> RelationshipDefinition {
		RelationshipDefinition {
			source_card_type: "Extended".to_string(),
			target_card_type: "Extended".to_string(),
			relationship: "relates to".to_string(),
			description: "Extended relationship type".to_string(),
		}
	}

	pub fn validate(source_card_type: &str, relationship: &str, target_card_type: &str) -> bool {
		let relationship =
			Self::get_by_relationship(source_card_type, relationship, target_card_type);
		relationship != Self::extended()
			&& relationship.source_card_type == source_card_type
			&& relationship.target_card_type == target_card_type
	}

	pub fn get_by_relationship(
		source_card_type: &str,
		relationship: &str,
		target_card_type: &str,
	) -> RelationshipDefinition {
		Self::definitions()
			.iter()
			.find(|def| {
				def.source_card_type == source_card_type
					&& def.relationship == relationship
					&& def.target_card_type == target_card_type
			})
			.cloned()
			.unwrap_or_else(Self::extended)
	}

	pub fn get_by_source_card_type(source_card_type: &str) -> Vec<RelationshipDefinition> {
		Self::definitions()
			.iter()
			.filter(|def| def.source_card_type == source_card_type)
			.cloned()
			.collect::<Vec<RelationshipDefinition>>()
	}

	pub fn _get_by_target_card_type(target_card_type: &str) -> Vec<RelationshipDefinition> {
		Self::definitions()
			.iter()
			.filter(|def| def.target_card_type == target_card_type)
			.cloned()
			.collect::<Vec<RelationshipDefinition>>()
			.into()
	}

	pub fn get_all() -> Vec<RelationshipDefinition> {
		Self::definitions().clone()
	}
}

const _: () = {
	let _ =
		RelationshipDefinition::get_by_source_card_type as fn(&str) -> Vec<RelationshipDefinition>;
	let _ = RelationshipDefinition::get_all as fn() -> Vec<RelationshipDefinition>;
};

#[cfg(test)]
#[path = "relationship_definition_tests.rs"]
mod relationship_definition_tests;
