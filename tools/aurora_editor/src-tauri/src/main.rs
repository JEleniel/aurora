#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

mod model_home;

use aurora_lib::{AuroraCard, AuroraLibError, validate_card};
use serde::Serialize;
use std::path::Path;

/// Response returned to the UI after validating an Aurora card payload.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationReport {
	valid: bool,
	messages: Vec<String>,
}

/// JSON payload returned when the UI loads card content from disk.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadedCardPayload {
	path: String,
	card: AuroraCard,
	raw: String,
}

/// Validate raw JSON input from the UI and report user-friendly errors.
#[tauri::command]
fn validate_card_json(card_json: &str) -> Result<ValidationReport, String> {
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

/// Return a structured overview of the selected model home directory.
#[tauri::command]
fn load_model_home(path: &str) -> Result<model_home::ModelHomeSummary, String> {
	model_home::load_model_home(path).map_err(|err: AuroraLibError| err.to_string())
}

/// Load a specific card JSON file inside the selected model home.
#[tauri::command]
fn load_card_file(root_path: &str, relative_path: &str) -> Result<LoadedCardPayload, String> {
	let opened = model_home::open_card(root_path, relative_path)
		.map_err(|err: AuroraLibError| err.to_string())?;
	let raw = serde_json::to_string_pretty(&opened.card).map_err(|err| err.to_string())?;
	Ok(LoadedCardPayload {
		path: normalize_path(&opened.absolute_path),
		card: opened.card,
		raw,
	})
}

fn normalize_path(path: &Path) -> String {
	path.to_string_lossy().replace('\\', "/")
}

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			validate_card_json,
			load_model_home,
			load_card_file
		])
		.run(tauri::generate_context!())
		.expect("error while running Aurora Editor");
}
