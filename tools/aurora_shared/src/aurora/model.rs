mod attribute;
mod card;
mod link;

pub use attribute::*;
pub use card::*;
pub use link::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
	collections::{HashMap, HashSet},
	fs,
};
use std::{ffi::OsStr, path::PathBuf};
use thiserror::Error;
use tracing::{debug, trace};

use crate::registry::CardDefinition;

const MODEL_MARKDOWN_TEMPLATE: &str = include_str!("model.template.md");

#[cfg(test)]
mod model_tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
	pub root_card: Card,
	pub cards: Vec<Card>,
	pub audit_log: super::AuditLog,
	pub model_home: PathBuf,
	pub mission_home: PathBuf,
}

impl Model {
	pub fn try_load(
		path: &PathBuf,
		card_schema: &serde_json::Value,
		audit_schema: &serde_json::Value,
	) -> Result<Self, ModelError> {
		let root_card = Card::try_load(path, card_schema)?;

		let model_home: PathBuf = path
			.parent()
			.ok_or_else(|| ModelError::InvalidParentPath(path.display().to_string()))?
			.to_path_buf();

		let mut mission_home: PathBuf = model_home.clone();
		mission_home.push(root_card.id.as_str());
		let audit_log_path = mission_home.join("AuditLog.json");
		let audit_log = super::AuditLog::try_load(&audit_log_path, audit_schema)?;

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
				if file_name == "AuditLog.json" || file_name == "Compact.json" {
					continue;
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

		Ok(model)
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
	pub fn validate_registry(&self) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();

		for card in &self.cards {
			warnings.extend(card.check_registry());
		}
		warnings
	}

	pub fn write(&self) -> Result<(), ModelError> {
		let ext = self
			.root_card
			.source_path
			.extension()
			.unwrap_or(OsStr::new("json"))
			.to_str()
			.unwrap();

		let mut mission_path = self.model_home.clone();
		mission_path.push(format!(
			"{}-{}.{}",
			&self.root_card.id,
			sanitize_filename(&self.root_card.name),
			ext
		));
		self.root_card.write(&mission_path);

		for card in &self.cards {
			let mut card_path = self.mission_home.clone();
			card_path.push(&card.card_type);
			fs::create_dir_all(&card_path)?;

			card_path.push(format!(
				"{}-{}.{}",
				card.id,
				sanitize_filename(&card.name),
				ext
			));
			card.write(&card_path);
		}
		Ok(())
	}

	pub fn write_markdown(&self, path: &PathBuf) -> Result<(), ModelError> {
		let mission_slug = sanitize_filename(&self.root_card.name);
		let mut readme_path = path.clone();
		readme_path.push(format!("README-{}-{}.md", self.root_card.id, mission_slug));

		let mission_md_path = &path.join(format!("{}-{}.md", self.root_card.id, mission_slug));

		let mut markdown = String::from(MODEL_MARKDOWN_TEMPLATE);

		let mission_link = format!(
			"**[Mission Card]({}-{}.md)**",
			self.root_card.id, mission_slug
		);

		markdown = markdown
			.replace("{{id}}", self.root_card.id.as_str())
			.replace("{{name}}", self.root_card.name.as_str())
			.replace("{{mission_link}}", &mission_link)
			.replace("{{description}}", self.root_card.description.as_str());

		let mut views: String = String::new();
		let mut view_path = path.clone();
		view_path.push(format!("{}-views", self.root_card.id));
		if view_path.exists() {
			for entry in fs::read_dir(&view_path)? {
				let entry = entry?;
				if entry.file_type()?.is_file() {
					if let Some(ext) = entry.path().extension() {
						if ext == "svg" {
							views.push_str(&format!(
								"\n![{}]({}-views/{})\n",
								entry.file_name().to_str().unwrap(),
								self.root_card.id,
								entry.path().file_name().unwrap().to_str().unwrap()
							));
						}
					}
				}
			}
		}
		if views.is_empty() {
			views.push_str("_No views available._");
		}
		markdown = markdown.replace("{{views}}", &views);

		let mut index: String = String::new();
		let mut card_types: Vec<String> = CardDefinition::get_all()
			.iter()
			.map(|c| c.card_type.to_string())
			.collect();
		card_types.sort_by(|a, b| a.cmp(b));
		for card_type in card_types {
			index.push_str(format!("### {}\n\n", card_type).as_str());

			for card in self.cards.iter().filter(|c| c.card_type == card_type) {
				let card_slug = sanitize_filename(&card.name);
				let card_link = format!(
					"- **[{} - {}]({}/{}/{}-{}.md)**: {}\n\n",
					card.id,
					card.name,
					self.root_card.id,
					card.card_type,
					card.id,
					card_slug,
					card.description
				);
				index.push_str(card_link.as_str());
			}
		}
		markdown = markdown.replace("{{index}}", &index);
		markdown = markdown.replace("\n\n\n", "\n");

		std::fs::write(&readme_path, markdown)?;

		self.root_card.write_markdown(
			mission_md_path,
			self.audit_log.entries_for_target(&self.root_card.id),
		);

		for card in &self.cards {
			let mut card_path = path.clone();
			card_path.push(self.root_card.id.as_str());
			card_path.push(&card.card_type);
			fs::create_dir_all(&card_path)?;
			let card_slug = sanitize_filename(&card.name);
			card_path.push(format!("{}-{}.md", card.id, card_slug));
			card.write_markdown(&card_path, self.audit_log.entries_for_target(&card.id));
		}
		Ok(())
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
		self.cards
			.iter()
			.map(|c| c.links.iter().map(|l| l.target.clone()))
			.flatten()
			.filter(|target_id| *target_id == self.root_card.id)
			.map(|target_id| format!("Card {} links to the mission card.", target_id))
			.collect()
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

fn sanitize_filename(name: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in name.chars() {
		if ch.is_ascii_alphanumeric() {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() {
			if !last_was_underscore {
				out.push('_');
				last_was_underscore = true;
			}
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("Failed to read model file: {0}")]
	ReadError(#[from] std::io::Error),
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
	#[error("{0}")]
	ValidationErrors(String),
}
