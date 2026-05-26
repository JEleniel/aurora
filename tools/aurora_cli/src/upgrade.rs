//! Upgrade helpers for migrating model files to the latest supported schema.

use anyhow::{Context, Result, bail};
use aurora_shared::{
	Aurora, MODEL_CONFIGURATION_VERSION, ModelConfiguration, ModelConfigurationCardDefinition,
	VIEW_CONFIGURATION_VERSION, ViewConfiguration, ViewConfigurationCardDefinition, ViewDefinition,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const SCHEMA_ASSETS: [(&str, &str); 5] = [
	(
		"Aurora.audit.schema.json",
		include_str!("../../../.github/aurora/schemas/Aurora.audit.schema.json"),
	),
	(
		"Aurora.card.schema.json",
		include_str!("../../../.github/aurora/schemas/Aurora.card.schema.json"),
	),
	(
		"Aurora.compact.schema.json",
		include_str!("../../../.github/aurora/schemas/Aurora.compact.schema.json"),
	),
	(
		"Aurora.modelconfiguration.schema.json",
		include_str!("../../../.github/aurora/schemas/Aurora.modelconfiguration.schema.json"),
	),
	(
		"Aurora.viewconfiguration.schema.json",
		include_str!("../../../.github/aurora/schemas/Aurora.viewconfiguration.schema.json"),
	),
];

const REFERENCE_TEXT_ASSETS: [(&str, &str); 2] = [
	(
		"Aurora.modelconfiguration.json",
		include_str!("../../../.github/aurora/reference/Aurora.modelconfiguration.json"),
	),
	(
		"Aurora.viewconfiguration.json",
		include_str!("../../../.github/aurora/reference/Aurora.viewconfiguration.json"),
	),
];

const REFERENCE_BINARY_ASSETS: [(&str, &[u8]); 1] = [(
	"SVGTemplate.svgz",
	include_bytes!("../../../.github/aurora/reference/SVGTemplate.svgz"),
)];
const REFERENCE_CONFIGURATION_FILES: [&str; 2] = [
	"Aurora.modelconfiguration.json",
	"Aurora.viewconfiguration.json",
];

const NO_BINARY_ASSETS: [(&str, &[u8]); 0] = [];

pub fn upgrade_in_place(input_path: &Path) -> Result<()> {
	let model_home = resolve_model_home(input_path)?;
	sync_model_home_assets(&model_home)?;

	let aurora = Aurora::try_load(&model_home)
		.with_context(|| format!("Failed to load model home {}", model_home.display()))?;

	let mut upgraded_cards = 0usize;

	for model in &aurora.models {
		let cards = std::iter::once(&model.root_card).chain(model.cards.iter());
		for card in cards {
			if upgrade_card_file(&card.source_path)? {
				upgraded_cards += 1;
			}
		}
	}

	info!("Upgraded {upgraded_cards} card(s).");
	Ok(())
}

fn resolve_model_home(path: &Path) -> Result<PathBuf> {
	if path.file_name() == Some(OsStr::new("aurora")) && path.is_dir() {
		return Ok(path.to_path_buf());
	}

	let candidate = path.join("aurora");
	if candidate.is_dir() {
		return Ok(candidate);
	}

	bail!(
		"The specified path is not a valid Aurora model home: {}",
		path.display()
	)
}

fn sync_model_home_assets(model_home: &Path) -> Result<()> {
	sync_directory_assets(
		&model_home.join("schemas"),
		&SCHEMA_ASSETS,
		&NO_BINARY_ASSETS,
	)?;
	sync_reference_assets(&model_home.join("reference"))?;
	upgrade_reference_configurations(model_home)?;
	Ok(())
}

fn sync_directory_assets(
	directory: &Path,
	text_assets: &[(&str, &str)],
	binary_assets: &[(&str, &[u8])],
) -> Result<()> {
	std::fs::create_dir_all(directory)
		.with_context(|| format!("Failed to create directory {}", directory.display()))?;

	let required = required_file_names(text_assets, binary_assets);
	prune_outdated_files(directory, &required)?;

	for (name, contents) in text_assets {
		write_asset_file(directory, name, contents.as_bytes())?;
	}
	for (name, contents) in binary_assets {
		write_asset_file(directory, name, contents)?;
	}

	Ok(())
}

fn sync_reference_assets(directory: &Path) -> Result<()> {
	std::fs::create_dir_all(directory)
		.with_context(|| format!("Failed to create directory {}", directory.display()))?;
	let mut required = required_file_names(&[], &REFERENCE_BINARY_ASSETS);
	required.extend(
		REFERENCE_CONFIGURATION_FILES
			.iter()
			.map(|name| (*name).to_string()),
	);
	prune_outdated_files(directory, &required)?;
	for (name, contents) in REFERENCE_BINARY_ASSETS {
		write_asset_file(directory, name, contents)?;
	}
	Ok(())
}

fn required_file_names(
	text_assets: &[(&str, &str)],
	binary_assets: &[(&str, &[u8])],
) -> HashSet<String> {
	text_assets
		.iter()
		.map(|(name, _)| (*name).to_string())
		.chain(binary_assets.iter().map(|(name, _)| (*name).to_string()))
		.collect()
}

fn prune_outdated_files(directory: &Path, required: &HashSet<String>) -> Result<()> {
	for entry in std::fs::read_dir(directory)
		.with_context(|| format!("Failed to read directory {}", directory.display()))?
	{
		let entry = entry.with_context(|| {
			format!(
				"Failed to read directory entry under {}",
				directory.display()
			)
		})?;
		if !entry
			.file_type()
			.with_context(|| format!("Failed to inspect {}", entry.path().display()))?
			.is_file()
		{
			continue;
		}

		let Some(name) = entry.file_name().to_str().map(|value| value.to_string()) else {
			continue;
		};
		if required.contains(&name) {
			continue;
		}

		std::fs::remove_file(entry.path()).with_context(|| {
			format!("Failed to remove outdated file {}", entry.path().display())
		})?;
		info!("Removed outdated file {}", entry.path().display());
	}

	Ok(())
}

fn write_asset_file(directory: &Path, file_name: &str, bytes: &[u8]) -> Result<()> {
	let path = directory.join(file_name);
	std::fs::write(&path, bytes)
		.with_context(|| format!("Failed to write required file {}", path.display()))?;
	Ok(())
}

fn upgrade_reference_configurations(model_home: &Path) -> Result<()> {
	let reference_dir = model_home.join("reference");
	let default_model: ModelConfiguration = serde_json::from_str(REFERENCE_TEXT_ASSETS[0].1)?;
	let default_view: ViewConfiguration = serde_json::from_str(REFERENCE_TEXT_ASSETS[1].1)?;
	let current_model = read_model_configuration(&reference_dir.join(REFERENCE_TEXT_ASSETS[0].0))?;
	let current_view = read_view_configuration(&reference_dir.join(REFERENCE_TEXT_ASSETS[1].0))?;
	write_reference_json(
		&reference_dir.join(REFERENCE_TEXT_ASSETS[0].0),
		&serde_json::to_value(upgrade_model_configuration(current_model, &default_model))?,
	)?;
	write_reference_json(
		&reference_dir.join(REFERENCE_TEXT_ASSETS[1].0),
		&serde_json::to_value(upgrade_view_configuration(current_view, &default_view))?,
	)?;
	Ok(())
}

fn read_model_configuration(path: &Path) -> Result<ModelConfiguration> {
	if !path.is_file() {
		return Ok(serde_json::from_str(REFERENCE_TEXT_ASSETS[0].1)?);
	}
	Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn read_view_configuration(path: &Path) -> Result<ViewConfiguration> {
	if !path.is_file() {
		return Ok(serde_json::from_str(REFERENCE_TEXT_ASSETS[1].1)?);
	}
	Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn write_reference_json(path: &Path, value: &Value) -> Result<()> {
	std::fs::write(path, serde_json::to_string_pretty(value)?)
		.with_context(|| format!("Failed to write upgraded reference file {}", path.display()))?;
	Ok(())
}

fn upgrade_model_configuration(
	current: ModelConfiguration,
	defaults: &ModelConfiguration,
) -> ModelConfiguration {
	let mut remaining: HashMap<String, ModelConfigurationCardDefinition> = current
		.cards
		.into_iter()
		.map(|card| (card.acronym.clone(), card))
		.collect();
	let mut cards = defaults.cards.clone();
	for card in &cards {
		remaining.remove(&card.acronym);
	}
	let mut custom_cards: Vec<ModelConfigurationCardDefinition> = remaining.into_values().collect();
	custom_cards.sort_by(|left, right| left.acronym.cmp(&right.acronym));
	cards.extend(custom_cards);
	ModelConfiguration {
		schema: current.schema.or_else(|| defaults.schema.clone()),
		version: Some(MODEL_CONFIGURATION_VERSION.to_string()),
		cards,
	}
}

fn upgrade_view_configuration(
	current: ViewConfiguration,
	defaults: &ViewConfiguration,
) -> ViewConfiguration {
	let mut appearances: HashMap<String, ViewConfigurationCardDefinition> = current
		.cards
		.into_iter()
		.map(|card| (card.acronym.clone(), card))
		.collect();
	let mut cards = defaults.cards.clone();
	for card in &cards {
		appearances.remove(&card.acronym);
	}
	let mut custom_cards: Vec<ViewConfigurationCardDefinition> =
		appearances.into_values().collect();
	custom_cards.sort_by(|left, right| left.acronym.cmp(&right.acronym));
	cards.extend(custom_cards);
	let mut views = defaults.views.clone();
	views.extend(custom_views(current.views, defaults.views.as_slice()));
	ViewConfiguration {
		schema: current.schema.or_else(|| defaults.schema.clone()),
		version: Some(VIEW_CONFIGURATION_VERSION.to_string()),
		available_icons: merged_icons(current.available_icons, defaults.available_icons.as_slice()),
		cards,
		domains: if current.domains.is_empty() {
			defaults.domains.clone()
		} else {
			current.domains
		},
		views,
	}
}

fn merged_icons(current: Vec<String>, defaults: &[String]) -> Vec<String> {
	let mut seen: HashSet<String> = HashSet::new();
	let mut icons = defaults.to_vec();
	for icon in &icons {
		seen.insert(icon.clone());
	}
	for icon in current {
		if seen.insert(icon.clone()) {
			icons.push(icon);
		}
	}
	icons
}

fn custom_views(current: Vec<ViewDefinition>, defaults: &[ViewDefinition]) -> Vec<ViewDefinition> {
	let default_names: HashSet<String> = defaults.iter().map(|view| view.name.clone()).collect();
	let mut custom: Vec<ViewDefinition> = current
		.into_iter()
		.filter(|view| !default_names.contains(&view.name))
		.collect();
	custom.sort_by(|left, right| left.name.cmp(&right.name));
	custom
}

fn upgrade_card_file(path: &Path) -> Result<bool> {
	let data = std::fs::read_to_string(path)
		.with_context(|| format!("Failed to read card file {}", path.display()))?;
	let mut json: Value = serde_json::from_str(&data)
		.with_context(|| format!("Failed to parse card JSON {}", path.display()))?;

	let changed = upgrade_card_json(&mut json, path)?;
	if !changed {
		return Ok(false);
	}

	let serialized = serde_json::to_string_pretty(&json)
		.with_context(|| format!("Failed to serialize upgraded card JSON {}", path.display()))?;
	std::fs::write(path, serialized)
		.with_context(|| format!("Failed to write upgraded card JSON {}", path.display()))?;

	Ok(true)
}

fn upgrade_card_json(json: &mut Value, path: &Path) -> Result<bool> {
	let Some(obj) = json.as_object_mut() else {
		warn!("Skipping non-object card JSON: {}", path.display());
		return Ok(false);
	};

	let existing_external_references =
		match parse_external_references(obj.get("external_references")) {
			Ok(values) => values,
			Err(message) => {
				warn!(
					"Card {} has invalid external_references ({message}); refusing to upgrade to avoid data loss.",
					path.display()
				);
				return Ok(false);
			}
		};

	let Some(mut attributes) = take_attributes_object(obj, path)? else {
		return Ok(false);
	};

	let (migrated_references, remove_references) =
		try_extract_migratable(&attributes, "references", path);
	let (migrated_external_reference, remove_external_reference) =
		try_extract_migratable(&attributes, "external_reference", path);

	let mut migrated = Vec::new();
	migrated.extend(migrated_references);
	migrated.extend(migrated_external_reference);

	let mut changed = false;
	if remove_references {
		attributes.remove("references");
		changed = true;
	}
	if remove_external_reference {
		attributes.remove("external_reference");
		changed = true;
	}

	let new_external_references =
		merge_preserving_order(existing_external_references.clone(), migrated);
	if new_external_references != existing_external_references {
		set_external_references(obj, &new_external_references);
		changed = true;
	}

	put_attributes_object(obj, attributes);
	Ok(changed)
}

fn merge_preserving_order(existing: Vec<String>, additional: Vec<String>) -> Vec<String> {
	let mut out: Vec<String> = Vec::new();
	let mut seen: HashSet<String> = HashSet::new();

	for value in existing.into_iter().chain(additional) {
		if seen.insert(value.clone()) {
			out.push(value);
		}
	}

	out
}

fn set_external_references(obj: &mut serde_json::Map<String, Value>, values: &[String]) {
	obj.insert(
		"external_references".to_string(),
		Value::Array(values.iter().cloned().map(Value::String).collect()),
	);
}

fn take_attributes_object(
	obj: &mut serde_json::Map<String, Value>,
	path: &Path,
) -> Result<Option<serde_json::Map<String, Value>>> {
	let Some(attributes) = obj.remove("attributes") else {
		return Ok(None);
	};
	let Value::Object(attributes) = attributes else {
		warn!(
			"Card {} has non-object attributes; refusing to upgrade to avoid data loss.",
			path.display()
		);
		obj.insert("attributes".to_string(), attributes);
		return Ok(None);
	};
	Ok(Some(attributes))
}

fn put_attributes_object(
	obj: &mut serde_json::Map<String, Value>,
	attributes: serde_json::Map<String, Value>,
) {
	obj.insert("attributes".to_string(), Value::Object(attributes));
}

fn try_extract_migratable(
	attributes: &serde_json::Map<String, Value>,
	key: &str,
	path: &Path,
) -> (Vec<String>, bool) {
	let Some(value) = attributes.get(key) else {
		return (Vec::new(), false);
	};
	match extract_non_empty_strings(value) {
		Ok(values) => {
			if values.is_empty() {
				return (Vec::new(), false);
			}
			(values, true)
		}
		Err(message) => {
			warn!(
				"Card {} has attributes.{key} that cannot be migrated ({message}); leaving it in place.",
				path.display()
			);
			(Vec::new(), false)
		}
	}
}

fn parse_external_references(
	value: Option<&Value>,
) -> std::result::Result<Vec<String>, &'static str> {
	let Some(value) = value else {
		return Ok(Vec::new());
	};
	extract_non_empty_strings(value)
}

fn extract_non_empty_strings(value: &Value) -> std::result::Result<Vec<String>, &'static str> {
	match value {
		Value::String(s) => {
			let trimmed = s.trim();
			if trimmed.is_empty() {
				return Err("string is empty");
			}
			Ok(vec![trimmed.to_string()])
		}
		Value::Array(values) => {
			let mut out: Vec<String> = Vec::new();
			for value in values {
				let Value::String(s) = value else {
					return Err("array contains non-string");
				};
				let trimmed = s.trim();
				if trimmed.is_empty() {
					return Err("array contains empty string");
				}
				out.push(trimmed.to_string());
			}
			Ok(out)
		}
		Value::Null => Ok(Vec::new()),
		_ => Err("unsupported type"),
	}
}
