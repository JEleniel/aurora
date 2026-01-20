pub mod model;

use std::{
	fs::File,
	io::{Read, Write},
	path::PathBuf,
	vec,
};

use jsonschema::{Validator, draft7::meta};
use serde_json::json;
use thiserror::Error;

use crate::{
	aurora::model::{Model, ModelError},
	cli::{bump_args::BumpArgs, output_args::OutputArgs},
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

	pub fn render_models(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		todo!()
	}

	pub fn render_cards(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		todo!()
	}

	pub fn render_all(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		let mut result = self.render_cards(args)?;
		result.extend(self.render_models(args)?);
		Ok(result)
	}

	pub fn compact(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		let mut results: Vec<String> = Vec::new();

		for model in &self.models {
			let compact_model = model.compact();
			let output = serde_json::to_string(&compact_model)?;
			let output_path = args
				.output_path
				.join(format!("AGENT-{}.json", model.mission_card.id));
			let mut file = File::create(&output_path)?;
			file.write_all(output.as_bytes())?;

			results.push(format!(
				"Wrote compact model for {} to {}.",
				model.mission_card.id,
				output_path.display()
			));
		}
		Ok(results)
	}

	pub fn bump_patch(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_patch(args)?;
				return Ok(vec![format!(
					"Bumped patch version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
	}

	pub fn bump_minor(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_minor(args)?;
				return Ok(vec![format!(
					"Bumped minor version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
	}

	pub fn bump_major(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_major(args)?;
				return Ok(vec![format!(
					"Bumped major version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
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
	#[error("A serialization/deserialization error occurred: {0}")]
	SerdeError(#[from] serde_json::Error),
	#[error("Card with ID '{0}' not found in any model.")]
	CardNotFound(String),
}
