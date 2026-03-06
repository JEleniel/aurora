use super::ViewRegistry;
use std::path::PathBuf;

fn read_testdata(rel_path: &str) -> String {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("testdata")
		.join(rel_path);
	std::fs::read_to_string(&path)
		.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
}

fn test_view_configuration() -> String {
	read_testdata("modelconfiguration/view_registry_test.json")
}

#[test]
fn try_get_all_returns_definitions() {
	let view_configuration = test_view_configuration();
	let registry = ViewRegistry::try_new_from_view_configuration(&view_configuration)
		.expect("view registry JSON should parse");
	let definitions = registry
		.try_get_all()
		.expect("view registry should return definitions");
	assert!(!definitions.is_empty());
}

#[test]
fn can_find_a_definition_by_name_in_returned_list() {
	let view_configuration = test_view_configuration();
	let registry = ViewRegistry::try_new_from_view_configuration(&view_configuration)
		.expect("view registry JSON should parse");
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
	let view_configuration = test_view_configuration();
	let registry = ViewRegistry::try_new_from_view_configuration(&view_configuration)
		.expect("view registry JSON should parse");
	let definitions = registry
		.try_get_all()
		.expect("view registry should return definitions");

	let found = definitions.iter().find(|d| d.name == "Unknown View");
	assert!(found.is_none());
}
