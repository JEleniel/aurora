use std::collections::{HashSet, VecDeque};

use serde::Serialize;

use crate::model::{AuroraModel, Card};

/// Severity levels emitted by the validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiagnosticSeverity {
	Error,
	Warning,
	Info,
}

/// Structured validation diagnostic message.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationDiagnostic {
	pub severity: DiagnosticSeverity,
	pub code: &'static str,
	pub message: String,
	pub card_id: Option<String>,
	pub path: Option<String>,
}

impl ValidationDiagnostic {
	fn error(code: &'static str, message: impl Into<String>) -> Self {
		ValidationDiagnostic {
			severity: DiagnosticSeverity::Error,
			code,
			message: message.into(),
			card_id: None,
			path: None,
		}
	}

	fn with_card(mut self, card: &Card) -> Self {
		self.card_id = Some(card.id.clone());
		self
	}

	fn with_path(mut self, path: impl Into<String>) -> Self {
		self.path = Some(path.into());
		self
	}
}

/// Collection of diagnostics returned by validation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationReport {
	pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
	pub fn has_errors(&self) -> bool {
		self.diagnostics
			.iter()
			.any(|diag| matches!(diag.severity, DiagnosticSeverity::Error))
	}
}

/// Validate an Aurora model according to basic invariants.
pub fn validate_model(model: &AuroraModel) -> ValidationReport {
	let mut report = ValidationReport::default();
	if model.is_empty() {
		report.diagnostics.push(ValidationDiagnostic::error(
			"EMPTY_MODEL",
			"No cards were loaded",
		));
		return report;
	}

	check_duplicate_ids(model, &mut report);
	let mission_ids = collect_mission_ids(model, &mut report);
	let known_ids: HashSet<String> = model.iter_cards().map(|card| card.id.clone()).collect();
	check_link_targets(model, &known_ids, &mut report);
	if let Some(root_id) = mission_ids.first() {
		check_reachability(model, root_id, &mut report);
	}
	check_secret_ownership(model, &mut report);

	report
}

fn is_case_insensitive_match(value: &str, expected: &str) -> bool {
	value.trim().eq_ignore_ascii_case(expected)
}

fn is_deleted(card: &Card) -> bool {
	card.status
		.as_deref()
		.is_some_and(|status| is_case_insensitive_match(status, "Deleted"))
}

fn is_secret_asset(card: &Card) -> bool {
	if !is_case_insensitive_match(&card.card_type, "Asset") {
		return false;
	}
	card.card_subtype
		.as_deref()
		.is_some_and(|subtype| is_case_insensitive_match(subtype, "Secret"))
}

fn check_duplicate_ids(model: &AuroraModel, report: &mut ValidationReport) {
	for (id, indices) in model.index_by_id() {
		if indices.len() > 1 {
			report.diagnostics.push(ValidationDiagnostic {
				severity: DiagnosticSeverity::Error,
				code: "DUPLICATE_CARD_ID",
				message: format!(
					"Card id {id} appears {} times; ids must be unique",
					indices.len()
				),
				card_id: Some(id.clone()),
				path: None,
			});
		}
	}
}

fn collect_mission_ids(model: &AuroraModel, report: &mut ValidationReport) -> Vec<String> {
	let mut missions: Vec<String> = model
		.iter_cards()
		.filter(|card| card.card_type == "Mission")
		.map(|card| card.id.clone())
		.collect();

	if missions.is_empty() {
		report.diagnostics.push(ValidationDiagnostic::error(
			"MISSING_MISSION",
			"No mission card was found",
		));
	} else if missions.len() > 1 {
		report.diagnostics.push(ValidationDiagnostic {
			severity: DiagnosticSeverity::Warning,
			code: "MULTIPLE_MISSIONS",
			message: format!(
				"{} mission cards were found; models typically define exactly one mission",
				missions.len()
			),
			card_id: None,
			path: None,
		});
		missions.sort();
	}

	missions
}

fn check_link_targets(
	model: &AuroraModel,
	known_ids: &HashSet<String>,
	report: &mut ValidationReport,
) {
	for card in model.iter_cards() {
		for (idx, link) in card.links.iter().enumerate() {
			if !known_ids.contains(&link.target) {
				report.diagnostics.push(
					ValidationDiagnostic::error(
						"MISSING_LINK_TARGET",
						format!(
							"Link uses relationship '{}' but target '{}' was not found",
							link.relationship, link.target
						),
					)
					.with_card(card)
					.with_path(format!("links[{idx}].target")),
				);
			}
			if link.relationship.trim().is_empty() {
				report.diagnostics.push(
					ValidationDiagnostic::error(
						"EMPTY_RELATIONSHIP",
						"Relationship name must not be empty",
					)
					.with_card(card)
					.with_path(format!("links[{idx}].relationship")),
				);
			}
		}
	}
}

fn check_reachability(model: &AuroraModel, root_id: &str, report: &mut ValidationReport) {
	let mut visited: HashSet<String> = HashSet::new();
	let mut queue: VecDeque<String> = VecDeque::new();
	queue.push_back(root_id.to_string());

	while let Some(id) = queue.pop_front() {
		if !visited.insert(id.clone()) {
			continue;
		}
		if let Some(card) = model.get(&id) {
			for link in &card.links {
				queue.push_back(link.target.clone());
			}
		}
	}

	for card in model.iter_cards() {
		if !visited.contains(&card.id) {
			report.diagnostics.push(
				ValidationDiagnostic::error(
					"UNREACHABLE_CARD",
					format!(
						"Card {} ({}) is not reachable from the mission root",
						card.id, card.card_type
					),
				)
				.with_card(card),
			);
		}
	}
}

fn check_secret_ownership(model: &AuroraModel, report: &mut ValidationReport) {
	let mut owned_by_any: HashSet<String> = HashSet::new();
	let mut owned_by_actor: HashSet<String> = HashSet::new();
	for source in model.iter_cards() {
		for link in &source.links {
			if !is_case_insensitive_match(&link.relationship, "owns") {
				continue;
			}
			owned_by_any.insert(link.target.clone());
			if is_case_insensitive_match(&source.card_type, "Actor") {
				owned_by_actor.insert(link.target.clone());
			}
		}
	}

	for card in model.iter_cards() {
		if is_deleted(card) || !is_secret_asset(card) {
			continue;
		}

		if !owned_by_any.contains(&card.id) {
			report.diagnostics.push(
				ValidationDiagnostic::error(
					"SECRET_MISSING_OWNER",
					"Secret assets must be owned via an incoming 'owns' relationship",
				)
				.with_card(card),
			);
			continue;
		}

		if !owned_by_actor.contains(&card.id) {
			report.diagnostics.push(ValidationDiagnostic {
				severity: DiagnosticSeverity::Warning,
				code: "SECRET_OWNED_BY_NON_ACTOR",
				message: "Secret assets should be owned by an Actor card".to_string(),
				card_id: Some(card.id.clone()),
				path: None,
			});
		}
	}
}

#[cfg(test)]
mod tests;
