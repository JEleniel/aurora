mod auditlog;
mod model;

pub use auditlog::*;
pub use model::*;

#[cfg(test)]
mod aurora_tests;

use serde_json::Value;
use std::{
	collections::HashMap,
	path::{Component, PathBuf},
};
use thiserror::Error;
use tracing::info;

/// A complete set of Aurora models in a shared Model Home
#[derive(Debug)]
pub struct Aurora {
	/// The `aurora` folder where the models live
	pub model_home: PathBuf,
	/// The Aurora models
	pub models: Vec<Model>,
	/// A copy of the schema located with the models
	pub card_schema: Value,
	/// A copy of the compact shema located with the models
	pub compact_schema: Value,
	/// A copy of the audit schema located with the models
	pub audit_schema: Value,
}

impl Aurora {
	/// Try to load a set of Aurora models from a given path
	/// It will automatically pick up if the Model Home is a child
	/// of the given path
	pub fn try_load(path: &PathBuf) -> Result<Self, AuroraError> {
		let mut model_home = path.clone();

		if let Some(folder) = Self::last_folder(path) {
			if folder != "aurora" {
				let test_path = path.join("aurora");
				if !test_path.is_dir() || !test_path.exists() {
					return Err(AuroraError::InvalidAuroraHome(path.display().to_string()));
				} else {
					model_home = test_path;
				}
			}
		} else {
			return Err(AuroraError::InvalidAuroraHome(path.display().to_string()));
		}

		let card_schema_path = model_home.join("Aurora.card.schema.json");
		let compact_schema_path = model_home.join("Aurora.compact.schema.json");
		let audit_schema_path = model_home.join("Aurora.audit.schema.json");
		if !card_schema_path.is_file()
			|| !compact_schema_path.is_file()
			|| !audit_schema_path.is_file()
		{
			return Err(AuroraError::SchemaLoadError);
		}
		let card_schema_data = std::fs::read_to_string(&card_schema_path)?;
		let card_schema: Value = serde_json::from_str(&card_schema_data)?;
		let compact_schema_data = std::fs::read_to_string(&compact_schema_path)?;
		let compact_schema: Value = serde_json::from_str(&compact_schema_data)?;
		let audit_schema_data = std::fs::read_to_string(&audit_schema_path)?;
		let audit_schema: Value = serde_json::from_str(&audit_schema_data)?;

		let mut models: Vec<Model> = Vec::new();
		for entry in std::fs::read_dir(&model_home)? {
			let entry = entry?;
			if entry.file_type()?.is_dir() {
				continue;
			}
			if let Some(file_name) = entry.file_name().to_str() {
				if file_name.starts_with("MIS-") {
					let model = Model::try_load(&entry.path(), &card_schema, &audit_schema)?;
					models.push(model);
				}
			}
		}

		info!(
			"Loaded {} models from {}",
			models.len(),
			model_home.display()
		);
		models.sort_by(|a, b| a.root_card.id.cmp(&b.root_card.id));

		Ok(Aurora {
			model_home,
			models,
			card_schema,
			compact_schema,
			audit_schema,
		})
	}

	/// Check all models against the official registry and
	/// emit warnings if they don't conf=orm
	pub fn check_registry(&self) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();

		for model in &self.models {
			warnings.extend(
				model
					.validate_registry()
					.into_iter()
					.map(|w| format!("{}: {}", model.root_card.id, w)),
			);
		}
		warnings
	}

	/// Validate that all models conform to the Inviariants
	pub fn validate(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		for model in &self.models {
			let model_errors = model.validate();
			if !model_errors.is_empty() {
				errors.extend(
					model_errors
						.into_iter()
						.map(|e| format!("Model {}: {}", model.root_card.id, e)),
				);
			}
		}
		errors
	}

	/// Get schema validation errors for all models
	pub fn get_schema_validation_errors(&self) -> HashMap<String, Vec<String>> {
		let mut errors: HashMap<String, Vec<String>> = HashMap::new();
		for model in &self.models {
			let model_errors = model.get_schema_validation_errors();
			if !model_errors.is_empty() {
				errors.insert(model.root_card.id.clone(), model_errors);
			}
		}
		errors
	}

	/// Write the entire set of models out as human friendly markdown files
	/// with a README.md index
	pub fn write_markdown(&self, path: &PathBuf) -> Result<(), AuroraError> {
		for model in &self.models {
			model.write_markdown(&path)?;
			info!(
				"Wrote {}: {} ({} cards)",
				model.root_card.id,
				model.root_card.name,
				model.cards.len(),
			);
		}

		Ok(())
	}

	pub fn write_compact(&self, path: &PathBuf) -> Result<(), AuroraError> {
		for model in &self.models {
			let mut mission_dir = path.clone();
			mission_dir.push(model.root_card.id.as_str());
			std::fs::create_dir_all(&mission_dir)?;

			let output_path = mission_dir.join("Compact.json");
			let compact = model.get_compact(Some("../Aurora.compact.schema.json".to_string()));
			let output = serde_json::to_string_pretty(&compact)?;
			std::fs::write(&output_path, output)?;
			info!(
				"Wrote compact {}: {} ({} cards)",
				model.root_card.id,
				model.root_card.name,
				model.cards.len(),
			);
		}

		Ok(())
	}

	fn last_folder(path: &PathBuf) -> Option<&std::ffi::OsStr> {
		path.components()
			.filter_map(|c| match c {
				Component::Normal(name) => Some(name),
				_ => None,
			})
			.last()
	}
}

#[derive(Debug, Error)]
pub enum AuroraError {
	#[error("A model error occurred: {0}")]
	ModelError(#[from] ModelError),
	#[error("The specified path is not a valid Aurora model home: {0}")]
	InvalidAuroraHome(String),
	#[error("An IO error occurred: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Schemas could not be loaded")]
	SchemaLoadError,
	#[error("A JSON schema error occurred: {0}")]
	JsonSchemaError(#[from] jsonschema::CompilationError),
	#[error("A JSON parse error occurred: {0}")]
	JsonParseError(#[from] serde_json::Error),
}
