use crate::models::Card;
use crate::AppState;

/// Validate a card against its type's schema.
///
/// Performs comprehensive validation to ensure the card conforms to all constraints
/// and requirements for its specific card type (driver, requirement, behavior, etc.).
/// Uses the schema validator to check fields, types, and relationships.
///
/// # Arguments
/// * `card` - Card to validate
/// * `state` - Application state (contains validator)
///
/// # Returns
/// Success message if validation passes, or detailed error description if validation fails.
#[tauri::command]
pub fn validate_card(card: Card, state: tauri::State<AppState>) -> Result<String, String> {
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

/// Log an error message from the frontend to backend logs.
///
/// Enables the frontend to write to the backend logger with context. Useful for
/// debugging client-side issues or capturing important state transitions. Log level
/// is flexible to support varying severity levels.
///
/// # Arguments
/// * `level` - Log level: "error", "warn", "info", or "debug"
/// * `message` - The message to log
/// * `context` - Optional context label (e.g., component name), defaults to "frontend"
///
/// # Returns
/// Success message after logging completes.
#[tauri::command]
pub fn log_error(
    level: String,
    message: String,
    context: Option<String>,
) -> Result<String, String> {
    let ctx = context.as_deref().unwrap_or("frontend");
    match level.as_str() {
        "error" => log::error!("[{}] {}", ctx, message),
        "warn" => log::warn!("[{}] {}", ctx, message),
        "info" => log::info!("[{}] {}", ctx, message),
        "debug" => log::debug!("[{}] {}", ctx, message),
        _ => log::info!("[{}] {}", ctx, message),
    }
    Ok("Error logged".to_string())
}
