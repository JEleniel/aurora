use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use serde::Serialize;

use crate::model::{AuroraModel, Card};
use crate::registry::{CardDefinition, RelationshipDefinition};

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

	fn warning(code: &'static str, message: impl Into<String>) -> Self {
		ValidationDiagnostic {
			severity: DiagnosticSeverity::Warning,
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
	validate_model_with_instructions(model, None)
}

/// Validate an Aurora model (embedded registries; instructions root ignored).
pub fn validate_model_with_instructions(
	model: &AuroraModel,
	instructions_root: Option<&Path>,
) -> ValidationReport {
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
	let _ = instructions_root;
	check_matrix_compliance(model, &mut report);

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

#[derive(Debug, Clone)]
struct RelationshipRule {
	sources: HashSet<String>,
	targets: HashSet<String>,
}

#[derive(Debug, Clone)]
struct MatrixRegistry {
	card_types: HashSet<String>,
	relationships: HashMap<String, RelationshipRule>,
}

fn check_matrix_compliance(model: &AuroraModel, report: &mut ValidationReport) {
	let registry = match load_matrix_registry(model) {
		Ok(Some(registry)) => registry,
		Ok(None) => return,
		Err(message) => {
			report.diagnostics.push(ValidationDiagnostic::warning(
				"RELATIONSHIP_MATRIX_UNAVAILABLE",
				message,
			));
			return;
		}
	};

	for card in model.iter_cards() {
		let card_type = canonical_card_type(&card.card_type);
		if !registry.card_types.contains(&card_type) {
			report.diagnostics.push(
				ValidationDiagnostic::warning(
					"UNKNOWN_CARD_TYPE",
					format!(
						"Card type '{}' is not listed in the registry definitions",
						card.card_type
					),
				)
				.with_card(card)
				.with_path("card_type"),
			);
		}
	}

	for card in model.iter_cards() {
		let source_type = canonical_card_type(&card.card_type);
		let source_known = registry.card_types.contains(&source_type);
		for (idx, link) in card.links.iter().enumerate() {
			let relationship = normalize_relationship(&link.relationship);
			if relationship.is_empty() {
				continue;
			}
			let rule = match registry.relationships.get(&relationship) {
				Some(rule) => rule,
				None => {
					report.diagnostics.push(
						ValidationDiagnostic::warning(
							"UNKNOWN_RELATIONSHIP",
							format!(
								"Relationship '{}' is not defined in the embedded relationship registry",
								link.relationship
							),
						)
						.with_card(card)
						.with_path(format!("links[{idx}].relationship")),
					);
					continue;
				}
			};

			if source_known && !rule.sources.contains(&source_type) {
				report.diagnostics.push(
					ValidationDiagnostic::warning(
						"RELATIONSHIP_SOURCE_NOT_ALLOWED",
						format!(
							"Relationship '{}' does not allow source card type '{}'",
							link.relationship, card.card_type
						),
					)
					.with_card(card)
					.with_path(format!("links[{idx}].relationship")),
				);
			}

			if let Some(target_card) = model.get(&link.target) {
				let target_type = canonical_card_type(&target_card.card_type);
				let target_known = registry.card_types.contains(&target_type);
				if target_known && !rule.targets.contains(&target_type) {
					report.diagnostics.push(
						ValidationDiagnostic::warning(
							"RELATIONSHIP_TARGET_NOT_ALLOWED",
							format!(
								"Relationship '{}' does not allow target card type '{}'",
								link.relationship, target_card.card_type
							),
						)
						.with_card(card)
						.with_path(format!("links[{idx}].target")),
					);
				}
			}
		}
	}
}

fn load_matrix_registry(_model: &AuroraModel) -> Result<Option<MatrixRegistry>, String> {
	let card_types = load_card_types_from_registry();
	if card_types.is_empty() {
		return Err("No card types were loaded from the registry".to_string());
	}
	let mut relationships = load_relationships_from_registry(&card_types);
	if let Some(rule) = relationships.get("transitions to").cloned() {
		relationships
			.entry("transistions to".to_string())
			.or_insert(rule);
	}
	Ok(Some(MatrixRegistry {
		card_types,
		relationships,
	}))
}

fn load_card_types_from_registry() -> HashSet<String> {
	let mut card_types = HashSet::new();
	for definition in CardDefinition::get_all() {
		let card_type = canonical_card_type(definition.card_type);
		if !card_type.is_empty() {
			card_types.insert(card_type);
		}
	}
	card_types
}

fn load_relationships_from_registry(
	card_types: &HashSet<String>,
) -> HashMap<String, RelationshipRule> {
	let mut relationships = HashMap::new();
	for definition in RelationshipDefinition::get_all() {
		let verb = normalize_relationship(definition.relationship);
		if verb.is_empty() {
			continue;
		}
		let sources = expand_card_type_set(definition.source_card_types, card_types);
		let targets = expand_card_type_set(definition.target_card_types, card_types);
		let entry = relationships
			.entry(verb)
			.or_insert_with(|| RelationshipRule {
				sources: HashSet::new(),
				targets: HashSet::new(),
			});
		entry.sources.extend(sources);
		entry.targets.extend(targets);
	}
	relationships
}

fn expand_card_type_set(
	entries: &[&'static str],
	all_card_types: &HashSet<String>,
) -> HashSet<String> {
	let mut types = HashSet::new();
	let mut exclude = HashSet::new();
	let mut include_all = false;
	for entry in entries {
		let trimmed = entry.trim();
		if trimmed.is_empty() {
			continue;
		}
		if trimmed == "*" {
			include_all = true;
			continue;
		}
		if let Some(rest) = trimmed.strip_prefix('!') {
			let normalized = canonical_card_type(rest);
			if !normalized.is_empty() {
				exclude.insert(normalized);
			}
			continue;
		}
		let normalized = canonical_card_type(trimmed);
		if !normalized.is_empty() {
			types.insert(normalized);
		}
	}
	if include_all || (!exclude.is_empty() && types.is_empty()) {
		types = all_card_types.clone();
	}
	for entry in exclude {
		types.remove(&entry);
	}
	types
}

fn normalize_relationship(value: &str) -> String {
	value.trim().trim_matches('`').to_ascii_lowercase()
}

fn canonical_card_type(value: &str) -> String {
	normalize_card_type(value).to_ascii_lowercase()
}

fn normalize_card_type(value: &str) -> String {
	let trimmed = value.trim().trim_matches('`');
	let base = match trimmed.split_once('(') {
		Some((head, _)) => head.trim(),
		None => trimmed,
	};
	base.trim().to_string()
}

#[cfg(test)]
mod tests;
