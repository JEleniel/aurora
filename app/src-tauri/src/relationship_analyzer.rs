/// Relationship analyzer for exploring card dependencies
/// Enables upstream/downstream analysis and impact assessment
use crate::models::ArchitectureModel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a single relationship entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEntry {
    pub id: String,
    pub name: String,
    pub card_type: String,
    pub depth: usize,
}

/// Complete relationship analysis for a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAnalysis {
    pub card_id: String,
    pub card_name: String,
    pub upstream: Vec<RelationshipEntry>,
    pub downstream: Vec<RelationshipEntry>,
    pub total_affected: usize,
    pub circular_refs: Vec<String>,
    pub analysis_depth: usize,
}

impl RelationshipAnalysis {
    /// Analyze relationships for a given card
    ///
    /// # Arguments
    /// * `card_id` - The card to analyze
    /// * `model` - The architecture model
    /// * `max_depth` - Maximum traversal depth (0 = immediate only, usize::MAX = all)
    ///
    /// # Returns
    /// RelationshipAnalysis with upstream/downstream dependencies
    pub fn analyze(
        card_id: &str,
        model: &ArchitectureModel,
        max_depth: usize,
    ) -> Self {
        let card = model.cards.get(card_id);
        let (card_name, card_type) = if let Some(c) = card {
            (c.name.clone(), format!("{:?}", c.r#type))
        } else {
            ("Unknown".to_string(), "Unknown".to_string())
        };

        let mut upstream = Vec::new();
        let mut downstream = Vec::new();
        let mut visited = HashSet::new();
        let mut circular_refs = HashSet::new();

        // Find direct links
        let mut upstream_direct = HashSet::new();
        let mut downstream_direct = HashSet::new();

        for link in &model.links {
            if let Some(target) = &link.target_id {
                if target == card_id {
                    upstream_direct.insert(link.source_id.clone());
                }
                if link.source_id == card_id {
                    downstream_direct.insert(target.clone());
                }
            }
        }

        // Traverse upstream (cards that link to this card)
        Self::traverse(
            card_id,
            &upstream_direct,
            model,
            max_depth,
            &mut visited,
            &mut circular_refs,
            true,
            &mut upstream,
        );

        // Reset visited for downstream traversal
        visited.clear();

        // Traverse downstream (cards this card links to)
        Self::traverse(
            card_id,
            &downstream_direct,
            model,
            max_depth,
            &mut visited,
            &mut circular_refs,
            false,
            &mut downstream,
        );

        let total_affected = upstream.len() + downstream.len();
        let circular_vec: Vec<String> = circular_refs.iter().cloned().collect();

        RelationshipAnalysis {
            card_id: card_id.to_string(),
            card_name,
            upstream,
            downstream,
            total_affected,
            circular_refs: circular_vec,
            analysis_depth: max_depth,
        }
    }

    /// Recursive traversal of relationships
    fn traverse(
        origin: &str,
        current_level: &HashSet<String>,
        model: &ArchitectureModel,
        max_depth: usize,
        visited: &mut HashSet<String>,
        circular_refs: &mut HashSet<String>,
        is_upstream: bool,
        results: &mut Vec<RelationshipEntry>,
    ) {
        if max_depth == 0 || current_level.is_empty() {
            return;
        }

        let mut next_level = HashSet::new();
        let current_depth = visited.len();

        for card_id in current_level {
            if visited.contains(card_id) {
                circular_refs.insert(card_id.clone());
                continue;
            }

            visited.insert(card_id.clone());

            // Get card info
            if let Some(card) = model.cards.get(card_id) {
                results.push(RelationshipEntry {
                    id: card_id.clone(),
                    name: card.name.clone(),
                    card_type: format!("{:?}", card.r#type),
                    depth: current_depth + 1,
                });

                // Find next level links
                for link in &model.links {
                    let next_id = if is_upstream {
                        // If going upstream, find who links to current card
                        if link.target_id.as_ref().map(|t| t == card_id).unwrap_or(false) {
                            Some(&link.source_id)
                        } else {
                            None
                        }
                    } else {
                        // If going downstream, find who current card links to
                        if link.source_id == *card_id {
                            link.target_id.as_ref()
                        } else {
                            None
                        }
                    };

                    if let Some(next_id) = next_id {
                        if next_id != origin && !visited.contains(next_id) {
                            next_level.insert(next_id.clone());
                        } else if next_id == origin {
                            circular_refs.insert(next_id.clone());
                        }
                    }
                }
            }
        }

        // Recurse to next level
        if current_depth < max_depth {
            Self::traverse(
                origin,
                &next_level,
                model,
                max_depth,
                visited,
                circular_refs,
                is_upstream,
                results,
            );
        }
    }

    /// Calculate impact if this card were to change
    pub fn calculate_impact(&self) -> ImpactAssessment {
        ImpactAssessment {
            direct_dependents: self
                .upstream
                .iter()
                .filter(|r| r.depth == 1)
                .count(),
            direct_dependencies: self
                .downstream
                .iter()
                .filter(|r| r.depth == 1)
                .count(),
            transitive_dependents: self
                .upstream
                .iter()
                .filter(|r| r.depth > 1)
                .count(),
            transitive_dependencies: self
                .downstream
                .iter()
                .filter(|r| r.depth > 1)
                .count(),
            circular_dependency_risk: !self.circular_refs.is_empty(),
            total_impact_scope: self.total_affected,
        }
    }
}

/// Impact assessment for change analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub direct_dependents: usize,
    pub direct_dependencies: usize,
    pub transitive_dependents: usize,
    pub transitive_dependencies: usize,
    pub circular_dependency_risk: bool,
    pub total_impact_scope: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Card, CardStatus, CardType};
    use chrono::Utc;

    #[test]
    fn test_relationship_analysis() {
        let mut model = ArchitectureModel::default();

        // Create test cards
        let card1 = Card {
            id: "1".to_string(),
            r#type: CardType::Driver,
            name: "Driver".to_string(),
            description: Some("Test driver".to_string()),
            version: None,
            status: Some(CardStatus::Draft),
            attributes: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        let card2 = Card {
            id: "2".to_string(),
            r#type: CardType::Requirement,
            name: "Requirement".to_string(),
            description: Some("Test requirement".to_string()),
            version: None,
            status: Some(CardStatus::Draft),
            attributes: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        model.cards.insert("1".to_string(), card1);
        model.cards.insert("2".to_string(), card2);

        // Create link: Driver -> Requirement
        let link = crate::models::Link {
            source_id: "1".to_string(),
            target_id: Some("2".to_string()),
            target_url: None,
            metadata: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        model.links.push(link);

        // Analyze from Requirement (upstream)
        let analysis = RelationshipAnalysis::analyze("2", &model, 5);

        assert_eq!(analysis.card_id, "2");
        assert_eq!(analysis.card_name, "Requirement");
        assert_eq!(analysis.upstream.len(), 1);
        assert_eq!(analysis.downstream.len(), 0);
        assert_eq!(analysis.upstream[0].id, "1");
    }
}
