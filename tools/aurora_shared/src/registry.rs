mod card_definition;
mod relationship_definition;
mod view_definition;

pub use card_definition::*;
pub use relationship_definition::*;
pub use view_definition::*;

/// The official registry of cards, relationships, and views included
/// in Aurora by default
pub struct Registry {}

impl Registry {
	/// Get the cards that are valid to link to from a specified card
	pub fn get_next_cards(card_type: &str) -> Vec<String> {
		RelationshipDefinition::get_by_source_card_type(card_type)
			.iter()
			.map(|rel| rel.target_card_type.to_string())
			.collect()
	}
}
