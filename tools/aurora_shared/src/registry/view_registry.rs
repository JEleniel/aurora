use serde::{Deserialize, Serialize};

use crate::registry::RegistryError;
use crate::registry::card_registry::ViewConfiguration;

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
	pub fn try_new_from_view_configuration(json: &str) -> Result<Self, RegistryError> {
		let view_configuration: ViewConfiguration = serde_json::from_str(json)?;
		Ok(Self {
			definitions: view_configuration.views,
		})
	}

	pub fn try_new_from_struct(view_configuration: &ViewConfiguration) -> Self {
		Self {
			definitions: view_configuration.views.clone(),
		}
	}

	pub fn try_get_all(&self) -> Result<Vec<ViewDefinition>, RegistryError> {
		Ok(self.definitions.clone())
	}
}

#[cfg(test)]
#[path = "view_registry_tests.rs"]
mod view_registry_tests;
