use crate::AppState;

/// Compute and retrieve model statistics.
///
/// Generates aggregate statistics about the current architecture model, including
/// total card counts, card type distribution, link counts, and other quantitative metrics.
/// Useful for dashboard displays and model overview.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// JSON object containing computed statistics, or error if state lock fails.
#[tauri::command]
pub fn get_statistics(state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::acquire_lock(&state)?;
    log::debug!("Computing model statistics");
    let stats = model.statistics();
    crate::serialize_json(&stats, "statistics")
}

/// Retrieve model metadata.
///
/// Returns metadata for the model (project name, description, root driver, version, timestamps).
/// This is distinct from card metadata and represents the overall architecture model's identity.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// JSON object containing model metadata fields.
#[tauri::command]
pub fn get_metadata(state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::acquire_lock(&state)?;
    log::debug!("Retrieving model metadata");
    crate::serialize_json(&model.metadata, "metadata")
}

/// Update model metadata fields.
///
/// Modifies the model's metadata. Each field is optional; providing None leaves it unchanged.
/// Typically used to set project name, description, or designate the root driver for
/// traceability visualization.
///
/// # Arguments
/// * `name` - New project name (optional)
/// * `description` - New project description (optional)
/// * `root_driver_id` - Card ID to use as root for traceability (optional)
/// * `state` - Application state
///
/// # Returns
/// Success message if update completed, or error if root_driver_id is invalid.
#[tauri::command(rename_all = "snake_case")]
pub fn update_metadata(
    name: Option<String>,
    description: Option<String>,
    root_driver_id: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Updating model metadata");
    crate::apply_metadata_updates(&mut model, name, description, root_driver_id);
    let success_msg = "Metadata updated successfully".to_string();
    log::info!("{}", success_msg);
    Ok(success_msg)
}
