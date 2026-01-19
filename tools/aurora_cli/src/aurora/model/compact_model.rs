use std::{fs::File, path::PathBuf};

use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use super::card::Link;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactModel {
	#[serde(rename = "$schema")]
	pub schema: String,
	pub cards: Vec<CompactCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactCard {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Map<String, Value>>,
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

impl CompactModel {
	pub fn load(path: &PathBuf, compact_validator: &Validator) -> Result<Self, CompactModelError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let json: Value = serde_json::from_reader(reader)?;

		compact_validator
			.validate(&json)
			.map_err(|e| CompactModelError::SchemaValidationError(e.to_string()))?;

		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let compact_model: CompactModel = serde_json::from_reader(reader)?;
		Ok(compact_model)
	}
}

#[derive(Debug, Error)]
pub enum CompactModelError {
	#[error("Schema validation error: {0}")]
	SchemaValidationError(String),
	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("JSON serialization/deserialization error: {0}")]
	SerdeJsonError(#[from] serde_json::Error),
}
