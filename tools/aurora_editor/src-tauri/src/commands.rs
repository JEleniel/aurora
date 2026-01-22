use aurora_lib::{AuroraCard, AuroraLibError, validate_card};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, State};

use crate::{model_home, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValidationReport {
	valid: bool,
	messages: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadedCardPayload {
	path: String,
	card: AuroraCard,
	raw: String,
}

#[tauri::command]
pub(crate) fn validate_card_json(card_json: &str) -> Result<ValidationReport, String> {
	let card: AuroraCard = serde_json::from_str(card_json)
		.map_err(|err| format!("failed to parse card JSON: {err}"))?;
	match validate_card(&card) {
		Ok(()) => Ok(ValidationReport {
			valid: true,
			messages: Vec::new(),
		}),
		Err(err) => Ok(ValidationReport {
			valid: false,
			messages: vec![err.to_string()],
		}),
	}
}

#[tauri::command]
pub(crate) fn load_model_home(
	app_handle: AppHandle,
	state: State<AppState>,
	path: &str,
) -> Result<model_home::ModelHomeSummary, String> {
	let model = model_home::load_model_home(path).map_err(|err: AuroraLibError| err.to_string())?;
	let root_path = model.root_path().to_path_buf();
	let summary = model.summary().clone();
	state.store_model(model)?;
	state.watch_model_home(&app_handle, root_path)?;
	Ok(summary)
}

#[tauri::command]
pub(crate) fn load_card_file(
	root_path: &str,
	relative_path: &str,
) -> Result<LoadedCardPayload, String> {
	let opened = model_home::open_card(root_path, relative_path)
		.map_err(|err: AuroraLibError| err.to_string())?;
	let raw = serde_json::to_string_pretty(&opened.card).map_err(|err| err.to_string())?;
	Ok(LoadedCardPayload {
		path: normalize_path(&opened.absolute_path),
		card: opened.card,
		raw,
	})
}

#[tauri::command]
pub(crate) fn filter_cards(
	state: State<AppState>,
	filter_id: Option<String>,
) -> Result<model_home::FilteredCards, String> {
	let active_filter = filter_id.as_deref();
	state.with_model_view(|model| model.filter_cards(active_filter))
}

#[tauri::command]
pub(crate) fn graph_neighborhood(
	state: State<AppState>,
	card_id: &str,
	filter_id: Option<String>,
	generations: Option<usize>,
) -> Result<model_home::GraphNeighborhood, String> {
	let active_filter = filter_id.as_deref();
	let depth = generations.unwrap_or(2).max(1);
	state.with_model(|model| model.graph_neighborhood(card_id, active_filter, depth))
}

fn normalize_path(path: &Path) -> String {
	path.to_string_lossy().replace('\\', "/")
}
