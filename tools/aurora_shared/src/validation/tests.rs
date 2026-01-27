use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn audit_trail() -> crate::model::AuditTrail {
	crate::model::AuditTrail {
		version: "0.0.1".to_string(),
		hash: None,
		history: Vec::new(),
	}
}

fn link(target: &str, relationship: &str) -> crate::model::Link {
	crate::model::Link {
		target: target.to_string(),
		relationship: relationship.to_string(),
	}
}

fn card(
	id: &str,
	card_type: &str,
	card_subtype: Option<&str>,
	links: Vec<crate::model::Link>,
) -> crate::model::Card {
	crate::model::Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: card_subtype.map(|value| value.to_string()),
		name: id.to_string(),
		description: "test".to_string(),
		status: None,
		links,
		audit_trail: audit_trail(),
		attributes: Value::Null,
		source_path: None,
		extra: BTreeMap::new(),
	}
}

fn model_with_cards(cards: Vec<crate::model::Card>) -> Result<crate::model::AuroraModel> {
	let temp = tempfile::tempdir()?;
	let home = crate::ModelHome::new(temp.path())?;
	Ok(crate::model::AuroraModel::new(home, cards))
}

fn has_diagnostic(
	report: &crate::ValidationReport,
	code: &str,
	severity: crate::DiagnosticSeverity,
) -> bool {
	report
		.diagnostics
		.iter()
		.any(|diag| diag.code == code && diag.severity == severity)
}

fn instructions_fixture_dir() -> Result<PathBuf> {
	let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	for ancestor in crate_dir.ancestors() {
		let candidate = ancestor.join(".github/instructions");
		if candidate.is_dir() {
			return Ok(candidate);
		}
	}
	Err(std::io::Error::new(
		std::io::ErrorKind::NotFound,
		"Unable to locate .github/instructions",
	)
	.into())
}

fn env_guard() -> &'static Mutex<()> {
	static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
	GUARD.get_or_init(|| Mutex::new(()))
}

fn set_instruction_env(path: &Path) {
	// SAFETY: Tests guard env mutation and restore values after each use.
	unsafe {
		env::set_var("AURORA_INSTRUCTIONS_ROOT", path);
	}
}

fn restore_instruction_env(previous: Option<String>) {
	// SAFETY: Tests guard env mutation and restore values after each use.
	unsafe {
		if let Some(value) = previous {
			env::set_var("AURORA_INSTRUCTIONS_ROOT", value);
		} else {
			env::remove_var("AURORA_INSTRUCTIONS_ROOT");
		}
	}
}

#[test]
fn secret_without_owner_emits_error() -> Result<()> {
	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![link("AST-001", "includes")],
		),
		card("AST-001", "Asset", Some("Secret"), Vec::new()),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);
	assert!(has_diagnostic(
		&report,
		"SECRET_MISSING_OWNER",
		crate::DiagnosticSeverity::Error
	));
	Ok(())
}

#[test]
fn secret_owned_by_actor_passes() -> Result<()> {
	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![link("ACT-001", "involves")],
		),
		card("ACT-001", "Actor", None, vec![link("AST-001", "owns")]),
		card("AST-001", "Asset", Some("Secret"), Vec::new()),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);
	assert!(!has_diagnostic(
		&report,
		"SECRET_MISSING_OWNER",
		crate::DiagnosticSeverity::Error
	));
	assert!(!has_diagnostic(
		&report,
		"SECRET_OWNED_BY_NON_ACTOR",
		crate::DiagnosticSeverity::Warning
	));
	Ok(())
}

#[test]
fn secret_owned_by_non_actor_emits_warning_only() -> Result<()> {
	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![link("SYS-001", "necessitates")],
		),
		card("SYS-001", "System", None, vec![link("AST-001", "owns")]),
		card("AST-001", "Asset", Some("Secret"), Vec::new()),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);
	assert!(!has_diagnostic(
		&report,
		"SECRET_MISSING_OWNER",
		crate::DiagnosticSeverity::Error
	));
	assert!(has_diagnostic(
		&report,
		"SECRET_OWNED_BY_NON_ACTOR",
		crate::DiagnosticSeverity::Warning
	));
	Ok(())
}

#[test]
fn unknown_relationship_warns_against_matrix() -> Result<()> {
	let _guard = env_guard()
		.lock()
		.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "env guard poisoned"))?;
	let instructions = instructions_fixture_dir()?;
	let original = env::var("AURORA_INSTRUCTIONS_ROOT").ok();
	set_instruction_env(&instructions);

	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![link("DRI-001", "unsupported")],
		),
		card("DRI-001", "Driver", None, Vec::new()),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);

	restore_instruction_env(original);
	assert!(has_diagnostic(
		&report,
		"UNKNOWN_RELATIONSHIP",
		crate::DiagnosticSeverity::Warning
	));
	Ok(())
}

#[test]
fn relationship_source_mismatch_warns_against_matrix() -> Result<()> {
	let _guard = env_guard()
		.lock()
		.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "env guard poisoned"))?;
	let instructions = instructions_fixture_dir()?;
	let original = env::var("AURORA_INSTRUCTIONS_ROOT").ok();
	set_instruction_env(&instructions);

	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![link("ACT-001", "involves")],
		),
		card("ACT-001", "Actor", None, vec![link("REQ-001", "drives")]),
		card("REQ-001", "Requirement", None, Vec::new()),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);

	restore_instruction_env(original);
	assert!(has_diagnostic(
		&report,
		"RELATIONSHIP_SOURCE_NOT_ALLOWED",
		crate::DiagnosticSeverity::Warning
	));
	Ok(())
}

#[test]
fn new_relationships_allowed_by_matrix() -> Result<()> {
	let _guard = env_guard()
		.lock()
		.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "env guard poisoned"))?;
	let instructions = instructions_fixture_dir()?;
	let original = env::var("AURORA_INSTRUCTIONS_ROOT").ok();
	set_instruction_env(&instructions);

	let cards = vec![
		card(
			"MIS-001",
			"Mission",
			None,
			vec![
				link("APP-001", "includes"),
				link("COM-001", "includes"),
				link("ART-001", "includes"),
				link("DTS-001", "includes"),
				link("DEP-001", "includes"),
				link("NOD-001", "includes"),
				link("CNS-001", "includes"),
			],
		),
		card(
			"APP-001",
			"Application",
			None,
			vec![link("COM-001", "reverse proxies")],
		),
		card("COM-001", "Component", None, Vec::new()),
		card(
			"ART-001",
			"Artifact",
			None,
			vec![link("DTS-001", "persists to")],
		),
		card("DTS-001", "Data Store", None, Vec::new()),
		card(
			"DEP-001",
			"Deployment",
			None,
			vec![link("NOD-001", "provides")],
		),
		card("NOD-001", "Node", None, Vec::new()),
		card(
			"CNS-001",
			"Constraint",
			None,
			vec![link("APP-001", "limits")],
		),
	];
	let model = model_with_cards(cards)?;
	let report = crate::validate_model(&model);

	restore_instruction_env(original);
	for code in [
		"UNKNOWN_RELATIONSHIP",
		"RELATIONSHIP_SOURCE_NOT_ALLOWED",
		"RELATIONSHIP_TARGET_NOT_ALLOWED",
	] {
		assert!(!has_diagnostic(
			&report,
			code,
			crate::DiagnosticSeverity::Warning
		));
	}
	Ok(())
}
