mod attribute;
mod audit_trail;
mod card;
mod link;

pub use attribute::*;
pub use card::*;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashSet, fs};
use std::{ffi::OsStr, path::PathBuf};
use thiserror::Error;
use tracing::{debug, trace};

use crate::registry::CardDefinition;

const MODEL_MARKDOWN_TEMPLATE: &str = include_str!("model.template.md");

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
	pub root_card: Card,
	pub cards: Vec<Card>,
	pub mission_home: PathBuf,
}

impl Model {
	pub fn try_load(path: &PathBuf, card_schema: &serde_json::Value) -> Result<Self, ModelError> {
		let root_card = Card::try_load(path, card_schema)?;

		let mut mission_home: PathBuf = path
			.parent()
			.ok_or_else(|| ModelError::InvalidParentPath(path.display().to_string()))?
			.to_path_buf()
			.clone();
		mission_home.push(root_card.id.as_str());

		let mut cards: Vec<Card> = Vec::new();
		let mut folders_to_visit: Vec<PathBuf> = vec![mission_home.clone()];
		while !folders_to_visit.is_empty() {
			if let Some(current_folder) = folders_to_visit.pop() {
				debug!("Scanning {}", current_folder.display());
				for entry in std::fs::read_dir(&current_folder)? {
					let entry = entry?;
					if entry.file_type()?.is_dir() {
						folders_to_visit.push(entry.path());
						trace!("Added {} to be scanned", entry.path().display());
						continue;
					} else {
						debug!("Loading card from {}", entry.path().display());
						let card = Card::try_load(&entry.path(), card_schema)?;
						cards.push(card);
					}
				}
			}
		}

		Ok(Model {
			root_card,
			cards,
			mission_home,
		})
	}

	pub fn validate(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		// Check for schema errors
		errors.extend(self.get_schema_validation_errors());

		// Check for invariant violations
		errors.extend(self.validate_single_mission());
		errors.extend(self.validate_no_mission_incoming_links());
		errors.extend(self.find_unlinked_cards());
		errors.extend(self.validate_broken_links());
		errors.extend(self.validate_bounary());
		errors.extend(self.validate_notes());

		errors
	}

	pub fn get_schema_validation_errors(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
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

		let mut mission_path = self.mission_home.clone();
		mission_path.push(format!(
			"{}-{}.{}",
			&self.root_card.id, self.root_card.name, ext
		));
		self.root_card.write(&mission_path);

		for card in &self.cards {
			let mut card_path = self.mission_home.clone();
			card_path.push(self.root_card.id.as_str());
			card_path.push(&card.card_type);
			fs::create_dir_all(&card_path)?;

			card_path.push(format!("{}.{}", card.id, ext));
			card.write(&card_path);
		}
		Ok(())
	}

	pub fn write_markdown(&self, path: &PathBuf) -> Result<(), ModelError> {
		let mut readme_path = path.clone();
		readme_path.push(format!(
			"README-{}-{}.md",
			self.root_card.id, self.root_card.name
		));

		let mission_md_path =
			&path.join(format!("{}-{}.md", self.root_card.id, self.root_card.name));

		let mut markdown = String::from(MODEL_MARKDOWN_TEMPLATE);

		let mission_link = format!(
			"**[Mission Card]({}-{}.md)**",
			self.root_card.id, self.root_card.name
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
				let card_link = format!(
					"- **[{} - {}]({}/{}/{}.md)**: {}\n\n",
					card.id,
					card.name,
					self.root_card.id,
					card.card_type,
					card.id,
					card.description
				);
				index.push_str(card_link.as_str());
			}
		}
		markdown = markdown.replace("{{index}}", &index);
		markdown = markdown.replace("\n\n\n", "\n");

		std::fs::write(&readme_path, markdown)?;

		self.root_card.write_markdown(mission_md_path);

