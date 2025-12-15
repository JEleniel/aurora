/// AURORA data models representing cards, links, and architectures
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Card types in the architecture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl CardType {
    /// Returns the folder name for this card type
    pub fn folder_name(&self) -> &'static str {
        match self {
            CardType::Driver => "driver",
            CardType::Requirement => "requirement",
            CardType::Behavior => "behavior",
            CardType::Interface => "interface",
            CardType::Constraint => "constraint",
            CardType::LogicalComponent => "logical-component",
            CardType::DeployableNode => "deployable-node",
            CardType::Actor => "actor",
            CardType::Test => "test",
            CardType::Artifact => "artifact",
            CardType::View => "view",
            CardType::Note => "note",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "driver" => Some(CardType::Driver),
            "requirement" => Some(CardType::Requirement),
            "behavior" => Some(CardType::Behavior),
            "interface" => Some(CardType::Interface),
            "constraint" => Some(CardType::Constraint),
            "logical-component" => Some(CardType::LogicalComponent),
            "deployable-node" => Some(CardType::DeployableNode),
            "actor" => Some(CardType::Actor),
            "test" => Some(CardType::Test),
            "artifact" => Some(CardType::Artifact),
            "view" => Some(CardType::View),
            "note" => Some(CardType::Note),
            _ => None,
        }
    }
}

/// Card status in the lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    Proposed,
    Approved,
    Implemented,
    Verified,
    Deprecated,
    Retired,
}

impl CardStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "proposed" => Some(CardStatus::Proposed),
            "approved" => Some(CardStatus::Approved),
            "implemented" => Some(CardStatus::Implemented),
            "verified" => Some(CardStatus::Verified),
            "deprecated" => Some(CardStatus::Deprecated),
            "retired" => Some(CardStatus::Retired),
            _ => None,
        }
    }
}

/// A card represents an element in the architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub r#type: CardType,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub status: Option<CardStatus>,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Card {
    pub fn new(id: String, r#type: CardType, name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Card {
            id,
            r#type,
            name,
            description,
            version: Some("1.0.0".to_string()),
            status: Some(CardStatus::Proposed),
            attributes: None,
            created_at: now,
            modified_at: now,
        }
    }
}

/// Link metadata with optional view context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkMetadata {
    pub weight: Option<f64>,
    pub confidence: Option<f64>,
    pub view_context: Option<String>,
    pub link_title: Option<String>,
}

/// A link represents a directional relationship between cards or to external URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub source_id: String,
    pub target_id: Option<String>,
    pub target_url: Option<String>,
    pub metadata: Option<LinkMetadata>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Link {
    pub fn new_internal(source_id: String, target_id: String) -> Self {
        let now = Utc::now();
        Link {
            source_id,
            target_id: Some(target_id),
            target_url: None,
            metadata: None,
            created_at: now,
            modified_at: now,
        }
    }

    pub fn new_external(source_id: String, target_url: String) -> Self {
        let now = Utc::now();
        Link {
            source_id,
            target_id: None,
            target_url: Some(target_url),
            metadata: None,
            created_at: now,
            modified_at: now,
        }
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub root_driver_id: Option<String>,
}

/// The complete architecture model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureModel {
    pub cards: HashMap<String, Card>,
    pub links: Vec<Link>,
    pub metadata: ProjectMetadata,
}

impl Default for ArchitectureModel {
    fn default() -> Self {
        ArchitectureModel {
            cards: HashMap::new(),
            links: Vec::new(),
            metadata: ProjectMetadata {
                name: None,
                description: None,
                version: Some("1.0.0".to_string()),
                root_driver_id: None,
            },
        }
    }
}

impl ArchitectureModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.insert(card.id.clone(), card);
    }

    pub fn remove_card(&mut self, id: &str) -> Option<Card> {
        // Also remove all links related to this card
        self.links
            .retain(|link| link.source_id != id && link.target_id.as_deref() != Some(id));
        self.cards.remove(id)
    }

    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn get_card(&self, id: &str) -> Option<&Card> {
        self.cards.get(id)
    }

    pub fn get_card_mut(&mut self, id: &str) -> Option<&mut Card> {
        self.cards.get_mut(id)
    }

    pub fn get_cards_by_type(&self, card_type: &CardType) -> Vec<&Card> {
        self.cards
            .values()
            .filter(|c| &c.r#type == card_type)
            .collect()
    }

    pub fn get_cards_by_status(&self, status: &CardStatus) -> Vec<&Card> {
        self.cards
            .values()
            .filter(|c| c.status.as_ref() == Some(status))
            .collect()
    }

    pub fn get_links_from(&self, source_id: &str) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|l| l.source_id == source_id)
            .collect()
    }

    pub fn get_links_to(&self, target_id: &str) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|l| l.target_id.as_deref() == Some(target_id))
            .collect()
    }

    /// Get statistics about the model
    pub fn statistics(&self) -> ModelStatistics {
        let mut cards_by_type = HashMap::new();
        let mut cards_by_status = HashMap::new();

        for card in self.cards.values() {
            *cards_by_type.entry(card.r#type.clone()).or_insert(0) += 1;
            if let Some(status) = &card.status {
                *cards_by_status.entry(status.clone()).or_insert(0) += 1;
            }
        }

        ModelStatistics {
            total_cards: self.cards.len(),
            total_links: self.links.len(),
            cards_by_type,
            cards_by_status,
        }
    }
}

/// Statistics about an architecture model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatistics {
    pub total_cards: usize,
    pub total_links: usize,
    pub cards_by_type: HashMap<CardType, usize>,
    pub cards_by_status: HashMap<CardStatus, usize>,
}
