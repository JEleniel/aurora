use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
	fs::File,
	io::BufWriter,
	path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
	#[serde(rename = "$schema")]
	pub schema: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Map<String, Value>>,
	pub audit_trail: AuditTrail,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub card_type: String,
	pub description: String,
	pub id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub links: Option<Vec<Link>>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
}

impl Card {
	pub fn validate(path: &PathBuf, validator: &Validator) -> Result<Vec<String>, CardError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let json: Value = serde_json::from_reader(reader)?;

		let evaluation = validator.evaluate(&json);

		let mut results: Vec<String> = Vec::new();
		for error in evaluation.iter_errors() {
			results.push(error.error.to_string())
		}
		for annotation in evaluation.iter_annotations() {
			results.push(format!(
				"Annotation at {}: {:?}",
				annotation.instance_location, annotation.annotations
			))
		}

		Ok(results)
	}

	pub fn load(path: &Path, validator: &Validator) -> Result<Self, CardError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let card: Card = serde_json::from_reader(reader)?;
		Ok(card)
	}

	pub fn write(&self, path: &Path) -> Result<(), CardError> {
		let file = File::create(path)?;
		let mut writer = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut writer, self)?;
		Ok(())
	}

	pub fn compact(&self) -> Result<Value, CardError> {
		let mut value = serde_json::to_value(self)?;
		if let Some(obj) = value.as_object_mut() {
			obj.remove("audit_trail");
		}
		Ok(value)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
	pub relationship: String,
	pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
	pub hash: Option<String>,
	pub history: Vec<HistoryEntry>,
	pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryEvent {
	Created,
	Edited,
	Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
	pub editor: String,
	pub event: HistoryEvent,
	pub timestamp: String,
}

#[derive(Debug, Error)]
pub enum CardError {
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("JSON error: {0}")]
	Json(#[from] serde_json::Error),
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
	#[error("Invalid card file name")]
	InvalidCardFileName,
	#[error("Schema validation error: {0}")]
	SchemaValidationError(String),
}
