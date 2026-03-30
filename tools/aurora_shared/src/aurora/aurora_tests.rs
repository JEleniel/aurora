use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::GzEncoder;

use serde_json::{Value, json};
use std::io::Write;

use super::{AuditChangeType, AuditLog, AuditLogEntry, Aurora, AuroraError, Card, Link, Model};
use crate::registry::{CardRegistry, ModelConfiguration, ViewConfiguration, ViewRegistry};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_testdata(rel_path: &str) -> String {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("testdata")
		.join(rel_path);
	std::fs::read_to_string(&path)
		.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
	let serialized = serde_json::to_string_pretty(value)?;
	std::fs::write(path, serialized)?;
	Ok(())
}

fn card_json(id: &str, card_type: &str, links: Vec<Value>) -> Value {
	json!({
		"$schema": "./schemas/Aurora.card.schema.json",
		"id": id,
		"card_type": card_type,
		"name": id,
		"description": "test",
		"links": links
	})
}

fn link_json(target: &str) -> Value {
	json!({
		"target": target,
		"relationship": "rel"
	})
}

fn write_schema_files(model_home: &Path) -> Result<()> {
	let schemas = model_home.join("schemas");
	let reference = model_home.join("reference");
	std::fs::create_dir_all(&schemas)?;
	std::fs::create_dir_all(&reference)?;

	for schema_file in [
		"Aurora.card.schema.json",
		"Aurora.compact.schema.json",
		"Aurora.audit.schema.json",
		"Aurora.modelconfiguration.schema.json",
		"Aurora.viewconfiguration.schema.json",
	] {
		std::fs::write(
			schemas.join(schema_file),
			read_testdata(&format!("model_home/schemas/{schema_file}")),
		)?;
	}
	std::fs::write(
		reference.join("Aurora.modelconfiguration.json"),
		read_testdata("model_home/reference/Aurora.modelconfiguration.json"),
	)?;
	std::fs::write(
		reference.join("Aurora.viewconfiguration.json"),
		read_testdata("model_home/reference/Aurora.viewconfiguration.json"),
	)?;
	write_svg_template_svgz(&reference)?;
	Ok(())
}

fn write_schema_files_without_svg_template(model_home: &Path) -> Result<()> {
	let schemas = model_home.join("schemas");
	let reference = model_home.join("reference");
	std::fs::create_dir_all(&schemas)?;
	std::fs::create_dir_all(&reference)?;

	for schema_file in [
		"Aurora.card.schema.json",
		"Aurora.compact.schema.json",
		"Aurora.audit.schema.json",
		"Aurora.modelconfiguration.schema.json",
		"Aurora.viewconfiguration.schema.json",
	] {
		std::fs::write(
			schemas.join(schema_file),
			read_testdata(&format!("model_home/schemas/{schema_file}")),
		)?;
	}
	std::fs::write(
		reference.join("Aurora.modelconfiguration.json"),
		read_testdata("model_home/reference/Aurora.modelconfiguration.json"),
	)?;
	std::fs::write(
		reference.join("Aurora.viewconfiguration.json"),
		read_testdata("model_home/reference/Aurora.viewconfiguration.json"),
	)?;
	Ok(())
}

fn write_svg_template_svgz(reference_dir: &Path) -> Result<()> {
	let svg_template = read_testdata("svg/template_minimal.svg");
	let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
	encoder.write_all(svg_template.as_bytes())?;
	let data = encoder.finish()?;
	std::fs::write(reference_dir.join("SVGTemplate.svgz"), data)?;
	Ok(())
}

fn write_audit_log(path: &Path) -> Result<()> {
	std::fs::write(path, "")?;
	Ok(())
}

fn test_aurora(models: Vec<Model>) -> Aurora {
	let model_configuration_text =
		read_testdata("model_home/reference/Aurora.modelconfiguration.json");
	let model_configuration: ModelConfiguration =
		serde_json::from_str(&model_configuration_text).expect("model configuration");
	let view_configuration_text =
		read_testdata("model_home/reference/Aurora.viewconfiguration.json");
	let view_configuration: ViewConfiguration =
		serde_json::from_str(&view_configuration_text).expect("view configuration");
	let card_registry =
		CardRegistry::try_new_from_structs(model_configuration.clone(), view_configuration.clone())
			.expect("card registry");
	let view_registry = ViewRegistry::try_new_from_struct(&view_configuration);
	let svg_template = read_testdata("svg/template_minimal.svg");

	Aurora {
		model_home: PathBuf::new(),
		models,
		card_schema: Value::Null,
		compact_schema: Value::Null,
		audit_schema: Value::Null,
		modelconfiguration_schema: Value::Null,
		viewconfiguration_schema: Value::Null,
		model_configuration,
		view_configuration,
		card_registry,
		view_registry,
		svg_template,
		load_warnings: Vec::new(),
		load_validation_errors: Vec::new(),
		_audit_log_locks: Vec::new(),
	}
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
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: super::Attributes::new(),
		external_references: Vec::new(),
		links,
		source_path: PathBuf::from(format!("{}.json", id)),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
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
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let aurora = Aurora::try_load(temp.path())?;
	assert_eq!(aurora.models.len(), 1);
	assert_eq!(aurora.model_home, aurora_home);
	Ok(())
}

