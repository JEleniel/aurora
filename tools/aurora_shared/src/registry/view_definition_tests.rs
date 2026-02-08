use super::ViewDefinition;

#[test]
fn get_all_returns_definitions() {
	let definitions = ViewDefinition::get_all();
	assert!(!definitions.is_empty());
}

#[test]
fn try_get_by_name_returns_definition() {
	let definitions = ViewDefinition::get_all();
	let definition = definitions[0].clone();
	let found = ViewDefinition::_try_get_by_name(&definition.name);
	assert!(found.is_some());
}

#[test]
fn try_get_by_name_returns_none_for_unknown() {
	let found = ViewDefinition::_try_get_by_name("Unknown View");
	assert!(found.is_none());
}
