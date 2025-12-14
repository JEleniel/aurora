use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core card types in AURORA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CardType {
    Driver,
    Requirement,
    Behavior,
    Interface,
    Constraint,
    LogicalComponent,
    DeployableNode,
    Actor,
    Test,
    Artifact,
    View,
    Note,
}

impl std::fmt::Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardType::Driver => write!(f, "driver"),
            CardType::Requirement => write!(f, "requirement"),
            CardType::Behavior => write!(f, "behavior"),
            CardType::Interface => write!(f, "interface"),
            CardType::Constraint => write!(f, "constraint"),
            CardType::LogicalComponent => write!(f, "logical-component"),
            CardType::DeployableNode => write!(f, "deployable-node"),
            CardType::Actor => write!(f, "actor"),
            CardType::Test => write!(f, "test"),
            CardType::Artifact => write!(f, "artifact"),
            CardType::View => write!(f, "view"),
            CardType::Note => write!(f, "note"),
        }
    }
}

/// Lifecycle status of a card
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    Proposed,
    Approved,
    Implemented,
    Verified,
    Deprecated,
    Retired,
}

impl std::fmt::Display for CardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardStatus::Proposed => write!(f, "proposed"),
            CardStatus::Approved => write!(f, "approved"),
            CardStatus::Implemented => write!(f, "implemented"),
            CardStatus::Verified => write!(f, "verified"),
            CardStatus::Deprecated => write!(f, "deprecated"),
            CardStatus::Retired => write!(f, "retired"),
        }
    }
}

/// Single change entry in a card's audit history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHistoryEntry {
    pub change_number: u64,
    pub event: String, // "created", "modified", "approved", etc.
    pub timestamp: DateTime<Utc>,
    pub by: String, // user identifier or agent name
    pub fields_modified: Vec<String>,
    pub previous_values: HashMap<String, serde_json::Value>,
    pub note: Option<String>,
}

/// Base card structure - represents any architectural element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String, // Format: "namespace:element-name"
    pub r#type: CardType,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>, // Semantic version
    pub status: Option<CardStatus>,
    pub rationale: Option<String>,
    pub priority: Option<String>,
    pub owner: Option<String>,

    // Change tracking
    pub change_counter: u64,
    pub last_modified: DateTime<Utc>,

    // Relationships
    pub relations: Vec<String>,   // Card IDs
    pub links: Vec<String>,       // Link artifact IDs
    pub constraints: Vec<String>, // Constraint card IDs

    // Metadata
    pub attributes: HashMap<String, serde_json::Value>,
    pub acceptance_criteria: Vec<String>,

    // Audit trail
    pub audit_history: Vec<AuditHistoryEntry>,

    // Provenance
    pub provenance: Option<HashMap<String, serde_json::Value>>,
}

impl Card {
    pub fn new(id: String, card_type: CardType, name: String) -> Self {
        let now = Utc::now();
        let mut audit_history = Vec::new();

        // Initial creation entry
        audit_history.push(AuditHistoryEntry {
            change_number: 1,
            event: "created".to_string(),
            timestamp: now,
            by: "system:new-card".to_string(),
            fields_modified: vec!["id".to_string(), "type".to_string(), "name".to_string()],
            previous_values: HashMap::new(),
            note: Some("Initial card creation".to_string()),
        });

        Self {
            id,
            r#type: card_type,
            name,
            description: None,
            version: Some("1.0.0".to_string()),
            status: Some(CardStatus::Proposed),
            rationale: None,
            priority: None,
            owner: None,
            change_counter: 1,
            last_modified: now,
            relations: Vec::new(),
            links: Vec::new(),
            constraints: Vec::new(),
            attributes: HashMap::new(),
            acceptance_criteria: Vec::new(),
            audit_history,
            provenance: None,
        }
    }

    /// Record a modification to this card
    pub fn record_change(
        &mut self,
        event: &str,
        fields_modified: Vec<String>,
        previous_values: HashMap<String, serde_json::Value>,
        by: String,
        note: Option<String>,
    ) {
        self.change_counter += 1;
        self.last_modified = Utc::now();

        self.audit_history.push(AuditHistoryEntry {
            change_number: self.change_counter,
            event: event.to_string(),
            timestamp: self.last_modified,
            by,
            fields_modified,
            previous_values,
            note,
        });
    }
}

/// Directional, untyped link between two cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub weight: Option<f64>,
    pub confidence: Option<f64>,
    pub view_context: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Link {
    pub fn new(source_id: String, target_id: String) -> Self {
        Self {
            id: format!("{}->{}", source_id, target_id),
            source_id,
            target_id,
            weight: None,
            confidence: None,
            view_context: None,
            metadata: HashMap::new(),
        }
    }
}

/// A view is a projection of the architecture model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub id: String,
    pub name: String,
    pub view_type: String, // "hierarchy", "matrix", "graph", "sequence", "state-machine", "centered"
    pub scope: Vec<String>, // Card IDs included in this view
    pub filter: Option<HashMap<String, serde_json::Value>>,
    pub sort_by: Option<Vec<String>>,
    pub format: String,             // "mermaid", "svg", "html", "markdown"
    pub relationships: Vec<String>, // Link IDs to include
}

/// Architecture model - contains all cards, links, and views
#[derive(Debug, Clone)]
pub struct ArchitectureModel {
    pub cards: HashMap<String, Card>,
    pub links: Vec<Link>,
    pub views: HashMap<String, View>,
    pub root_driver_id: Option<String>,
}

impl ArchitectureModel {
    pub fn new() -> Self {
        Self {
            cards: HashMap::new(),
            links: Vec::new(),
            views: HashMap::new(),
            root_driver_id: None,
        }
    }

    pub fn add_card(&mut self, card: Card) -> Result<(), String> {
        if self.cards.contains_key(&card.id) {
            return Err(format!("Card with id '{}' already exists", card.id));
        }

        // Track root driver
        if card.r#type == CardType::Driver && card.id == "root:root-driver" {
            self.root_driver_id = Some(card.id.clone());
        }

        self.cards.insert(card.id.clone(), card);
        Ok(())
    }

    pub fn add_link(&mut self, link: Link) -> Result<(), String> {
        // Verify both ends exist
        if !self.cards.contains_key(&link.source_id) {
            return Err(format!("Source card '{}' not found", link.source_id));
        }
        if !self.cards.contains_key(&link.target_id) {
            return Err(format!("Target card '{}' not found", link.target_id));
        }

        self.links.push(link);
        Ok(())
    }

    pub fn get_card(&self, id: &str) -> Option<&Card> {
        self.cards.get(id)
    }

    pub fn get_card_mut(&mut self, id: &str) -> Option<&mut Card> {
        self.cards.get_mut(id)
    }
}

impl Default for ArchitectureModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = Card::new(
            "test:my-card".to_string(),
            CardType::Driver,
            "Test Card".to_string(),
        );
        assert_eq!(card.id, "test:my-card");
        assert_eq!(card.name, "Test Card");
        assert_eq!(card.change_counter, 1);
        assert_eq!(card.audit_history.len(), 1);
    }

    #[test]
    fn test_card_modification() {
        let mut card = Card::new(
            "test:my-card".to_string(),
            CardType::Driver,
            "Test Card".to_string(),
        );
        card.record_change(
            "modified",
            vec!["name".to_string()],
            vec![(
                "name".to_string(),
                serde_json::Value::String("Test Card".to_string()),
            )]
            .into_iter()
            .collect(),
            "user:test".to_string(),
            Some("Updated name".to_string()),
        );

        assert_eq!(card.change_counter, 2);
        assert_eq!(card.audit_history.len(), 2);
    }
}
