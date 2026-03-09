//! Editor session orchestration for bounded-load model-home startup.

use std::path::Path;

use aurora_shared::{
	Card, CardRef, ModelHomeSession, ModelHomeSessionError, ModelIndex, ModelIndexError,
	ModelRootCard,
};
use thiserror::Error;

/// Bounded-working-set editor session.
pub struct EditorSession {
	model_home_session: ModelHomeSession,
	index: ModelIndex,
}

impl EditorSession {
	/// Open the editor session without fully materializing every card.
	pub fn open(path: &Path) -> Result<Self, EditorSessionError> {
		let model_home_session = ModelHomeSession::try_open_for_update(path)?;
		let index = ModelIndex::open(&model_home_session.model_home)?;
		Ok(Self {
			model_home_session,
			index,
		})
	}

	/// Root mission cards available immediately after startup.
	pub fn roots(&self) -> &[ModelRootCard] {
		self.model_home_session.roots()
	}

	/// Resolved model-home directory for this session.
	pub fn model_home(&self) -> &Path {
		&self.model_home_session.model_home
	}

	/// Search the model home using the live in-memory index.
	pub fn search(&self, query: &str) -> Result<Vec<CardRef>, EditorSessionError> {
		Ok(self.index.search(query)?)
	}

	/// Load a full card document on demand by its card ID.
	pub fn load_card(&self, card_id: &str) -> Result<Option<Card>, EditorSessionError> {
		let Some(relative_path) = self.index.resolve_card_path(card_id)? else {
			return Ok(None);
		};
		Ok(Some(
			self.model_home_session
				.load_card_by_relative_path(&relative_path)?,
		))
	}
}

/// Errors raised while opening or using an editor session.
#[derive(Debug, Error)]
pub enum EditorSessionError {
	#[error("Model-home session failed: {0}")]
	ModelHomeSession(#[from] ModelHomeSessionError),
	#[error("Search index failed: {0}")]
	ModelIndex(#[from] ModelIndexError),
}

#[cfg(test)]
mod tests {
	use super::{EditorSession, EditorSessionError};
	use serde_json::json;
	use std::path::Path;
	use std::time::{Duration, Instant};

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn load_card_fetches_full_details_on_demand() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": [{"target": "ACT-001", "relationship": "contains"}]
			}),
		)?;
		write_card(
			&model_home.join("MIS-001").join("ACT-001-Alpha.json"),
			json!({
				"$schema": "../schemas/Aurora.card.schema.json",
				"id": "ACT-001",
				"card_type": "Activity",
				"name": "Alpha Workflow",
				"description": "loaded lazily",
				"links": []
			}),
		)?;

		let session = EditorSession::open(temp.path())?;
		let results = session.search("alpha")?;
		assert_eq!(
			results.first().map(|card| card.id.as_str()),
			Some("ACT-001")
		);

		let card = session.load_card("ACT-001")?.expect("card should exist");
		assert_eq!(card.description, "loaded lazily");
		Ok(())
	}

	#[test]
	fn open_refuses_second_writer_for_same_model_home() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": []
			}),
		)?;

		let first = EditorSession::open(temp.path())?;
		let second = EditorSession::open(temp.path());
		assert!(matches!(
			second,
			Err(EditorSessionError::ModelHomeSession(_))
		));
		drop(first);
		Ok(())
	}

	#[test]
	fn open_stays_within_two_seconds_for_typical_fixture() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		seed_typical_fixture(&model_home, 2_000, 3_500)?;

		let started = Instant::now();
		let session = EditorSession::open(temp.path())?;
		let elapsed = started.elapsed();

		assert_eq!(session.roots().len(), 1);
		assert!(
			elapsed <= Duration::from_secs(2),
			"expected startup under 2s, got {:?}",
			elapsed
		);
		Ok(())
	}

	fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> {
		let model_home = root.join("aurora");
		std::fs::create_dir_all(model_home.join("schemas"))?;
		std::fs::create_dir_all(model_home.join("MIS-001"))?;
		std::fs::write(model_home.join("MIS-001").join("AuditLog.ndjson"), b"")?;
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

	fn write_card(path: &Path, value: serde_json::Value) -> Result<()> {
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		std::fs::write(path, serde_json::to_string_pretty(&value)?)?;
		Ok(())
	}

	fn seed_typical_fixture(model_home: &Path, card_count: usize, link_count: usize) -> Result<()> {
		let non_root_count = card_count.saturating_sub(1);
		let ids = (1..=non_root_count)
			.map(|index| format!("ACT-{index:04}"))
			.collect::<Vec<_>>();

		let root_links = ids
			.iter()
			.take(non_root_count.min(8))
			.map(|target| json!({ "target": target, "relationship": "contains" }))
			.collect::<Vec<_>>();
		write_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": root_links
			}),
		)?;

		let mut remaining_links = link_count.saturating_sub(non_root_count.min(8));
		for (index, id) in ids.iter().enumerate() {
			let mut links = Vec::new();
			if remaining_links > 0 && index + 1 < ids.len() {
				links.push(json!({ "target": ids[index + 1], "relationship": "relates" }));
				remaining_links -= 1;
			}
			if remaining_links > 0 && index + 5 < ids.len() {
				links.push(json!({ "target": ids[index + 5], "relationship": "supports" }));
				remaining_links -= 1;
			}
			if remaining_links > 0 && index + 25 < ids.len() {
				links.push(json!({ "target": ids[index + 25], "relationship": "traces" }));
				remaining_links -= 1;
			}

			write_card(
				&model_home.join("MIS-001").join(format!("{id}.json")),
				json!({
					"$schema": "../schemas/Aurora.card.schema.json",
					"id": id,
					"card_type": "Activity",
					"name": format!("Activity {index:04}"),
					"description": "synthetic fixture",
					"links": links
				}),
			)?;
		}

		Ok(())
	}
}
