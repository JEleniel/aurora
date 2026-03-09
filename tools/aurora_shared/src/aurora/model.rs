mod attribute;
mod card;
mod edit_history;
mod link;

pub use attribute::*;
pub use card::*;
pub use edit_history::*;
pub use link::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, trace};

use crate::registry::CardRegistry;

const MODEL_MARKDOWN_TEMPLATE: &str = include_str!("model.template.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ModelLoadMode {
	#[default]
	ReadOnly,
	ReadWrite,
}

#[derive(Debug)]
pub(crate) struct LoadedModel {
	pub(crate) model: Model,
	pub(crate) audit_log_lock: Option<super::AuditLogFileLock>,
}

#[path = "model/model_write_support.rs"]
pub(super) mod model_write_support;

#[cfg(test)]
mod model_tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
	pub root_card: Card,
	pub cards: Vec<Card>,
	pub audit_log: super::AuditLog,
	pub model_home: PathBuf,
	pub mission_home: PathBuf,
}

impl Model {
	pub fn try_load(
		path: &Path,
		card_schema: &serde_json::Value,
		audit_schema: &serde_json::Value,
	) -> Result<Self, ModelError> {
		Ok(
			Self::try_load_with_mode(path, card_schema, audit_schema, ModelLoadMode::ReadOnly)?
				.model,
		)
	}

	pub(crate) fn try_load_for_update(
		path: &Path,
		card_schema: &serde_json::Value,
		audit_schema: &serde_json::Value,
	) -> Result<LoadedModel, ModelError> {
		Self::try_load_with_mode(path, card_schema, audit_schema, ModelLoadMode::ReadWrite)
	}

	fn try_load_with_mode(
		path: &Path,
		card_schema: &serde_json::Value,
		audit_schema: &serde_json::Value,
		load_mode: ModelLoadMode,
	) -> Result<LoadedModel, ModelError> {
		let root_card = Card::try_load(path, card_schema)?;

		let model_home: PathBuf = path
			.parent()
			.ok_or_else(|| ModelError::InvalidParentPath(path.display().to_string()))?
			.to_path_buf();

		let mut mission_home: PathBuf = model_home.clone();
		mission_home.push(root_card.id.as_str());
		let audit_log_path = mission_home.join("AuditLog.ndjson");
		let (audit_log, audit_log_lock) = load_audit_log(&audit_log_path, audit_schema, load_mode)?;

		let mut cards: Vec<Card> = Vec::new();
		let mut folders_to_visit: Vec<PathBuf> = vec![mission_home.clone()];
		while let Some(current_folder) = folders_to_visit.pop() {
			debug!("Scanning {}", current_folder.display());
			for entry in std::fs::read_dir(&current_folder)? {
				let entry = entry?;
				let entry_path = entry.path();
				if entry.file_type()?.is_dir() {
					folders_to_visit.push(entry_path);
					trace!(
						"Added {} to be scanned",
						folders_to_visit.last().unwrap().display()
					);
					continue;
				}

				if entry_path.extension().and_then(|s| s.to_str()) != Some("json") {
					continue;
				}

				let file_name = entry_path
					.file_name()
					.and_then(|s| s.to_str())
					.unwrap_or_default();
				if file_name == "AuditLog.ndjson" || file_name == "Compact.json" {
					continue;
				}

				if cards.len() >= 99999 {
					return Err(ModelError::ModelTooLarge);
				}

				debug!("Loading card from {}", entry_path.display());
				let card = Card::try_load(&entry_path, card_schema)?;
				if card.card_type == "Mission" {
					return Err(ModelError::UnexpectedMissionCard(
						entry_path.display().to_string(),
					));
				}
				cards.push(card);
			}
		}

		let model = Model {
			root_card,
			cards,
			audit_log,
			model_home,
			mission_home,
		};

		Ok(LoadedModel {
			model,
			audit_log_lock,
		})
	}

