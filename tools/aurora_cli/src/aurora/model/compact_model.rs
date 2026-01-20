use std::{fs::File, path::PathBuf};

use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::aurora::model::{Model, compact_card::CompactCard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactModel {
	#[serde(rename = "$schema")]
	pub schema: String,
	pub cards: Vec<CompactCard>,
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

impl From<&Model> for CompactModel {
	fn from(model: &Model) -> Self {
		Self {
			schema: "Aurora.compact.schema.json".to_string(),
			cards: model.cards.values().map(CompactCard::from).collect(),
		}
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
