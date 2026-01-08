use crate::models::{Card, CardStatus, CardType};
use crate::AppState;

/// Create a new card in the architecture model.
///
/// Instantiates a new card with the specified type, name, and optional description,
/// then adds it to the current architecture. Card IDs should be unique.
///
/// # Arguments
/// * `id` - Unique identifier for the card (must not already exist)
/// * `card_type` - Type string (e.g., "driver", "requirement", "behavior")
/// * `name` - Display name for the card
/// * `description` - Optional detailed description
/// * `state` - Application state
///
/// # Returns
/// Success message with card ID, or error if type is invalid or ID exists.
#[tauri::command(rename_all = "snake_case")]
pub fn create_card(
    id: String,
    card_type: String,
    name: String,
    description: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let type_enum = crate::parse_card_type(&card_type)?;
    let card = Card::new(id.clone(), type_enum, name, description);
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Creating card: id={}, type={}", id, card_type);
    model.add_card(card);
    let success_msg = format!("Card '{}' created successfully", id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Delete a card and all its associated links from the model.
///
/// Removes a card by ID. Cascades to remove any links that reference this card
/// as source or target.
///
/// # Arguments
/// * `id` - Card ID to delete
/// * `state` - Application state
///
/// # Returns
/// Success message if card existed and was deleted, or error if not found.
#[tauri::command]
pub fn delete_card(id: String, state: tauri::State<AppState>) -> Result<String, String> {
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Deleting card: {}", id);
    model.remove_card(&id).ok_or_else(|| {
        let err = crate::models::AppError::not_found(
            "Card not found",
            format!("Card with ID '{}' does not exist", id),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })?;
    let success_msg = format!("Card '{}' deleted successfully", id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Retrieve all cards in the model.
///
/// Returns a JSON-serialized vector of all cards currently stored in the model.
/// Useful for initializing the UI or displaying complete inventory.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// JSON array of all cards, or error if state lock cannot be acquired.
#[tauri::command]
pub fn get_cards(state: tauri::State<AppState>) -> Result<String, String> {
    let model = crate::acquire_lock(&state)?;
    log::debug!("Retrieving all cards");
    let cards: Vec<_> = model.cards.values().collect();
    crate::serialize_json(&cards, "cards")
}

/// Retrieve cards filtered by type.
///
/// Returns only cards matching the specified type. Card types include driver, requirement,
/// behavior, interface, constraint, actor, test, and others as defined in the schema.
///
/// # Arguments
/// * `card_type` - Type to filter by (must be valid card type)
/// * `state` - Application state
///
/// # Returns
/// JSON array of matching cards, or error if type is invalid or lock fails.
#[tauri::command(rename_all = "snake_case")]
pub fn get_cards_by_type(
    card_type: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let type_enum = crate::parse_card_type(&card_type)?;
    let model = crate::acquire_lock(&state)?;
    log::debug!("Retrieving cards of type: {}", card_type);
    let cards = model.get_cards_by_type(&type_enum);
    crate::serialize_json(&cards, "filtered cards")
}

/// Update card properties (name, description, and/or status).
///
/// Modifies a card's basic properties. At least one field must be provided to perform an update.
/// Empty/None fields are left unchanged. Status can be set to values like "draft", "approved", "active", etc.
///
/// # Arguments
/// * `id` - Card ID to update
/// * `name` - New display name (optional, leave unchanged if None)
/// * `description` - New description (optional, leave unchanged if None)
/// * `status` - New status (optional, leave unchanged if None)
/// * `state` - Application state
///
/// # Returns
/// Success message if update succeeded, or error if card not found or status invalid.
#[tauri::command]
pub fn update_card(
    id: String,
    name: Option<String>,
    description: Option<String>,
    status: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = crate::acquire_lock(&state)?;
    log::info!("Updating card: {}", id);
    let card = model.get_card_mut(&id).ok_or_else(|| {
        let err = crate::models::AppError::not_found(
            "Card not found",
            format!("Card with ID '{}' does not exist", id),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })?;
    if let Some(new_name) = name {
        card.name = new_name;
    }
    if let Some(new_description) = description {
        card.description = Some(new_description);
    }
    if let Some(status_str) = status {
        card.status = CardStatus::from_str(&status_str);
    }
    card.modified_at = chrono::Utc::now();
    let success_msg = format!("Card '{}' updated successfully", id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}
