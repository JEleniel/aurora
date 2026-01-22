mod commands;
mod model_home;
mod state;

pub fn run() -> Result<(), tauri::Error> {
	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.manage(state::AppState::default())
		.invoke_handler(tauri::generate_handler![
			commands::validate_card_json,
			commands::load_model_home,
			commands::load_card_file,
			commands::filter_cards,
			commands::graph_neighborhood
		])
		.run(tauri::generate_context!())
}
