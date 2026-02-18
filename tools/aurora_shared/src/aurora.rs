mod auditlog;
mod model;

pub use auditlog::*;
pub use model::*;

#[cfg(test)]
mod aurora_tests;

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use tracing::info;

use crate::registry::RegistryError;
use crate::registry::{CardRegistry, ModelConfiguration, ViewRegistry};

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
	/// A copy of the model-configuration schema located with the models
	pub modelconfiguration_schema: Value,
	/// Parsed model configuration loaded from `reference/Aurora.modelconfiguration.json`
	pub model_configuration: ModelConfiguration,
	/// Parsed card registry derived from model configuration
	pub card_registry: CardRegistry,
	/// Parsed view registry derived from model configuration
	pub view_registry: ViewRegistry,
	/// The SVG template loaded from `reference/SVGTemplate.svg`
	pub svg_template: String,
	/// Warnings collected during initial load checks.
	pub load_warnings: Vec<String>,
	/// Validation errors discovered while performing initial load checks.
	pub load_validation_errors: Vec<String>,
}

impl Aurora {
	/// Try to load a set of Aurora models from a given path
	/// It will automatically pick up if the Model Home is a child
	/// of the given path
	pub fn try_load(path: &Path) -> Result<Self, AuroraError> {
		let mut model_home = path.to_path_buf();
		let mut load_warnings: Vec<String> = Vec::new();

		let folder = Self::last_folder(path)
			.ok_or_else(|| AuroraError::InvalidAuroraHome(path.display().to_string()))?;
		let test_path = path.join("aurora");
		if folder != "aurora" && !test_path.is_dir() {
			return Err(AuroraError::InvalidAuroraHome(path.display().to_string()));
		}
		if folder != "aurora" {
			model_home = test_path;
		}

		let schema_dir = model_home.join("schemas");
		let reference_dir = model_home.join("reference");

		let card_schema_path = schema_dir.join("Aurora.card.schema.json");
		let compact_schema_path = schema_dir.join("Aurora.compact.schema.json");
		let audit_schema_path = schema_dir.join("Aurora.audit.schema.json");
		let modelconfiguration_schema_path =
			schema_dir.join("Aurora.modelconfiguration.schema.json");
		let modelconfiguration_path = reference_dir.join("Aurora.modelconfiguration.json");
		let svg_template_path = reference_dir.join("SVGTemplate.svg");

		for required in [
			&card_schema_path,
			&compact_schema_path,
			&audit_schema_path,
			&modelconfiguration_schema_path,
			&modelconfiguration_path,
			&svg_template_path,
		] {
			if !required.is_file() {
				return Err(AuroraError::RequiredFileMissing(
					required.display().to_string(),
				));
			}
		}

		let card_schema_data = std::fs::read_to_string(&card_schema_path)?;
		let card_schema: Value = serde_json::from_str(&card_schema_data)?;
		let compact_schema_data = std::fs::read_to_string(&compact_schema_path)?;
		let compact_schema: Value = serde_json::from_str(&compact_schema_data)?;
		let audit_schema_data = std::fs::read_to_string(&audit_schema_path)?;
		let audit_schema: Value = serde_json::from_str(&audit_schema_data)?;
		let modelconfiguration_schema_data =
			std::fs::read_to_string(&modelconfiguration_schema_path)?;
		let modelconfiguration_schema: Value =
			serde_json::from_str(&modelconfiguration_schema_data)?;
		let modelconfiguration_data = std::fs::read_to_string(&modelconfiguration_path)?;
		let modelconfiguration_json: Value = serde_json::from_str(&modelconfiguration_data)?;
		let svg_template = std::fs::read_to_string(&svg_template_path)?;

		Self::check_schema_reference(
			&modelconfiguration_path,
			&modelconfiguration_json,
			"Aurora.modelconfiguration.schema.json",
			&mut load_warnings,
		)?;

		let modelconfiguration_compiled =
			jsonschema::JSONSchema::compile(&modelconfiguration_schema)?;
		if let Err(errors) = modelconfiguration_compiled.validate(&modelconfiguration_json) {
			let messages = errors.map(|error| error.to_string()).collect::<Vec<_>>();
			return Err(AuroraError::ReferenceValidationFailed(messages));
		}

		let model_configuration: ModelConfiguration =
			serde_json::from_str(&modelconfiguration_data)?;
		let card_registry = CardRegistry::try_new_from_struct(model_configuration.clone())?;
		let view_registry = ViewRegistry::try_new_from_struct(&model_configuration);

		let svg_icon_ids = extract_svg_icon_ids(&svg_template);
		for icon in &model_configuration.available_icons {
			if !svg_icon_ids.contains(icon) {
				return Err(AuroraError::ReferenceValidationFailed(vec![format!(
					"reference/Aurora.modelconfiguration.json declares icon '{}' but reference/SVGTemplate.svg is missing group id 'i-{}'.",
					icon, icon
				)]));
			}
			if svg_icon_group_is_empty(&svg_template, icon) {
				return Err(AuroraError::ReferenceValidationFailed(vec![format!(
					"reference/Aurora.modelconfiguration.json declares icon '{}' but reference/SVGTemplate.svg has an empty group for id 'i-{}'.",
					icon, icon
				)]));
			}
		}

		let mut models: Vec<Model> = Vec::new();
		for entry in std::fs::read_dir(&model_home)? {
			let entry = entry?;
			if entry.file_type()?.is_dir() {
				continue;
			}
			if let Some(file_name) = entry.file_name().to_str()
				&& file_name.starts_with("MIS-")
				&& file_name.ends_with(".json")
			{
				let model = Model::try_load(&entry.path(), &card_schema, &audit_schema)?;
				models.push(model);
			}
		}

		info!(
			"Loaded {} models from {}",
			models.len(),
			model_home.display()
		);
		models.sort_by(|a, b| a.root_card.id.cmp(&b.root_card.id));

		let mut aurora = Aurora {
			model_home,
			models,
			card_schema,
			compact_schema,
			audit_schema,
			modelconfiguration_schema,
			model_configuration,
			card_registry,
			view_registry,
			svg_template,
			load_warnings,
			load_validation_errors: Vec::new(),
		};

		aurora.load_validation_errors = aurora.validate();
		aurora.load_warnings = aurora.check_registry();

		Ok(aurora)
	}

