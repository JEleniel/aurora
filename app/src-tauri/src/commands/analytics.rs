use crate::AppState;

/// Generate a traceability matrix between two card types.
///
/// Creates a matrix showing all links (traceability) between cards of two specified types.
/// For example, matrices can show which requirements trace to which tests, or which
/// drivers influence which requirements. The result is a structured table suitable
/// for traceability reports and coverage analysis.
///
/// # Arguments
/// * `source_type` - Origin card type (e.g., "driver", "requirement")
/// * `target_type` - Destination card type (e.g., "requirement", "test")
/// * `state` - Application state
///
/// # Returns
/// JSON matrix structure showing relationships, or error if types are invalid.
#[tauri::command(rename_all = "snake_case")]
pub fn generate_traceability_matrix(
    source_type: String,
    target_type: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    log::info!(
        "Generating traceability matrix: {} -> {}",
        source_type,
        target_type
    );

    let model = crate::acquire_lock(&state)?;

    let source = crate::parse_card_type(&source_type)?;
    let target = crate::parse_card_type(&target_type)?;

    let matrix = crate::traceability_matrix::TraceabilityMatrix::generate(&model, source, target);

    crate::serialize_json(&matrix, "traceability matrix")
}

/// Generate a complete dependency graph of the architecture.
///
/// Constructs a directed graph representing all dependencies and relationships
/// across the entire model. Useful for visualization, impact analysis, and
/// understanding the overall structure of the architecture. The graph can
/// identify critical paths and dependencies.
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// JSON graph structure (nodes and edges), or error if lock fails.
#[tauri::command]
pub fn generate_dependency_graph(state: tauri::State<AppState>) -> Result<String, String> {
    log::info!("Generating dependency graph");

    let model = crate::acquire_lock(&state)?;
    let graph = crate::dependency_graph::DependencyGraph::generate(&model);

    crate::serialize_json(&graph, "dependency graph")
}
