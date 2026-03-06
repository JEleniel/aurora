use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{Card, CardError, Link, NewCard};
use crate::registry::CardRegistry;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_testdata(rel_path: &str) -> String {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("testdata")
		.join(rel_path);
	std::fs::read_to_string(&path)
		.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
}

fn card_schema() -> Value {
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": ["$schema", "id", "card_type", "name", "description", "links"],
		"properties": {
			"$schema": { "type": "string" },
			"id": { "type": "string" },
			"card_type": { "type": "string" },
			"name": { "type": "string" },
			"description": { "type": "string" },
			"links": { "type": "array" }
		}
	})
}

fn write_schema(model_home: &Path) -> Result<()> {
	let schema_dir = model_home.join("schemas");
	std::fs::create_dir_all(&schema_dir)?;
	std::fs::write(
		schema_dir.join("Aurora.card.schema.json"),
		serde_json::to_string_pretty(&card_schema())?,
	)?;
	Ok(())
}

fn build_card(id: &str, card_type: &str) -> Card {
	Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: None,
		name: id.to_string(),
		description: "desc".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::super::Attributes::new(),
		external_references: Vec::new(),
		links: Vec::<Link>::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	}
}

#[test]
fn create_assigns_unique_ids_using_registry_acronyms() -> Result<()> {
	let model_configuration = read_testdata("modelconfiguration/card_tests_target_only.json");
	let view_configuration =
		read_testdata("modelconfiguration/card_tests_target_only_viewconfiguration.json");
	let registry =
		CardRegistry::try_new_from_configurations(&model_configuration, &view_configuration)?;
	let definition = registry
		.definitions
		.first()
		.expect("registry should contain at least one card definition")
		.clone();

	let mut existing_ids = HashSet::from([
		format!("{}-001", definition.acronym),
		format!("{}-002", definition.acronym),
	]);
	let new_card = NewCard {
		card_type: definition.card_type.clone(),
		card_subtype: None,
		name: "Requirement A".to_string(),
		description: "desc".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::super::Attributes::new(),
		external_references: Vec::new(),
		links: Vec::new(),
	};

	let first = Card::create(new_card.clone(), &registry, &existing_ids)?;
	assert_eq!(first.id, format!("{}-003", definition.acronym));
	assert!(!existing_ids.contains(&first.id));
	existing_ids.insert(first.id.clone());

	let second = Card::create(new_card, &registry, &existing_ids)?;
	assert_eq!(second.id, format!("{}-004", definition.acronym));
	assert_ne!(first.id, second.id);
	Ok(())
}

#[test]
fn write_infers_schema_reference_for_nested_card_paths() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;
	let card_path = temp
		.path()
		.join("MIS-001")
		.join("Requirement")
		.join("REQ-001-REQ_001.json");

	let card = build_card("REQ-001", "Requirement");
	card.write(&card_path)?;

	let contents = std::fs::read_to_string(&card_path)?;
	assert!(contents.contains("\"$schema\": \"../../schemas/Aurora.card.schema.json\""));
	Ok(())
}

#[test]
fn write_rejects_invalid_card_id_without_persisting_file() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;
	let card_path = temp.path().join("BAD-Card.json");
	let card = build_card("bad", "Requirement");

	let error = card.write(&card_path).expect_err("write should fail");
	match error {
		CardError::ValidationErrors(errors) => {
			assert!(
				errors
					.iter()
					.any(|error| error.contains("invalid ID format"))
			);
		}
		other => panic!("expected validation error, got {other:?}"),
	}
	assert!(!card_path.exists());
	Ok(())
}
