mod model;
mod persistence;
mod query;
mod commands;

use crate::commands::AppState;
use crate::model::ArchitectureModel;
use crate::persistence::{AutoSaveConfig, AutoSaveManager, PersistenceManager};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Disable GPU acceleration on ARM systems
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        std::env::set_var("WEBKIT_DISABLE_WEBPAGE_ANIMATIONS", "1");
    }
    
    // Initialize logging
    env_logger::init();

    // Set up panic hook to log instead of silently crash
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic"
        };
        
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        eprintln!("!!! PANIC: {} at {}", msg, location);
        log::error!("Application panic: {} at {}", msg, location);
        
        // Call the default panic hook to maintain normal panic behavior
        default_panic(panic_info);
    }));

    // Create app state
    let persistence = Arc::new(PersistenceManager::new(
        "./aurora_project",
        AutoSaveConfig::default(),
    ));

    let autosave = Arc::new(AutoSaveManager::new(persistence.clone()));

    let app_state = AppState {
        model: Arc::new(RwLock::new(ArchitectureModel::new())),
        persistence,
        autosave,
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Card commands
            commands::create_card,
            commands::get_card,
            commands::update_card,
            commands::delete_card,
            commands::list_cards,
            // Link commands
            commands::create_link,
            commands::list_links,
            commands::get_links_for_card,
            // Query commands
            commands::query_find_by_type,
            commands::query_find_path,
            commands::query_get_statistics,
            // Persistence commands
            commands::load_project,
            commands::save_project,
            commands::get_autosave_status,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Fatal error running Tauri application: {}", e);
            log::error!("Fatal error running Tauri application: {}", e);
            std::process::exit(1);
        });
}

pub fn main() {
    run();
}
