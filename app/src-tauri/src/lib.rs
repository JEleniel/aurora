pub mod dependency_graph;
pub mod models;
pub mod schema_validator;
pub mod traceability_matrix;
pub mod zip_handler;

use dependency_graph::DependencyGraph;
use models::{AppError, ArchitectureModel, Card, CardStatus, CardType, Link};
use schema_validator::SchemaValidator;
use std::path::PathBuf;
use std::sync::Mutex;
use traceability_matrix::TraceabilityMatrix;

// Global architecture model state
pub struct AppState {
    pub model: Mutex<ArchitectureModel>,
    pub validator: SchemaValidator,
}

// Helper: Acquire model lock with error handling
fn acquire_lock<'a>(
    state: &'a tauri::State<AppState>,
) -> Result<std::sync::MutexGuard<'a, ArchitectureModel>, String> {
    state.model.lock().map_err(|e| {
        let err = AppError::locking_error(
            "Failed to access model storage",
            format!("Could not acquire lock: {}", e),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

// Helper: Serialize to JSON with error handling
fn serialize_json<T: serde::Serialize>(value: &T, context: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| {
        let err = AppError::serialization_error(
            format!("Failed to serialize {}", context),
            format!("JSON serialization error: {}", e),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

// Helper: Parse card type with validation
fn parse_card_type(card_type: &str) -> Result<CardType, String> {
    CardType::from_str(card_type).ok_or_else(|| {
        let err = AppError::validation(
            "Invalid card type specified",
            format!("Unknown card type: '{}'. Valid types are: driver, requirement, behavior, interface, constraint, logical-component, deployable-node, actor, test, artifact, view, note", card_type)
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

// Helper: Import model from ZIP file path
fn import_model(path: &str) -> Result<ArchitectureModel, String> {
    let file_path = PathBuf::from(path);
    log::info!("Importing architecture from: {}", path);
    zip_handler::import_from_zip(&file_path).map_err(|e| {
        let err = AppError::file_error(
            "Failed to load architecture file",
            format!("Could not load from {}: {}", path, e),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

// Helper: Export model to ZIP file path
fn export_model(model: &ArchitectureModel, path: &str) -> Result<(), String> {
    let file_path = PathBuf::from(path);
    log::info!("Exporting architecture to: {}", path);
    zip_handler::export_to_zip(model, &file_path).map_err(|e| {
        let err = AppError::file_error(
            "Failed to save architecture file",
            format!("Could not save to {}: {}", path, e),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

/// Load architecture from ZIP file
#[tauri::command]
fn load_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let model = import_model(&path)?;
    let mut app_model = acquire_lock(&state)?;
    *app_model = model;
    let success_msg = format!("Successfully loaded architecture from {}", path);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Save current architecture to ZIP file
#[tauri::command]
fn save_architecture(path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let model = acquire_lock(&state)?;
    export_model(&*model, &path)?;
    let success_msg = format!("Successfully saved architecture to {}", path);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Create a new card
#[tauri::command(rename_all = "snake_case")]
fn create_card(
    id: String,
    card_type: String,
    name: String,
    description: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let type_enum = parse_card_type(&card_type)?;
    let card = Card::new(id.clone(), type_enum, name, description);
    let mut model = acquire_lock(&state)?;
    log::info!("Creating card: id={}, type={}", id, card_type);
    model.add_card(card);
    let success_msg = format!("Card '{}' created successfully", id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Delete a card
#[tauri::command]
fn delete_card(id: String, state: tauri::State<AppState>) -> Result<String, String> {
    let mut model = acquire_lock(&state)?;
    log::info!("Deleting card: {}", id);
    model.remove_card(&id).ok_or_else(|| {
        let err = AppError::not_found(
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

/// Get all cards
#[tauri::command]
fn get_cards(state: tauri::State<AppState>) -> Result<String, String> {
    let model = acquire_lock(&state)?;
    log::debug!("Retrieving all cards");
    let cards: Vec<_> = model.cards.values().collect();
    serialize_json(&cards, "cards")
}

/// Get cards by type
#[tauri::command(rename_all = "snake_case")]
fn get_cards_by_type(card_type: String, state: tauri::State<AppState>) -> Result<String, String> {
    let type_enum = parse_card_type(&card_type)?;
    let model = acquire_lock(&state)?;
    log::debug!("Retrieving cards of type: {}", card_type);
    let cards = model.get_cards_by_type(&type_enum);
    serialize_json(&cards, "filtered cards")
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
    let mut model = acquire_lock(&state)?;
    log::info!("Updating card: {}", id);
    let card = model.get_card_mut(&id).ok_or_else(|| {
        let err = AppError::not_found(
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

// Helper: Create link from parameters
fn create_link_from_params(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
) -> Result<Link, String> {
    if let Some(id) = target_id {
        Ok(Link::new_internal(source_id, id))
    } else if let Some(url) = target_url {
        Ok(Link::new_external(source_id, url))
    } else {
        let err = AppError::validation(
            "Invalid link configuration",
            "Either target_id or target_url must be provided",
        );
        log::error!("{}", err.technical_message);
        Err(String::from(err))
    }
}

/// Create a link
#[tauri::command(rename_all = "snake_case")]
fn create_link(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let link = create_link_from_params(source_id.clone(), target_id, target_url)?;
    let mut model = acquire_lock(&state)?;
    log::info!("Creating link from: {}", source_id);
    model.add_link(link);
    let success_msg = format!("Link created successfully from {}", source_id);
    log::info!("{}", success_msg);
    Ok(success_msg)
}

/// Delete a link
#[tauri::command(rename_all = "snake_case")]
fn delete_link(
    source_id: String,
    target_id: Option<String>,
    target_url: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = acquire_lock(&state)?;
    log::info!("Deleting link from: {}", source_id);
    let removed = model.remove_link(&source_id, target_id.as_deref(), target_url.as_deref());
    if removed {
        let success_msg = format!("Link deleted successfully from {}", source_id);
        log::info!("{}", success_msg);
        Ok(success_msg)
    } else {
        let err = AppError::not_found(
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

/// Get all links
#[tauri::command]
fn get_links(state: tauri::State<AppState>) -> Result<String, String> {
    let model = acquire_lock(&state)?;
    log::debug!("Retrieving all links");
    serialize_json(&model.links, "links")
}

/// Get model statistics
#[tauri::command]
fn get_statistics(state: tauri::State<AppState>) -> Result<String, String> {
    let model = acquire_lock(&state)?;
    log::debug!("Computing model statistics");
    let stats = model.statistics();
    serialize_json(&stats, "statistics")
}

/// Get model metadata
#[tauri::command]
fn get_metadata(state: tauri::State<AppState>) -> Result<String, String> {
    let model = acquire_lock(&state)?;
    log::debug!("Retrieving model metadata");
    serialize_json(&model.metadata, "metadata")
}

// Helper: Apply metadata updates to model
fn apply_metadata_updates(
    model: &mut ArchitectureModel,
    name: Option<String>,
    description: Option<String>,
    root_driver_id: Option<String>,
) {
    if let Some(n) = name {
        model.metadata.name = Some(n);
    }
    if let Some(d) = description {
        model.metadata.description = Some(d);
    }
    if let Some(id) = root_driver_id {
        model.metadata.root_driver_id = Some(id);
    }
}

/// Update model metadata
#[tauri::command(rename_all = "snake_case")]
fn update_metadata(
    name: Option<String>,
    description: Option<String>,
    root_driver_id: Option<String>,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut model = acquire_lock(&state)?;
    log::info!("Updating model metadata");
    apply_metadata_updates(&mut model, name, description, root_driver_id);
    let success_msg = "Metadata updated successfully".to_string();
    log::info!("{}", success_msg);
    Ok(success_msg)
}

// Helper: Log message at appropriate level
fn log_message(level: &str, context: &str, message: &str) {
    match level {
        "error" => log::error!("[{}] {}", context, message),
        "warn" => log::warn!("[{}] {}", context, message),
        "info" => log::info!("[{}] {}", context, message),
        "debug" => log::debug!("[{}] {}", context, message),
        _ => log::info!("[{}] {}", context, message),
    }
}

/// Log an error from the frontend to backend logs
#[tauri::command]
fn log_error(level: String, message: String, context: Option<String>) -> Result<String, String> {
    let ctx = context.as_deref().unwrap_or("frontend");
    log_message(&level, ctx, &message);
    Ok("Error logged".to_string())
}

/// Validate a card against its type's schema
#[tauri::command]
fn validate_card(card: Card, state: tauri::State<AppState>) -> Result<String, String> {
    log::info!("Validating card: {}", card.name);
    match state.validator.validate_card(&card) {
        Ok(()) => {
            let success_msg = format!("Card '{}' passed validation", card.name);
            log::info!("{}", success_msg);
            Ok(success_msg)
        }
        Err(e) => {
            log::warn!("Card validation failed for '{}': {}", card.name, e);
            Err(e)
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn generate_traceability_matrix(
    source_type: String,
    target_type: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    log::info!(
        "Generating traceability matrix: {} -> {}",
        source_type,
        target_type
    );

    let model = acquire_lock(&state)?;

    let source = parse_card_type(&source_type)?;
    let target = parse_card_type(&target_type)?;

    let matrix = TraceabilityMatrix::generate(&model, source, target);

    serialize_json(&matrix, "traceability matrix")
}

#[tauri::command]
fn generate_dependency_graph(state: tauri::State<AppState>) -> Result<String, String> {
    log::info!("Generating dependency graph");

    let model = acquire_lock(&state)?;
    let graph = DependencyGraph::generate(&model);

    serialize_json(&graph, "dependency graph")
}

#[tauri::command]
fn select_file(window: tauri::Window) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = window
        .dialog()
        .file()
        .add_filter("ZIP Files", &["zip"])
        .blocking_pick_file();

    Ok(file_path.map(|p| p.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        // Log to stdout
        .chain(std::io::stdout())
        // Log to file
        .chain(fern::log_file("aurora.log").unwrap_or_else(|_| {
            eprintln!("Warning: Could not open aurora.log for writing");
            std::fs::File::create("aurora.log").expect("Failed to create log file")
        }))
        .apply()
        .expect("Failed to initialize logging");

    log::info!("AURORA application starting");

    // Initialize schema validator from the schemas directory
    let schema_dir = std::path::Path::new("../schemas");
    let validator = SchemaValidator::new(schema_dir).unwrap_or_else(|e| {
        log::warn!("Failed to initialize schema validator: {}", e);
        SchemaValidator::new(std::path::Path::new(".")).expect("Failed to create empty validator")
    });

    let app_state = AppState {
        model: Mutex::new(ArchitectureModel::new()),
        validator,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            delete_link,
            get_links,
            get_statistics,
            get_metadata,
            update_metadata,
            log_error,
            validate_card,
            generate_traceability_matrix,
            generate_dependency_graph,
            select_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    log::info!("AURORA application shutting down");
}
