/// Relationship analysis commands for card dependency exploration
use crate::relationship_analyzer::RelationshipAnalysis;
use crate::{acquire_lock, AppState};
use tauri::State;

/// Analyze relationships (upstream and downstream) for a card
///
/// # Arguments
/// * `card_id` - The card to analyze
/// * `max_depth` - Maximum traversal depth (usize::MAX for unlimited)
///
/// # Returns
/// RelationshipAnalysis with upstream/downstream dependencies and impact metrics
///
/// # Errors
/// Returns `Err` if card not found or lock acquisition fails
#[tauri::command]
pub fn analyze_relationships(
    card_id: String,
    max_depth: usize,
    state: State<AppState>,
) -> Result<RelationshipAnalysis, String> {
    let model = acquire_lock(&state)?;
    Ok(RelationshipAnalysis::analyze(&card_id, &model, max_depth))
}
