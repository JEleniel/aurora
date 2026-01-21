#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use aurora_lib::{AuroraCard, validate_card};
use serde::Serialize;

/// Response returned to the UI after validating an Aurora card payload.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationReport {
	valid: bool,
	messages: Vec<String>,
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

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![validate_card_json])
		.run(tauri::generate_context!())
		.expect("error while running Aurora Editor");
}
