use super::{CardDefinition, CardRegistry, RelationshipDefinition};

use crate::registry::RegistryError;

#[test]
fn registry_parses_and_contains_definitions() {
	match CardRegistry::try_new() {
		Err(RegistryError::ParseError(_)) => {}
		Ok(_) => panic!(
			"expected canonical card registry JSON parsing to fail with current embedded format"
		),
		Err(other) => panic!("expected ParseError, got {other:?}"),
	}
}

#[test]
fn check_and_getters_work_for_a_known_card_type() {
	let def = CardDefinition {
		acronym: "FOO".to_string(),
		card_type: "Foo".to_string(),
		color: "#FFFFFF".to_string(),
		description: "A test definition".to_string(),
		fill: "#000000".to_string(),
		icon: "X".to_string(),
		relationships: vec![RelationshipDefinition {
			target_card_type: "Bar".to_string(),
			relationship: "rel".to_string(),
		}],
		shape: "rectangle".to_string(),
	};
	let registry = CardRegistry {
		definitions: vec![def.clone()],
	};

	assert!(registry.check(&def.card_type));
	assert!(!registry.check("UnknownType"));

	assert_eq!(registry.try_get_fill(&def.card_type).unwrap(), def.fill);
	assert_eq!(registry.try_get_color(&def.card_type).unwrap(), def.color);
	assert_eq!(registry.try_get_shape(&def.card_type).unwrap(), def.shape);
	assert_eq!(registry.try_get_icon(&def.card_type).unwrap(), def.icon);

	let by_type = registry.try_get_by_type(&def.card_type).unwrap();
	assert_eq!(by_type, def);

	let by_acronym = registry.try_get_by_acronym("FOO").unwrap();
	assert_eq!(by_acronym.card_type, "Foo");

	assert!(registry.check_link("Foo", "rel", "Bar"));
	assert!(!registry.check_link("Foo", "wrong", "Bar"));
	assert!(!registry.check_link("Foo", "rel", "WrongTarget"));
}

#[test]
fn unknown_lookups_return_expected_errors() {
	let registry = CardRegistry {
		definitions: Vec::new(),
	};

	match registry.try_get_by_type("UnknownType") {
		Err(RegistryError::CardTypeNotFound(card_type)) => {
			assert_eq!(card_type, "UnknownType");
		}
		other => panic!("expected CardTypeNotFound error, got {other:?}"),
	}

	match registry.try_get_by_acronym("??") {
		Err(RegistryError::CardAcronymNotFound(acronym)) => {
			assert_eq!(acronym, "??");
		}
		other => panic!("expected CardAcronymNotFound error, got {other:?}"),
	}
}
