use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;
use crate::registry::card_registry::ModelConfiguration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
	pub description: String,
	pub included_card_types: Vec<String>,
	pub name: String,
	pub root_card_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRegistry {
	definitions: Vec<ViewDefinition>,
}

impl ViewRegistry {
	pub fn try_new() -> Result<Self, RegistryError> {
		let model_configuration = include_str!(
			"../../../../.github/agents/aurora/reference/Aurora.modelconfiguration.json"
		);
		Self::try_new_from_model_configuration(model_configuration)
	}

	pub fn try_new_from_model_configuration(json: &str) -> Result<Self, RegistryError> {
		let model_configuration: ModelConfiguration = serde_json::from_str(json)?;
		Ok(Self {
			definitions: model_configuration.views,
		})
	}

	pub fn try_new_from_struct(model_configuration: &ModelConfiguration) -> Self {
		Self {
			definitions: model_configuration.views.clone(),
		}
	}

	pub fn try_get_all(&self) -> Result<Vec<ViewDefinition>, RegistryError> {
		Ok(self.definitions.clone())
	}
}

#[cfg(test)]
#[path = "view_registry_tests.rs"]
mod view_registry_tests;
