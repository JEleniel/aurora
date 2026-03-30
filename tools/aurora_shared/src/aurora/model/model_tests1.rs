use chrono::{TimeZone, Utc};
use std::path::PathBuf;

use super::super::{Attributes, Card, sanitize_filename};
use super::{
	Model, Result, audit_schema, card_json, card_json_with_links, card_schema, missing, write_json,
};
use crate::{AuditChangeType, AuditLog, AuditLogEntry, ModelError};

#[test]
fn sanitize_filename_strips_symbols() {
	let input = "Alpha- Beta!! Gamma__ Delta";
	let expected = "Alpha_Beta_Gamma_Delta";
	let actual = sanitize_filename(input);
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

	let root_card = Card {
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
		attributes: Attributes::new(),
		external_references: Vec::new(),
		links: Vec::new(),
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	};

	let feature_card = Card {
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
		attributes: Attributes::new(),
		external_references: Vec::new(),
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

	let mission_slug = sanitize_filename(mission_name);
	let card_slug = sanitize_filename(card_name);
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

	let root_card = Card {
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
		attributes: Attributes::new(),
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
		cards: Vec::new(),
		audit_log,
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let views_dir = output_path.join("MIS-001").join("Views");
	std::fs::create_dir_all(&views_dir)?;
	std::fs::write(views_dir.join("Process.svg"), "<svg></svg>")?;

	model.write_markdown(&output_path)?;

	let mission_slug = sanitize_filename(mission_name);
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
