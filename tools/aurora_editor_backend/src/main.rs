#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod dto;
mod errors;
mod file_ops;
mod session;

use app_state::AppState;
use commands::{
	close_model, current_model, discover_models, open_model, refresh_model, render_all_docs,
	render_markdown_docs, render_views_docs, update_card, validate_active_model,
	write_compact_model_file,
};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
	init_tracing();
	info!("Starting Aurora Editor backend");

	tauri::Builder::default()
		.manage(AppState::default())
		.invoke_handler(tauri::generate_handler![
			discover_models,
			open_model,
			refresh_model,
			current_model,
			validate_active_model,
			update_card,
			render_markdown_docs,
			render_views_docs,
			render_all_docs,
			write_compact_model_file,
			close_model
		])
		.run(tauri::generate_context!())
		.expect("Failed to run Aurora Editor backend");
}

fn init_tracing() {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	let subscriber = fmt().with_env_filter(filter).with_target(false).finish();
	let _ = tracing::subscriber::set_global_default(subscriber);
}
