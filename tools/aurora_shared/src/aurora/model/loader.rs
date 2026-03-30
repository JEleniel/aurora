use serde_json::Value;
use std::path::{Path, PathBuf};

use super::{Card, Model, ModelError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadMode {
	#[default]
	ReadOnly,
	ReadWrite,
}

#[derive(Debug)]
pub struct LoadedModel {
	pub(crate) model: Model,
	pub(crate) audit_log_lock: Option<super::super::AuditLogFileLock>,
}

impl Model {
	pub fn try_load(
		path: &Path,
		card_schema: &Value,
		audit_schema: &Value,
		load_mode: LoadMode,
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
			tracing::debug!("Scanning {}", current_folder.display());
			for entry in std::fs::read_dir(&current_folder)? {
				let entry = entry?;
				let entry_path = entry.path();
				if entry.file_type()?.is_dir() {
					folders_to_visit.push(entry_path);
					tracing::trace!(
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

				tracing::debug!("Loading card from {}", entry_path.display());
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
}

fn load_audit_log(
	path: &Path,
	audit_schema: &Value,
	load_mode: LoadMode,
) -> Result<
	(
		super::super::AuditLog,
		Option<super::super::AuditLogFileLock>,
	),
	ModelError,
> {
	match load_mode {
		LoadMode::ReadOnly => Ok((super::super::AuditLog::try_load(path, audit_schema)?, None)),
		LoadMode::ReadWrite => {
			let loaded = super::super::AuditLog::try_load_for_update(path, audit_schema)?;
			Ok((loaded.audit_log, Some(loaded.lock)))
		}
	}
}
