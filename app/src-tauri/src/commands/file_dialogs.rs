/// Open a file picker dialog for selecting an existing file.
///
/// Displays native OS file selection dialog filtered to ZIP files only.
/// Used for importing architecture models from disk. Returns the absolute
/// path to the selected file, or None if dialog was cancelled.
///
/// # Arguments
/// * `window` - Tauri window for dialog display
///
/// # Returns
/// Some(path) with absolute file path, or None if user cancelled.
#[tauri::command]
pub fn select_file(window: tauri::Window) -> Result<Option<String>, String> {
	use tauri_plugin_dialog::DialogExt;

	let file_path = window
		.dialog()
		.file()
		.add_filter("ZIP Files", &["zip"])
		.blocking_pick_file();

	Ok(file_path.map(|p| p.to_string()))
}

/// Open a file picker dialog for selecting a save location.
///
/// Displays native OS file save dialog filtered to ZIP files. Used for exporting
/// the current architecture model. Returns the absolute path selected by the user,
/// or None if the dialog was cancelled. May include suggested filename.
///
/// # Arguments
/// * `window` - Tauri window for dialog display
///
/// # Returns
/// Some(path) with absolute file path for save location, or None if user cancelled.
#[tauri::command]
pub fn select_save_file(window: tauri::Window) -> Result<Option<String>, String> {
	use tauri_plugin_dialog::DialogExt;

	let file_path = window
		.dialog()
		.file()
		.add_filter("ZIP Files", &["zip"])
		.blocking_save_file();

	Ok(file_path.map(|p| p.to_string()))
}
