use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{EditCommand, EditHistory, EditHistoryError};
use crate::{Attributes, AuditLog, Link, Model, ModelError};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn card_schema() -> Value {
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": ["$schema", "id", "card_type", "name", "description", "links", "version"],
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

fn write_schema(model_home: &Path) -> Result<()> {
	let schema_dir = model_home.join("schemas");
	std::fs::create_dir_all(&schema_dir)?;
	std::fs::write(
		schema_dir.join("Aurora.card.schema.json"),
		serde_json::to_string_pretty(&card_schema())?,
	)?;
	Ok(())
}

fn make_model(model_home: &Path, description: &str, root_target: &str) -> Model {
	let mission_home = model_home.join("MIS-001");
	let feature_path = mission_home
		.join("Feature")
		.join("FEA-001-Feature_One.json");

	Model {
		root_card: crate::Card {
			schema: None,
			id: "MIS-001".to_string(),
			card_type: "Mission".to_string(),
			card_subtype: None,
			name: "Mission Alpha".to_string(),
			description: "Root description".to_string(),
			version: Some("1.0.0".to_string()),
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: Attributes::new(),
			external_references: Vec::new(),
			links: vec![Link {
				target: root_target.to_string(),
				relationship: "rel".to_string(),
			}],
			source_path: model_home.join("MIS-001-Mission_Alpha.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		},
		cards: vec![crate::Card {
			schema: None,
			id: "FEA-001".to_string(),
			card_type: "Feature".to_string(),
			card_subtype: None,
			name: "Feature One".to_string(),
			description: description.to_string(),
			version: Some("1.0.0".to_string()),
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: Attributes::new(),
			external_references: Vec::new(),
			links: Vec::new(),
			source_path: feature_path,
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		}],
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: mission_home.join("AuditLog.ndjson"),
			validation_errors: Vec::new(),
		},
		model_home: model_home.to_path_buf(),
		mission_home,
	}
}

fn feature_path(model_home: &Path) -> PathBuf {
	model_home
		.join("MIS-001")
		.join("Feature")
		.join("FEA-001-Feature_One.json")
}

#[test]
fn push_reports_undo_redo_availability() {
	let model = make_model(Path::new("/tmp"), "before", "FEA-001");
	let mut updated = model.clone();
	updated.cards[0].description = "after".to_string();

	let mut history = EditHistory::new();
	assert!(!history.can_undo());
	assert!(!history.can_redo());

	history.push(EditCommand::new(&model, &updated));

	assert!(history.can_undo());
	assert!(!history.can_redo());
}

#[test]
fn history_discards_oldest_entry_when_capacity_is_exceeded() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;

	let mut current = make_model(temp.path(), "version 0", "FEA-001");
	current.write()?;

	let mut history = EditHistory::with_capacity(2);
	for version in 1..=3 {
		let mut next = current.clone();
		next.cards[0].description = format!("version {version}");
		history.push(EditCommand::new(&current, &next));
		current = next;
	}
	current.write()?;

	history.undo(&mut current)?;
	assert_eq!(current.cards[0].description, "version 2");
	history.undo(&mut current)?;
	assert_eq!(current.cards[0].description, "version 1");
	assert!(!history.can_undo());

	match history.undo(&mut current) {
		Err(EditHistoryError::NothingToUndo) => Ok(()),
		other => Err(format!("expected empty-history error, got {other:?}").into()),
	}
}

#[test]
fn undo_and_redo_persist_model_snapshots() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;

	let before = make_model(temp.path(), "before change", "FEA-001");
	before.write()?;

	let mut current = before.clone();
	current.cards[0].description = "after change".to_string();
	current.write()?;

	let mut history = EditHistory::new();
	history.push(EditCommand::new(&before, &current));

	history.undo(&mut current)?;
	assert_eq!(current.cards[0].description, "before change");
	let persisted_before = std::fs::read_to_string(feature_path(temp.path()))?;
	assert!(persisted_before.contains("before change"));
	assert!(history.can_redo());

	history.redo(&mut current)?;
	assert_eq!(current.cards[0].description, "after change");
	let persisted_after = std::fs::read_to_string(feature_path(temp.path()))?;
	assert!(persisted_after.contains("after change"));
	assert!(history.can_undo());
	Ok(())
}

#[test]
fn undo_rolls_back_in_memory_and_on_disk_when_validation_fails() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;

	let invalid_before = make_model(temp.path(), "before change", "REQ-999");
	let mut current = make_model(temp.path(), "after change", "FEA-001");
	current.write()?;

	let mut history = EditHistory::new();
	history.push(EditCommand::new(&invalid_before, &current));

	let error = history
		.undo(&mut current)
		.expect_err("undo should reject invalid snapshot");
	match error {
		EditHistoryError::ModelError(ModelError::ValidationErrors(errors)) => {
			assert!(errors.iter().any(|entry| entry.contains("broken link")));
		}
		other => return Err(format!("expected validation error, got {other:?}").into()),
	}

	assert_eq!(current.root_card.links[0].target, "FEA-001");
	assert_eq!(current.cards[0].description, "after change");
	assert!(history.can_undo());
	assert!(!history.can_redo());

	let persisted = std::fs::read_to_string(feature_path(temp.path()))?;
	assert!(persisted.contains("after change"));
	Ok(())
}
