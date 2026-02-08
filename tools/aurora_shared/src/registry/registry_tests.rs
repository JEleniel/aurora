use super::{Registry, RelationshipDefinition};

#[test]
fn get_next_cards_includes_known_targets() {
	let definitions = RelationshipDefinition::get_all();
	assert!(!definitions.is_empty());
	let definition = &definitions[0];

	let next_cards = Registry::get_next_cards(&definition.source_card_type);
	assert!(next_cards.contains(&definition.target_card_type));
}

#[test]
fn get_next_cards_handles_unknown_source() {
	let next_cards = Registry::get_next_cards("UnknownType");
	assert!(next_cards.is_empty());
}
