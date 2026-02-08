use crate::{registry::CardDefinition, registry::RelationshipDefinition};
use jsonschema::{CompilationError, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

use super::{Attributes, Link, attributes_markdown};

const CARD_MARKDOWN_TEMPLATE: &str = include_str!("card.template.md");

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
	#[serde(rename = "$schema", default)]
	pub schema: Option<String>,
	pub id: String,
	pub card_type: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	pub version: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub boundary: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub notes: Option<String>,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub attributes: Attributes,
	pub links: Vec<Link>,
	#[serde(skip)]
	pub source_path: PathBuf,
	#[serde(skip)]
	pub validation_errors: Vec<String>,
}

impl Card {
	pub fn try_load(path: &Path, card_schema: &serde_json::Value) -> Result<Self, CardError> {
		debug!("Loading card from {}", path.display());

		let data = std::fs::read_to_string(path)?;

		let card_json: serde_json::Value = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		let validation_errors = Self::validate_against_schema(&card_json, card_schema)?;

		let mut card: Card = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		card.source_path = path.to_path_buf();
		card.validation_errors = validation_errors;
		Ok(card)
	}

	pub fn check_registry(&self) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();
		if !CardDefinition::validate(&self.card_type) {
			warnings.push(format!("Card has unknown card type: {}", self.card_type));
		}
		for link in self.links.iter() {
			let target_card_def = CardDefinition::get_by_acronym(&link.target[0..3]);
			if !CardDefinition::validate(target_card_def.card_type.as_str()) {
				warnings.push(format!(
					"Card links to unknown target card: {}",
					link.target
				));
				continue;
			};

			if !RelationshipDefinition::validate(
				&self.card_type,
				&link.relationship,
				target_card_def.card_type.as_str(),
			) {
				warnings.push(format!(
					"Card {} has unknown relationship '{}' to target card {}",
					self.id, link.relationship, link.target
				));
			}
		}

		warnings
	}

	pub fn write(&self, path: &Path) {
		let serialized = serde_json::to_string_pretty(self).unwrap();
		std::fs::write(path, serialized).unwrap();
	}

	pub fn write_markdown<'a>(
		&self,
		path: &Path,
		audit_entries: impl Iterator<Item = &'a super::super::AuditLogEntry>,
	) {
		let mut markdown: String = String::from(CARD_MARKDOWN_TEMPLATE);

		let subtype = match &self.card_subtype {
			Some(subtype) => format!(" ({})", subtype.as_str()),
			None => "".to_string(),
		};

		let status = match &self.status {
			Some(status) => format!("**Status**: {}\n", status.as_str()),
			None => "".to_string(),
		};

		let boundary = match &self.boundary {
			Some(boundary) => format!("**Boundary**: {}\n", boundary),
			None => "".to_string(),
		};

		let notes = match &self.notes {
			Some(notes) => format!("## Notes\n\n{}\n", notes),
			None => "".to_string(),
		};

		let attributes = attributes_markdown(&self.attributes);

		let mut links: String = String::new();
		if self.links.is_empty() {
			links.push_str("_No links defined._");
		} else {
			for link in &self.links {
				links.push_str(link.get_markdown().as_str());
			}
		}

		let mut history = String::new();
		history.push_str("| Timestamp | Editor | Change |\n");
		history.push_str("|-----------|--------|--------|\n");
		let mut any = false;
		for entry in audit_entries {
			any = true;
			history.push_str(&format!(
				"| {} | {} | {} |\n",
				entry
					.timestamp
					.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
				entry.editor,
				entry.change_type.as_str(),
			));
		}
		if !any {
			history.push_str("| _No entries_ |  |  |\n");
		}

		markdown = markdown
			.replace("{{card_type}}", &self.card_type)
			.replace("{{card_subtype}}", &subtype)
			.replace("{{id}}", &self.id)
			.replace("{{name}}", &self.name)
			.replace("{{description}}", &self.description)
			.replace("{{status}}", &status)
			.replace("{{boundary}}", &boundary)
			.replace("{{notes}}", &notes)
			.replace("{{attributes}}", &attributes)
			.replace("{{links}}", &links)
			.replace("{{version}}", &self.version)
			.replace("{{history}}", &history)
			.replace("|\n\n", "|\n")
			.replace("\n\n\n\n", "\n\n");

		std::fs::write(path, markdown).unwrap();
	}

	pub fn get_compact(&self) -> Value {
		let mut value = serde_json::to_value(self).unwrap();
		Self::remove_key(&mut value, "$schema");

		value
	}

	fn validate_against_schema(
		card_json: &Value,
		card_schema: &Value,
	) -> Result<Vec<String>, CardError> {
		let compiled_schema = JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(card_schema)?;

		let result = compiled_schema.validate(card_json);
		match result {
			Ok(_) => Ok(vec![]),
			Err(errors) => {
				let mut error_messages = Vec::new();
				for error in errors {
					error_messages.push(error.to_string());
				}
				Ok(error_messages)
			}
		}
	}

	fn remove_key(value: &mut Value, key: &str) {
		if let Value::Object(map) = value {
			map.remove(key);
		}
	}
}

#[derive(Debug, Error)]
pub enum CardError {
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Parse error at {0}: {1}")]
	ParseError(String, serde_json::Error),
	#[error("Invalid card file: {0}")]
	InvalidCard(String),
	#[error("Invalid filename: {0}")]
	InvalidFilename(String),
	#[error("Schema compilation error: {0}")]
	SchemaCompilationError(#[from] CompilationError),
	#[error("Card not found: {0}")]
	CardNotFound(String),
}

#[cfg(test)]
#[path = "card_tests.rs"]
mod card_tests;
