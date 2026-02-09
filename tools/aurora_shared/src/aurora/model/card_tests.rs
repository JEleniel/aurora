use std::path::PathBuf;

use serde_json::{Value, json};

use super::{Card, Link};
use crate::registry::CardRegistry;

fn card_schema(required_extra: bool) -> Value {
	let mut required = vec!["id", "card_type", "name", "description", "links", "version"];
	if required_extra {
		required.push("extra");
	}
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": required,
		"properties": {
			"$schema": { "type": "string" },
			"id": { "type": "string" },
			"card_type": { "type": "string" },
			"name": { "type": "string" },
			"description": { "type": "string" },
			"links": { "type": "array" },
			"version": { "type": "string" }
		}
	})
}

fn card_json(id: &str, card_type: &str) -> Value {
	json!({
		"$schema": "./Aurora.card.schema.json",
		"id": id,
		"card_type": card_type,
		"name": id,
		"description": "test",
		"links": [],
		"version": "1.0.0"
	})
}

fn write_json(path: &PathBuf, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
	let serialized = serde_json::to_string_pretty(value)?;
	std::fs::write(path, serialized)?;
	Ok(())
}

fn build_card(id: &str, card_type: &str, links: Vec<Link>) -> Card {
	Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: None,
		name: id.to_string(),
		description: "desc".to_string(),
		version: "1.0.0".to_string(),
		status: None,
		boundary: None,
		notes: None,
		attributes: super::Attributes::new(),
		links,
		source_path: PathBuf::from(format!("{}.json", id)),
		validation_errors: Vec::new(),
	}
}

#[test]
fn try_load_reads_card() -> Result<(), Box<dyn std::error::Error>> {
	let temp = tempfile::tempdir()?;
	let card_path = temp.path().join("REQ-001.json");
	write_json(&card_path, &card_json("REQ-001", "Requirement"))?;

	let card = Card::try_load(&card_path, &card_schema(false))?;
	assert_eq!(card.id, "REQ-001");
	assert!(card.validation_errors.is_empty());
	Ok(())
}

#[test]
fn try_load_collects_schema_errors() -> Result<(), Box<dyn std::error::Error>> {
	let temp = tempfile::tempdir()?;
	let card_path = temp.path().join("REQ-001.json");
	write_json(&card_path, &card_json("REQ-001", "Requirement"))?;

	let card = Card::try_load(&card_path, &card_schema(true))?;
	assert!(!card.validation_errors.is_empty());
	Ok(())
}

#[test]
fn check_registry_warns_on_unknown() -> Result<(), Box<dyn std::error::Error>> {
	let registry = CardRegistry::try_new()?;
	let target_def = registry
		.definitions
		.iter()
		.find(|def| def.acronym.as_bytes().len() == 3)
		.expect("registry should contain at least one 3-letter card acronym");

	let card = build_card(
		"UNK-001",
		"UnknownType",
		vec![Link {
			target: format!("{}-001", target_def.acronym),
			relationship: "rel".to_string(),
		}],
	);

	let warnings = card.check_registry()?;
	assert!(!warnings.is_empty());
	Ok(())
}

#[test]
fn check_registry_accepts_known_relationships() -> Result<(), Box<dyn std::error::Error>> {
	let registry = CardRegistry::try_new()?;
	let source_def = registry
		.definitions
		.iter()
		.find(|def| !def.relationships.is_empty())
		.expect("registry should contain at least one card type with relationships");
	let rel = source_def
		.relationships
		.first()
		.expect("relationship list was unexpectedly empty");
	let target_def = registry.try_get_by_type(&rel.target_card_type)?;
	assert_eq!(
		target_def.acronym.as_bytes().len(),
		3,
		"card acronyms must be 3 bytes because Card::check_registry slices target[0..3]"
	);

	let card = build_card(
		"SRC-001",
		&source_def.card_type,
		vec![Link {
			target: format!("{}-001", target_def.acronym),
			relationship: rel.relationship.clone(),
		}],
	);

	let warnings = card.check_registry()?;
	assert!(
		warnings.is_empty(),
		"expected no warnings, got: {warnings:?}"
	);
	Ok(())
}

#[test]
fn write_outputs_file() -> Result<(), Box<dyn std::error::Error>> {
	let temp = tempfile::tempdir()?;
	let card_path = temp.path().join("REQ-001.json");
	let card = build_card("REQ-001", "Requirement", Vec::new());

	card.write(&card_path);
	let contents = std::fs::read_to_string(&card_path)?;
	assert!(contents.contains("REQ-001"));
	Ok(())
}

#[test]
fn write_markdown_outputs_file() -> Result<(), Box<dyn std::error::Error>> {
	let temp = tempfile::tempdir()?;
	let card_path = temp.path().join("REQ-001.md");
	let card = build_card("REQ-001", "Requirement", Vec::new());

	card.write_markdown(
		&card_path,
		None,
		std::iter::empty::<&crate::AuditLogEntry>(),
	);
	let contents = std::fs::read_to_string(&card_path)?;
	assert!(contents.contains("REQ-001"));
	Ok(())
}

#[test]
fn get_compact_removes_schema() {
	let mut card = build_card("REQ-001", "Requirement", Vec::new());
	card.schema = Some("schema.json".to_string());
	let compact = card.get_compact();

	if let Value::Object(map) = compact {
		assert!(!map.contains_key("$schema"));
	} else {
		panic!("expected compact object");
	}
}
