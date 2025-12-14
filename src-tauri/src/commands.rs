use crate::model::{Card, CardType, Link, ArchitectureModel};
use crate::persistence::{AutoSaveConfig, PersistenceManager, AutoSaveManager};
use crate::query::QueryEngine;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state shared between Tauri commands
pub struct AppState {
    pub model: Arc<RwLock<ArchitectureModel>>,
    pub persistence: Arc<PersistenceManager>,
    pub autosave: Arc<AutoSaveManager>,
}

/// Response wrapper for all API returns
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

// ============================================================================
// CARD COMMANDS
// ============================================================================

#[tauri::command]
pub async fn create_card(
    state: State<'_, AppState>,
    id: String,
    card_type: String,
    name: String,
) -> Result<ApiResponse<Card>, String> {
    let card_type = match card_type.as_str() {
        "driver" => CardType::Driver,
        "requirement" => CardType::Requirement,
        "behavior" => CardType::Behavior,
        "interface" => CardType::Interface,
        "constraint" => CardType::Constraint,
        "logical-component" => CardType::LogicalComponent,
        "deployable-node" => CardType::DeployableNode,
        "actor" => CardType::Actor,
        "test" => CardType::Test,
        "artifact" => CardType::Artifact,
        "view" => CardType::View,
        "note" => CardType::Note,
        _ => return Err("Invalid card type".to_string()),
    };

    let card = Card::new(id, card_type, name);
    let card_copy = card.clone();

    // Add to model
    let mut model = state.model.write().await;
    if let Err(e) = model.add_card(card) {
        return Ok(ApiResponse::error(e));
    }

    // Queue for auto-save
    state.autosave.queue_save(card_copy.clone()).await;

    Ok(ApiResponse::ok(card_copy))
}

#[tauri::command]
pub async fn get_card(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<Option<Card>>, String> {
    let model = state.model.read().await;
    let card = model.get_card(&id).cloned();
    Ok(ApiResponse::ok(card))
}

#[tauri::command]
pub async fn update_card(
    state: State<'_, AppState>,
    id: String,
    updates: serde_json::Value,
) -> Result<ApiResponse<Card>, String> {
    let mut model = state.model.write().await;

    if let Some(card) = model.get_card_mut(&id) {
        // Update fields from JSON
        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            card.name = name.to_string();
        }
        if let Some(description) = updates.get("description").and_then(|v| v.as_str()) {
            card.description = Some(description.to_string());
        }
        if let Some(status) = updates.get("status").and_then(|v| v.as_str()) {
            card.status = match status {
                "proposed" => Some(crate::model::CardStatus::Proposed),
                "approved" => Some(crate::model::CardStatus::Approved),
                "implemented" => Some(crate::model::CardStatus::Implemented),
                "verified" => Some(crate::model::CardStatus::Verified),
                "deprecated" => Some(crate::model::CardStatus::Deprecated),
                "retired" => Some(crate::model::CardStatus::Retired),
                _ => None,
            };
        }

        // Record change
        let prev_values = std::collections::HashMap::new();
        let changed_fields = if let Some(obj) = updates.as_object() {
            obj.keys().cloned().collect()
        } else {
            // If updates is not an object, return error instead of panicking
            return Ok(ApiResponse::error("updates must be a JSON object".to_string()));
        };

        card.record_change(
            "modified",
            changed_fields,
            prev_values,
            "user:web-ui".to_string(),
            Some("Card updated via UI".to_string()),
        );

        let card_copy = card.clone();
        
        // Queue for auto-save
        state.autosave.queue_save(card_copy.clone()).await;

        Ok(ApiResponse::ok(card_copy))
    } else {
        Ok(ApiResponse::error(format!("Card '{}' not found", id)))
    }
}

#[tauri::command]
pub async fn delete_card(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    let mut model = state.model.write().await;
    let existed = model.cards.remove(&id).is_some();
    Ok(ApiResponse::ok(existed))
}

