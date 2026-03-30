use std::path::PathBuf;

use serde_json::{Value, json};

use super::Model;
use crate::AuditLog;
use crate::registry::CardRegistry;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn missing(message: &str) -> Box<dyn std::error::Error> {
	Box::new(std::io::Error::other(message))
}

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

fn write_json(path: &PathBuf, value: &Value) -> Result<()> {
	let serialized = serde_json::to_string_pretty(value)?;
	std::fs::write(path, serialized)?;
	Ok(())
}

#[path = "model_tests1.rs"]
mod model_tests1;

#[test]
fn write_markdown_skips_empty_card_types_in_index() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();

	let root_card = super::Card {
		schema: None,
		id: "MIS-001".to_string(),
		card_type: "Mission".to_string(),
		card_subtype: None,
		name: "Mission".to_string(),
		description: "Root description".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let only_card = super::Card {
		schema: None,
		id: "FEA-001".to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: "Feature One".to_string(),
		description: "desc".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let model = Model {
		root_card,
		cards: vec![only_card],
		audit_log,
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	model.write_markdown(&output_path)?;

	let readme_path = output_path.join("README-MIS-001-Mission.md");
	let readme = std::fs::read_to_string(readme_path)?;
	if !readme.contains("### Feature") {
		return Err(missing("expected Feature section in index"));
	}

	// Find a card type from the registry that is not present and ensure it is not emitted.
	let model_configuration = read_testdata("modelconfiguration/model_tests_registry.json");
	let view_configuration = read_testdata("modelconfiguration/model_tests_viewconfiguration.json");
	let registry =
		CardRegistry::try_new_from_configurations(&model_configuration, &view_configuration)?;
	let forbidden = registry
		.definitions
		.iter()
		.map(|def| def.card_type.as_str())
		.find(|t| *t != "Feature" && *t != "Mission")
		.ok_or_else(|| {
			missing("registry should contain a card type besides Feature and Mission")
		})?;
	if readme.contains(&format!("### {forbidden}\n")) {
		return Err(missing("index should skip empty card types"));
	}

	Ok(())
}

#[test]
fn write_markdown_links_are_relative_and_point_to_slugged_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();
	let mission_name = "Mission Name!";

	let root_card = super::Card {
		schema: None,
		id: "MIS-001".to_string(),
		card_type: "Mission".to_string(),
		card_subtype: None,
		name: mission_name.to_string(),
		description: "Root description".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: vec![super::Link {
			target: "FEA-001".to_string(),
			relationship: "rel".to_string(),
		}],
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let feature = super::Card {
		schema: None,
		id: "FEA-001".to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: "Feature: One?".to_string(),
		description: "desc".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: vec![super::Link {
			target: "MIS-001".to_string(),
			relationship: "back".to_string(),
		}],
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let model = Model {
		root_card,
		cards: vec![feature],
		audit_log,
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	model.write_markdown(&output_path)?;

	let mission_slug = super::sanitize_filename(mission_name);
	let mission_md = output_path.join(format!("MIS-001-{}.md", mission_slug));
	let mission_contents = std::fs::read_to_string(&mission_md)?;
	if !mission_contents.contains("- rel [FEA-001](MIS-001/Feature/FEA-001-Feature_One.md)") {
		return Err(missing("mission links are not correctly relative"));
	}

	let feature_md = output_path
		.join("MIS-001")
		.join("Feature")
		.join("FEA-001-Feature_One.md");
	let feature_contents = std::fs::read_to_string(&feature_md)?;
	if !feature_contents.contains(&format!(
		"- back [MIS-001](../../MIS-001-{}.md)",
		mission_slug
	)) {
		return Err(missing("card links to mission are not correctly relative"));
	}

	Ok(())
}

#[test]
fn write_outputs_model_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let schema_dir = temp.path().join("schemas");
	std::fs::create_dir_all(&schema_dir)?;
	std::fs::write(
		schema_dir.join("Aurora.card.schema.json"),
		serde_json::to_string_pretty(&card_schema())?,
	)?;
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
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: vec![super::Link {
			target: "FEA-001".to_string(),
			relationship: "rel".to_string(),
		}],
		source_path: root_path.clone(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let feature_card = super::Card {
		schema: None,
		id: "FEA-001".to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: "Feature One".to_string(),
		description: "Feature description".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links: Vec::new(),
		source_path: mission_home.join("Feature").join("FEA-001.json"),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: Vec::new(),
		source_path: mission_home.join("AuditLog.ndjson"),
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
	let feature_contents = std::fs::read_to_string(feature_path)?;
	if !feature_contents.contains("\"$schema\": \"../../schemas/Aurora.card.schema.json\"") {
		return Err(missing(
			"missing inferred schema reference in feature card file",
		));
	}

	Ok(())
}
