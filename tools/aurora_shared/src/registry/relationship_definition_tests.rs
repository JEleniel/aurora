use super::RelationshipDefinition;

#[test]
fn get_all_returns_definitions() {
	let definitions = RelationshipDefinition::get_all();
	assert!(!definitions.is_empty());
}

#[test]
fn get_by_relationship_matches_definition() {
	let definitions = RelationshipDefinition::get_all();
	let definition = definitions[0].clone();
	let found = RelationshipDefinition::get_by_relationship(
		&definition.source_card_type,
		&definition.relationship,
		&definition.target_card_type,
	);
	assert_eq!(found, definition);
}

#[test]
fn validate_accepts_known_relationship() {
	let definitions = RelationshipDefinition::get_all();
	let definition = definitions[0].clone();
	let valid = RelationshipDefinition::validate(
		&definition.source_card_type,
		&definition.relationship,
		&definition.target_card_type,
	);
	assert!(valid);
}

#[test]
fn validate_rejects_unknown_relationship() {
	let valid = RelationshipDefinition::validate("Unknown", "nope", "Unknown");
	assert!(!valid);
}

#[test]
fn get_by_source_card_type_includes_definition() {
	let definitions = RelationshipDefinition::get_all();
	let definition = definitions[0].clone();
	let matches = RelationshipDefinition::get_by_source_card_type(&definition.source_card_type);
	assert!(matches.iter().any(|entry| entry == &definition));
}

#[test]
fn get_by_target_card_type_includes_definition() {
	let definitions = RelationshipDefinition::get_all();
	let definition = definitions[0].clone();
	let matches = RelationshipDefinition::_get_by_target_card_type(&definition.target_card_type);
	assert!(matches.iter().any(|entry| entry == &definition));
}