#[test]
fn try_load_allows_missing_svg_template_for_validation() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files_without_svg_template(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let aurora = Aurora::try_load(temp.path())?;
	assert!(aurora.svg_template.trim().is_empty());
	assert!(
		aurora
			.load_warnings
			.iter()
			.any(|warning| warning.contains("Missing SVG template")),
		"expected missing-template warning"
	);
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
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;
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

	let aurora = test_aurora(vec![model]);

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

	let aurora = test_aurora(vec![model]);

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
				changes: Vec::new(),
			}],
			source_path: PathBuf::new(),
			validation_errors: Vec::new(),
		},
		model_home: PathBuf::new(),
		mission_home: PathBuf::new(),
	};

	let aurora = test_aurora(vec![model]);

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

	let aurora = test_aurora(vec![model]);

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

#[test]
fn try_load_for_update_rejects_second_writer_for_same_model_home() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let first = Aurora::try_load_for_update(temp.path())?;
	let second = Aurora::try_load_for_update(temp.path());
	assert!(matches!(
		second,
		Err(AuroraError::ModelError(super::ModelError::ModelLocked(_)))
	));
	drop(first);

	let reopened = Aurora::try_load_for_update(temp.path())?;
	assert_eq!(reopened.models.len(), 1);
	Ok(())
}

#[test]
fn write_model_configuration_creates_one_time_config_backup() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let mut aurora = Aurora::try_load_for_update(temp.path())?;
	let mut updated = aurora.model_configuration.clone();
	updated.cards[0].description = "updated description".to_string();

	aurora.write_model_configuration(updated)?;

	let backup_path = aurora_home
		.join("backups")
		.join("MIS-001-config-backup.zip");
	assert!(backup_path.exists());
	let written = std::fs::read_to_string(
		aurora_home
			.join("reference")
			.join("Aurora.modelconfiguration.json"),
	)?;
	assert!(written.contains("updated description"));
	Ok(())
}

#[test]
fn write_view_configuration_preserves_existing_config_backup() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let backup_path = aurora_home
		.join("backups")
		.join("MIS-001-config-backup.zip");
	std::fs::create_dir_all(backup_path.parent().expect("backup parent"))?;
	std::fs::write(&backup_path, b"original backup")?;

	let mut aurora = Aurora::try_load_for_update(temp.path())?;
	let mut updated = aurora.view_configuration.clone();
	updated.available_icons.push("new-icon".to_string());

	aurora.write_view_configuration(updated)?;

	assert_eq!(std::fs::read(&backup_path)?, b"original backup");
	Ok(())
}

#[test]
fn write_model_configuration_blocks_write_when_backup_fails() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let aurora_home = temp.path().join("aurora");
	std::fs::create_dir_all(&aurora_home)?;
	write_schema_files(&aurora_home)?;

	let root_path = aurora_home.join("MIS-001-Alpha.json");
	let mission_home = aurora_home.join("MIS-001");
	std::fs::create_dir_all(&mission_home)?;
	write_json(&root_path, &card_json("MIS-001", "Mission", vec![]))?;
	write_audit_log(&mission_home.join("AuditLog.ndjson"))?;

	let original_path = aurora_home
		.join("reference")
		.join("Aurora.modelconfiguration.json");
	let original_contents = std::fs::read_to_string(&original_path)?;
	std::fs::write(aurora_home.join("backups"), b"not a directory")?;

	let mut aurora = Aurora::try_load_for_update(temp.path())?;
	let mut updated = aurora.model_configuration.clone();
	updated.cards[0].description = "blocked write".to_string();

	let result = aurora.write_model_configuration(updated);
	assert!(matches!(result, Err(AuroraError::BackupError(_))));
	assert_eq!(std::fs::read_to_string(&original_path)?, original_contents);
	Ok(())
}