	pub fn validate(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		// Check for schema errors
		errors.extend(self.get_schema_validation_errors());

		// Check for invariant violations
		errors.extend(self.validate_no_mission_incoming_links());
		errors.extend(self.validate_reachability_and_orphans());
		errors.extend(self.validate_broken_links());

		errors
	}

	pub fn get_schema_validation_errors(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		if !self.audit_log.validation_errors.is_empty() {
			for error in self.audit_log.validation_errors.iter() {
				errors.push(format!("AuditLog: {}", error));
			}
		}
		if !self.root_card.validation_errors.is_empty() {
			for error in self.root_card.validation_errors.iter() {
				errors.push(format!("{}: {}", self.root_card.id.clone(), error,));
			}
		}

		for card in &self.cards {
			if card.validation_errors.is_empty() {
				continue;
			}
			for error in card.validation_errors.iter() {
				errors.push(format!("{}: {}", card.id.clone(), error,));
			}
		}
		errors
	}

	/// Check against the registry
	pub fn validate_registry(&self, registry: &CardRegistry) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();

		for card in &self.cards {
			warnings.extend(card.check_registry(registry));
		}
		warnings.extend(self.root_card.check_registry(registry));
		warnings
	}

	pub fn get_schema_validation_warnings(&self) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();
		for warning in &self.root_card.validation_warnings {
			warnings.push(format!("{}: {}", self.root_card.id, warning));
		}
		for card in &self.cards {
			for warning in &card.validation_warnings {
				warnings.push(format!("{}: {}", card.id, warning));
			}
		}
		warnings
	}

	pub fn validate_acronym_consistency(&self, registry: &CardRegistry) -> Vec<String> {
		let canonical_by_type: HashMap<String, String> = registry
			.definitions
			.iter()
			.map(|definition| (definition.card_type.clone(), definition.acronym.clone()))
			.collect();

		let mut non_canonical_by_type: HashMap<String, String> = HashMap::new();
		let mut errors: Vec<String> = Vec::new();

		for card in std::iter::once(&self.root_card).chain(self.cards.iter()) {
			let prefix = id_prefix(&card.id).unwrap_or_default().to_string();
			if prefix.len() != 3 {
				errors.push(format!(
					"Card {} has invalid ID prefix '{}' (expected 3 uppercase letters).",
					card.id, prefix
				));
				continue;
			}

			if let Some(expected) = canonical_by_type.get(&card.card_type) {
				if expected != &prefix {
					errors.push(format!(
						"Card {} has prefix '{}' but card type '{}' requires canonical acronym '{}'.",
						card.id, prefix, card.card_type, expected
					));
				}
				continue;
			}

			if let Some(existing) = non_canonical_by_type.get(&card.card_type) {
				if existing != &prefix {
					errors.push(format!(
						"Non-canonical card type '{}' uses inconsistent acronyms '{}' and '{}'.",
						card.card_type, existing, prefix
					));
				}
			} else {
				non_canonical_by_type.insert(card.card_type.clone(), prefix);
			}
		}

		errors
	}

	pub fn get_compact(&self, schema_ref: Option<String>) -> Value {
		let mut cards: Vec<Value> = Vec::new();
		cards.push(self.root_card.get_compact());
		for card in &self.cards {
			cards.push(card.get_compact());
		}

		let mut root = serde_json::Map::new();
		if let Some(schema_ref) = schema_ref {
			root.insert("$schema".to_string(), Value::String(schema_ref));
		}
		root.insert("cards".to_string(), Value::Array(cards));
		Value::Object(root)
	}

	/// Test Invariant 2a: All cards lead away from Mission
	fn validate_no_mission_incoming_links(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		for card in &self.cards {
			for link in &card.links {
				if link.target == self.root_card.id {
					errors.push(format!("Card {} links to the mission card.", card.id));
				}
			}
		}
		errors
	}

	/// Test Invariant 2b: All cards reachable from Mission
	/// Test Invariant 3: No orphans
	fn validate_reachability_and_orphans(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		let mut cards_by_id: HashMap<&str, &Card> = HashMap::new();
		cards_by_id.insert(&self.root_card.id, &self.root_card);
		for card in &self.cards {
			cards_by_id.insert(&card.id, card);
		}

		let mut incoming: HashMap<&str, usize> = HashMap::new();
		for card in cards_by_id.values() {
			for link in &card.links {
				if let Some(target) = cards_by_id.get(link.target.as_str()) {
					let count = incoming.entry(target.id.as_str()).or_insert(0);
					*count += 1;
				}
			}
		}

		for card in &self.cards {
			if incoming.get(card.id.as_str()).copied().unwrap_or(0) == 0 {
				errors.push(format!("Card {} is orphaned (no incoming links).", card.id));
			}
		}

		let mut reachable: HashSet<&str> = HashSet::new();
		let mut stack: Vec<&str> = vec![self.root_card.id.as_str()];
		while let Some(id) = stack.pop() {
			if !reachable.insert(id) {
				continue;
			}
			let card = cards_by_id.get(id).copied().unwrap_or(&self.root_card);
			for link in &card.links {
				stack.push(link.target.as_str());
			}
		}

		for card in &self.cards {
			if !reachable.contains(card.id.as_str()) {
				errors.push(format!(
					"Card {} is not reachable from mission {}.",
					card.id, self.root_card.id
				));
			}
		}

		errors
	}

	/// Test Invariant 6: No broken links
	fn validate_broken_links(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		let mut known: HashSet<&str> = HashSet::new();
		known.insert(self.root_card.id.as_str());
		for card in &self.cards {
			known.insert(card.id.as_str());
		}

		for card in std::iter::once(&self.root_card).chain(self.cards.iter()) {
			for link in &card.links {
				if !known.contains(link.target.as_str()) {
					errors.push(format!(
						"Card {} has a broken link to {}.",
						card.id, link.target
					));
				}
			}
		}
		errors
	}
}

