use crate::models::ArchitectureModel;
use crate::AppState;
use std::path::PathBuf;

/// Load architecture from ZIP file.
///
/// Reads a ZIP file containing an AURORA architecture and loads it into the application state.
/// The ZIP file must contain properly formatted card and link data as organized by the export format.
///
/// # Arguments
/// * `path` - Full file system path to the ZIP file
/// * `state` - Application state containing the architecture model
///
/// # Returns
/// Success message indicating the file was loaded, or error message if loading failed.
///
/// # Errors
/// Returns an error if:
/// - The file path is invalid or the file doesn't exist
/// - The ZIP structure is malformed
/// - Card or link data cannot be deserialized
#[tauri::command]
pub fn load_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::import_model(&path)?;
    let mut app_model = crate::acquire_lock(&state)?;
    *app_model = model;
    let success_msg = format!("Successfully loaded architecture from {}", path);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Save current architecture to ZIP file.
///
/// Exports the entire architecture (all cards and links) to a ZIP file with
/// type-organized folder structure for easy inspection and version control.
///
/// # Arguments
/// * `path` - Full file system path where the ZIP file should be created
/// * `state` - Application state containing the architecture model to save
///
/// # Returns
/// Success message indicating the file was saved, or error message if saving failed.
///
/// # Errors
/// Returns an error if:
/// - The destination path is not writable
/// - Card or link data cannot be serialized
/// - Zip file creation fails
#[tauri::command]
pub fn save_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::acquire_lock(&state)?;
    crate::export_model(&*model, &path)?;
    let success_msg = format!("Successfully saved architecture to {}", path);
    log::info!("{}", success_msg);
    Ok(success_msg)
}
