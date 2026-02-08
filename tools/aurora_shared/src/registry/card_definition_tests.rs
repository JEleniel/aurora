use super::CardDefinition;

#[test]
fn get_all_returns_definitions() {
	let definitions = CardDefinition::get_all();
	assert!(!definitions.is_empty());
}

#[test]
fn get_by_type_matches_definition() {
	let definitions = CardDefinition::get_all();
	let definition = definitions[0].clone();
	let found = CardDefinition::get_by_type(&definition.card_type);
	assert_eq!(found, definition);
}

#[test]
fn get_by_acronym_matches_definition() {
	let definitions = CardDefinition::get_all();
	let definition = definitions[0].clone();
	let found = CardDefinition::get_by_acronym(&definition.acronym);
	assert_eq!(found, definition);
}

#[test]
fn validate_rejects_unknown_card_type() {
	assert!(!CardDefinition::validate("NotAType"));
}

#[test]
fn property_helpers_match_definition() {
	let definitions = CardDefinition::get_all();
	let definition = definitions[0].clone();
	assert_eq!(
		CardDefinition::get_fill(&definition.card_type),
		definition.fill
	);
	assert_eq!(
		CardDefinition::get_color(&definition.card_type),
		definition.color
	);
	assert_eq!(
		CardDefinition::get_shape(&definition.card_type),
		definition.shape
	);
	assert_eq!(
		CardDefinition::get_icon(&definition.card_type),
		definition.icon
	);
}
