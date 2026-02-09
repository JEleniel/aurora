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
use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::{debug, trace};

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
		path: &Path,
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
			warnings.extend(
				card.check_registry()
					.unwrap_or_else(|e| vec![format!("{}: Registry error: {}", card.id, e)]),
			);
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

	pub fn write_markdown(&self, path: &Path) -> Result<(), ModelError> {
		let mission_slug = sanitize_filename(&self.root_card.name);
		let mut readme_path = path.to_path_buf();
		readme_path.push(format!("README-{}-{}.md", self.root_card.id, mission_slug));

		let mission_md_path = path.join(format!("{}-{}.md", self.root_card.id, mission_slug));

		// Map card IDs to their markdown output paths so we can produce correct relative links.
		let mut markdown_paths_by_id: HashMap<String, PathBuf> = HashMap::new();
		markdown_paths_by_id.insert(self.root_card.id.clone(), mission_md_path.clone());
		for card in &self.cards {
			let card_slug = sanitize_filename(&card.name);
			let card_md_path = path
				.join(self.root_card.id.as_str())
				.join(card.card_type.as_str())
				.join(format!("{}-{}.md", card.id, card_slug));
			markdown_paths_by_id.insert(card.id.clone(), card_md_path);
		}

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
		let view_path = path.join(self.root_card.id.as_str()).join("Views");
		if view_path.exists() {
			let mut svg_files: Vec<String> = Vec::new();
			for entry in fs::read_dir(&view_path)? {
				let entry = entry?;
				let is_svg = entry.file_type()?.is_file()
					&& entry.path().extension().and_then(|s| s.to_str()) == Some("svg");
				if is_svg {
					svg_files.push(entry.file_name().to_string_lossy().into_owned());
				}
			}
			svg_files.sort();
			for file_name in svg_files {
				views.push_str(&format!(
					"\n![{}]({}/Views/{})\n",
					file_name, self.root_card.id, file_name
				));
			}
		}
		if views.is_empty() {
			views.push_str("_No views available._");
		}
		markdown = markdown.replace("{{views}}", &views);

		let mut index: String = String::new();
		// Only emit card types that exist in this model.
		let mut card_types: Vec<String> = self.cards.iter().map(|c| c.card_type.clone()).collect();
		card_types.sort();
		card_types.dedup();
		for card_type in card_types {
			let cards_of_type: Vec<&Card> = self
				.cards
				.iter()
				.filter(|c| c.card_type == card_type)
				.collect();
			if cards_of_type.is_empty() {
				continue;
			}

			index.push_str(format!("### {}\n\n", card_type).as_str());
			for card in cards_of_type {
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
		// Keep at most a single blank line between sections.
		while markdown.contains("\n\n\n") {
			markdown = markdown.replace("\n\n\n", "\n\n");
		}

		std::fs::write(&readme_path, markdown)?;

		self.root_card.write_markdown(
			&mission_md_path,
			Some(&markdown_paths_by_id),
			self.audit_log.entries_for_target(&self.root_card.id),
		);

		for card in &self.cards {
			let mut card_path = path.to_path_buf();
			card_path.push(self.root_card.id.as_str());
			card_path.push(&card.card_type);
			fs::create_dir_all(&card_path)?;
			let card_slug = sanitize_filename(&card.name);
			card_path.push(format!("{}-{}.md", card.id, card_slug));
			card.write_markdown(
				&card_path,
				Some(&markdown_paths_by_id),
				self.audit_log.entries_for_target(&card.id),
			);
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
			.flat_map(|card| card.links.iter())
			.filter(|link| link.target == self.root_card.id)
			.map(|link| format!("Card {} links to the mission card.", link.target))
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
	#[error("Model exceeds maximum allowed size of 99999 cards.")]
	ModelTooLarge,
}
