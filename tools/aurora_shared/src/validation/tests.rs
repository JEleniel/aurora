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
				link("DRI-001", "establishes"),
				link("APP-001", "necessitates"),
				link("DEP-001", "necessitates"),
				link("ACT-001", "involves"),
				link("ACT-002", "involves"),
				link("BND-001", "includes"),
			],
		),
		card(
			"BND-001",
			"Boundary",
			None,
			vec![link("APP-001", "contains")],
		),
		card("DRI-001", "Driver", None, vec![link("REQ-001", "drives")]),
		card("REQ-001", "Requirement", None, Vec::new()),
		card(
			"APP-001",
			"Application",
			None,
			vec![link("COM-001", "comprises"), link("TES-001", "implements")],
		),
		card(
			"COM-001",
			"Component",
			None,
			vec![
				link("INT-001", "calls"),
				link("ART-001", "generates"),
				link("FEA-001", "implements"),
				link("CLS-001", "implements"),
				link("STM-001", "runs"),
			],
		),
		card("INT-001", "Interface", None, Vec::new()),
		card("FEA-001", "Feature", None, vec![link("CAP-001", "enables")]),
		card(
			"CAP-001",
			"Capability",
			None,
			vec![link("REQ-001", "satisfies"), link("PRO-001", "runs")],
		),
		card(
			"PRO-001",
			"Process",
			None,
			vec![link("ATV-001", "starts with")],
		),
		card(
			"ATV-001",
			"Activity",
			None,
			vec![link("EVT-001", "receives"), link("ACT-001", "triggers")],
		),
		card("EVT-001", "Event", None, vec![link("PRD-001", "triggers")]),
		card(
			"PRD-001",
			"Condition",
			None,
			vec![link("STE-001", "transitions (true/false)")],
		),
		card(
			"STM-001",
			"State Machine",
			None,
			vec![link("STA-001", "starts in")],
		),
		card(
			"STA-001",
			"State",
			None,
			vec![link("PRD-001", "transitions to")],
		),
		card("STE-001", "State", None, Vec::new()),
		card(
			"ART-001",
			"Artifact",
			None,
			vec![link("AST-001", "is"), link("DTS-001", "persists to")],
		),
		card("AST-001", "Asset", None, Vec::new()),
		card(
			"ACT-001",
			"Actor",
			None,
			vec![
				link("STR-001", "desires"),
				link("ADR-001", "makes"),
				link("CTL-001", "implements"),
				link("AST-001", "owns"),
				link("ATV-001", "performs"),
				link("APP-001", "uses"),
			],
		),
		card("STR-001", "Story", None, vec![link("CNS-001", "implies")]),
		card(
			"CNS-001",
			"Constraint",
			None,
			vec![link("REQ-001", "limits")],
		),
		card("ADR-001", "ADR", None, vec![link("REQ-001", "documents")]),
		card(
			"CTL-001",
			"Control",
			None,
			vec![link("RIS-001", "mitigates"), link("AST-001", "protects")],
		),
		card("ACT-002", "Actor", None, vec![link("THR-001", "presents")]),
		card("THR-001", "Threat", None, vec![link("RIS-001", "presents")]),
		card("RIS-001", "Risk", None, Vec::new()),
		card(
			"DEP-001",
			"Deployment",
			None,
			vec![link("NOD-001", "deploys")],
		),
		card(
			"NOD-001",
			"Node",
			None,
			vec![
				link("NIN-001", "instantiates"),
				link("COM-001", "hosts"),
				link("DTS-001", "hosts"),
			],
		),
		card(
			"NIN-001",
			"Node Instance",
			None,
			vec![link("DTS-001", "hosts")],
		),
		card(
			"DTS-001",
			"Data Store",
			None,
			vec![link("INT-002", "exposes")],
		),
		card("INT-002", "Interface", None, Vec::new()),
		card("TES-001", "Test", None, Vec::new()),
		card("CLS-001", "Class", None, Vec::new()),
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
