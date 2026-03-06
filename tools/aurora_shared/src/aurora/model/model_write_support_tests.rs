use serde_json::{Value, json};

use super::{Model, ModelError};
use crate::{Attributes, AuditLog, Link};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

fn write_schema(model_home: &std::path::Path) -> Result<()> {
	let schema_dir = model_home.join("schemas");
	std::fs::create_dir_all(&schema_dir)?;
	std::fs::write(
		schema_dir.join("Aurora.card.schema.json"),
		serde_json::to_string_pretty(&card_schema())?,
	)?;
	Ok(())
}

#[test]
fn write_rejects_invalid_model_without_persisting_any_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	write_schema(temp.path())?;
	let mission_home = temp.path().join("MIS-001");

	let root_card = super::Card {
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
			target: "REQ-999".to_string(),
			relationship: "rel".to_string(),
		}],
		source_path: temp.path().join("MIS-001-Mission_Alpha.json"),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let model = Model {
		root_card,
		cards: Vec::new(),
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: mission_home.join("AuditLog.ndjson"),
			validation_errors: Vec::new(),
		},
		model_home: temp.path().to_path_buf(),
		mission_home,
	};

	let error = model.write().expect_err("write should fail");
	match error {
		ModelError::ValidationErrors(errors) => {
			assert!(errors.iter().any(|error| error.contains("broken link")));
		}
		other => panic!("expected validation error, got {other:?}"),
	}
	assert!(!temp.path().join("MIS-001-Mission_Alpha.json").exists());
	Ok(())
}