	/// Check all models against the official registry and
	/// emit warnings if they don't conf=orm
	pub fn check_registry(&self) -> Vec<String> {
		let mut warnings: Vec<String> = self.load_warnings.clone();

		for model in &self.models {
			warnings.extend(
				model
					.get_schema_validation_warnings()
					.into_iter()
					.map(|warning| format!("{}: {}", model.root_card.id, warning)),
			);

			warnings.extend(
				model
					.validate_registry(&self.card_registry)
					.into_iter()
					.map(|w| format!("{}: {}", model.root_card.id, w)),
			);

			warnings.extend(self.boundary_warnings(model));
			warnings.extend(self.custom_acronym_view_warnings(model));
		}
		warnings.sort();
		warnings.dedup();
		warnings
	}

	/// Validate that all models conform to the Inviariants
	pub fn validate(&self) -> Vec<String> {
		let mut errors: Vec<String> = self.load_validation_errors.clone();

		for model in &self.models {
			let model_errors = model.validate();
			if !model_errors.is_empty() {
				errors.extend(
					model_errors
						.into_iter()
						.map(|e| format!("Model {}: {}", model.root_card.id, e)),
				);
			}

			for error in model.validate_acronym_consistency(&self.card_registry) {
				errors.push(format!("Model {}: {}", model.root_card.id, error));
			}

			for error in self.validate_icon_overrides(model) {
				errors.push(format!("Model {}: {}", model.root_card.id, error));
			}

			for error in self.validate_root_safety(model) {
				errors.push(format!("Model {}: {}", model.root_card.id, error));
			}
		}
		errors.sort();
		errors.dedup();
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
	pub fn write_markdown(&self, path: &Path) -> Result<(), AuroraError> {
		for model in &self.models {
			model.write_markdown(path)?;
			info!(
				"Wrote {}: {} ({} cards)",
				model.root_card.id,
				model.root_card.name,
				model.cards.len(),
			);
		}

		Ok(())
	}

	pub fn write_compact(&self, path: &Path) -> Result<(), AuroraError> {
		for model in &self.models {
			let mut mission_dir = path.to_path_buf();
			mission_dir.push(model.root_card.id.as_str());
			std::fs::create_dir_all(&mission_dir)?;

			let output_path = mission_dir.join("Compact.json");
			let compact =
				model.get_compact(Some("../schemas/Aurora.compact.schema.json".to_string()));
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

	fn last_folder(path: &Path) -> Option<&std::ffi::OsStr> {
		path.components()
			.filter_map(|c| match c {
				Component::Normal(name) => Some(name),
				_ => None,
			})
			.next_back()
	}

	fn check_schema_reference(
		json_path: &Path,
		json_value: &Value,
		expected_schema_file_name: &str,
		warnings: &mut Vec<String>,
	) -> Result<(), AuroraError> {
		let schema_ref = json_value.get("$schema").and_then(Value::as_str);
		let Some(schema_ref) = schema_ref else {
			return Err(AuroraError::MissingSchemaReference(
				json_path.display().to_string(),
			));
		};

		let Some(parent) = json_path.parent() else {
			warnings.push(format!(
				"{}: $schema path could not be resolved.",
				json_path.display()
			));
			return Ok(());
		};

		let resolved = parent.join(schema_ref);
		match std::fs::canonicalize(&resolved) {
			Ok(path) => {
				let file_name = path
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or_default();
				if file_name != expected_schema_file_name {
					warnings.push(format!(
						"{}: $schema resolves to '{}' but '{}' is expected.",
						json_path.display(),
						file_name,
						expected_schema_file_name
					));
				}
			}
			Err(_) => warnings.push(format!(
				"{}: $schema '{}' could not be resolved; continuing with expected schema '{}'.",
				json_path.display(),
				schema_ref,
				expected_schema_file_name
			)),
		}

		Ok(())
	}

	fn validate_icon_overrides(&self, model: &Model) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
			if let Some(icon) = &card.icon
				&& !self.card_registry.has_icon(icon)
			{
				errors.push(format!(
					"Card {} has icon override '{}' which is not present in reference/Aurora.modelconfiguration.json available_icons.",
					card.id, icon
				));
			}
		}
		errors
	}

	fn validate_root_safety(&self, model: &Model) -> Vec<String> {
		let cards = model_card_index(model);
		let mut errors: Vec<String> = Vec::new();

		for view in self
			.view_registry
			.try_get_all()
			.unwrap_or_default()
			.into_iter()
		{
			let included: HashSet<String> = view
				.included_card_types
				.iter()
				.chain(view.root_card_types.iter())
				.cloned()
				.collect();

			let roots: Vec<String> = cards
				.values()
				.filter_map(|card| {
					let prefix = id_prefix(&card.id)?;
					if view.root_card_types.iter().any(|root| root == prefix) {
						Some(card.id.clone())
					} else {
						None
					}
				})
				.collect();

			let adjacency = rendered_view_adjacency(&cards, roots.as_slice(), &included);

			for root in roots {
				if root_participates_in_cycle(&root, &adjacency) {
					errors.push(format!(
						"View '{}' root '{}' participates in a cycle.",
						view.name, root
					));
				}
			}
		}

		errors
	}

	fn boundary_warnings(&self, model: &Model) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();
		for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
			if let Some(boundary) = &card.boundary
				&& boundary.trim().is_empty()
			{
				warnings.push(format!(
					"{}: boundary is empty or whitespace; boundary grouping will be ignored.",
					card.id
				));
			}
		}
		warnings
	}

	fn custom_acronym_view_warnings(&self, model: &Model) -> Vec<String> {
		let canonical_acronyms: HashSet<String> = self
			.card_registry
			.definitions
			.iter()
			.map(|definition| definition.acronym.clone())
			.collect();

		let view_acronyms: HashSet<String> = self
			.view_registry
			.try_get_all()
			.unwrap_or_default()
			.into_iter()
			.flat_map(|view| {
				view.root_card_types
					.into_iter()
					.chain(view.included_card_types)
			})
			.collect();

		let mut warnings: Vec<String> = Vec::new();
		for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
			let Some(prefix) = id_prefix(&card.id) else {
				continue;
			};
			if canonical_acronyms.contains(prefix) {
				continue;
			}
			if !view_acronyms.contains(prefix) {
				warnings.push(format!(
					"Custom acronym '{}' is present in model but not included in any view definition.",
					prefix
				));
			}
		}

		warnings.sort();
		warnings.dedup();
		warnings
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
	#[error("Missing required file in model home: {0}")]
	RequiredFileMissing(String),
	#[error("Missing required $schema property in JSON file: {0}")]
	MissingSchemaReference(String),
	#[error("Reference validation failed: {0:?}")]
	ReferenceValidationFailed(Vec<String>),
	#[error("A JSON schema error occurred: {0}")]
	JsonSchemaError(#[from] jsonschema::CompilationError),
	#[error("A JSON parse error occurred: {0}")]
	JsonParseError(#[from] serde_json::Error),
	#[error("A registry error occurred: {0}")]
	RegistryError(#[from] RegistryError),
}

fn extract_svg_icon_ids(svg_template: &str) -> HashSet<String> {
	let mut out: HashSet<String> = HashSet::new();
	let mut search_start = 0usize;
	while let Some(index) = svg_template[search_start..].find("id=\"i-") {
		let start = search_start + index + "id=\"i-".len();
		let remainder = &svg_template[start..];
		if let Some(end) = remainder.find('"') {
			let icon = &remainder[..end];
			if !icon.trim().is_empty() {
				out.insert(icon.to_string());
			}
			search_start = start + end + 1;
		} else {
			break;
		}
	}
	out
}

fn svg_icon_group_is_empty(svg_template: &str, icon: &str) -> bool {
	let needle = format!("id=\"i-{}\"", icon);
	let Some(id_start) = svg_template.find(needle.as_str()) else {
		return false;
	};

	let Some(group_start) = svg_template[..id_start].rfind("<g") else {
		return false;
	};
	let Some(open_end_rel) = svg_template[id_start..].find('>') else {
		return false;
	};
	let open_end = id_start + open_end_rel;
	let open_tag = &svg_template[group_start..=open_end];
	if open_tag.trim_end().ends_with("/>") {
		return true;
	}

	let Some(close_rel) = svg_template[open_end + 1..].find("</g>") else {
		return false;
	};
	let close_start = open_end + 1 + close_rel;
	let content = &svg_template[open_end + 1..close_start];
	content.trim().is_empty()
}

fn id_prefix(id: &str) -> Option<&str> {
	id.split('-').next()
}

fn model_card_index<'a>(model: &'a Model) -> HashMap<String, &'a Card> {
	let mut cards: HashMap<String, &Card> = HashMap::new();
	for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
		cards.insert(card.id.clone(), card);
	}
	cards
}