		for card in &self.cards {
			let mut card_path = path.clone();
			card_path.push(self.root_card.id.as_str());
			card_path.push(&card.card_type);
			fs::create_dir_all(&card_path)?;

			card_path.push(format!("{}.md", card.id));
			card.write_markdown(&card_path);
		}
		Ok(())
	}

	pub fn get_compact(&self) -> Value {
		let value: Value = json![
			{
				"$schema": "Aurora.compact.schema.json",
				"cards": [
					self.root_card.get_compact(),
					for card in &self.cards {
						card.get_compact();
					}
				]
			}
		];
		value
	}

	/// Test Invariant 1: Only one mission at the root
	fn validate_single_mission(&self) -> Vec<String> {
		let missions: Vec<&Card> = self
			.cards
			.iter()
			.filter(|c| c.id.starts_with("MIS"))
			.collect();
		let mut results: Vec<String> = Vec::new();
		if missions.len() > 0 {
			for mission in missions {
				results.push(format!(
					"Model {}: extra Mission card at {}",
					self.root_card.id,
					mission.source_path.display()
				));
			}
		}
		results
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
	/// Test Invariant 5: No unlinked cards
	fn find_unlinked_cards(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		let mut left: HashSet<String> = HashSet::new();
		for card_id in self.cards.iter().map(|c| c.id.clone()) {
			left.insert(card_id.clone());
		}

		for link in &self.root_card.links {
			left.remove(&link.target);
		}

		for card in &self.cards {
			for link in &card.links {
				left.remove(&link.target);
			}
		}
		for unlinked in left {
			errors.push(format!("Card {} is unlinked.", unlinked));
		}
		errors
	}

	/// Test Invariant 6: No broken links
	fn validate_broken_links(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		for card in &self.cards {
			for link in &card.links {
				if link.target != self.root_card.id
					&& !self.cards.iter().filter(|c| c.id == link.target).count() == 0
				{
					errors.push(format!(
						"Card {} has a broken link to {}.",
						card.id, link.target
					));
				}
			}
		}
		errors
	}

	/// Test Invariant 7: Special cards: Boundary
	fn validate_bounary(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();

		// At least one outgoing link with 'contains' relationship
		for card in &self
			.cards
			.iter()
			.filter(|c| c.card_type == "Boundary")
			.collect::<Vec<_>>()
		{
			if card.links.is_empty() {
				errors.push(format!(
					"Special Boundary card {} must have at least one outgoing link.",
					card.id
				));
			}
			if card
				.links
				.iter()
				.filter(|l| l.relationship != "contains")
				.count() > 0
			{
				errors.push(format!(
					"Special Boundary card {} can only have 'contains' outgoing relationships.",
					card.id
				));
			}

			// No incoming links except 'includes' relationships
			if self
				.cards
				.iter()
				.flat_map(|c| c.links.iter())
				.filter(|l| l.target == card.id)
				.any(|l| l.relationship != "includes")
			{
				errors.push(format!(
					"Special Boundary card {} can only have 'includes' incoming relationships.",
					card.id
				));
			}
		}

		errors
	}

	/// Test Invariant 7: Special cards: Notes
	fn validate_notes(&self) -> Vec<String> {
		let mut errors: Vec<String> = Vec::new();
		for card in self
			.cards
			.iter()
			.filter(|c| c.card_type == "Note")
			.collect::<Vec<_>>()
		{
			if !card.links.is_empty() {
				errors.push(format!(
					"Special card Note {} should not have outgoing links.",
					card.id
				));
			}
			let incoming: usize = self
				.cards
				.iter()
				.map(|c| c.links.iter().map(|l| l.target.clone()))
				.flatten()
				.filter(|target_id| *target_id == card.id)
				.count();
			if incoming > 1 {
				errors.push(format!(
					"Special card Note {} should not have more than one incoming link.",
					card.id
				));
			}
		}
		errors
	}
}

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("Failed to read model file: {0}")]
	ReadError(#[from] std::io::Error),
	#[error("A Card error has occurred: {0}")]
	CardError(#[from] CardError),
	#[error("Invalid filename")]
	InvalidFilename,
	#[error("Invalid parent path for model root: {0}")]
	InvalidParentPath(String),
}
