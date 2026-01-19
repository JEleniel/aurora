pub mod model;

use std::{fs::File, io::Read, path::PathBuf};

use jsonschema::{Validator, draft7::meta};
use serde_json::json;
use thiserror::Error;

use crate::{
	aurora::model::{Model, ModelError},
	logging::LoggingError,
};

pub struct Aurora {
	pub path: PathBuf,
	pub card_schema_validator: Validator,
	pub compact_schema_validator: Validator,
	pub models: Vec<Model>,
}

impl Aurora {
	pub fn load(path: &PathBuf) -> Result<Self, AuroraError> {
		let aurora_path = Self::find_aurora_path(path)?;
		let schema_path = aurora_path.join("Aurora.schema.json");
		let compact_schema_path = aurora_path.join("Aurora.compact.schema.json");

		let card_validator = Self::validate_and_load_schema(&schema_path)?;
		let compact_validator = Self::validate_and_load_schema(&compact_schema_path)?;

		let models = Model::load(&aurora_path, &card_validator, &compact_validator)?;
		Ok(Self {
			path: aurora_path,
			card_schema_validator: card_validator,
			compact_schema_validator: compact_validator,
			models,
		})
	}

	pub fn validate(&self) -> Result<Vec<String>, AuroraError> {
		let mut all_errors: Vec<String> = Vec::new();
		for model in &self.models {
			all_errors.extend(model.card_schema_errors.clone());
			all_errors.extend(model.card_invariant_errors.clone());
		}
		Ok(all_errors)
	}

	fn find_aurora_path(path: &PathBuf) -> Result<PathBuf, AuroraError> {
		let mut path = path.clone();
		if path.is_dir() {
			if !path.ends_with("aurora") && !path.ends_with("aurora/") {
				path = path.join("aurora/");
				if !path.exists() {
					return Err(AuroraError::InvalidPath(path.display().to_string()));
				} else {
					return Ok(path);
				}
			} else {
				if !path.exists() {
					return Err(AuroraError::InvalidPath(path.display().to_string()));
				}
				return Ok(path);
			};
		}
		Ok(path)
	}

	fn validate_and_load_schema(path: &PathBuf) -> Result<Validator, AuroraError> {
		let file = File::open(path)?;
		let mut data: String = String::new();
		let mut reader = std::io::BufReader::new(&file);
		reader.read_to_string(&mut data)?;
		let schema = json!(data);
		if meta::is_valid(&schema) {
			Ok(Validator::new(&schema)
				.map_err(|err| AuroraError::SchemaValidationError(err.to_string()))?)
		} else {
			Err(AuroraError::InvalidSchema)
		}
	}
}

/// Errors that can occur while loading, validating, or rendering Aurora models.
#[derive(Debug, Error)]
pub enum AuroraError {
	#[error("An I/O error occurred: {0}")]
	IoError(#[from] std::io::Error),
	#[error("The Aurora.schema.json file is invalid")]
	InvalidSchema,
	#[error("Schema validation failed: {0}")]
	SchemaValidationError(String),
	#[error("A Model error occurred: {0}")]
	ModelError(#[from] ModelError),
	#[error("The specified path is not a valid Aurora directory: {0}")]
	InvalidPath(String),
	#[error("A logging error occurred: {0}")]
	LoggingError(#[from] LoggingError),
}
