//! In-memory representation of Aurora cards and models.
mod card;
mod compact_card;
mod compact_model;

pub use card::*;
pub use compact_model::{CompactModel, CompactModelError};
use jsonschema::Validator;
use log::debug;
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

use crate::cli::bump_args::BumpArgs;

/// Fully materialized Aurora model state.
#[derive(Debug, Clone)]
pub struct Model {
	pub mission_card: Card,
	pub compact_model: Option<CompactModel>,
	pub cards: HashMap<String, Card>,
	pub adjacency: HashMap<String, Vec<String>>,
	pub card_schema_errors: Vec<String>,
	pub card_invariant_errors: Vec<String>,
}

impl Model {
	pub fn load(
		path: &PathBuf,
		card_validator: &Validator,
		compact_validator: &Validator,
	) -> Result<Vec<Self>, ModelError> {
		// Load all the Mission cards to identify models
		let mut models: Vec<Model> = Self::load_models(path)?;
		if models.is_empty() {
			return Err(ModelError::ModelNotFound);
		}

		// Load all the cards for each model
		for mut model in models.iter_mut() {
			let mission_path = path.join(&model.mission_card.id);
			if !mission_path.exists() {
				return Err(ModelError::ModelPathNotFound(
					mission_path.display().to_string(),
				));
			}
			Self::load_cards(&mut model, &mission_path, card_validator)?;
			// Load the compact model if it exists
			model.compact_model =
				Self::load_compact(&path, &model.mission_card.id, &compact_validator)?;
		}

		Ok(models)
	}

	pub fn compact(&self) -> CompactModel {
		CompactModel::from(self)
	}

	pub fn bump_patch(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_patch(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	pub fn bump_minor(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_minor(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	pub fn bump_major(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_major(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	fn load_models(path: &PathBuf) -> Result<Vec<Self>, ModelError> {
		let regex_mission_card = Regex::new(r"^MIS-\d{3}.json$")?;
		let mut models: Vec<Self> = Vec::new();
		for entry in path.read_dir()? {
			let path = entry?.path();

			if path.is_file() {
				if let Some(file_name) = path.file_name() {
					if regex_mission_card.is_match(
						file_name
							.to_str()
							.ok_or(ModelError::InvalidFileName(file_name.display().to_string()))?,
					) {
						let mission_card = Card::load(&path)?;

						let model = Self {
							mission_card,
							cards: HashMap::new(),
							adjacency: HashMap::new(),
							card_schema_errors: Vec::new(),
							card_invariant_errors: Vec::new(),
							compact_model: None,
						};
						models.push(model);
					}
				}
			}
		}

		Ok(models)
	}

	fn load_cards(
		model: &mut Model,
		path: &PathBuf,
		card_validator: &Validator,
	) -> Result<(), ModelError> {
		let regex_card: Regex = Regex::new(r"^[A-Z]{3}-\d{3}.json$")?;

		let mut pending_folders: Vec<PathBuf> = Vec::new();

		for entry in path.read_dir()? {
			let entry = entry?;
			if entry.path().is_dir() {
				pending_folders.push(entry.path());
			}
		}

		while let Some(folder) = pending_folders.pop() {
			for entry in folder.read_dir()? {
				let entry_path = entry?.path();
				if entry_path.is_dir() {
					pending_folders.push(entry_path);
				} else if entry_path.is_file() {
					if let Some(file_name) = path.file_name() {
						if !regex_card
							.is_match(&file_name.to_str().ok_or(CardError::InvalidCardFileName)?)
						{
							debug!("Skipping non-card file {}", entry_path.display());
							continue;
						}

						let mut results = Card::validate(&entry_path, card_validator)?;
						if !results.is_empty() {
							model.card_schema_errors.append(&mut results);
						}
						let card = Card::load(&entry_path)?;
						model.add_card(card);
					}
				}
			}
		}

		Ok(())
	}

	fn load_compact(
		path: &PathBuf,
		model_id: &str,
		compact_validator: &Validator,
	) -> Result<Option<CompactModel>, ModelError> {
		let path = path.join(format!("AGENT-{}.json", model_id));
		if path.exists() {
			Ok(Some(CompactModel::load(&path, compact_validator)?))
		} else {
			Ok(None)
		}
	}

	fn add_card(&mut self, card: Card) {
		self.cards.insert(card.id.clone(), card.clone());
		if let Some(links) = &card.links {
			for link in links {
				self.adjacency
					.entry(card.id.clone())
					.or_insert_with(Vec::new)
					.push(link.target.clone());
			}
		}
	}
}

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("An error occurred while loading a compact model: {0}")]
	CompactModelError(#[from] CompactModelError),
	#[error("Invalid file name: {0}")]
	InvalidFileName(String),
	#[error("No model found at specified path {0}")]
	PathNotFound(String),
	#[error("Schema file not found")]
	SchemaNotFound,
	#[error("Compact schema file not found")]
	CompactSchemaNotFound,
	#[error("Invalid schema: {0}")]
	InvalidSchema(String),
	#[error("Model root file not found")]
	ModelNotFound,
	#[error("Schema file too large, max {0} GB ({1} GB): {2}")]
	SchemaTooLarge(usize, usize, String),
	#[error("Io error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
	#[error("JSON error: {0}")]
	JsonError(#[from] serde_json::Error),
	#[error("Card error: {0}")]
	CardError(#[from] CardError),
	#[error("Card not found: {0}")]
	CardNotFound(String),
	#[error("Cards not found")]
	CardsNotFound,
	#[error("model path not found: {0}")]
	ModelPathNotFound(String),
}
