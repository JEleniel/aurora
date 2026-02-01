use serde::{Deserialize, Serialize};

const DEFINITIONS: &str =
	include_str!("../../../../.github/agents/details/2-Relationship_Definitions.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDefinition {
	pub source_card_type: &'static str,
	pub target_card_type: &'static str,
	pub relationship: &'static str,
	pub description: &'static str,
}

impl RelationshipDefinition {
	pub const EXTENDED: RelationshipDefinition = RelationshipDefinition {
		source_card_type: "Extended",
		target_card_type: "Extended",
		relationship: "relates to",
		description: "Extended relationship type",
	};

	pub fn validate(source_card_type: &str, relationship: &str, target_card_type: &str) -> bool {
		let relationship =
			Self::get_by_relationship(source_card_type, relationship, target_card_type);
		relationship != Self::EXTENDED
			&& relationship.source_card_type == source_card_type
			&& relationship.target_card_type == target_card_type
	}

	pub fn get_by_relationship(
		source_card_type: &str,
		relationship: &str,
		target_card_type: &str,
	) -> RelationshipDefinition {
		let definitions = serde_json::from_str::<Vec<RelationshipDefinition>>(DEFINITIONS)
			.unwrap_or(vec![Self::EXTENDED]);

		definitions
			.iter()
			.find(|def| {
				def.source_card_type == source_card_type
					&& def.relationship == relationship
					&& def.target_card_type == target_card_type
			})
			.cloned()
			.unwrap_or(Self::EXTENDED)
	}

	pub fn get_by_source_card_type(source_card_type: &str) -> Vec<RelationshipDefinition> {
		let definitions = serde_json::from_str::<Vec<RelationshipDefinition>>(DEFINITIONS)
			.unwrap_or(vec![Self::EXTENDED]);

		definitions
			.iter()
			.filter(|def| def.source_card_type.contains(&source_card_type))
			.cloned()
			.collect::<Vec<RelationshipDefinition>>()
	}

	pub fn _get_by_target_card_type(target_card_type: &str) -> Vec<RelationshipDefinition> {
		let definitions = serde_json::from_str::<Vec<RelationshipDefinition>>(DEFINITIONS)
			.unwrap_or(vec![Self::EXTENDED]);

		definitions
			.iter()
			.filter(|def| def.target_card_type.contains(&target_card_type))
			.cloned()
			.collect::<Vec<RelationshipDefinition>>()
			.into()
	}

	pub fn get_all() -> Vec<RelationshipDefinition> {
		serde_json::from_str::<Vec<RelationshipDefinition>>(DEFINITIONS)
			.unwrap_or(vec![Self::EXTENDED])
	}
}
