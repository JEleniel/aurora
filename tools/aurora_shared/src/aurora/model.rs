mod attribute;
mod card;
mod compactor;
mod edit_history;
mod link;
mod loader;
mod persistence;
mod validator;

pub use attribute::*;
pub use card::*;
pub use edit_history::*;
pub use link::*;
pub use loader::{LoadMode, LoadedModel};
pub(crate) use persistence::{sanitize_card_type_folder, sanitize_filename};
pub use validator::ValidationReport;
pub(crate) use validator::id_prefix;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use thiserror::Error;

use crate::registry::CardRegistry;

const MODEL_MARKDOWN_TEMPLATE: &str = include_str!("model.template.md");

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

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("Failed to read model file: {0}")]
	ReadError(#[from] std::io::Error),
	#[error("Model is locked by another write session: {0}")]
	ModelLocked(String),
	#[error("A Card error has occurred: {0}")]
	CardError(#[from] CardError),
	#[error("An AuditLog error has occurred: {0}")]
	AuditLogError(super::AuditLogError),
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

impl From<super::AuditLogError> for ModelError {
	fn from(error: super::AuditLogError) -> Self {
		match error {
			super::AuditLogError::ModelLocked(path) => Self::ModelLocked(path),
			other => Self::AuditLogError(other),
		}
	}
}
