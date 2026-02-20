use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};

use super::{Model, ModelError};
use crate::registry::CardRegistry;
use crate::{AuditChangeType, AuditLog, AuditLogEntry};

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
	std::fs::write(mission_home.join("AuditLog.ndjson"), "")?;
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
	std::fs::write(mission_home.join("AuditLog.ndjson"), "")?;

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
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let feature_card = super::Card {
		schema: None,
		id: card_id.to_string(),
		card_type: "Feature".to_string(),
		card_subtype: None,
		name: card_name.to_string(),
		description: "Feature description".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let audit_log = AuditLog {
		schema: None,
		history: vec![AuditLogEntry {
			timestamp: Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap(),
			editor: "Tester".to_string(),
			target: card_id.to_string(),
			change_type: AuditChangeType::Create,
			changes: Vec::new(),
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
fn write_markdown_embeds_views_from_mission_views_dir() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let output_path = temp.path().to_path_buf();
	let mission_name = "Mission Alpha";

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
		cards: Vec::new(),
		audit_log,
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let views_dir = output_path.join("MIS-001").join("Views");
	std::fs::create_dir_all(&views_dir)?;
	std::fs::write(views_dir.join("Process.svg"), "<svg></svg>")?;

	model.write_markdown(&output_path)?;

	let mission_slug = super::sanitize_filename(mission_name);
	let readme_path = output_path.join(format!("README-MIS-001-{}.md", mission_slug));
	let readme = std::fs::read_to_string(readme_path)?;
	if !readme.contains("![Process.svg](MIS-001/Views/Process.svg)") {
		return Err(missing("missing view embed"));
	}
	if !readme.contains("## Views\n\n![Process.svg](MIS-001/Views/Process.svg)\n\n## Card Index") {
		return Err(missing("views section lost surrounding blank lines"));
	}

	Ok(())
}

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
	let registry = CardRegistry::try_new_from_model_configuration(&model_configuration)?;
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
		links: Vec::new(),
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

	Ok(())
}
