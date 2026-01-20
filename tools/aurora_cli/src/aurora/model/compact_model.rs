use std::{fs::File, path::Path};

use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::aurora::model::{
	Model,
	compact_card::{CompactCard, CompactCardBorrowed},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactModel {
	#[serde(rename = "$schema")]
	pub schema: String,
	pub cards: Vec<CompactCard>,
}

impl CompactModel {
	pub fn load(path: &Path, compact_validator: &Validator) -> Result<Self, CompactModelError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let json: Value = serde_json::from_reader(reader)?;

		compact_validator
			.validate(&json)
			.map_err(|e| CompactModelError::SchemaValidationError(e.to_string()))?;

		let compact_model: CompactModel = serde_json::from_value(json)?;
		Ok(compact_model)
	}
}

impl From<&Model> for CompactModel {
	fn from(model: &Model) -> Self {
		let mut cards: Vec<CompactCard> = Vec::new();
		cards.push(CompactCard::from(&model.mission_card));
		for card in model.cards.values() {
			cards.push(CompactCard::from(card));
		}
		Self {
			schema: "Aurora.compact.schema.json".to_string(),
			cards,
		}
	}
}

#[derive(Debug, Serialize)]
pub struct CompactModelBorrowed<'a> {
	#[serde(rename = "$schema")]
	pub schema: &'a str,
	pub cards: Vec<CompactCardBorrowed<'a>>,
}

impl<'a> From<&'a Model> for CompactModelBorrowed<'a> {
	fn from(model: &'a Model) -> Self {
		let mut cards = Vec::with_capacity(model.cards.len() + 1);
		cards.push(CompactCardBorrowed::from(&model.mission_card));
		for card in model.cards.values() {
			cards.push(CompactCardBorrowed::from(card));
		}
		Self {
			schema: "Aurora.compact.schema.json",
			cards,
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
