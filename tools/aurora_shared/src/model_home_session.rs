//! Lightweight write-session support for editor-style model-home access.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::{AuditLogError, AuditLogFileLock, Card, CardError};

/// Minimal metadata for a mission root card available at session startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRootCard {
	pub id: String,
	pub name: String,
	pub source_path: PathBuf,
}

/// Exclusive editor-oriented model-home session with lazy card loading.
#[derive(Debug)]
pub struct ModelHomeSession {
	pub model_home: PathBuf,
	card_schema: Value,
	roots: Vec<ModelRootCard>,
	_audit_log_locks: Vec<AuditLogFileLock>,
}

impl ModelHomeSession {
	/// Open a model home for editor use without fully materializing all cards.
	pub fn try_open_for_update(path: &Path) -> Result<Self, ModelHomeSessionError> {
		let model_home = resolve_model_home(path)?;
		let card_schema_path = model_home.join("schemas").join("Aurora.card.schema.json");
		if !card_schema_path.is_file() {
			return Err(ModelHomeSessionError::RequiredFileMissing(
				card_schema_path.display().to_string(),
			));
		}

		let card_schema_data = std::fs::read_to_string(&card_schema_path)?;
		let card_schema: Value = serde_json::from_str(&card_schema_data)?;

		let mut roots: Vec<ModelRootCard> = Vec::new();
		let mut audit_log_locks: Vec<AuditLogFileLock> = Vec::new();
		for entry in std::fs::read_dir(&model_home)? {
			let entry = entry?;
			if entry.file_type()?.is_dir() {
				continue;
			}
			if let Some(file_name) = entry.file_name().to_str()
				&& file_name.starts_with("MIS-")
				&& file_name.ends_with(".json")
			{
				let root_card = Card::try_load(&entry.path(), &card_schema)?;
				let audit_log_path = model_home
					.join(root_card.id.as_str())
					.join("AuditLog.ndjson");
				if !audit_log_path.is_file() {
					return Err(ModelHomeSessionError::RequiredFileMissing(
						audit_log_path.display().to_string(),
					));
				}
				audit_log_locks.push(AuditLogFileLock::try_acquire(&audit_log_path).map_err(
					|error| match error {
						AuditLogError::ModelLocked(path) => {
							ModelHomeSessionError::ModelLocked(path)
						}
						AuditLogError::IoError(error) => ModelHomeSessionError::IoError(error),
						other => ModelHomeSessionError::AuditLog(other),
					},
				)?);
				roots.push(ModelRootCard {
					id: root_card.id,
					name: root_card.name,
					source_path: entry.path(),
				});
			}
		}

		if roots.is_empty() {
			return Err(ModelHomeSessionError::InvalidAuroraHome(
				model_home.display().to_string(),
			));
		}
		roots.sort_by(|left, right| left.id.cmp(&right.id));

		Ok(Self {
			model_home,
			card_schema,
			roots,
			_audit_log_locks: audit_log_locks,
		})
	}

	/// Root missions discovered during startup.
	pub fn roots(&self) -> &[ModelRootCard] {
		self.roots.as_slice()
	}

	/// Load a single card on demand using a model-home-relative path.
	pub fn load_card_by_relative_path(
		&self,
		relative_path: &Path,
	) -> Result<Card, ModelHomeSessionError> {
		let normalized = normalize_relative_card_path(relative_path)?;
		let full_path = self.model_home.join(&normalized);
		Ok(Card::try_load(&full_path, &self.card_schema)?)
	}
}

fn resolve_model_home(path: &Path) -> Result<PathBuf, ModelHomeSessionError> {
	if path
		.file_name()
		.and_then(|name| name.to_str())
		.is_some_and(|name| name == "aurora")
		&& path.is_dir()
	{
		return Ok(path.to_path_buf());
	}

	let nested = path.join("aurora");
	if nested.is_dir() {
		return Ok(nested);
	}

	Err(ModelHomeSessionError::InvalidAuroraHome(
		path.display().to_string(),
	))
}

fn normalize_relative_card_path(path: &Path) -> Result<PathBuf, ModelHomeSessionError> {
	let mut normalized = PathBuf::new();
	for component in path.components() {
		match component {
			Component::Normal(segment) => normalized.push(segment),
			Component::CurDir => {}
			Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
				return Err(ModelHomeSessionError::PathOutsideModelHome(
					path.display().to_string(),
				));
			}
		}
	}

	if normalized.as_os_str().is_empty() {
		return Err(ModelHomeSessionError::PathOutsideModelHome(
			path.display().to_string(),
		));
	}

	Ok(normalized)
}

