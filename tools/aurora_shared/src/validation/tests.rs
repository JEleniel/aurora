use std::collections::BTreeMap;

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
