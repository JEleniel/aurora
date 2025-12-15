pub mod models;
pub mod zip_handler;

use models::{ArchitectureModel, Card, CardStatus, CardType, Link};
use std::path::PathBuf;
use std::sync::Mutex;

// Global architecture model state
pub struct AppState {
    pub model: Mutex<ArchitectureModel>,
}

/// Load architecture from ZIP file
#[tauri::command]
fn load_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let file_path = PathBuf::from(&path);
    let model = zip_handler::import_from_zip(&file_path)
        .map_err(|e| format!("Failed to load architecture: {}", e))?;

    let mut app_model = state.model.lock().unwrap();
    *app_model = model;

    Ok(format!("Loaded architecture from {}", path))
}

/// Save current architecture to ZIP file
#[tauri::command]
fn save_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let file_path = PathBuf::from(&path);
    let model = state.model.lock().unwrap();

    zip_handler::export_to_zip(&*model, &file_path)
        .map_err(|e| format!("Failed to save architecture: {}", e))?;

    Ok(format!("Saved architecture to {}", path))
}

/// Create a new card
#[tauri::command]
fn create_card(
    id: String,
    card_type: String,
    name: String,
    description: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let card_type =
        CardType::from_str(&card_type).ok_or_else(|| "Invalid card type".to_string())?;

    let card = Card::new(id, card_type, name, description);
    let mut model = state.model.lock().unwrap();
    model.add_card(card);

    Ok("Card created successfully".to_string())
}

/// Delete a card
#[tauri::command]
fn delete_card(id: String, state: tauri::State<AppState>) -> Result<String, String> {
    let mut model = state.model.lock().unwrap();
    model
        .remove_card(&id)
        .ok_or_else(|| "Card not found".to_string())?;

    Ok("Card deleted successfully".to_string())
}

/// Get all cards
#[tauri::command]
fn get_cards(state: tauri::State<AppState>) -> Result<String, String> {
    let model = state.model.lock().unwrap();
    let cards: Vec<_> = model.cards.values().collect();
    serde_json::to_string(&cards).map_err(|e| e.to_string())
}

/// Get cards by type
#[tauri::command]
fn get_cards_by_type(card_type: String, state: tauri::State<AppState>) -> Result<String, String> {
    let card_type =
        CardType::from_str(&card_type).ok_or_else(|| "Invalid card type".to_string())?;

    let model = state.model.lock().unwrap();
    let cards = model.get_cards_by_type(&card_type);
    serde_json::to_string(&cards).map_err(|e| e.to_string())
}

/// Update card
#[tauri::command]
fn update_card(
    id: String,
    name: Option<String>,
    description: Option<String>,
    status: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = state.model.lock().unwrap();
    let card = model
        .get_card_mut(&id)
        .ok_or_else(|| "Card not found".to_string())?;

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

    Ok("Card updated successfully".to_string())
}

/// Create a link
#[tauri::command]
fn create_link(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let link = if let Some(target_id) = target_id {
        Link::new_internal(source_id, target_id)
    } else if let Some(target_url) = target_url {
        Link::new_external(source_id, target_url)
    } else {
        return Err("Either target_id or target_url must be provided".to_string());
    };

    let mut model = state.model.lock().unwrap();
    model.add_link(link);

    Ok("Link created successfully".to_string())
}

/// Get all links
#[tauri::command]
fn get_links(state: tauri::State<AppState>) -> Result<String, String> {
    let model = state.model.lock().unwrap();
    serde_json::to_string(&model.links).map_err(|e| e.to_string())
}

/// Get model statistics
#[tauri::command]
fn get_statistics(state: tauri::State<AppState>) -> Result<String, String> {
    let model = state.model.lock().unwrap();
    let stats = model.statistics();
    serde_json::to_string(&stats).map_err(|e| e.to_string())
}

/// Get model metadata
#[tauri::command]
fn get_metadata(state: tauri::State<AppState>) -> Result<String, String> {
    let model = state.model.lock().unwrap();
    serde_json::to_string(&model.metadata).map_err(|e| e.to_string())
}

/// Update model metadata
#[tauri::command]
fn update_metadata(
    name: Option<String>,
    description: Option<String>,
    root_driver_id: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = state.model.lock().unwrap();

    if let Some(n) = name {
        model.metadata.name = Some(n);
    }
    if let Some(d) = description {
        model.metadata.description = Some(d);
    }
    if let Some(id) = root_driver_id {
        model.metadata.root_driver_id = Some(id);
    }

    Ok("Metadata updated successfully".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        model: Mutex::new(ArchitectureModel::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            load_architecture,
            save_architecture,
            create_card,
            delete_card,
            get_cards,
            get_cards_by_type,
            update_card,
            create_link,
            get_links,
            get_statistics,
            get_metadata,
            update_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
