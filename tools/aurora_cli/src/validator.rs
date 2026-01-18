//! Schema and invariant validation for Aurora models.

use crate::error::AuroraCliError;
use crate::model::Model;
use chrono::{SecondsFormat, Utc};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;

/// Aggregated validation results used for reporting and exit codes.
#[derive(Debug, Clone)]
pub struct ValidationOutcome {
	pub card_count: usize,
	pub schema_errors: Vec<String>,
	pub invariant_errors: Vec<String>,
}

impl ValidationOutcome {
	pub fn is_success(&self) -> bool {
		self.schema_errors.is_empty() && self.invariant_errors.is_empty()
	}

	pub fn total_errors(&self) -> usize {
		self.schema_errors.len() + self.invariant_errors.len()
	}
}

pub fn validate_model(model: &Model) -> Result<ValidationOutcome, AuroraCliError> {
	let schema_errors = validate_schema(model)?;
	let invariant_errors = validate_invariants(model);
	Ok(ValidationOutcome {
		card_count: model.len(),
		schema_errors,
		invariant_errors,
	})
}

pub fn validation_report_markdown(model: &Model, outcome: &ValidationOutcome) -> String {
	let mission = model.mission();
	let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
	let mut report = String::new();
	report.push_str("# Validation Report\n\n");
	report.push_str(&format!("- Model root: {}\n", model.root().display()));
	report.push_str(&format!(
		"- Schema copy: {}\n",
		model.schema_path().display()
	));
	report.push_str(&format!("- Mission: {} ({})\n", mission.name, mission.id));
	report.push_str(&format!("- Cards validated: {}\n", outcome.card_count));
	report.push_str(&format!("- Generated: {}\n\n", timestamp));

	report.push_str("## Schema Validation\n\n");
	append_section(
		&mut report,
		&outcome.schema_errors,
		"All cards conform to Aurora.schema.json",
	);

	report.push_str("\n## Invariant Validation\n\n");
	append_section(
		&mut report,
		&outcome.invariant_errors,
		"All invariants satisfied",
	);

	report
}

fn append_section(buf: &mut String, issues: &[String], success_message: &str) {
	if issues.is_empty() {
		buf.push_str(&format!("- ✅ {}\n", success_message));
		return;
	}
	buf.push_str(&format!("- ❌ {} issue(s) detected\n", issues.len()));
	for issue in issues {
		buf.push_str(&format!("  - {}\n", issue));
	}
}

fn validate_schema(model: &Model) -> Result<Vec<String>, AuroraCliError> {
	let schema_text =
		fs::read_to_string(model.schema_path()).map_err(|source| AuroraCliError::Io {
			path: model.schema_path().to_path_buf(),
			source,
		})?;
	let schema_json: Value =
		serde_json::from_str(&schema_text).map_err(|source| AuroraCliError::Json {
			path: model.schema_path().to_path_buf(),
			source,
		})?;
	let compiled =
		JSONSchema::compile(&schema_json).map_err(|err| AuroraCliError::SchemaCompilation {
			path: model.schema_path().to_path_buf(),
			message: err.to_string(),
		})?;
	let mut issues = Vec::new();
	for card in model.cards() {
		if let Err(errors) = compiled.validate(&card.raw) {
			let details: Vec<String> = errors.map(|err| err.to_string()).collect();
			issues.push(format!("{}: {}", card.id, details.join("; ")));
		}
	}
	Ok(issues)
}

fn validate_invariants(model: &Model) -> Vec<String> {
	let mut issues = Vec::new();
	let mut inbound: HashMap<&str, usize> = HashMap::new();
	for card in model.cards() {
		inbound.insert(card.id.as_str(), 0);
	}

	for card in model.cards() {
		for link in &card.links {
			if !model.contains(&link.target) {
				let relation = link.relationship.as_deref().unwrap_or("links to");
				issues.push(format!(
					"{} {} missing target {}",
					card.id, relation, link.target
				));
			}
			if let Some(count) = inbound.get_mut(link.target.as_str()) {
				*count += 1;
			}
		}
	}

	for card in model.cards() {
		if card.is_mission() {
			continue;
		}
		if inbound.get(card.id.as_str()).copied().unwrap_or(0) == 0 {
			issues.push(format!("{} has no incoming links", card.id));
		}
	}

	let reachable = reachable_from(model);
	for card in model.cards() {
		if !reachable.contains(card.id.as_str()) {
			issues.push(format!(
				"{} is not reachable from Mission {}",
				card.id,
				model.mission_id()
			));
		}
	}

	issues
}

fn reachable_from(model: &Model) -> HashSet<&str> {
	let mut stack = vec![model.mission_id()];
	let mut visited: HashSet<&str> = HashSet::new();
	let adjacency = build_adjacency(model);
	while let Some(node) = stack.pop() {
		if !visited.insert(node) {
			continue;
		}
		if let Some(neighbors) = adjacency.get(node) {
			for &neighbor in neighbors {
				if !visited.contains(neighbor) {
					stack.push(neighbor);
				}
			}
		}
	}
	visited
}

fn build_adjacency(model: &Model) -> HashMap<&str, Vec<&str>> {
	let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
	for card in model.cards() {
		let entry = adj.entry(card.id.as_str()).or_default();
		entry.extend(card.links.iter().map(|link| link.target.as_str()));
	}
	adj
}
