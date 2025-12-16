/// Traceability matrix generation and analysis
/// Produces a matrix showing links between card types (e.g., Drivers × Requirements × Behaviors)
use crate::models::{ArchitectureModel, Card, CardType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a single cell in the traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixCell {
    /// Source card ID
    pub source_id: String,
    /// Target card ID
    pub target_id: String,
    /// Number of links between these cards
    pub link_count: usize,
    /// Whether a direct link exists
    pub has_link: bool,
}

/// Represents a row in the traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixRow {
    /// Card ID (source)
    pub id: String,
    /// Card name
    pub name: String,
    /// Card type
    pub card_type: String,
    /// Links to target cards
    pub cells: Vec<MatrixCell>,
    /// Link count summary
    pub total_links: usize,
}

/// Complete traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    /// Target card type (column headers)
    pub target_type: String,
    /// Source card type (row headers)
    pub source_type: String,
    /// Matrix rows (one per source card)
    pub rows: Vec<MatrixRow>,
    /// Column headers (target cards)
    pub columns: Vec<String>,
    /// Coverage statistics
    pub coverage: f64,
    /// Orphaned sources (no outgoing links)
    pub orphaned_sources: Vec<String>,
    /// Unreferenced targets (no incoming links)
    pub unreferenced_targets: Vec<String>,
}

impl TraceabilityMatrix {
    /// Generate traceability matrix for source and target card types
    pub fn generate(
        model: &ArchitectureModel,
        source_type: CardType,
        target_type: CardType,
    ) -> Self {
        // Collect source and target cards
        let sources: Vec<&Card> = model
            .cards
            .values()
            .filter(|c| c.r#type == source_type)
            .collect();
        let targets: Vec<&Card> = model
            .cards
            .values()
            .filter(|c| c.r#type == target_type)
            .collect();

        // Build link map: source_id -> Vec<target_id>
        let mut link_map: HashMap<String, Vec<String>> = HashMap::new();
        for link in &model.links {
            if let Some(target_id) = &link.target_id {
                if let Some(source_card) = model.cards.get(&link.source_id) {
                    if source_card.r#type == source_type {
                        if let Some(target_card) = model.cards.get(target_id) {
                            if target_card.r#type == target_type {
                                link_map
                                    .entry(link.source_id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(target_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // Build columns (target card IDs)
        let mut columns: Vec<String> = targets.iter().map(|c| c.id.clone()).collect();
        columns.sort();

        // Build rows
        let mut rows = Vec::new();
        let mut covered_targets = HashSet::new();

        for source in &sources {
            let cell_links = link_map.get(&source.id).cloned().unwrap_or_default();
            let total_links = cell_links.len();

            let cells: Vec<MatrixCell> = columns
                .iter()
                .map(|target_id| {
                    let has_link = cell_links.contains(target_id);
                    if has_link {
                        covered_targets.insert(target_id.clone());
                    }
                    MatrixCell {
                        source_id: source.id.clone(),
                        target_id: target_id.clone(),
                        link_count: if has_link { 1 } else { 0 },
                        has_link,
                    }
                })
                .collect();

            rows.push(MatrixRow {
                id: source.id.clone(),
                name: source.name.clone(),
                card_type: format!("{:?}", source.r#type),
                cells,
                total_links,
            });
        }

        // Calculate coverage
        let coverage = if !targets.is_empty() {
            (covered_targets.len() as f64 / targets.len() as f64) * 100.0
        } else {
            0.0
        };

        // Find orphaned sources and unreferenced targets
        let orphaned_sources: Vec<String> = rows
            .iter()
            .filter(|r| r.total_links == 0)
            .map(|r| r.id.clone())
            .collect();

        let unreferenced_targets: Vec<String> = targets
            .iter()
            .filter(|t| !covered_targets.contains(&t.id))
            .map(|t| t.id.clone())
            .collect();

        TraceabilityMatrix {
            target_type: format!("{:?}", target_type),
            source_type: format!("{:?}", source_type),
            rows,
            columns,
            coverage,
            orphaned_sources,
            unreferenced_targets,
        }
    }

    /// Get a specific cell's link status
    pub fn get_cell(&self, row_idx: usize, col_idx: usize) -> Option<&MatrixCell> {
        self.rows
            .get(row_idx)
            .and_then(|row| row.cells.get(col_idx))
    }

    /// Get all links from a source card
    pub fn get_source_links(&self, row_idx: usize) -> Option<Vec<&MatrixCell>> {
        self.rows
            .get(row_idx)
            .map(|row| row.cells.iter().filter(|c| c.has_link).collect())
    }

    /// Get all incoming links to a target card
    pub fn get_target_links(&self, col_idx: usize) -> Vec<&MatrixCell> {
        self.rows
            .iter()
            .flat_map(|row| row.cells.get(col_idx))
            .filter(|c| c.has_link)
            .collect()
    }
}

/// Gap analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub gap_type: String,
    pub source_id: String,
    pub target_id: Option<String>,
    pub severity: String,
    pub recommendation: String,
}

impl GapAnalysis {
    /// Analyze traceability gaps
    pub fn analyze(matrix: &TraceabilityMatrix) -> Vec<GapAnalysis> {
        let mut gaps = Vec::new();

        // Check for orphaned sources
        for source_id in &matrix.orphaned_sources {
            gaps.push(GapAnalysis {
                gap_type: "orphaned_source".to_string(),
                source_id: source_id.clone(),
                target_id: None,
                severity: "high".to_string(),
                recommendation: format!(
                    "Source {} has no links to any target. Please create at least one link.",
                    source_id
                ),
            });
        }

        // Check for unreferenced targets
        for target_id in &matrix.unreferenced_targets {
            gaps.push(GapAnalysis {
                gap_type: "unreferenced_target".to_string(),
                source_id: target_id.clone(),
                target_id: None,
                severity: "medium".to_string(),
                recommendation: format!(
                    "Target {} is not referenced by any source. Consider removing or linking it.",
                    target_id
                ),
            });
        }

        gaps
    }
}
