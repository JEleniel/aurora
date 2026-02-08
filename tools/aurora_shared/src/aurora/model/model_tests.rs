use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};

use super::{Model, ModelError};
use crate::{AuditChangeType, AuditLog, AuditLogEntry};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn missing(message: &str) -> Box<dyn std::error::Error> {
	Box::new(std::io::Error::new(std::io::ErrorKind::Other, message))
}

fn card_schema() -> Value {
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": ["id", "card_type", "name", "description", "links", "version"],
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

fn audit_schema() -> Value {
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": ["$schema", "history"],
		"properties": {
			"$schema": { "type": "string" },
			"history": { "type": "array", "minItems": 0 }
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

fn card_json_with_links(id: &str, card_type: &str, targets: &[&str]) -> Value {
	let links: Vec<Value> = targets
		.iter()
		.map(|target| {
			json!({
				"target": target,
				"relationship": "rel"
			})
		})
		.collect();
	json!({
		"$schema": "./Aurora.card.schema.json",
		"id": id,
		"card_type": card_type,
		"name": id,
		"description": "test",
		"links": links,
		"version": "1.0.0"
	})
}

fn audit_log_json() -> Value {
	json!({
		"$schema": "../Aurora.audit.schema.json",
		"history": []
	})
}

fn write_json(path: &PathBuf, value: &Value) -> Result<()> {
	let serialized = serde_json::to_string_pretty(value)?;
	std::fs::write(path, serialized)?;
	Ok(())
}

#[test]
fn sanitize_filename_strips_symbols() {
	let input = "Alpha- Beta!! Gamma__ Delta";
	let expected = "Alpha_Beta_Gamma_Delta";
	let actual = super::sanitize_filename(input);
	assert_eq!(actual, expected);
}

#[test]
fn try_load_rejects_nested_mission_card() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let root_path = temp.path().join("MIS-001-Alpha.json");
	let mission_home = temp.path().join("MIS-001");
	let mission_folder = mission_home.join("Mission");
	std::fs::create_dir_all(&mission_folder)?;

	write_json(&root_path, &card_json("MIS-001", "Mission"))?;
	write_json(&mission_home.join("AuditLog.json"), &audit_log_json())?;
	write_json(
		&mission_folder.join("MIS-999-Extra.json"),
		&card_json("MIS-999", "Mission"),
	)?;

	let card_schema = card_schema();
	let audit_schema = audit_schema();
	let result = Model::try_load(&root_path, &card_schema, &audit_schema);

	match result {
		Err(ModelError::UnexpectedMissionCard(_)) => Ok(()),
		Err(error) => Err(Box::new(error)),
		Ok(_) => Err(missing("expected nested mission guard error")),
	}
}

#[test]
fn try_load_rejects_invalid_model() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let root_path = temp.path().join("MIS-001-Alpha.json");
	let mission_home = temp.path().join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;

	write_json(
		&root_path,
		&card_json_with_links("MIS-001", "Mission", &["REQ-999"]),
	)?;
	write_json(&mission_home.join("AuditLog.json"), &audit_log_json())?;

	let card_schema = card_schema();
	let audit_schema = audit_schema();
	let model = Model::try_load(&root_path, &card_schema, &audit_schema)?;
	let errors = model.validate();
	if !errors.iter().any(|error| error.contains("broken link")) {
		return Err(missing("expected broken link error"));
	}
	Ok(())
}

#[test]
fn write_markdown_uses_sanitized_names_and_audit_history() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();
	let mission_name = "Mission Name!";
	let card_name = "Feature: One?";
	let card_id = "FEA-001";

	let root_card = super::Card {
		schema: None,
		id: "MIS-001".to_string(),
		card_type: "Mission".to_string(),
		card_subtype: None,
		name: mission_name.to_string(),
		description: "Root description".to_string(),
		version: "1.0.0".to_string(),
		status: None,
		boundary: None,
		notes: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let feature_card = super::Card {
		schema: None,
		id: card_id.to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: card_name.to_string(),
		description: "Feature description".to_string(),
		version: "1.0.0".to_string(),
		status: None,
		boundary: None,
		notes: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: vec![AuditLogEntry {
			timestamp: Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap(),
			editor: "Tester".to_string(),
			target: card_id.to_string(),
			change_type: AuditChangeType::Create,
		}],
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let model = Model {
		root_card,
		cards: vec![feature_card],
		audit_log,
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	model.write_markdown(&output_path)?;

	let mission_slug = super::sanitize_filename(mission_name);
	let card_slug = super::sanitize_filename(card_name);
	let mission_md = output_path.join(format!("MIS-001-{}.md", mission_slug));
	let card_md = output_path
		.join("MIS-001")
		.join("Feature")
		.join(format!("{}-{}.md", card_id, card_slug));

	if !mission_md.exists() {
		return Err(missing("missing mission markdown"));
	}
	if !card_md.exists() {
		return Err(missing("missing card markdown"));
	}

	let card_content = std::fs::read_to_string(card_md)?;
	if !card_content.contains("## Audit Log") {
		return Err(missing("missing audit log section"));
	}
	if !card_content.contains("Tester") {
		return Err(missing("missing audit log entry"));
	}

	Ok(())
}

#[test]
fn write_outputs_model_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let root_path = temp.path().join("MIS-001-Alpha.json");
	let mission_home = temp.path().join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;

	let root_card = super::Card {
		schema: None,
		id: "MIS-001".to_string(),
		card_type: "Mission".to_string(),
		card_subtype: None,
		name: "Mission Alpha".to_string(),
		description: "Root description".to_string(),
		version: "1.0.0".to_string(),
		status: None,
		boundary: None,
		notes: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: root_path.clone(),
		validation_errors: Vec::new(),
	};

	let feature_card = super::Card {
		schema: None,
		id: "FEA-001".to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: "Feature One".to_string(),
		description: "Feature description".to_string(),
		version: "1.0.0".to_string(),
		status: None,
		boundary: None,
		notes: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: mission_home.join("Feature").join("FEA-001.json"),
		validation_errors: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: Vec::new(),
		source_path: mission_home.join("AuditLog.json"),
		validation_errors: Vec::new(),
	};

	let model = Model {
		root_card,
		cards: vec![feature_card],
		audit_log,
		model_home: temp.path().to_path_buf(),
		mission_home,
	};

	model.write()?;

	let mission_files = std::fs::read_dir(temp.path())?
		.filter_map(|entry| entry.ok())
		.filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
		.any(|entry| entry.file_name().to_string_lossy().starts_with("MIS-001-"));
	if !mission_files {
		return Err(missing("missing root card file"));
	}

	let feature_path = temp
		.path()
		.join("MIS-001")
		.join("Feature")
		.join("FEA-001-Feature_One.json");
	if !feature_path.exists() {
		return Err(missing("missing feature card file"));
	}

	Ok(())
}
