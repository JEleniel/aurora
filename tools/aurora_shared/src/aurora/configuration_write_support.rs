use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use super::model::model_write_support::write_single_file_transactionally;
use super::{Aurora, AuroraError};
use crate::registry::{CardRegistry, ModelConfiguration, ViewConfiguration, ViewRegistry};
use crate::{ConfigBackupManager, ConfigBackupRequest};

const MODEL_CONFIGURATION_SCHEMA_REF: &str = "../schemas/Aurora.modelconfiguration.schema.json";
const VIEW_CONFIGURATION_SCHEMA_REF: &str = "../schemas/Aurora.viewconfiguration.schema.json";

impl Aurora {
	pub fn write_model_configuration(
		&mut self,
		configuration: ModelConfiguration,
	) -> Result<(), AuroraError> {
		let model_configuration = prepare_model_configuration(configuration);
		let card_registry = CardRegistry::try_new_from_structs(
			model_configuration.clone(),
			self.view_configuration.clone(),
		)?;
		let serialized =
			serialize_and_validate(&model_configuration, &self.modelconfiguration_schema)?;
		self.ensure_config_backup()?;
		write_configuration_file(&self.model_configuration_path(), serialized)?;
		self.apply_configuration_update(
			model_configuration,
			self.view_configuration.clone(),
			card_registry,
		);
		Ok(())
	}

	pub fn write_view_configuration(
		&mut self,
		configuration: ViewConfiguration,
	) -> Result<(), AuroraError> {
		let view_configuration = prepare_view_configuration(configuration);
		let card_registry = CardRegistry::try_new_from_structs(
			self.model_configuration.clone(),
			view_configuration.clone(),
		)?;
		let serialized =
			serialize_and_validate(&view_configuration, &self.viewconfiguration_schema)?;
		self.ensure_config_backup()?;
		write_configuration_file(&self.view_configuration_path(), serialized)?;
		self.apply_configuration_update(
			self.model_configuration.clone(),
			view_configuration,
			card_registry,
		);
		Ok(())
	}

	fn ensure_config_backup(&self) -> Result<(), AuroraError> {
		let request = ConfigBackupRequest::new(self.model_home.clone(), self.primary_mission_id()?);
		ConfigBackupManager::ensure_backup(&request)?;
		Ok(())
	}

	fn primary_mission_id(&self) -> Result<&str, AuroraError> {
		self.models
			.first()
			.map(|model| model.root_card.id.as_str())
			.ok_or_else(|| AuroraError::InvalidAuroraHome(self.model_home.display().to_string()))
	}

	fn model_configuration_path(&self) -> PathBuf {
		configuration_path(&self.model_home, "Aurora.modelconfiguration.json")
	}

	fn view_configuration_path(&self) -> PathBuf {
		configuration_path(&self.model_home, "Aurora.viewconfiguration.json")
	}

	fn apply_configuration_update(
		&mut self,
		model_configuration: ModelConfiguration,
		view_configuration: ViewConfiguration,
		card_registry: CardRegistry,
	) {
		self.model_configuration = model_configuration;
		self.view_configuration = view_configuration;
		self.card_registry = card_registry;
		self.view_registry = ViewRegistry::try_new_from_struct(&self.view_configuration);
		self.load_warnings.clear();
		self.load_validation_errors.clear();
		self.load_validation_errors = self.validate();
		self.load_warnings = self.check_registry();
	}
}

fn prepare_model_configuration(mut configuration: ModelConfiguration) -> ModelConfiguration {
	if configuration.schema.is_none() {
		configuration.schema = Some(MODEL_CONFIGURATION_SCHEMA_REF.to_string());
	}
	configuration
}

fn prepare_view_configuration(mut configuration: ViewConfiguration) -> ViewConfiguration {
	if configuration.schema.is_none() {
		configuration.schema = Some(VIEW_CONFIGURATION_SCHEMA_REF.to_string());
	}
	configuration
}

fn serialize_and_validate<T: Serialize>(
	configuration: &T,
	schema: &Value,
) -> Result<String, AuroraError> {
	let json = serde_json::to_value(configuration)?;
	validate_configuration_json(schema, &json)?;
	Ok(serde_json::to_string_pretty(configuration)?)
}

fn validate_configuration_json(schema: &Value, json: &Value) -> Result<(), AuroraError> {
	let compiled = jsonschema::JSONSchema::compile(schema)?;
	if let Err(errors) = compiled.validate(json) {
		let messages = errors.map(|error| error.to_string()).collect::<Vec<_>>();
		return Err(AuroraError::ReferenceValidationFailed(messages));
	}
	Ok(())
}

fn write_configuration_file(path: &Path, contents: String) -> Result<(), AuroraError> {
	write_single_file_transactionally(path.to_path_buf(), contents)?;
	Ok(())
}

fn configuration_path(model_home: &Path, file_name: &str) -> PathBuf {
	model_home.join("reference").join(file_name)
}
