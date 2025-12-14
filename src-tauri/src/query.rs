use crate::model::{ArchitectureModel, Card, CardType, Link};
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};

/// Query engine for traversing and searching the architecture model
pub struct QueryEngine {
    model: ArchitectureModel,
}

impl QueryEngine {
    pub fn new(model: ArchitectureModel) -> Self {
        Self { model }
    }

    /// Find all cards by type
    pub fn find_by_type(&self, card_type: CardType) -> Vec<&Card> {
        self.model
            .cards
            .values()
            .filter(|c| c.r#type == card_type)
            .collect()
    }

    /// Find cards by name (partial match)
    pub fn find_by_name(&self, name: &str) -> Vec<&Card> {
        let lower = name.to_lowercase();
        self.model
            .cards
            .values()
            .filter(|c| c.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Find all outgoing links from a card
    pub fn get_outgoing_links(&self, card_id: &str) -> Vec<&Link> {
        self.model
            .links
            .iter()
            .filter(|l| l.source_id == card_id)
            .collect()
    }

    /// Find all incoming links to a card
    pub fn get_incoming_links(&self, card_id: &str) -> Vec<&Link> {
        self.model
            .links
            .iter()
            .filter(|l| l.target_id == card_id)
            .collect()
    }

    /// Get all cards directly linked (in or out)
    pub fn get_neighbors(&self, card_id: &str) -> Vec<&Card> {
        let mut neighbors = Vec::new();
        let mut seen = HashSet::new();

        for link in &self.model.links {
            if link.source_id == card_id && seen.insert(&link.target_id) {
                if let Some(card) = self.model.cards.get(&link.target_id) {
                    neighbors.push(card);
                }
            } else if link.target_id == card_id && seen.insert(&link.source_id) {
                if let Some(card) = self.model.cards.get(&link.source_id) {
                    neighbors.push(card);
                }
            }
        }

        neighbors
    }

    /// Find all cards reachable from a starting card (BFS)
    pub fn find_reachable(&self, card_id: &str) -> Vec<&Card> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(card_id.to_string());
        visited.insert(card_id.to_string());

        while let Some(current_id) = queue.pop_front() {
            if let Some(card) = self.model.cards.get(&current_id) {
                result.push(card);

                for link in self.get_outgoing_links(&current_id) {
                    if !visited.contains(&link.target_id) {
                        visited.insert(link.target_id.clone());
                        queue.push_back(link.target_id.clone());
                    }
                }
            }
        }

        result
    }

    /// Find path between two cards (BFS)
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to.to_string()];
                let mut current = to.to_string();

                while let Some(p) = parent.get(&current) {
                    path.push(p.clone());
                    current = p.clone();
                }

                path.reverse();
                return Some(path);
            }

            for link in self.get_outgoing_links(&current) {
                if !visited.contains(&link.target_id) {
                    visited.insert(link.target_id.clone());
                    parent.insert(link.target_id.clone(), current.clone());
                    queue.push_back(link.target_id.clone());
                }
            }
        }

        None
    }

    /// Get all cards that depend on a given card (reverse reachability)
    pub fn find_dependents(&self, card_id: &str) -> Vec<&Card> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(card_id.to_string());
        visited.insert(card_id.to_string());

        while let Some(current_id) = queue.pop_front() {
            if let Some(card) = self.model.cards.get(&current_id) {
                if current_id != card_id {
                    result.push(card);
                }

                for link in self.get_incoming_links(&current_id) {
                    if !visited.contains(&link.source_id) {
                        visited.insert(link.source_id.clone());
                        queue.push_back(link.source_id.clone());
                    }
                }
            }
        }

        result
    }

    /// Count cards by type
    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for card in self.model.cards.values() {
            *counts.entry(card.r#type.to_string()).or_insert(0) += 1;
        }

        counts
    }

    /// Get statistics about the model
    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_cards".to_string(), self.model.cards.len());
        stats.insert("total_links".to_string(), self.model.links.len());
        stats.insert("total_views".to_string(), self.model.views.len());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> ArchitectureModel {
        let mut model = ArchitectureModel::new();

        let mut card1 = Card::new(
            "test:driver-1".to_string(),
            CardType::Driver,
            "Driver 1".to_string(),
        );
        let mut card2 = Card::new(
            "test:req-1".to_string(),
            CardType::Requirement,
            "Requirement 1".to_string(),
        );
        let mut card3 = Card::new(
            "test:req-2".to_string(),
            CardType::Requirement,
            "Requirement 2".to_string(),
        );

        let _ = model.add_card(card1);
        let _ = model.add_card(card2);
        let _ = model.add_card(card3);

        let link1 = Link::new("test:driver-1".to_string(), "test:req-1".to_string());
        let link2 = Link::new("test:req-1".to_string(), "test:req-2".to_string());

        let _ = model.add_link(link1);
        let _ = model.add_link(link2);

        model
    }

    #[test]
    fn test_find_by_type() {
        let model = create_test_model();
        let engine = QueryEngine::new(model);
        let drivers = engine.find_by_type(CardType::Driver);
        assert_eq!(drivers.len(), 1);
    }

    #[test]
    fn test_find_path() {
        let model = create_test_model();
        let engine = QueryEngine::new(model);
        let path = engine.find_path("test:driver-1", "test:req-2");
        assert!(path.is_some());
        if let Some(path) = path {
            assert_eq!(path.len(), 3);
            assert_eq!(path[0], "test:driver-1");
            assert_eq!(path[2], "test:req-2");
        }
    }
}
