//! Tauri backend entrypoint for the Aurora Editor.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audit;
mod commands;
mod state;
mod types;
mod workspace;

use anyhow::{Result, anyhow};
use state::EditorState;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
	init_tracing()?;
	tauri::Builder::default()
		.manage(EditorState::new())
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			commands::health_check,
			commands::set_workspace,
			commands::workspace_status,
			commands::discover_models,
			commands::load_model_snapshot,
			commands::validate_model_snapshot,
			commands::render_card_markdown,
			commands::render_views_bundle,
			commands::render_all_assets,
			commands::write_compact_export,
			commands::create_card,
			commands::update_card,
			commands::delete_card,
		])
		.run(tauri::generate_context!())
		.map_err(|err| anyhow!(err))
}

fn init_tracing() -> Result<()> {
	let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
	fmt()
		.with_env_filter(filter)
		.with_target(false)
		.try_init()
		.map_err(|err| anyhow!(err))?;
	Ok(())
}
