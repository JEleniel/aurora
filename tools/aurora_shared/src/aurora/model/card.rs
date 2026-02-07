use crate::{
	aurora::{
		Attribute,
		model::{audit_trail::AuditTrail, link::Link},
	},
	registry::CardDefinition,
	registry::RelationshipDefinition,
};
use jsonschema::{CompilationError, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;
use tracing::debug;

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
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	pub description: String,
	#[serde(default)]
	pub attributes: Vec<Attribute>,
	pub links: Vec<Link>,
	pub audit_trail: AuditTrail,
	#[serde(skip)]
	pub source_path: PathBuf,
	#[serde(skip)]
	pub validation_errors: Vec<String>,
}

impl Card {
	pub fn try_load(path: &PathBuf, card_schema: &serde_json::Value) -> Result<Self, CardError> {
		debug!("Loading card from {}", path.display());

		let data = std::fs::read_to_string(&path).map_err(|e| CardError::IoError(e))?;

		let card_json: serde_json::Value = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		let validation_errors = Self::validate_against_schema(&card_json, card_schema)?;

		let mut card: Card = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		card.source_path = path.clone();
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

	pub fn write(&self, path: &PathBuf) {
		let serialized = serde_json::to_string_pretty(self).unwrap();
		std::fs::write(path, serialized).unwrap();
	}

	pub fn write_markdown(&self, path: &PathBuf) {
		let mut markdown: String = String::from(CARD_MARKDOWN_TEMPLATE);

		let subtype = match &self.card_subtype {
			Some(subtype) => format!(" ({})", subtype.as_str()),
			None => "".to_string(),
		};

		let status = match &self.status {
			Some(status) => format!("**Status**: {}\n", status.as_str()),
			None => "".to_string(),
		};

		let hash = if let Some(hash) = &self.audit_trail.hash {
			format!("Hash: {}", hash.as_str())
		} else {
			"".to_string()
		};

		let mut attributes: String = String::new();
		if self.attributes.is_empty() {
			attributes.push_str("_No attributes defined._");
		} else {
			for attrib in &self.attributes {
				attributes.push_str(attrib.get_markdown().as_str());
			}
		}

		let mut links: String = String::new();
		if self.links.is_empty() {
			links.push_str("_No links defined._");
		} else {
			for link in &self.links {
				links.push_str(link.get_markdown().as_str());
			}
		}

		let history = self.audit_trail.get_history_markdown();

		markdown = markdown
			.replace("{{card_type}}", &self.card_type)
			.replace("{{card_subtype}}", &subtype)
			.replace("{{id}}", &self.id)
			.replace("{{name}}", &self.name)
			.replace("{{description}}", &self.description)
			.replace("{{status}}", &status)
			.replace("{{attributes}}", &attributes)
			.replace("{{links}}", &links)
			.replace("{{version}}", &self.audit_trail.version)
			.replace("{{hash}}", &hash)
			.replace("{{history}}", &history)
			.replace("|\n\n", "|\n")
			.replace("\n\n\n\n", "\n\n");

		std::fs::write(path, markdown).unwrap();
	}

	pub fn get_compact(&self) -> Value {
		let mut value = serde_json::to_value(self).unwrap();
		Self::remove_key(&mut value, "$schema");
		Self::remove_key(&mut value, "audit_trail");

		value
	}

	fn validate_against_schema(
		card_json: &Value,
		card_schema: &Value,
	) -> Result<Vec<String>, CardError> {
		let compiled_schema = JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(card_schema)?;

		let result = compiled_schema.validate(&card_json);
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
