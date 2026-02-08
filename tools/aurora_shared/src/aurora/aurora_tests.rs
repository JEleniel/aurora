use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{AuditChangeType, AuditLog, AuditLogEntry, Aurora, AuroraError, Card, Link, Model};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn write_json(path: &Path, value: &Value) -> Result<()> {
	let serialized = serde_json::to_string_pretty(value)?;
	std::fs::write(path, serialized)?;
	Ok(())
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

fn compact_schema() -> Value {
	json!({
		"type": "object",
		"additionalProperties": true
	})
}

fn audit_log_json() -> Value {
	json!({
		"$schema": "../Aurora.audit.schema.json",
		"history": []
	})
}

fn card_json(id: &str, card_type: &str, links: Vec<Value>) -> Value {
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

fn link_json(target: &str) -> Value {
	json!({
		"target": target,
		"relationship": "rel"
	})
}

fn write_schema_files(model_home: &Path) -> Result<()> {
	write_json(&model_home.join("Aurora.card.schema.json"), &card_schema())?;
	write_json(
		&model_home.join("Aurora.compact.schema.json"),
		&compact_schema(),
	)?;
	write_json(
		&model_home.join("Aurora.audit.schema.json"),
		&audit_schema(),
	)?;
	Ok(())
}

fn build_card(id: &str, card_type: &str, targets: &[&str]) -> Card {
	let links = targets
		.iter()
		.map(|target| Link {
			target: target.to_string(),
			relationship: "rel".to_string(),
		})
		.collect();

	Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: None,
		name: format!("{} name", id),
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
fn try_load_discovers_model_home() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_json(&mission_home.join("AuditLog.json"), &audit_log_json())?;

	let aurora = Aurora::try_load(temp.path())?;
	assert_eq!(aurora.models.len(), 1);
	assert_eq!(aurora.model_home, aurora_home);
	Ok(())
}

#[test]
fn check_registry_reports_unknown_card_type() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	let unknown_folder = mission_home.join("Unknown");
	std::fs::create_dir_all(&unknown_folder)?;

	write_json(
		&root_path,
		&card_json("MIS-001", "Mission", vec![link_json("UNK-001")]),
	)?;
	write_json(&mission_home.join("AuditLog.json"), &audit_log_json())?;
	write_json(
		&unknown_folder.join("UNK-001.json"),
		&card_json("UNK-001", "UnknownType", vec![]),
	)?;

	let aurora = Aurora::try_load(&aurora_home)?;
	let warnings = aurora.check_registry();
	assert!(
		warnings
			.iter()
			.any(|warning| warning.contains("unknown card type"))
	);
	Ok(())
}

#[test]
fn validate_reports_model_errors() {
	let root = build_card("MIS-001", "Mission", &["REQ-404"]);
	let model = Model {
		root_card: root,
		cards: Vec::new(),
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: PathBuf::new(),
			validation_errors: Vec::new(),
		},
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let aurora = Aurora {
		model_home: PathBuf::new(),
		models: vec![model],
		card_schema: Value::Null,
		compact_schema: Value::Null,
		audit_schema: Value::Null,
	};

	let errors = aurora.validate();
	assert!(errors.iter().any(|error| error.contains("broken link")));
}

#[test]
fn get_schema_validation_errors_returns_entries() {
	let mut root = build_card("MIS-001", "Mission", &[]);
	root.validation_errors = vec!["bad".to_string()];
	let model = Model {
		root_card: root,
		cards: Vec::new(),
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: PathBuf::new(),
			validation_errors: vec!["audit".to_string()],
		},
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let aurora = Aurora {
		model_home: PathBuf::new(),
		models: vec![model],
		card_schema: Value::Null,
		compact_schema: Value::Null,
		audit_schema: Value::Null,
	};

	let errors = aurora.get_schema_validation_errors();
	let entries = errors.get("MIS-001").expect("missing error entry");
	assert!(entries.iter().any(|entry| entry.contains("MIS-001")));
	assert!(entries.iter().any(|entry| entry.contains("AuditLog")));
}

#[test]
fn write_markdown_writes_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();

	let root = build_card("MIS-001", "Mission", &[]);
	let model = Model {
		root_card: root,
		cards: Vec::new(),
		audit_log: AuditLog {
			schema: None,
			history: vec![AuditLogEntry {
				timestamp: chrono::Utc::now(),
				editor: "Tester".to_string(),
				target: "MIS-001".to_string(),
				change_type: AuditChangeType::Create,
			}],
			source_path: PathBuf::new(),
			validation_errors: Vec::new(),
		},
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let aurora = Aurora {
		model_home: PathBuf::new(),
		models: vec![model],
		card_schema: Value::Null,
		compact_schema: Value::Null,
		audit_schema: Value::Null,
	};

	aurora.write_markdown(&output_path)?;

	let has_readme = std::fs::read_dir(&output_path)?
		.filter_map(|entry| entry.ok())
		.any(|entry| {
			entry
				.file_name()
				.to_string_lossy()
				.starts_with("README-MIS-001-")
		});
	if !has_readme {
		return Err("missing README output".into());
	}

	Ok(())
}

#[test]
fn write_compact_writes_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();

	let root = build_card("MIS-001", "Mission", &[]);
	let model = Model {
		root_card: root,
		cards: Vec::new(),
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: PathBuf::new(),
			validation_errors: Vec::new(),
		},
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let aurora = Aurora {
		model_home: PathBuf::new(),
		models: vec![model],
		card_schema: Value::Null,
		compact_schema: Value::Null,
		audit_schema: Value::Null,
	};

	aurora.write_compact(&output_path)?;

	let compact_path = output_path.join("MIS-001").join("Compact.json");
	if !compact_path.exists() {
		return Err("missing compact output".into());
	}

	let compact_content = std::fs::read_to_string(compact_path)?;
	assert!(compact_content.contains("\"cards\""));
	Ok(())
}

#[test]
fn try_load_rejects_invalid_home() {
	let temp = tempfile::tempdir().expect("temp dir");
	let result = Aurora::try_load(temp.path());
	assert!(matches!(result, Err(AuroraError::InvalidAuroraHome(_))));
}