/// Errors raised while establishing or using a lazy editor model-home session.
#[derive(Debug, Error)]
pub enum ModelHomeSessionError {
	#[error("The specified path is not a valid Aurora model home: {0}")]
	InvalidAuroraHome(String),
	#[error("Model is locked by another write session: {0}")]
	ModelLocked(String),
	#[error("Missing required file in model home: {0}")]
	RequiredFileMissing(String),
	#[error("The requested card path escapes the model home: {0}")]
	PathOutsideModelHome(String),
	#[error("Card loading failed: {0}")]
	Card(#[from] CardError),
	#[error("Audit log setup failed: {0}")]
	AuditLog(#[from] AuditLogError),
	#[error("An IO error occurred: {0}")]
	IoError(#[from] std::io::Error),
	#[error("A JSON parse error occurred: {0}")]
	JsonParseError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
	use super::{ModelHomeSession, ModelHomeSessionError};
	use serde_json::json;
	use std::path::Path;

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn try_open_for_update_loads_root_cards_without_loading_children() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_root_card(&model_home)?;
		std::fs::write(model_home.join("MIS-001").join("AuditLog.ndjson"), b"")?;
		write_child_card(
			&model_home.join("MIS-001").join("ACT-001-Alpha.json"),
			json!({
				"$schema": "../schemas/Aurora.card.schema.json",
				"id": "ACT-001",
				"card_type": "Activity",
				"name": "Alpha",
				"description": "lazy detail"
			}),
		)?;

		let session = ModelHomeSession::try_open_for_update(temp.path())?;
		assert_eq!(session.roots().len(), 1);
		assert_eq!(session.roots()[0].id, "MIS-001");

		let detail = session.load_card_by_relative_path(Path::new("MIS-001/ACT-001-Alpha.json"));
		assert!(matches!(detail, Err(ModelHomeSessionError::Card(_))));
		Ok(())
	}

	#[test]
	fn try_open_for_update_rejects_second_writer() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_root_card(&model_home)?;
		std::fs::write(model_home.join("MIS-001").join("AuditLog.ndjson"), b"")?;

		let first = ModelHomeSession::try_open_for_update(temp.path())?;
		let second = ModelHomeSession::try_open_for_update(temp.path());
		assert!(matches!(second, Err(ModelHomeSessionError::ModelLocked(_))));
		drop(first);

		let reopened = ModelHomeSession::try_open_for_update(temp.path())?;
		assert_eq!(reopened.roots().len(), 1);
		Ok(())
	}

	fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> {
		let model_home = root.join("aurora");
		std::fs::create_dir_all(model_home.join("schemas"))?;
		std::fs::create_dir_all(model_home.join("MIS-001"))?;
		std::fs::write(
			model_home.join("schemas").join("Aurora.card.schema.json"),
			serde_json::to_string_pretty(&json!({
				"$schema": "http://json-schema.org/draft-07/schema#",
				"type": "object",
				"required": ["$schema", "id", "card_type", "name", "description", "links"],
				"properties": {
					"$schema": { "type": "string" },
					"id": { "type": "string" },
					"card_type": { "type": "string" },
					"card_subtype": { "type": "string" },
					"name": { "type": "string" },
					"description": { "type": "string" },
					"version": { "type": "string" },
					"status": { "type": "string" },
					"boundary": { "type": "string" },
					"notes": { "type": "string" },
					"icon": { "type": "string" },
					"attributes": { "type": "object" },
					"external_references": { "type": "array", "items": { "type": "string" } },
					"links": {
						"type": "array",
						"items": {
							"type": "object",
							"required": ["target", "relationship"],
							"properties": {
								"target": { "type": "string" },
								"relationship": { "type": "string" }
							}
						}
					}
				}
			}))?,
		)?;
		Ok(model_home)
	}

	fn write_root_card(model_home: &Path) -> Result<()> {
		write_child_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": []
			}),
		)
	}

	fn write_child_card(path: &Path, card: serde_json::Value) -> Result<()> {
		std::fs::write(path, serde_json::to_string_pretty(&card)?)?;
		Ok(())
	}
}
