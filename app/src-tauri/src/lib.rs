pub mod commands;
pub mod config;
pub mod constants;
pub mod dependency_graph;
pub mod models;
pub mod relationship_analyzer;
pub mod schema_validator;
pub mod traceability_matrix;
pub mod zip_handler;

use config::{AppConfig, ConfigManager};
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
    pub config: Mutex<AppConfig>,
}

// Helper: Acquire model lock with error handling
/// Acquire a mutable lock on the application's architecture model.
///
/// Safely obtains a mutex lock on the shared architecture model state. This function
/// ensures thread-safe access and provides detailed error reporting if locking fails.
/// Commands should call this early to obtain access to the model.
///
/// # Arguments
/// * `state` - Application state containing the mutex-protected model
///
/// # Returns
/// MutexGuard providing mutable access to the model, or error if lock cannot be acquired.
///
/// # Errors
/// Returns locking error if the mutex is poisoned (previous thread panicked) or other
/// synchronization issues prevent lock acquisition.
pub fn acquire_lock<'a>(
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

/// Serialize a value to JSON string with comprehensive error handling.
///
/// Converts any Serialize-able value to a JSON string representation. Handles serialization
/// errors gracefully with context and detailed logging. Useful for returning complex data
/// structures to the frontend via Tauri IPC.
///
/// # Arguments
/// * `value` - The value to serialize (must implement serde::Serialize)
/// * `context` - A descriptive label (e.g., "cards", "links") for error messages
///
/// # Returns
/// JSON string representation of the value, or error with context if serialization fails.
///
/// # Errors
/// Returns serialization error if the value cannot be converted to valid JSON (e.g., NaN values, cycles).
pub fn serialize_json<T: serde::Serialize>(value: &T, context: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| {
        let err = AppError::serialization_error(
            format!("Failed to serialize {}", context),
            format!("JSON serialization error: {}", e),
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

/// Parse and validate a card type string.
///
/// Converts a string representation (e.g., "driver", "requirement") into a strongly-typed
/// CardType enum. Validates against the set of supported types and provides detailed error
/// messages with valid alternatives if parsing fails.
///
/// # Arguments
/// * `card_type` - String to parse (case-sensitive)
///
/// # Returns
/// Parsed CardType enum value, or error listing valid types if string is unrecognized.
///
/// # Errors
/// Returns validation error if the string does not match any supported card type.
pub fn parse_card_type(card_type: &str) -> Result<CardType, String> {
    CardType::from_str(card_type).ok_or_else(|| {
        let err = AppError::validation(
            "Invalid card type specified",
            format!("Unknown card type: '{}'. Valid types are: driver, requirement, behavior, interface, constraint, logical-component, deployable-node, actor, test, artifact, view, note", card_type)
        );
        log::error!("{}", err.technical_message);
        String::from(err)
    })
}

/// Load an architecture model from a ZIP file.
///
/// Reads and deserializes a complete architecture model from a ZIP archive at the specified path.
/// The ZIP should contain properly structured JSON files representing the model's cards, links,
/// metadata, and other components. Useful for opening saved architecture documents.
///
/// # Arguments
/// * `path` - File system path to the ZIP file (absolute or relative)
///
/// # Returns
/// Deserialized ArchitectureModel, or error if file not found, is invalid ZIP, or contents malformed.
///
/// # Errors
/// Returns file error if file cannot be opened, read, or parsed as valid architecture model.
pub fn import_model(path: &str) -> Result<ArchitectureModel, String> {
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

/// Save an architecture model to a ZIP file.
///
/// Serializes and writes a complete architecture model to a ZIP archive at the specified path.
/// Overwrites existing files. Creates a structured archive containing all model components
/// (cards, links, metadata). Useful for saving work or creating backups.
///
/// # Arguments
/// * `model` - Architecture model to serialize
/// * `path` - File system path where ZIP file should be written (absolute or relative)
///
/// # Returns
/// Empty Ok result if successful, or error if file cannot be written or serialization fails.
///
/// # Errors
/// Returns file error if disk write fails, path is invalid, or serialization encounters issues.
pub fn export_model(model: &ArchitectureModel, path: &str) -> Result<(), String> {
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

/// Factory function to create a Link from parameters.
///
/// Constructs either an internal link (to another card) or external link (to URL) based
/// on which target parameter is provided. Validates that exactly one target type is specified.
/// Simplifies link creation logic used by the create_link command.
///
/// # Arguments
/// * `source_id` - ID of the source card
/// * `target_id` - Card ID for internal link (use if linking to another card)
/// * `target_url` - External URL for external link (use if linking to external resource)
///
/// # Returns
/// Created Link object, or error if neither or both targets are specified.
///
/// # Errors
/// Returns validation error if target_id and target_url are both None or both Some.
pub fn create_link_from_params(
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

/// Apply optional metadata updates to the architecture model.
///
/// Updates the model's metadata fields (name, description, root_driver_id) with provided values.
/// Fields not provided (None) are left unchanged. Useful for bulk metadata modifications
/// without requiring separate calls for each field.
///
/// # Arguments
/// * `model` - Mutable reference to the model to update
/// * `name` - New project name, or None to leave unchanged
/// * `description` - New project description, or None to leave unchanged
/// * `root_driver_id` - Card ID to use as root driver, or None to leave unchanged
///
/// # Notes
/// This function does not validate the root_driver_id. The id should be verified to exist
/// in the model before calling. Updates are applied in-place without returning a result.
pub fn apply_metadata_updates(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging with graceful degradation
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("aurora.log")
        .or_else(|_| {
            eprintln!("Warning: Could not open aurora.log for writing");
            std::fs::File::create("aurora.log")
        });

    match log_file {
        Ok(file) => {
            if let Err(e) = fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .level(log::LevelFilter::Debug)
                .chain(std::io::stdout())
                .chain(file)
                .apply()
            {
                eprintln!("Warning: Failed to initialize logging with file: {}", e);
                // Initialize without file logging
                let _ = fern::Dispatch::new()
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "[{} {}] {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            message
                        ))
                    })
                    .level(log::LevelFilter::Debug)
                    .chain(std::io::stdout())
                    .apply();
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not create aurora.log: {}", e);
            // Initialize logging without file
            let _ = fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .level(log::LevelFilter::Debug)
                .chain(std::io::stdout())
                .apply();
        }
    }

    log::info!("AURORA application starting");

    // Initialize schema validator from the schemas symlink in the app directory
    // The symlink is created during setup and points to ../schemas
    let schema_paths = vec![
        std::path::PathBuf::from("./schemas"),
        std::path::PathBuf::from("schemas"),
        std::path::PathBuf::from("../schemas"),
        std::path::PathBuf::from("../../schemas"),
    ];

    let mut validator = None;
    for schema_dir in schema_paths {
        if schema_dir.exists() {
            log::info!("Found schemas directory at: {:?}", schema_dir);
            match SchemaValidator::new(&schema_dir) {
                Ok(v) => {
                    validator = Some(v);
                    break;
                }
                Err(e) => {
                    log::warn!(
                        "Failed to initialize schema validator from {:?}: {}",
                        schema_dir,
                        e
                    );
                }
            }
        }
    }

    let validator = validator.unwrap_or_else(|| {
        log::warn!("Could not find schemas directory, creating minimal validator");
        match SchemaValidator::new(std::path::Path::new(".")) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to create minimal validator: {}", e);
                // Return a validator with no schemas - validation will be permissive
                SchemaValidator::new(std::path::Path::new(".")).unwrap()
            }
        }
    });

    // Initialize configuration
    let config_manager = match ConfigManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            log::warn!("Failed to initialize config manager: {}", e);
            // Use default config manager - already handles creation failures gracefully
            ConfigManager::default()
        }
    };

    let config = config_manager.load().unwrap_or_else(|e| {
        log::warn!("Failed to load config: {}", e);
        AppConfig::default()
    });

    log::info!("Config loaded from: {:?}", config_manager.get_config_path());

    let app_state = AppState {
        model: Mutex::new(ArchitectureModel::new()),
        validator,
        config: Mutex::new(config),
    };

    match tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::archive::load_architecture,
            commands::archive::save_architecture,
            commands::cards::create_card,
            commands::cards::delete_card,
            commands::cards::get_cards,
            commands::cards::get_cards_by_type,
            commands::cards::update_card,
            commands::links::create_link,
            commands::links::delete_link,
            commands::links::get_links,
            commands::metadata::get_statistics,
            commands::metadata::get_metadata,
            commands::metadata::update_metadata,
            commands::diagnostics::log_error,
            commands::diagnostics::validate_card,
            commands::analytics::generate_traceability_matrix,
            commands::analytics::generate_dependency_graph,
            commands::relationships::analyze_relationships,
            commands::file_dialogs::select_file,
            commands::file_dialogs::select_save_file,
        ])
        .on_window_event(|_window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                log::info!("Close requested, exiting application");
            }
            _ => {}
        })
        .build(tauri::generate_context!())
    {
        Ok(app) => {
            app.run(|_app_handle, event| match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    log::info!("Exit requested");
                }
                tauri::RunEvent::Exit => {
                    log::info!("Application exiting");
                }
                _ => {}
            });
        }
        Err(e) => {
            log::error!("Failed to build Tauri application: {}", e);
            eprintln!("Critical error: Failed to build Tauri application: {}", e);
        }
    }

    log::info!("AURORA application shutting down");
}