fn load_audit_log(
	path: &Path,
	audit_schema: &Value,
	load_mode: ModelLoadMode,
) -> Result<(super::AuditLog, Option<super::AuditLogFileLock>), ModelError> {
	match load_mode {
		ModelLoadMode::ReadOnly => Ok((super::AuditLog::try_load(path, audit_schema)?, None)),
		ModelLoadMode::ReadWrite => {
			let loaded = super::AuditLog::try_load_for_update(path, audit_schema)
				.map_err(map_audit_log_error)?;
			Ok((loaded.audit_log, Some(loaded.lock)))
		}
	}
}

fn map_audit_log_error(error: super::AuditLogError) -> ModelError {
	match error {
		super::AuditLogError::ModelLocked(path) => ModelError::ModelLocked(path),
		other => ModelError::AuditLogError(other),
	}
}

fn sanitize_filename(name: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in name.chars() {
		if ch.is_ascii_alphanumeric() {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() && !last_was_underscore {
			out.push('_');
			last_was_underscore = true;
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}

fn sanitize_card_type_folder(card_type: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in card_type.chars() {
		if ch.is_ascii_alphanumeric() || ch == '_' {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() && !last_was_underscore {
			out.push('_');
			last_was_underscore = true;
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}

fn id_prefix(id: &str) -> Option<&str> {
	id.split('-').next()
}

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("Failed to read model file: {0}")]
	ReadError(#[from] std::io::Error),
	#[error("Model is locked by another write session: {0}")]
	ModelLocked(String),
	#[error("A Card error has occurred: {0}")]
	CardError(#[from] CardError),
	#[error("An AuditLog error has occurred: {0}")]
	AuditLogError(#[from] super::AuditLogError),
	#[error("Invalid filename")]
	InvalidFilename,
	#[error("Invalid parent path for model root: {0}")]
	InvalidParentPath(String),
	#[error("Unexpected Mission card at {0}")]
	UnexpectedMissionCard(String),
	#[error("Model validation failed: {0:?}")]
	ValidationErrors(Vec<String>),
	#[error("Model exceeds maximum allowed size of 99999 cards.")]
	ModelTooLarge,
}
