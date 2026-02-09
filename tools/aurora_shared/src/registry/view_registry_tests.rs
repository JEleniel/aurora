use super::ViewRegistry;

#[test]
fn try_get_all_returns_definitions() {
	let registry = ViewRegistry::try_new().expect("view registry JSON should parse");
	let definitions = registry
		.try_get_all()
		.expect("view registry should return definitions");
	assert!(!definitions.is_empty());
}

#[test]
fn can_find_a_definition_by_name_in_returned_list() {
	let registry = ViewRegistry::try_new().expect("view registry JSON should parse");
	let definitions = registry
		.try_get_all()
		.expect("view registry should return definitions");
	let definition = definitions
		.first()
		.expect("view registry should contain at least one definition");

	let found = definitions.iter().find(|d| d.name == definition.name);
	assert!(found.is_some());
}

#[test]
fn find_by_name_returns_none_for_unknown() {
	let registry = ViewRegistry::try_new().expect("view registry JSON should parse");
	let definitions = registry
		.try_get_all()
		.expect("view registry should return definitions");

	let found = definitions.iter().find(|d| d.name == "Unknown View");
	assert!(found.is_none());
}
