use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Attributes, Card, CardError, Link};
use crate::registry::CardRegistry;

#[derive(Debug, Clone)]
pub struct NewCard {
	pub card_type: String,
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	pub version: Option<String>,
	pub status: Option<String>,
	pub boundary: Option<String>,
	pub notes: Option<String>,
	pub icon: Option<String>,
	pub attributes: Attributes,
	pub external_references: Vec<String>,
	pub links: Vec<Link>,
}

#[derive(Debug)]
pub(crate) struct PreparedCardWrite {
	pub(crate) card: Card,
	pub(crate) serialized: String,
}

impl Card {
	pub fn create(
		new_card: NewCard,
		registry: &CardRegistry,
		existing_ids: &HashSet<String>,
	) -> Result<Self, CardError> {
		let acronym = registry.try_get_acronym_for_type(&new_card.card_type)?;
		let id = next_card_id(&acronym, existing_ids);

		Ok(Self {
			schema: None,
			id,
			card_type: new_card.card_type,
			card_subtype: new_card.card_subtype,
			name: new_card.name,
			description: new_card.description,
			version: new_card.version,
			status: new_card.status,
			boundary: new_card.boundary,
			notes: new_card.notes,
			icon: new_card.icon,
			attributes: new_card.attributes,
			external_references: new_card.external_references,
			links: new_card.links,
			source_path: PathBuf::new(),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		})
	}

	pub fn write(&self, path: &Path) -> Result<(), CardError> {
		let prepared = self.prepare_write(path)?;
		let parent = path
			.parent()
			.ok_or_else(|| CardError::InvalidFilename(path.display().to_string()))?;
		std::fs::create_dir_all(parent)?;
		std::fs::write(path, prepared.serialized)?;
		Ok(())
	}

	pub(crate) fn prepare_write(&self, path: &Path) -> Result<PreparedCardWrite, CardError> {
		let mut card = self.clone();
		card.schema = Some(infer_schema_reference(path)?);
		card.source_path = path.to_path_buf();
		card.validation_errors.clear();
		card.validation_warnings.clear();

		let mut errors = validate_against_target_schema(&card, path)?;
		if let Some(error) = validate_card_id(&card.id) {
			errors.push(error);
		}

		if !errors.is_empty() {
			return Err(CardError::ValidationErrors(errors));
		}

		let serialized = serde_json::to_string_pretty(&card).map_err(|error| {
			CardError::InvalidCard(format!("Failed to serialize card {}: {error}", card.id))
		})?;

		Ok(PreparedCardWrite { card, serialized })
	}
}

fn validate_against_target_schema(card: &Card, path: &Path) -> Result<Vec<String>, CardError> {
	let schema_path = locate_card_schema(path)?;
	let schema_data = std::fs::read_to_string(&schema_path)?;
	let schema_json: Value = serde_json::from_str(&schema_data).map_err(|error| {
		CardError::InvalidCard(format!(
			"Failed to parse schema {}: {error}",
			schema_path.display()
		))
	})?;
	let card_json = serde_json::to_value(card).map_err(|error| {
		CardError::InvalidCard(format!("Failed to encode card {}: {error}", card.id))
	})?;
	let mut errors = Card::validate_against_schema(&card_json, &schema_json)?;
	let mut warnings = Vec::new();
	Card::validate_schema_reference(
		path,
		&card_json,
		"Aurora.card.schema.json",
		&mut errors,
		&mut warnings,
	);
	Ok(errors)
}

fn infer_schema_reference(path: &Path) -> Result<String, CardError> {
	let parent = path
		.parent()
		.ok_or_else(|| CardError::InvalidFilename(path.display().to_string()))?;
	let schema_path = locate_card_schema(path)?;
	let relative = super::relative_path(parent, &schema_path);
	Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn locate_card_schema(path: &Path) -> Result<PathBuf, CardError> {
	let parent = path
		.parent()
		.ok_or_else(|| CardError::InvalidFilename(path.display().to_string()))?;

	for ancestor in parent.ancestors() {
		let candidate = ancestor.join("schemas").join("Aurora.card.schema.json");
		if candidate.is_file() {
			return Ok(candidate);
		}
	}

	Err(CardError::SchemaReference(format!(
		"schemas/Aurora.card.schema.json not found for {}",
		path.display()
	)))
}

fn validate_card_id(id: &str) -> Option<String> {
	let Some((prefix, suffix)) = id.split_once('-') else {
		return Some(format!(
			"Card {} has invalid ID format (expected PREFIX-NUMBER).",
			id
		));
	};
	if prefix.len() != 3 || !prefix.chars().all(|ch| ch.is_ascii_uppercase()) {
		return Some(format!(
			"Card {} has invalid ID prefix '{}' (expected 3 uppercase letters).",
			id, prefix
		));
	}
	if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
		return Some(format!(
			"Card {} has invalid numeric suffix '{}' (expected digits only).",
			id, suffix
		));
	}
	None
}

fn next_card_id(acronym: &str, existing_ids: &HashSet<String>) -> String {
	let max_suffix = existing_ids
		.iter()
		.filter_map(|existing_id| parse_suffix(existing_id, acronym))
		.max()
		.unwrap_or(0);

	let mut next = max_suffix + 1;
	loop {
		let candidate = format!("{acronym}-{next:03}");
		if !existing_ids.contains(&candidate) {
			return candidate;
		}
		next += 1;
	}
}

fn parse_suffix(existing_id: &str, expected_prefix: &str) -> Option<u32> {
	let (prefix, suffix) = existing_id.split_once('-')?;
	if prefix != expected_prefix || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
		return None;
	}
	suffix.parse().ok()
}

#[cfg(test)]
#[path = "card_persistence_tests.rs"]
mod card_persistence_tests;