fn rendered_view_adjacency(
	cards: &HashMap<String, &Card>,
	roots: &[String],
	included_acronyms: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
	let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
	for root in roots {
		adjacency.entry(root.clone()).or_default();
		let mut stack: Vec<String> = vec![root.clone()];
		let mut visited: HashSet<String> = HashSet::from([root.clone()]);

		while let Some(node_id) = stack.pop() {
			let Some(card) = cards.get(&node_id) else {
				continue;
			};
			for link in &card.links {
				let Some(target_card) = cards.get(&link.target) else {
					continue;
				};
				let Some(prefix) = id_prefix(&target_card.id) else {
					continue;
				};
				if !included_acronyms.contains(prefix) {
					continue;
				}

				adjacency
					.entry(node_id.clone())
					.or_default()
					.push(target_card.id.clone());
				adjacency.entry(target_card.id.clone()).or_default();

				if visited.insert(target_card.id.clone()) {
					stack.push(target_card.id.clone());
				}
			}
		}
	}

	for neighbors in adjacency.values_mut() {
		neighbors.sort();
		neighbors.dedup();
	}

	adjacency
}

fn root_participates_in_cycle(root: &str, adjacency: &HashMap<String, Vec<String>>) -> bool {
	let mut stack: Vec<String> = adjacency
		.get(root)
		.cloned()
		.unwrap_or_default()
		.into_iter()
		.collect();
	let mut visited: HashSet<String> = HashSet::new();

	while let Some(node) = stack.pop() {
		if node == root {
			return true;
		}
		if !visited.insert(node.clone()) {
			continue;
		}
		for next in adjacency.get(&node).cloned().unwrap_or_default() {
			stack.push(next);
		}
	}

	false
}

#[cfg(test)]
mod tests {
	use super::svg_icon_group_is_empty;

	#[test]
	fn detects_empty_self_closing_icon_group() {
		let svg = "<svg><defs><g id=\"i-wrench\" /></defs></svg>";
		assert!(svg_icon_group_is_empty(svg, "wrench"));
	}

	#[test]
	fn accepts_non_empty_icon_group() {
		let svg = "<svg><defs><g id=\"i-wrench\"><path d=\"M0 0\" /></g></defs></svg>";
		assert!(!svg_icon_group_is_empty(svg, "wrench"));
	}
}