#[tauri::command]
pub async fn list_cards(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Card>>, String> {
    let model = state.model.read().await;
    let cards: Vec<Card> = model.cards.values().cloned().collect();
    Ok(ApiResponse::ok(cards))
}

// ============================================================================
// LINK COMMANDS
// ============================================================================

#[tauri::command]
pub async fn create_link(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<ApiResponse<Link>, String> {
    let link = Link::new(source_id, target_id);
    let link_copy = link.clone();

    let mut model = state.model.write().await;
    if let Err(e) = model.add_link(link) {
        return Ok(ApiResponse::error(e));
    }

    Ok(ApiResponse::ok(link_copy))
}

#[tauri::command]
pub async fn list_links(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Link>>, String> {
    let model = state.model.read().await;
    let links = model.links.clone();
    Ok(ApiResponse::ok(links))
}

#[tauri::command]
pub async fn get_links_for_card(
    state: State<'_, AppState>,
    card_id: String,
) -> Result<ApiResponse<(Vec<Link>, Vec<Link>)>, String> {
    let model = state.model.read().await;
    let outgoing = model
        .links
        .iter()
        .filter(|l| l.source_id == card_id)
        .cloned()
        .collect();
    let incoming = model
        .links
        .iter()
        .filter(|l| l.target_id == card_id)
        .cloned()
        .collect();
    Ok(ApiResponse::ok((outgoing, incoming)))
}

// ============================================================================
// QUERY COMMANDS
// ============================================================================

#[tauri::command]
pub async fn query_find_by_type(
    state: State<'_, AppState>,
    card_type: String,
) -> Result<ApiResponse<Vec<Card>>, String> {
    let model = state.model.read().await;
    let card_type = match card_type.as_str() {
        "driver" => CardType::Driver,
        "requirement" => CardType::Requirement,
        "behavior" => CardType::Behavior,
        "interface" => CardType::Interface,
        "constraint" => CardType::Constraint,
        "logical-component" => CardType::LogicalComponent,
        "deployable-node" => CardType::DeployableNode,
        "actor" => CardType::Actor,
        "test" => CardType::Test,
        "artifact" => CardType::Artifact,
        "view" => CardType::View,
        "note" => CardType::Note,
        _ => return Err("Invalid card type".to_string()),
    };

    let engine = QueryEngine::new(model.clone());
    let cards: Vec<Card> = engine
        .find_by_type(card_type)
        .into_iter()
        .cloned()
        .collect();

    Ok(ApiResponse::ok(cards))
}

#[tauri::command]
pub async fn query_find_path(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<ApiResponse<Option<Vec<String>>>, String> {
    let model = state.model.read().await;
    let engine = QueryEngine::new(model.clone());
    let path = engine.find_path(&from, &to);
    Ok(ApiResponse::ok(path))
}

#[tauri::command]
pub async fn query_get_statistics(
    state: State<'_, AppState>,
) -> Result<ApiResponse<std::collections::HashMap<String, usize>>, String> {
    let model = state.model.read().await;
    let engine = QueryEngine::new(model.clone());
    let stats = engine.get_statistics();
    Ok(ApiResponse::ok(stats))
}

// ============================================================================
// PERSISTENCE COMMANDS
// ============================================================================

#[tauri::command]
pub async fn load_project(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ApiResponse<std::collections::HashMap<String, usize>>, String> {
    let persistence = Arc::new(PersistenceManager::new(
        project_path,
        AutoSaveConfig::default(),
    ));

    match persistence.load_model().await {
        Ok(model) => {
            let mut state_model = state.model.write().await;
            *state_model = model;
            
            let engine = QueryEngine::new(state_model.clone());
            Ok(ApiResponse::ok(engine.get_statistics()))
        }
        Err(e) => Ok(ApiResponse::error(format!("Failed to load project: {}", e))),
    }
}

#[tauri::command]
pub async fn save_project(
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    if let Err(e) = state.autosave.flush_all().await {
        return Ok(ApiResponse::error(format!("Failed to save: {}", e)));
    }

    Ok(ApiResponse::ok(true))
}

#[tauri::command]
pub async fn get_autosave_status(
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, String> {
    let pending = state.autosave.pending_count().await;
    Ok(ApiResponse::ok(pending))
}
