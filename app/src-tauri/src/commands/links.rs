use crate::models::Link;
use crate::AppState;

/// Create a link between two model elements.
///
/// Establishes a directed link from a source card to either another card (internal link)
/// or an external URL. Links represent relationships such as dependencies, references,
/// or trace-ability connections between architectural elements.
///
/// # Arguments
/// * `source_id` - ID of the source card (must exist)
/// * `target_id` - ID of target card for internal links (None if using target_url)
/// * `target_url` - External URL for external links (None if using target_id)
/// * `state` - Application state
///
/// # Returns
/// Success message if link created, or error if source not found or parameters invalid.
#[tauri::command(rename_all = "snake_case")]
pub fn create_link(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let link = crate::create_link_from_params(source_id.clone(), target_id, target_url)?;
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Creating link from: {}", source_id);
    model.add_link(link);
    let success_msg = format!("Link created successfully from {}", source_id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Delete a link between two model elements.
///
/// Removes a link by source ID and either target card ID or external URL.
/// At least one target must match for the link to be found and removed.
///
/// # Arguments
/// * `source_id` - ID of the source card
/// * `target_id` - ID of target card (used if link is internal)
/// * `target_url` - External URL (used if link is external)
/// * `state` - Application state
///
/// # Returns
/// Success message if link was found and removed, or error if link not found.
#[tauri::command(rename_all = "snake_case")]
pub fn delete_link(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Deleting link from: {}", source_id);
    let removed = model.remove_link(&source_id, target_id.as_deref(), target_url.as_deref());
    if removed {
        let success_msg = format!("Link deleted successfully from {}", source_id);
        log::info!("{}", success_msg);
        Ok(success_msg)
    } else {
        let err = crate::models::AppError::not_found(
            "Link not found",
            format!(
                "Could not find link from {} to {:?} or {:?}",
                source_id, target_id, target_url
            ),
        );
        log::error!("{}", err.technical_message);
        Err(String::from(err))
    }
}

/// Retrieve all links in the model.
///
/// Returns a JSON-serialized list of all links currently stored, both internal
/// (card-to-card) and external (card-to-URL).
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// JSON array of all links, or error if state lock cannot be acquired.
#[tauri::command]
pub fn get_links(state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::acquire_lock(&state)?;
    log::debug!("Retrieving all links");
    crate::serialize_json(&model.links, "links")
}
