//! Application state for the Aurora editor UI.

use std::path::{Path, PathBuf};

use aurora_shared::{ValidationReport, discover_model_homes, load_model, validate_model};

use crate::constants::DEFAULT_MODEL_HOME;
use crate::error::EditorError;
use crate::model::EditorModel;

/// Summary of validation counts for the selected mission.
#[derive(Debug, Clone, Copy)]
pub struct ValidationSummary {
	/// Total error count.
	pub errors: usize,
	/// Total warning count.
	pub warnings: usize,
	/// Total info count.
	pub infos: usize,
}

/// Mission-scoped snapshot used by the editor UI.
#[derive(Debug, Clone)]
pub struct MissionSnapshot {
	pub mission_id: String,
	pub mission_name: String,
	pub model: EditorModel,
	pub validation: ValidationReport,
}

/// State container for the editor UI.
#[derive(Debug, Clone)]
pub struct AppState {
	pub load_path: String,
	pub model_home: Option<PathBuf>,
	pub missions: Vec<MissionSnapshot>,
	pub selected_mission_index: Option<usize>,
	pub selected_card_id: Option<String>,
	pub search_query: String,
	pub last_error: Option<String>,
}

impl AppState {
	/// Create an empty editor state with the default model path.
	pub fn new() -> Self {
		Self {
			load_path: DEFAULT_MODEL_HOME.to_string(),
			model_home: None,
			missions: Vec::new(),
			selected_mission_index: None,
			selected_card_id: None,
			search_query: String::new(),
			last_error: None,
		}
	}

	/// Return the currently selected mission snapshot.
	pub fn current_mission(&self) -> Option<&MissionSnapshot> {
		self.selected_mission_index
			.and_then(|index| self.missions.get(index))
	}

	/// Compute aggregated validation counts for the selected mission.
	pub fn validation_summary(&self) -> Option<ValidationSummary> {
		let mission = self.current_mission()?;
		let mut summary = ValidationSummary {
			errors: 0,
			warnings: 0,
			infos: 0,
		};
		for diagnostic in &mission.validation.diagnostics {
			match diagnostic.severity {
				aurora_shared::DiagnosticSeverity::Error => summary.errors += 1,
				aurora_shared::DiagnosticSeverity::Warning => summary.warnings += 1,
				aurora_shared::DiagnosticSeverity::Info => summary.infos += 1,
			}
		}
		Some(summary)
	}

	/// Select a mission snapshot by index.
	pub fn select_mission(&mut self, index: usize) {
		if index >= self.missions.len() {
			return;
		}
		self.selected_mission_index = Some(index);
		let mission_id = self.missions[index].mission_id.clone();
		self.selected_card_id = Some(mission_id);
	}

	/// Select a card by id.
	pub fn select_card(&mut self, card_id: String) {
		self.selected_card_id = Some(card_id);
	}

	/// Reset the load path to the default model home.
	pub fn load_default(&mut self) {
		self.load_path = DEFAULT_MODEL_HOME.to_string();
	}

	/// Load a model from the provided path.
	pub fn load_from_path(&mut self, path: &Path) -> Result<(), EditorError> {
		let mut homes = discover_model_homes(path)?;
		homes.sort_by(|left, right| {
			left.root()
				.to_string_lossy()
				.cmp(&right.root().to_string_lossy())
		});
		let home = homes
			.first()
			.cloned()
			.ok_or_else(|| EditorError::ModelHomeNotFound {
				path: path.to_path_buf(),
			})?;
		let model = load_model(&home)?;
		let mut missions = model.split_by_mission();
		missions.sort_by(|left, right| mission_id(left).cmp(&mission_id(right)));

		let mut snapshots = Vec::new();
		for mission_model in missions {
			let validation = validate_model(&mission_model);
			let editor_model = EditorModel::new(mission_model)?;
			let mission_id = editor_model.mission_id().to_string();
			let mission_name = editor_model
				.mission_name()
				.unwrap_or("Unnamed Mission")
				.to_string();
			snapshots.push(MissionSnapshot {
				mission_id,
				mission_name,
				model: editor_model,
				validation,
			});
		}

		self.model_home = Some(home.root().to_path_buf());
		self.missions = snapshots;
		self.selected_mission_index = if self.missions.is_empty() {
			None
		} else {
			Some(0)
		};
		self.selected_card_id = self
			.selected_mission_index
			.and_then(|index| self.missions.get(index))
			.map(|mission| mission.mission_id.clone());
		self.last_error = None;
		Ok(())
	}

	/// Record an error message for display in the UI.
	pub fn set_error(&mut self, error: EditorError) {
		self.last_error = Some(error.to_string());
	}
}

fn mission_id(model: &aurora_shared::AuroraModel) -> String {
	model
		.iter_cards()
		.find(|card| card.card_type == "Mission")
		.map(|card| card.id.clone())
		.unwrap_or_else(|| "MIS-UNKNOWN".to_string())
}
